use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::EnvConfig;
use crate::db::Db;
use crate::types::{RtdsActivity, UserActivity, UserPosition};
use crate::utils::{self, Logger};

// RTDS WebSocket URL (Polymarket real-time data stream)
const RTDS_URL: &str = "wss://ws-live-data.polymarket.com";
const POSITION_UPDATE_INTERVAL_SECS: u64 = 30;
const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_DELAY_SECS: u64 = 5;

// Global flag for graceful shutdown
static RUNNING: AtomicBool = AtomicBool::new(true);

// Stop monitor (called on shutdown)
pub fn stop_trade_monitor() {
    RUNNING.store(false, Ordering::SeqCst);
    Logger::info("Trade monitor shutdown requested...");
}

pub struct TradeMonitorHandle {
    _tx: broadcast::Sender<()>,
}

// Init: show DB stats, positions, balances
async fn init(
    config: &EnvConfig,
    db: &Db,
    http_client: &reqwest::Client,
) -> Result<()> {
    // Count activities per trader
    let mut counts = Vec::new();
    for addr in &config.user_addresses {
        let c = db.count_activities(addr).await?;
        counts.push(c);
    }
    Logger::clear_line();
    Logger::db_connection(&config.user_addresses, &counts);

    // Fetch & display your positions
    let my_positions_url = format!(
        "https://data-api.polymarket.com/positions?user={}",
        config.proxy_wallet
    );
    let current_balance = utils::get_usdc_balance(
        &config.rpc_url,
        &config.usdc_contract_address,
        &config.proxy_wallet,
    )
    .await
    .unwrap_or(0.0);

    match utils::fetch_data(
        http_client,
        &my_positions_url,
        config.request_timeout_ms,
        config.network_retry_limit,
    )
    .await
    {
        Ok(data) => {
            if let Some(arr) = data.as_array() {
                let mut total_value = 0.0;
                let mut initial_value = 0.0;
                let mut weighted_pnl = 0.0;
                for pos in arr.iter() {
                    let value = pos
                        .get("currentValue")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let initial = pos
                        .get("initialValue")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let pnl = pos
                        .get("percentPnl")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    total_value += value;
                    initial_value += initial;
                    weighted_pnl += value * pnl;
                }
                let my_overall_pnl = if total_value > 0.0 {
                    weighted_pnl / total_value
                } else {
                    0.0
                };

                let mut top_positions = arr.clone();
                top_positions.sort_by(|a, b| {
                    let pnl_a = a
                        .get("percentPnl")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let pnl_b = b
                        .get("percentPnl")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    pnl_b.partial_cmp(&pnl_a).unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_positions: Vec<_> = top_positions.iter().take(5).cloned().collect();

                Logger::clear_line();
                Logger::my_positions(
                    &config.proxy_wallet,
                    arr.len(),
                    &top_positions,
                    my_overall_pnl,
                    total_value,
                    initial_value,
                    current_balance,
                );
            } else {
                Logger::clear_line();
                Logger::my_positions(
                    &config.proxy_wallet,
                    0,
                    &[],
                    0.0,
                    0.0,
                    0.0,
                    current_balance,
                );
            }
        }
        Err(e) => {
            Logger::error(&format!("Failed to fetch your positions: {}", e));
        }
    }

    let mut position_counts = Vec::new();
    let mut position_details = Vec::new();
    let mut profitabilities = Vec::new();
    for addr in &config.user_addresses {
        let positions = db.get_positions(addr).await?;
        position_counts.push(positions.len());

        let mut total_value = 0.0;
        let mut weighted_pnl = 0.0;
        for pos in &positions {
            let value = pos.current_value.unwrap_or(0.0);
            let pnl = pos.percent_pnl.unwrap_or(0.0);
            total_value += value;
            weighted_pnl += value * pnl;
        }
        let overall_pnl = if total_value > 0.0 {
            weighted_pnl / total_value
        } else {
            0.0
        };
        profitabilities.push(overall_pnl);

        let mut sorted_positions = positions.clone();
        sorted_positions.sort_by(|a, b| {
            let pnl_a = a.percent_pnl.unwrap_or(0.0);
            let pnl_b = b.percent_pnl.unwrap_or(0.0);
            pnl_b.partial_cmp(&pnl_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_positions: Vec<serde_json::Value> = sorted_positions
            .iter()
            .take(3)
            .filter_map(|p| serde_json::to_value(p).ok())
            .collect();
        position_details.push(top_positions);
    }
    Logger::clear_line();
    Logger::traders_positions(
        &config.user_addresses,
        &position_counts,
        &position_details,
        &profitabilities,
    );

    Ok(())
}

// Process trade from RTDS: validate timestamp, check duplicates, save to DB
async fn process_trade_activity(
    db: &Db,
    config: &EnvConfig,
    activity: &RtdsActivity,
    address: &str,
) -> Result<()> {
    // Normalize timestamp (handle both ms & sec formats)
    let ts = activity.timestamp.unwrap_or(0);
    let ts_ms = if ts > 1_000_000_000_000 {
        ts
    } else {
        ts * 1000
    };
    // Skip if trade too old (configurable threshold)
    let hours_ago = (chrono::Utc::now().timestamp_millis() - ts_ms) as f64 / (1000.0 * 3600.0);
    if hours_ago > config.too_old_timestamp_hours as f64 {
        return Ok(());
    }

    // Skip if no tx hash or already processed
    let tx_hash = activity.transaction_hash.as_deref().unwrap_or("");
    if tx_hash.is_empty() {
        return Ok(());
    }
    if db.find_activity_by_tx(address, tx_hash).await?.is_some() {
        return Ok(());
    }

    let doc = UserActivity {
        id: None,
        proxy_wallet: activity.proxy_wallet.clone(),
        timestamp: activity.timestamp,
        condition_id: activity.condition_id.clone(),
        activity_type: Some("TRADE".to_string()),
        size: activity.size,
        usdc_size: Some(activity.usdc_size()),
        transaction_hash: activity.transaction_hash.clone(),
        price: activity.price,
        asset: activity.asset.clone(),
        side: activity.side.clone(),
        outcome_index: activity.outcome_index,
        title: activity.title.clone(),
        slug: activity.slug.clone(),
        icon: activity.icon.clone(),
        event_slug: activity.event_slug.clone(),
        outcome: activity.outcome.clone(),
        name: activity.name.clone(),
        pseudonym: None,
        bio: None,
        profile_image: None,
        profile_image_optimized: None,
        bot: Some(false),
        bot_executed_time: Some(0),
        my_bought_size: None,
    };

    db.insert_activity(address, &doc).await?;
    Logger::info(&format!(
        "New trade detected for {}",
        Logger::format_address(address)
    ));
    Ok(())
}

async fn update_positions(
    config: &EnvConfig,
    db: &Db,
    http_client: &reqwest::Client,
) -> Result<()> {
    for addr in &config.user_addresses {
        match utils::fetch_data(
            http_client,
            &format!("https://data-api.polymarket.com/positions?user={}", addr),
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await
        {
            Ok(data) => {
                if let Some(arr) = data.as_array() {
                    for p in arr {
                        if let Ok(pos) = serde_json::from_value::<UserPosition>(p.clone()) {
                            let _ = db.upsert_position(addr, &pos).await;
                        }
                    }
                }
            }
            Err(e) => {
                Logger::error(&format!(
                    "Error updating positions for {}: {}",
                    Logger::format_address(addr),
                    e
                ));
            }
        }
    }
    Ok(())
}

// Connect to RTDS WebSocket & subscribe to trade activity (with auto-reconnect)
async fn connect_rtds(
    config: Arc<EnvConfig>,
    db: Arc<Db>,
    http_client: Arc<reqwest::Client>,
    reconnect_attempts: Arc<std::sync::atomic::AtomicU32>,
) -> Result<()> {
    loop {
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }

        Logger::info(&format!("Connecting to RTDS at {}...", RTDS_URL));

        match connect_async(RTDS_URL).await {
            Ok((ws_stream, _)) => {
                Logger::success("RTDS WebSocket connected");
                reconnect_attempts.store(0, Ordering::SeqCst);

                let (mut write, mut read) = ws_stream.split();

                // Subscribe to trade activity for each tracked trader
                let subscriptions: Vec<_> = config
                    .user_addresses
                    .iter()
                    .map(|_| json!({"topic": "activity", "type": "trades"}))
                    .collect();

                let subscribe_message = json!({
                    "action": "subscribe",
                    "subscriptions": subscriptions
                });

                // Send subscription msg
                if let Err(e) = write.send(Message::Text(subscribe_message.to_string())).await {
                    Logger::error(&format!("Failed to send subscription: {}", e));
                    continue;
                }

                Logger::success(&format!(
                    "Subscribed to RTDS for {} trader(s) - monitoring in real-time",
                    config.user_addresses.len()
                ));

                let db_msg = db.clone();
                let config_msg = config.clone();
                let mut message_task = tokio::spawn(async move {
                    while RUNNING.load(Ordering::SeqCst) {
                        match read.next().await {
                            Some(Ok(Message::Text(t))) => {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&t) {
                                    if parsed.get("action").and_then(|a| a.as_str()) == Some("subscribed")
                                        || parsed.get("status").and_then(|s| s.as_str()) == Some("subscribed")
                                    {
                                        Logger::info("RTDS subscription confirmed");
                                        continue;
                                    }

                                    if parsed.get("topic").and_then(|t| t.as_str()) == Some("activity")
                                        && parsed.get("type").and_then(|t| t.as_str()) == Some("trades")
                                    {
                                        if let Some(payload) = parsed.get("payload") {
                                            if let Ok(activity) =
                                                serde_json::from_value::<RtdsActivity>(payload.clone())
                                            {
                                                let proxy = activity
                                                    .proxy_wallet
                                                    .as_deref()
                                                    .unwrap_or("")
                                                    .to_lowercase();
                                                if config_msg
                                                    .user_addresses
                                                    .iter()
                                                    .any(|a| a.to_lowercase() == proxy)
                                                {
                                                    let _ = process_trade_activity(
                                                        &db_msg,
                                                        &config_msg,
                                                        &activity,
                                                        &proxy,
                                                    )
                                                    .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                Logger::warning("RTDS WebSocket closed");
                                break;
                            }
                            Some(Err(e)) => {
                                Logger::error(&format!("RTDS WebSocket error: {}", e));
                                break;
                            }
                            None => break,
                            _ => continue,
                        }
                    }
                });

                message_task.await.ok();
            }
            Err(e) => {
                Logger::error(&format!("Failed to connect to RTDS: {}", e));
            }
        }

        if RUNNING.load(Ordering::SeqCst) {
            let attempts = reconnect_attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if attempts < MAX_RECONNECT_ATTEMPTS {
                let delay = RECONNECT_DELAY_SECS * attempts.min(5) as u64;
                Logger::info(&format!(
                    "Reconnecting to RTDS in {}s (attempt {}/{})...",
                    delay, attempts, MAX_RECONNECT_ATTEMPTS
                ));
                sleep(Duration::from_secs(delay)).await;
            } else {
                Logger::error(&format!(
                    "Max reconnection attempts ({}) reached. Please restart the bot.",
                    MAX_RECONNECT_ATTEMPTS
                ));
                break;
            }
        }
    }

    Ok(())
}

pub async fn run_trade_monitor(
    config: &EnvConfig,
    db: &Db,
    http_client: &reqwest::Client,
) -> Result<TradeMonitorHandle> {
    RUNNING.store(true, Ordering::SeqCst);

    init(config, db, http_client).await?;

    Logger::success(&format!(
        "Monitoring {} trader(s) using RTDS (Real-Time Data Stream)",
        config.user_addresses.len()
    ));
    Logger::separator();

    Logger::info("First run: marking all historical trades as processed...");
    for addr in &config.user_addresses {
        let n = db.mark_historical_processed(addr).await?;
        if n > 0 {
            Logger::info(&format!(
                "Marked {} historical trades as processed for {}",
                n,
                Logger::format_address(addr)
            ));
        }
    }
    Logger::success("\nHistorical trades processed. Now monitoring for new trades only.");
    Logger::separator();

    let config_arc = Arc::new(config.clone());
    let db_arc = Arc::new(db.clone());
    let http_arc = Arc::new(http_client.clone());
    let reconnect_attempts = Arc::new(std::sync::atomic::AtomicU32::new(0));

    let db_pos = db_arc.clone();
    let config_pos = config_arc.clone();
    let http_pos = http_arc.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(POSITION_UPDATE_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            let _ = update_positions(&config_pos, &db_pos, &http_pos).await;
        }
    });

    let config_ws = config_arc.clone();
    let db_ws = db_arc.clone();
    let http_ws = http_arc.clone();
    let reconnect_ws = reconnect_attempts.clone();
    tokio::spawn(async move {
        let _ = connect_rtds(config_ws, db_ws, http_ws, reconnect_ws).await;
    });

    let (tx, _) = broadcast::channel::<()>(1);
    Ok(TradeMonitorHandle { _tx: tx })
}
