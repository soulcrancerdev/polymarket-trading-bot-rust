use anyhow::Result;
use alloy::signers::local::PrivateKeySigner;
use polymarket_client_sdk::clob::Client as ClobClient;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::Normal;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

use crate::config::EnvConfig;
use crate::db::Db;
use crate::types::{UserActivity, UserPosition};
use crate::utils::{create_clob_client, fetch_data, get_usdc_balance, post_order, Logger};

// Min USD to aggregate trades (small trades get batched)
const TRADE_AGGREGATION_MIN_TOTAL_USD: f64 = 1.0;

// Global flag to stop executor gracefully
static IS_RUNNING: AtomicBool = AtomicBool::new(true);

// Trade + trader address wrapper
#[derive(Debug, Clone)]
struct TradeWithUser {
    trade: UserActivity,
    user_address: String,
}

// Aggregated trade group (batches small trades together)
#[derive(Debug, Clone)]
struct AggregatedTrade {
    user_address: String,
    condition_id: Option<String>,
    asset: Option<String>,
    side: String,
    slug: Option<String>,
    event_slug: Option<String>,
    trades: Vec<TradeWithUser>,
    total_usdc_size: f64,
    average_price: f64,
    first_trade_time: Instant,
    last_trade_time: Instant,
}

// Thread-safe buffer for aggregating trades
type AggregationBuffer = Arc<Mutex<HashMap<String, AggregatedTrade>>>;

// Fetch unprocessed trades from DB for all tracked traders
async fn read_temp_trades(config: &EnvConfig, db: &Db) -> Result<Vec<TradeWithUser>> {
    let mut all_trades = Vec::new();

    for user_address in &config.user_addresses {
        let trades = db.find_unprocessed_trades(user_address).await?;
        for trade in trades {
            all_trades.push(TradeWithUser {
                trade,
                user_address: user_address.clone(),
            });
        }
    }

    Ok(all_trades)
}

// Generate key for grouping trades (user:condition:asset:side)
fn get_aggregation_key(trade: &TradeWithUser) -> String {
    format!(
        "{}:{}:{}:{}",
        trade.user_address,
        trade.trade.condition_id.as_deref().unwrap_or(""),
        trade.trade.asset.as_deref().unwrap_or(""),
        trade.trade.side.as_deref().unwrap_or("BUY")
    )
}

// Add trade to aggregation buffer (batches small trades)
async fn add_to_aggregation_buffer(
    buffer: &AggregationBuffer,
    trade: TradeWithUser,
) -> Result<()> {
    let key = get_aggregation_key(&trade);
    let mut buffer_guard = buffer.lock().await;
    let now = Instant::now();

    // Update existing group or create new one
    if let Some(existing) = buffer_guard.get_mut(&key) {
        existing.trades.push(trade.clone());
        existing.total_usdc_size += trade.trade.usdc_size.unwrap_or(0.0);
        // Recalc weighted avg price
        let mut total_value = 0.0;
        for t in &existing.trades {
            let usdc = t.trade.usdc_size.unwrap_or(0.0);
            let price = t.trade.price.unwrap_or(0.0);
            total_value += usdc * price;
        }
        if existing.total_usdc_size > 0.0 {
            existing.average_price = total_value / existing.total_usdc_size;
        }
        existing.last_trade_time = now;
    } else {
        // New aggregation group
        let usdc_size = trade.trade.usdc_size.unwrap_or(0.0);
        let price = trade.trade.price.unwrap_or(0.0);
        buffer_guard.insert(
            key,
            AggregatedTrade {
                user_address: trade.user_address.clone(),
                condition_id: trade.trade.condition_id.clone(),
                asset: trade.trade.asset.clone(),
                side: trade.trade.side.clone().unwrap_or_else(|| "BUY".to_string()),
                slug: trade.trade.slug.clone(),
                event_slug: trade.trade.event_slug.clone(),
                trades: vec![trade],
                total_usdc_size: usdc_size,
                average_price: price,
                first_trade_time: now,
                last_trade_time: now,
            },
        );
    }

    Ok(())
}

async fn get_ready_aggregated_trades(
    buffer: &AggregationBuffer,
    window_seconds: u64,
    db: &Db,
) -> Result<Vec<AggregatedTrade>> {
    let mut ready = Vec::new();
    let now = Instant::now();
    let window_duration = Duration::from_secs(window_seconds);

    let mut buffer_guard = buffer.lock().await;
    let mut keys_to_remove = Vec::new();

    for (key, agg) in buffer_guard.iter() {
        let time_elapsed = now.duration_since(agg.first_trade_time);

        if time_elapsed >= window_duration {
            if agg.total_usdc_size >= TRADE_AGGREGATION_MIN_TOTAL_USD {
                ready.push(agg.clone());
            } else {
                let asset_display = agg
                    .slug
                    .as_ref()
                    .or(agg.asset.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                Logger::info(&format!(
                    "Trade aggregation for {} on {}: ${:.2} total from {} trades below minimum (${}) - skipping",
                    Logger::format_address(&agg.user_address),
                    asset_display,
                    agg.total_usdc_size,
                    agg.trades.len(),
                    TRADE_AGGREGATION_MIN_TOTAL_USD
                ));

                for trade in &agg.trades {
                    if let Some(ref id) = trade.trade.id {
                        db.update_activity(
                            &trade.user_address,
                            id,
                            &mongodb::bson::doc! { "bot": true },
                        )
                        .await?;
                    }
                }
            }
            keys_to_remove.push(key.clone());
        }
    }

    for key in keys_to_remove {
        buffer_guard.remove(&key);
    }

    Ok(ready)
}

// Execute trades immediately (no aggregation)
async fn do_trading(
    config: &EnvConfig,
    trades: &[TradeWithUser],
    clob_client: &ClobClient<Authenticated<Normal>>,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    for trade in trades {
        // Mark as processing in DB
        if let Some(ref id) = trade.trade.id {
            db.update_activity(
                &trade.user_address,
                id,
                &mongodb::bson::doc! { "botExcutedTime": 1_i64 },
            )
            .await?;
        }

        // Log trade details
        Logger::trade(
            &trade.user_address,
            trade.trade.side.as_deref().unwrap_or("UNKNOWN"),
            crate::utils::TradeDetails {
                asset: trade.trade.asset.clone(),
                side: trade.trade.side.clone(),
                amount: trade.trade.usdc_size,
                price: trade.trade.price,
                slug: trade.trade.slug.clone(),
                event_slug: trade.trade.event_slug.clone(),
                transaction_hash: trade.trade.transaction_hash.clone(),
                title: trade.trade.title.clone(),
            },
        );

        // Fetch positions for both wallets (yours & trader's)
        let my_positions_url = format!(
            "https://data-api.polymarket.com/positions?user={}",
            config.proxy_wallet
        );
        let user_positions_url = format!(
            "https://data-api.polymarket.com/positions?user={}",
            trade.user_address
        );

        let my_positions_data: serde_json::Value = fetch_data(
            http_client,
            &my_positions_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;
        let user_positions_data: serde_json::Value = fetch_data(
            http_client,
            &user_positions_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;

        let my_positions: Vec<UserPosition> = if let Some(arr) = my_positions_data.as_array() {
            arr.iter()
                .filter_map(|p| serde_json::from_value::<UserPosition>(p.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        let user_positions: Vec<UserPosition> = if let Some(arr) = user_positions_data.as_array() {
            arr.iter()
                .filter_map(|p| serde_json::from_value::<UserPosition>(p.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        let condition_id = trade.trade.condition_id.as_deref();
        let my_position = my_positions
            .iter()
            .find(|p| p.condition_id.as_deref() == condition_id);
        let user_position = user_positions
            .iter()
            .find(|p| p.condition_id.as_deref() == condition_id);

        // Get balances & calc trader's portfolio value
        let my_balance = get_usdc_balance(
            &config.rpc_url,
            &config.usdc_contract_address,
            &config.proxy_wallet,
        )
        .await
        .unwrap_or(0.0);

        let user_balance: f64 = user_positions
            .iter()
            .map(|p| p.current_value.unwrap_or(0.0))
            .sum();

        Logger::balance(my_balance, user_balance, &trade.user_address);

        // Determine order type & execute
        let condition = if trade.trade.side.as_deref().unwrap_or("") == "BUY" {
            "buy"
        } else {
            "sell"
        };

        post_order(
            config,
            clob_client,
            condition,
            my_position,
            user_position,
            &trade.trade,
            my_balance,
            user_balance,
            &trade.user_address,
            http_client,
            db,
            signer,
        )
        .await?;

        Logger::separator();
    }

    Ok(())
}

async fn do_aggregated_trading(
    config: &EnvConfig,
    aggregated_trades: &[AggregatedTrade],
    clob_client: &ClobClient<Authenticated<Normal>>,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    for agg in aggregated_trades {
        Logger::header(&format!(
            "📊 AGGREGATED TRADE ({} trades combined)",
            agg.trades.len()
        ));
        Logger::info(&format!(
            "Market: {}",
            agg.slug
                .as_ref()
                .or(agg.asset.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("unknown")
        ));
        Logger::info(&format!("Side: {}", agg.side));
        Logger::info(&format!("Total volume: ${:.2}", agg.total_usdc_size));
        Logger::info(&format!("Average price: ${:.4}", agg.average_price));

        for trade in &agg.trades {
            if let Some(ref id) = trade.trade.id {
                db.update_activity(
                    &trade.user_address,
                    id,
                    &mongodb::bson::doc! { "botExcutedTime": 1_i64 },
                )
                .await?;
            }
        }

        let my_positions_url = format!(
            "https://data-api.polymarket.com/positions?user={}",
            config.proxy_wallet
        );
        let user_positions_url = format!(
            "https://data-api.polymarket.com/positions?user={}",
            agg.user_address
        );

        let my_positions_data: serde_json::Value = fetch_data(
            http_client,
            &my_positions_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;
        let user_positions_data: serde_json::Value = fetch_data(
            http_client,
            &user_positions_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;

        let my_positions: Vec<UserPosition> = if let Some(arr) = my_positions_data.as_array() {
            arr.iter()
                .filter_map(|p| serde_json::from_value::<UserPosition>(p.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        let user_positions: Vec<UserPosition> = if let Some(arr) = user_positions_data.as_array() {
            arr.iter()
                .filter_map(|p| serde_json::from_value::<UserPosition>(p.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        let condition_id = agg.condition_id.as_deref();
        let my_position = my_positions
            .iter()
            .find(|p| p.condition_id.as_deref() == condition_id);
        let user_position = user_positions
            .iter()
            .find(|p| p.condition_id.as_deref() == condition_id);

        let my_balance = get_usdc_balance(
            &config.rpc_url,
            &config.usdc_contract_address,
            &config.proxy_wallet,
        )
        .await
        .unwrap_or(0.0);

        let user_balance: f64 = user_positions
            .iter()
            .map(|p| p.current_value.unwrap_or(0.0))
            .sum();

        Logger::balance(my_balance, user_balance, &agg.user_address);

        let mut synthetic_trade = agg.trades[0].trade.clone();
        synthetic_trade.usdc_size = Some(agg.total_usdc_size);
        synthetic_trade.price = Some(agg.average_price);
        synthetic_trade.side = Some(agg.side.clone());

        let condition = if agg.side == "BUY" { "buy" } else { "sell" };

        post_order(
            config,
            clob_client,
            condition,
            my_position,
            user_position,
            &synthetic_trade,
            my_balance,
            user_balance,
            &agg.user_address,
            http_client,
            db,
            signer,
        )
        .await?;

        Logger::separator();
    }

    Ok(())
}

// Main executor loop - polls DB for trades & executes orders
pub async fn run_trade_executor(
    config: &EnvConfig,
    db: &Db,
    http_client: &reqwest::Client,
) -> Result<()> {
    // Init CLOB client & signer
    let (clob_client, mut signer) = create_clob_client(config).await?;

    Logger::success(&format!(
        "Trade executor ready for {} trader(s)",
        config.user_addresses.len()
    ));
    if config.trade_aggregation_enabled {
        Logger::info(&format!(
            "Trade aggregation enabled: {}s window, ${} minimum",
            config.trade_aggregation_window_seconds, TRADE_AGGREGATION_MIN_TOTAL_USD
        ));
    }

    // Init aggregation buffer (even if disabled, keeps code simpler)
    let aggregation_buffer: AggregationBuffer = if config.trade_aggregation_enabled {
        Arc::new(Mutex::new(HashMap::new()))
    } else {
        Arc::new(Mutex::new(HashMap::new()))
    };

    // Poll every 300ms for new trades
    let mut last_check = Instant::now();
    let poll_interval = Duration::from_millis(300);

    while IS_RUNNING.load(Ordering::Relaxed) {
        // Fetch unprocessed trades from DB
        let trades = match read_temp_trades(config, db).await {
            Ok(t) => t,
            Err(e) => {
                Logger::error(&format!("Failed to read trades: {}", e));
                sleep(poll_interval).await;
                continue;
            }
        };

        // Aggregation mode: batch small trades, execute large ones immediately
        if config.trade_aggregation_enabled {
            if !trades.is_empty() {
                Logger::clear_line();
                Logger::info(&format!(
                    "📥 {} new trade{} detected",
                    trades.len(),
                    if trades.len() > 1 { "s" } else { "" }
                ));

                for trade in &trades {
                    let usdc_size = trade.trade.usdc_size.unwrap_or(0.0);
                    let side = trade.trade.side.as_deref().unwrap_or("");
                    // Small BUY trades go to buffer, everything else executes immediately
                    if side == "BUY" && usdc_size < TRADE_AGGREGATION_MIN_TOTAL_USD {
                        let asset_display = trade
                            .trade
                            .slug
                            .as_ref()
                            .or(trade.trade.asset.as_ref())
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");
                        Logger::info(&format!(
                            "Adding ${:.2} {} trade to aggregation buffer for {}",
                            usdc_size, side, asset_display
                        ));
                        if let Err(e) = add_to_aggregation_buffer(&aggregation_buffer, trade.clone()).await {
                            Logger::error(&format!("Failed to add trade to aggregation buffer: {}", e));
                        }
                    } else {
                        Logger::clear_line();
                        Logger::header("⚡ IMMEDIATE TRADE (above threshold)");
                        if let Err(e) = do_trading(
                            config,
                            &[trade.clone()],
                            &clob_client,
                            http_client,
                            db,
                            &mut signer,
                        )
                        .await
                        {
                            Logger::error(&format!("Trade executor error: {}", e));
                        }
                    }
                }
                last_check = Instant::now();
            }

            let ready_aggregations = match get_ready_aggregated_trades(
                &aggregation_buffer,
                config.trade_aggregation_window_seconds,
                db,
            )
            .await
            {
                Ok(agg) => agg,
                Err(e) => {
                    Logger::error(&format!("Failed to get ready aggregated trades: {}", e));
                    Vec::new()
                }
            };
            if !ready_aggregations.is_empty() {
                Logger::clear_line();
                Logger::header(&format!(
                    "⚡ {} AGGREGATED TRADE{} READY",
                    ready_aggregations.len(),
                    if ready_aggregations.len() > 1 { "S" } else { "" }
                ));
                if let Err(e) = do_aggregated_trading(
                    config,
                    &ready_aggregations,
                    &clob_client,
                    http_client,
                    db,
                    &mut signer,
                )
                .await
                {
                    Logger::error(&format!("Trade executor error: {}", e));
                }
                last_check = Instant::now();
            }

            if trades.is_empty() && ready_aggregations.is_empty() {
                if last_check.elapsed() > Duration::from_millis(300) {
                    let buffered_count = aggregation_buffer.lock().await.len();
                    if buffered_count > 0 {
                        Logger::waiting(
                            config.user_addresses.len(),
                            Some(&format!("{} trade group(s) pending", buffered_count)),
                        );
                    } else {
                        Logger::waiting(config.user_addresses.len(), None);
                    }
                    last_check = Instant::now();
                }
            }
        } else {
            if !trades.is_empty() {
                Logger::clear_line();
                Logger::header(&format!(
                    "⚡ {} NEW TRADE{} TO COPY",
                    trades.len(),
                    if trades.len() > 1 { "S" } else { "" }
                ));
                if let Err(e) = do_trading(config, &trades, &clob_client, http_client, db, &mut signer)                .await {
                    Logger::error(&format!("Trade executor error: {}", e));
                }
                last_check = Instant::now();
            }
            
            if trades.is_empty() {
                if last_check.elapsed() > Duration::from_millis(300) {
                    Logger::waiting(config.user_addresses.len(), None);
                    last_check = Instant::now();
                }
            }
        }

        if !IS_RUNNING.load(Ordering::Relaxed) {
            break;
        }
        sleep(poll_interval).await;
    }

    Logger::info("Trade executor stopped");
    Ok(())
}

pub fn stop_trade_executor() {
    IS_RUNNING.store(false, Ordering::Relaxed);
    Logger::info("Trade executor shutdown requested...");
}
