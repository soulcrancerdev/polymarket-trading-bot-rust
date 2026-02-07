use anyhow::Result;
use alloy::signers::local::PrivateKeySigner;
use polymarket_client_sdk::clob::Client as ClobClient;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::clob::types::{OrderType as SdkOrderType, Amount, Side};
use polymarket_client_sdk::types::Decimal;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{get_trade_multiplier, EnvConfig};
use crate::db::Db;
use crate::types::{UserActivity, UserPosition};
use crate::utils::{fetch_data, Logger};

// Min order sizes (PM API requirements)
const MIN_ORDER_SIZE_USD: f64 = 1.0;
const MIN_ORDER_SIZE_TOKENS: f64 = 1.0;

// Extract error msg from API response (handles nested error objs)
fn extract_order_error(response: &serde_json::Value) -> Option<String> {
    if response.is_null() {
        return None;
    }

    if let Some(s) = response.as_str() {
        return Some(s.to_string());
    }

    if let Some(obj) = response.as_object() {
        if let Some(error_val) = obj.get("error") {
            if let Some(s) = error_val.as_str() {
                return Some(s.to_string());
            }
            // Check nested error obj
            if let Some(nested) = error_val.as_object() {
                if let Some(err) = nested.get("error").and_then(|v| v.as_str()) {
                    return Some(err.to_string());
                }
                if let Some(msg) = nested.get("message").and_then(|v| v.as_str()) {
                    return Some(msg.to_string());
                }
            }
        }

        if let Some(s) = obj.get("errorMsg").and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }

        if let Some(s) = obj.get("message").and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }

    None
}

// Check if error is balance/allowance related (retry won't help)
fn is_insufficient_balance_or_allowance_error(message: Option<&str>) -> bool {
    let Some(msg) = message else {
        return false;
    };
    let lower = msg.to_lowercase();
    lower.contains("not enough balance") || lower.contains("allowance")
}

// Main order router - dispatches to buy/sell/merge strategies
pub async fn post_order(
    config: &EnvConfig,
    clob_client: &ClobClient,
    condition: &str,
    my_position: Option<&UserPosition>,
    user_position: Option<&UserPosition>,
    trade: &UserActivity,
    my_balance: f64,
    _user_balance: f64,
    user_address: &str,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    match condition {
        "merge" => {
            execute_merge_strategy(config, trade, my_position, user_address, clob_client, http_client, db, signer).await?;
        }
        "buy" => {
            execute_buy_strategy(config, trade, my_position, my_balance, user_address, clob_client, http_client, db, signer).await?;
        }
        "sell" => {
            execute_sell_strategy(config, trade, my_position, user_position, user_address, clob_client, http_client, db, signer).await?;
        }
        _ => {
            Logger::error(&format!("Unknown condition: {}", condition));
        }
    }
    Ok(())
}

// Merge strategy: sell entire position at best bid (FOK orders)
async fn execute_merge_strategy(
    config: &EnvConfig,
    trade: &UserActivity,
    my_position: Option<&UserPosition>,
    user_address: &str,
    clob_client: &ClobClient,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    Logger::info("Executing MERGE strategy...");
    
    // Need a position to merge
    let my_position = match my_position {
        Some(p) => p,
        None => {
            Logger::warning("No position to merge");
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            return Ok(());
        }
    };

    let asset = trade.asset.as_deref().unwrap_or("");
    if asset.is_empty() {
        Logger::warning("No asset specified");
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    let mut remaining = my_position.size.unwrap_or(0.0);

    // Skip if position too small (below PM min)
    if remaining < MIN_ORDER_SIZE_TOKENS {
        Logger::warning(&format!(
            "Position size ({:.2} tokens) too small to merge - skipping",
            remaining
        ));
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    let mut retry = 0u32;
    let mut abort_due_to_funds = false;

        while remaining > 0.0 && retry < config.retry_limit {
        let book_url = format!(
            "{}/book?token_id={}",
            config.clob_http_url.trim_end_matches('/'),
            asset
        );
        let book: serde_json::Value = fetch_data(
            http_client,
            &book_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;
        
        let bids = book
            .get("bids")
            .and_then(|b| b.as_array())
            .ok_or_else(|| anyhow::anyhow!("No bids"))?;

        if bids.is_empty() {
            Logger::warning("No bids available in order book");
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            break;
        }

        let best_bid = bids
            .iter()
            .filter_map(|b| {
                let price: f64 = b
                    .get("price")
                    .and_then(|p| p.as_str())
                    .and_then(|s| s.parse().ok())?;
                let size: f64 = b
                    .get("size")
                    .and_then(|s| s.as_str())
                    .and_then(|s| s.parse().ok())?;
                Some((price, size))
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let (price, size) = match best_bid {
            Some((p, s)) => (p, s),
            None => {
                Logger::warning("No bids in order book");
                break;
            }
        };

        Logger::info(&format!("Best bid: {} @ ${:.4}", size, price));

        let sell_amount = if remaining <= size {
            remaining
        } else {
            size
        };

        let exp_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 90;
        let exp = chrono::DateTime::from_timestamp(exp_secs as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
        let token_id = alloy::primitives::U256::from_str_radix(
            asset.trim_start_matches("0x"),
            16,
        )
        .or_else(|_| alloy::primitives::U256::from_str(&asset))?;
        let decimal_size = Decimal::from_str(&format!("{:.4}", sell_amount))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let decimal_price =
            Decimal::from_str(&format!("{:.2}", price))
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        let order = clob_client
            .limit_order()
            .token_id(token_id)
            .size(decimal_size)
            .price(decimal_price)
            .side(Side::Sell)
            .order_type(SdkOrderType::FOK)
            .expiration(exp)
            .build()
            .await?;
        let signed = clob_client.sign(&signer, order).await?;
        let resp = clob_client.post_order(signed).await?;

        let error_msg = resp.error_msg.as_deref();

        if resp.error_msg.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            retry = 0;
            Logger::order_result(
                true,
                &format!("Sold {:.2} tokens at ${:.4}", sell_amount, price),
            );
            remaining -= sell_amount;
        } else {
            if is_insufficient_balance_or_allowance_error(error_msg) {
                abort_due_to_funds = true;
                Logger::warning(&format!(
                    "Order rejected: {}",
                    error_msg.unwrap_or("Insufficient balance or allowance")
                ));
                Logger::warning(
                    "Skipping remaining attempts. Top up funds or check allowance.",
                );
                break;
            }
            retry += 1;
            Logger::warning(&format!(
                "Order failed (attempt {}/{}){}",
                retry,
                config.retry_limit,
                error_msg.map(|m| format!(" - {}", m)).unwrap_or_default()
            ));
        }
    }

    if let Some(ref id) = trade.id {
        let mut update_doc = mongodb::bson::doc! { "bot": true };
        if abort_due_to_funds {
            update_doc.insert("botExcutedTime", config.retry_limit as i64);
        } else if retry >= config.retry_limit {
            update_doc.insert("botExcutedTime", retry as i64);
        }
        db.update_activity(user_address, id, &update_doc).await?;
    }

    Ok(())
}

// Buy strategy: copy trader's buy order (with size limits & multipliers)
async fn execute_buy_strategy(
    config: &EnvConfig,
    trade: &UserActivity,
    my_position: Option<&UserPosition>,
    my_balance: f64,
    user_address: &str,
    clob_client: &ClobClient,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    Logger::info("Executing BUY strategy...");
    Logger::info(&format!("Your balance: ${:.2}", my_balance));
    Logger::info(&format!("Trader bought: ${:.2}", trade.usdc_size.unwrap_or(0.0)));

    let asset = trade.asset.as_deref().unwrap_or("");
    if asset.is_empty() {
        Logger::warning("No asset specified");
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    // Calc current position value (for position limits)
    let current_position_value = my_position
        .map(|p| p.size.unwrap_or(0.0) * p.avg_price.unwrap_or(0.0))
        .unwrap_or(0.0);

    // Calc order size based on strategy (percentage/fixed/adaptive)
    let order_calc = crate::config::calculate_order_size(
        &config.copy_strategy_config,
        trade.usdc_size.unwrap_or(0.0),
        my_balance,
        current_position_value,
    );

    Logger::info(&format!("📊 {}", order_calc.reasoning));

    // Skip if below min order size
    if order_calc.final_amount < config.copy_strategy_config.min_order_size_usd {
        Logger::warning(&format!("❌ Cannot execute: {}", order_calc.reasoning));
        if order_calc.below_minimum {
            Logger::warning("💡 Increase COPY_SIZE or wait for larger trades");
        }
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    let mut remaining = order_calc.final_amount;
    let mut available_balance = my_balance;

    let mut retry = 0u32;
    let mut abort_due_to_funds = false;
    let mut total_bought_tokens = 0.0;

    while remaining > 0.0 && retry < config.retry_limit {
        let book_url = format!(
            "{}/book?token_id={}",
            config.clob_http_url.trim_end_matches('/'),
            asset
        );
        let book: serde_json::Value = fetch_data(
            http_client,
            &book_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;
        
        let asks = book
            .get("asks")
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow::anyhow!("No asks"))?;

        if asks.is_empty() {
            Logger::warning("No asks available in order book");
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            break;
        }

        let best_ask = asks
            .iter()
            .filter_map(|a| {
                let price: f64 = a
                    .get("price")
                    .and_then(|p| p.as_str())
                    .and_then(|s| s.parse().ok())?;
                let size: f64 = a
                    .get("size")
                    .and_then(|s| s.as_str())
                    .and_then(|s| s.parse().ok())?;
                Some((price, size))
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let (best_price, best_size) = match best_ask {
            Some(x) => x,
            None => {
                Logger::warning("No asks in order book");
                break;
            }
        };

        Logger::info(&format!("Best ask: {} @ ${:.4}", best_size, best_price));

        if remaining < MIN_ORDER_SIZE_USD {
            Logger::info(&format!(
                "Remaining amount (${:.2}) below minimum - completing trade",
                remaining
            ));
            if let Some(ref id) = trade.id {
                let mut update_doc = mongodb::bson::doc! { "bot": true };
                if total_bought_tokens > 0.0 {
                    update_doc.insert("myBoughtSize", total_bought_tokens);
                }
                db.update_activity(user_address, id, &update_doc).await?;
            }
            break;
        }

        let max_order_size = best_size * best_price;
        let order_size = remaining.min(max_order_size);

        if order_size < MIN_ORDER_SIZE_USD {
            Logger::info(&format!(
                "Order size (${:.2}) below minimum (${}) - completing trade",
                order_size, MIN_ORDER_SIZE_USD
            ));
            if let Some(ref id) = trade.id {
                let mut update_doc = mongodb::bson::doc! { "bot": true };
                if total_bought_tokens > 0.0 {
                    update_doc.insert("myBoughtSize", total_bought_tokens);
                }
                db.update_activity(user_address, id, &update_doc).await?;
            }
            break;
        }

        if available_balance < order_size {
            Logger::warning(&format!(
                "Insufficient balance: Need ${:.2} but only have ${:.2}",
                order_size, available_balance
            ));
            abort_due_to_funds = true;
            break;
        }

        Logger::info(&format!(
            "Creating order: ${:.2} @ ${:.4} (Balance: ${:.2})",
            order_size, best_price, available_balance
        ));

        let exp_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 90;
        let exp = chrono::DateTime::from_timestamp(exp_secs as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
        let token_id = alloy::primitives::U256::from_str_radix(
            asset.trim_start_matches("0x"),
            16,
        )
        .or_else(|_| alloy::primitives::U256::from_str(&asset))?;
        let decimal_amount =
            Decimal::from_str(&format!("{:.2}", order_size))
                .map_err(|e| anyhow::anyhow!("Decimal: {}", e))?;
        let order = clob_client
            .market_order()
            .token_id(token_id)
            .amount(Amount::usdc(decimal_amount)?)
            .side(Side::Buy)
            .order_type(SdkOrderType::FOK)
            .expiration(exp)
            .build()
            .await?;
        let signed = clob_client.sign(&signer, order).await?;
        let resp = clob_client.post_order(signed).await?;

        let error_msg = resp.error_msg.as_deref();

        if resp.error_msg.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            retry = 0;
            let tokens_bought = order_size / best_price;
            total_bought_tokens += tokens_bought;
            Logger::order_result(
                true,
                &format!(
                    "Bought ${:.2} at ${:.4} ({:.2} tokens)",
                    order_size, best_price, tokens_bought
                ),
            );
            remaining -= order_size;
            available_balance -= order_size;
        } else {
            if is_insufficient_balance_or_allowance_error(error_msg) {
                abort_due_to_funds = true;
                Logger::warning(&format!(
                    "Order rejected: {}",
                    error_msg.unwrap_or("Insufficient balance or allowance")
                ));
                Logger::warning(
                    "Skipping remaining attempts. Top up funds or check allowance.",
                );
                break;
            }
            retry += 1;
            Logger::warning(&format!(
                "Order failed (attempt {}/{}){}",
                retry,
                config.retry_limit,
                error_msg.map(|m| format!(" - {}", m)).unwrap_or_default()
            ));
        }
    }

    if let Some(ref id) = trade.id {
        let mut update_doc = mongodb::bson::doc! { "bot": true };
        if abort_due_to_funds {
            update_doc.insert("botExcutedTime", config.retry_limit as i64);
        } else if retry >= config.retry_limit {
            update_doc.insert("botExcutedTime", retry as i64);
        }
        if total_bought_tokens > 0.0 {
            update_doc.insert("myBoughtSize", total_bought_tokens);
        }
        db.update_activity(user_address, id, &update_doc).await?;
    }

    if total_bought_tokens > 0.0 {
        Logger::info(&format!(
            "📝 Tracked purchase: {:.2} tokens for future sell calculations",
            total_bought_tokens
        ));
    }

    Ok(())
}

async fn execute_sell_strategy(
    config: &EnvConfig,
    trade: &UserActivity,
    my_position: Option<&UserPosition>,
    user_position: Option<&UserPosition>,
    user_address: &str,
    clob_client: &ClobClient,
    http_client: &reqwest::Client,
    db: &Db,
    signer: &mut PrivateKeySigner,
) -> Result<()> {
    Logger::info("Executing SELL strategy...");

    let my_position = match my_position {
        Some(p) => p,
        None => {
            Logger::warning("No position to sell");
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            return Ok(());
        }
    };

    let asset = trade.asset.as_deref().unwrap_or("");
    if asset.is_empty() {
        Logger::warning("No asset specified");
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    let previous_buys = db
        .find_all_buy_activities_for_asset(user_address, asset, &trade.condition_id)
        .await?;
    let total_bought_tokens: f64 = previous_buys
        .iter()
        .filter_map(|t| t.my_bought_size)
        .sum();

    if total_bought_tokens > 0.0 {
        Logger::info(&format!(
            "📊 Found {} previous purchases: {:.2} tokens bought",
            previous_buys.len(),
            total_bought_tokens
        ));
    }

    let mut remaining = if user_position.is_none() {
        Logger::info(&format!(
            "Trader closed entire position → Selling all your {:.2} tokens",
            my_position.size.unwrap_or(0.0)
        ));
        my_position.size.unwrap_or(0.0)
    } else {
        let up = user_position.unwrap();
        let trader_sell_percent = trade.size.unwrap_or(0.0)
            / (up.size.unwrap_or(0.0) + trade.size.unwrap_or(0.0));
        let trader_position_before = up.size.unwrap_or(0.0) + trade.size.unwrap_or(0.0);

        Logger::info(&format!(
            "Position comparison: Trader has {:.2} tokens, You have {:.2} tokens",
            trader_position_before,
            my_position.size.unwrap_or(0.0)
        ));
        Logger::info(&format!(
            "Trader selling: {:.2} tokens ({:.2}% of their position)",
            trade.size.unwrap_or(0.0),
            trader_sell_percent * 100.0
        ));

        let base_sell_size = if total_bought_tokens > 0.0 {
            Logger::info(&format!(
                "Calculating from tracked purchases: {:.2} × {:.2}% = {:.2} tokens",
                total_bought_tokens,
                trader_sell_percent * 100.0,
                total_bought_tokens * trader_sell_percent
            ));
            total_bought_tokens * trader_sell_percent
        } else {
            Logger::warning(&format!(
                "No tracked purchases found, using current position: {:.2} × {:.2}% = {:.2} tokens",
                my_position.size.unwrap_or(0.0),
                trader_sell_percent * 100.0,
                my_position.size.unwrap_or(0.0) * trader_sell_percent
            ));
            my_position.size.unwrap_or(0.0) * trader_sell_percent
        };

        let multiplier = get_trade_multiplier(
            &config.copy_strategy_config,
            trade.usdc_size.unwrap_or(0.0),
        );
        let calculated = base_sell_size * multiplier;

        if (multiplier - 1.0).abs() > 1e-9 {
            Logger::info(&format!(
                "Applying {}x multiplier (based on trader's ${:.2} order): {:.2} → {:.2} tokens",
                multiplier,
                trade.usdc_size.unwrap_or(0.0),
                base_sell_size,
                calculated
            ));
        }

        calculated
    };

    if remaining < MIN_ORDER_SIZE_TOKENS {
        Logger::warning(&format!(
            "❌ Cannot execute: Sell amount {:.2} tokens below minimum ({:.2} token)",
            remaining, MIN_ORDER_SIZE_TOKENS
        ));
        Logger::warning("💡 This happens when position sizes are too small or mismatched");
        if let Some(ref id) = trade.id {
            db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                .await?;
        }
        return Ok(());
    }

    if remaining > my_position.size.unwrap_or(0.0) {
        Logger::warning(&format!(
            "⚠️  Calculated sell {:.2} tokens > Your position {:.2} tokens",
            remaining,
            my_position.size.unwrap_or(0.0)
        ));
        Logger::warning(&format!(
            "Capping to maximum available: {:.2} tokens",
            my_position.size.unwrap_or(0.0)
        ));
        remaining = my_position.size.unwrap_or(0.0);
    }

    let mut retry = 0u32;
    let mut abort_due_to_funds = false;
    let mut total_sold_tokens = 0.0;

    while remaining > 0.0 && retry < config.retry_limit {
        let book_url = format!(
            "{}/book?token_id={}",
            config.clob_http_url.trim_end_matches('/'),
            asset
        );
        let book: serde_json::Value = fetch_data(
            http_client,
            &book_url,
            config.request_timeout_ms,
            config.network_retry_limit,
        )
        .await?;
        
        let bids = book
            .get("bids")
            .and_then(|b| b.as_array())
            .ok_or_else(|| anyhow::anyhow!("No bids"))?;

        if bids.is_empty() {
            Logger::warning("No bids available in order book");
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            break;
        }

        let best_bid = bids
            .iter()
            .filter_map(|b| {
                let price: f64 = b
                    .get("price")
                    .and_then(|p| p.as_str())
                    .and_then(|s| s.parse().ok())?;
                let size: f64 = b
                    .get("size")
                    .and_then(|s| s.as_str())
                    .and_then(|s| s.parse().ok())?;
                Some((price, size))
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let (price, size) = match best_bid {
            Some((p, s)) => (p, s),
            None => {
                Logger::warning("No bids in order book");
                break;
            }
        };

        Logger::info(&format!("Best bid: {} @ ${:.4}", size, price));

        if remaining < MIN_ORDER_SIZE_TOKENS {
            Logger::info(&format!(
                "Remaining amount ({:.2} tokens) below minimum - completing trade",
                remaining
            ));
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            break;
        }

        let sell_amount = remaining.min(size);

        if sell_amount < MIN_ORDER_SIZE_TOKENS {
            Logger::info(&format!(
                "Order amount ({:.2} tokens) below minimum - completing trade",
                sell_amount
            ));
            if let Some(ref id) = trade.id {
                db.update_activity(user_address, id, &mongodb::bson::doc! { "bot": true })
                    .await?;
            }
            break;
        }

        let exp_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 90;
        let exp = chrono::DateTime::from_timestamp(exp_secs as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
        let token_id = alloy::primitives::U256::from_str_radix(
            asset.trim_start_matches("0x"),
            16,
        )
        .or_else(|_| alloy::primitives::U256::from_str(&asset))?;
        let decimal_size = Decimal::from_str(&format!("{:.4}", sell_amount))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let decimal_price =
            Decimal::from_str(&format!("{:.2}", price))
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        let order = clob_client
            .limit_order()
            .token_id(token_id)
            .size(decimal_size)
            .price(decimal_price)
            .side(Side::Sell)
            .order_type(SdkOrderType::FOK)
            .expiration(exp)
            .build()
            .await?;
        let signed = clob_client.sign(&signer, order).await?;
        let resp = clob_client.post_order(signed).await?;

        let error_msg = resp.error_msg.as_deref();

        if resp.error_msg.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            retry = 0;
            total_sold_tokens += sell_amount;
            Logger::order_result(
                true,
                &format!("Sold {:.2} tokens at ${:.4}", sell_amount, price),
            );
            remaining -= sell_amount;
        } else {
            if is_insufficient_balance_or_allowance_error(error_msg) {
                abort_due_to_funds = true;
                Logger::warning(&format!(
                    "Order rejected: {}",
                    error_msg.unwrap_or("Insufficient balance or allowance")
                ));
                Logger::warning(
                    "Skipping remaining attempts. Top up funds or check allowance.",
                );
                break;
            }
            retry += 1;
            Logger::warning(&format!(
                "Order failed (attempt {}/{}){}",
                retry,
                config.retry_limit,
                error_msg.map(|m| format!(" - {}", m)).unwrap_or_default()
            ));
        }
    }

    if total_sold_tokens > 0.0 && total_bought_tokens > 0.0 {
        let sell_percentage = total_sold_tokens / total_bought_tokens;

        if sell_percentage >= 0.99 {
            db.update_many_activities(
                user_address,
                asset,
                &trade.condition_id,
                &mongodb::bson::doc! { "myBoughtSize": 0.0 },
            )
            .await?;
            Logger::info(&format!(
                "🧹 Cleared purchase tracking (sold {:.1}% of position)",
                sell_percentage * 100.0
            ));
        } else {
            for buy in &previous_buys {
                if let Some(ref id) = buy.id {
                    let new_size = (buy.my_bought_size.unwrap_or(0.0)) * (1.0 - sell_percentage);
                    db.update_activity(
                        user_address,
                        id,
                        &mongodb::bson::doc! { "myBoughtSize": new_size },
                    )
                    .await?;
                }
            }
            Logger::info(&format!(
                "📝 Updated purchase tracking (sold {:.1}% of tracked position)",
                sell_percentage * 100.0
            ));
        }
    }

    if let Some(ref id) = trade.id {
        let mut update_doc = mongodb::bson::doc! { "bot": true };
        if abort_due_to_funds {
            update_doc.insert("botExcutedTime", config.retry_limit as i64);
        } else if retry >= config.retry_limit {
            update_doc.insert("botExcutedTime", retry as i64);
        }
        db.update_activity(user_address, id, &update_doc).await?;
    }

    Ok(())
}

