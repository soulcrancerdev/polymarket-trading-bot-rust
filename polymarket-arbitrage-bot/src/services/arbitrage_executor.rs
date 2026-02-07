use crate::config::{Env, MIN_ORDER_SIZE_USD, TOKEN_AMOUNT};
use crate::services::create_clob_client::{ClobClient, OrderResponse, OrderSide, OrderType};
use crate::utils::logger::log_error;
use anyhow::{anyhow, Result};
use colored::*;

// Trading constants (FYI: Polymarket has strict precision requirements)
const MIN_TOKEN_AMOUNT: f64 = 5.0; // Min tokens per order
const PRICE_DECIMALS: usize = 4; // Price precision (4 decimals)
const TOKEN_DECIMALS: usize = 2; // Token qty precision (2 decimals)
const PRECISION_EPSILON: f64 = 0.000001; // Float comparison threshold

#[derive(Debug, Clone)]
pub struct ArbitrageOrderResult {
    pub success: bool,
    pub token_id: String,
    pub side: String,
    pub amount: f64,
    pub price: f64,
    pub tokens_bought: Option<f64>,
    pub error: Option<String>,
}

// Floor value to N decimals (AFAIK: required for Polymarket API precision)
fn floor_to_decimals(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).floor() / multiplier
}

// Calculate precise amounts that meet API requirements (IMO: this is tricky due to rounding)
fn calculate_precise_amounts(target_token_amount: f64, price: f64) -> (f64, f64) {
    let mut token_amount = floor_to_decimals(target_token_amount, TOKEN_DECIMALS);
    
    if token_amount < 0.01 {
        token_amount = 0.01;
    }

    // Iterate to find amounts that satisfy precision (FYI: max 10 iterations)
    for _ in 0..10 {
        let usdc_amount = floor_to_decimals(token_amount * price, PRICE_DECIMALS);
        let calculated_token_amount = usdc_amount / price;
        let floored_token_amount = floor_to_decimals(calculated_token_amount, TOKEN_DECIMALS);
        let difference = (calculated_token_amount - floored_token_amount).abs();

        // Check if precision is good enough (BTW: must be exact within epsilon)
        if difference < PRECISION_EPSILON && floored_token_amount >= 0.01 {
            return (floored_token_amount, usdc_amount);
        }

        token_amount = floored_token_amount;
        if token_amount < 0.01 {
            token_amount = 0.01; // Enforce minimum
            let final_usdc = floor_to_decimals(token_amount * price, PRICE_DECIMALS);
            return (token_amount, final_usdc);
        }
    }

    let final_usdc = floor_to_decimals(token_amount * price, PRICE_DECIMALS);
    (token_amount, final_usdc)
}

fn create_error_result(token_id: &str, side: &str, error: String) -> ArbitrageOrderResult {
    ArbitrageOrderResult {
        success: false,
        token_id: token_id.to_string(),
        side: side.to_string(),
        amount: 0.0,
        price: 0.0,
        tokens_bought: None,
        error: Some(error),
    }
}

// Execute buy order for arbitrage (FYI: handles precision and validation)
pub async fn execute_buy_order(
    clob_client: &ClobClient,
    token_id: &str,
    side: &str,
    amount_usdc: f64,
    ask_price: f64,
) -> ArbitrageOrderResult {
    // Validate inputs (IMO: fail fast on bad data)
    if token_id.trim().is_empty() {
        return create_error_result(token_id, side, "Invalid tokenId".to_string());
    }

    if amount_usdc < MIN_ORDER_SIZE_USD {
        return create_error_result(
            token_id,
            side,
            format!("Order size (${:.2}) below minimum (${:.2})", amount_usdc, MIN_ORDER_SIZE_USD),
        );
    }

    if ask_price <= 0.0 || !ask_price.is_finite() {
        return create_error_result(token_id, side, format!("Invalid ask price: {}", ask_price));
    }

    let floored_price = floor_to_decimals(ask_price, PRICE_DECIMALS);
    if floored_price <= 0.0 || !floored_price.is_finite() {
        return create_error_result(token_id, side, format!("Invalid floored price: {}", floored_price));
    }

    // Calculate token quantity (AFAIK: ensure we meet minimums)
    let initial_share_quantity = amount_usdc / floored_price;
    let min_share_quantity = MIN_ORDER_SIZE_USD / floored_price;
    let mut share_quantity = initial_share_quantity.max(min_share_quantity).max(MIN_TOKEN_AMOUNT);

    let (precise_token_amount, floored_amount_usdc) = calculate_precise_amounts(share_quantity, floored_price);
    let mut share_quantity = precise_token_amount;
    let mut floored_amount_usdc = floored_amount_usdc;

    // Ensure minimums (BTW: round up if needed to meet min token requirement)
    if share_quantity < MIN_TOKEN_AMOUNT {
        share_quantity = (share_quantity * 100.0).ceil() / 100.0;
        if share_quantity < MIN_TOKEN_AMOUNT {
            share_quantity = MIN_TOKEN_AMOUNT; // Force to minimum
        }
        let (t, u) = calculate_precise_amounts(share_quantity, floored_price);
        share_quantity = t;
        floored_amount_usdc = u;
    }

    // Final validation (FYI: precision adjustments might drop below minimum)
    if floored_amount_usdc < MIN_ORDER_SIZE_USD {
        return create_error_result(
            token_id,
            side,
            format!("After precision adjustment, USDC amount (${:.2}) below minimum (${:.2})", floored_amount_usdc, MIN_ORDER_SIZE_USD),
        );
    }

    println!(
        "{}",
        format!(
            "[{}] Executing at ${:.4} (original: ${:.4})\n  Amount: ${:.4} USDC\n  Share quantity: {:.2} tokens\n  TokenID: {}...",
            side, floored_price, ask_price, floored_amount_usdc, share_quantity, &token_id[..token_id.len().min(20)]
        )
        .cyan()
    );

    // Create and submit order (IMO: this is where we actually trade)
    match clob_client
        .create_market_order(
            OrderSide::Buy,
            token_id,
            floored_amount_usdc,
            floored_price,
        )
        .await
    {
        Ok(signed_order) => {
            match clob_client.post_order(&signed_order, OrderType::FAK).await {
                Ok(resp) => {
                    if resp.success {
                        let tokens_bought = floored_amount_usdc / floored_price;
                        println!(
                            "{}",
                            format!(
                                "\n✓✓✓ [{}] ORDER COMPLETED ✓✓✓\n  Order ID: {}\n  Amount: ${:.4} USDC\n  Price: ${:.4}\n  Tokens Bought: {:.2} tokens\n",
                                side,
                                resp.order_id.as_ref().unwrap_or(&"N/A".to_string()),
                                floored_amount_usdc,
                                floored_price,
                                tokens_bought
                            )
                            .green()
                        );

                        ArbitrageOrderResult {
                            success: true,
                            token_id: token_id.to_string(),
                            side: side.to_string(),
                            amount: floored_amount_usdc,
                            price: floored_price,
                            tokens_bought: Some(tokens_bought),
                            error: None,
                        }
                    } else {
                        let error_msg = resp.error.unwrap_or_else(|| "Unknown error".to_string());
                        println!("{}", format!("✗ [{}] Order failed: {}", side, error_msg).red());
                        log_error(&format!("[{}] Order failed: {}", side, error_msg), Some(&format!("executeBuyOrder-{}", side)));
                        create_error_result(token_id, side, error_msg)
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to post order: {}", e);
                    println!("{}", format!("✗ [{}] {}", side, error_msg).red());
                    log_error(&error_msg, Some(&format!("executeBuyOrder-{}", side)));
                    create_error_result(token_id, side, error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to create order: {}", e);
            println!("{}", format!("✗ [{}] {}", side, error_msg).red());
            log_error(&error_msg, Some(&format!("executeBuyOrder-{}", side)));
            create_error_result(token_id, side, error_msg)
        }
    }
}

fn calculate_trade_amounts(up_price: f64, down_price: f64) -> (f64, f64, f64) {
    let token_amount = floor_to_decimals(TOKEN_AMOUNT, TOKEN_DECIMALS);
    let up_amount_usdc = floor_to_decimals(token_amount * up_price, PRICE_DECIMALS);
    let down_amount_usdc = floor_to_decimals(token_amount * down_price, PRICE_DECIMALS);
    (token_amount, up_amount_usdc, down_amount_usdc)
}

// Execute arbitrage trade (FYI: buys both UP and DOWN simultaneously)
pub async fn execute_arbitrage_trade(
    clob_client: &ClobClient,
    up_token_id: &str,
    down_token_id: &str,
    up_price: f64,
    down_price: f64,
    _up_bid_price: f64, // Unused (would be for liquidation)
    _down_bid_price: f64, // Unused
    env: &Env,
) -> Result<(ArbitrageOrderResult, ArbitrageOrderResult, bool)> {
    // Validate inputs (AFAIK: fail fast on bad data)
    if up_token_id.trim().is_empty() || down_token_id.trim().is_empty() {
        return Err(anyhow!("Invalid token IDs"));
    }

    if up_price <= 0.0 || !up_price.is_finite() || down_price <= 0.0 || !down_price.is_finite() {
        return Err(anyhow!("Invalid prices"));
    }

    let (token_amount, up_amount_usdc, down_amount_usdc) = calculate_trade_amounts(up_price, down_price);

    // Check minimums (BTW: both sides must meet minimum order size)
    if up_amount_usdc < MIN_ORDER_SIZE_USD || down_amount_usdc < MIN_ORDER_SIZE_USD {
        return Err(anyhow!(
            "Order sizes below minimum: UP=${:.2}, DOWN=${:.2}",
            up_amount_usdc,
            down_amount_usdc
        ));
    }

    println!(
        "{}",
        format!(
            "\n⚡ Executing arbitrage trade: Buying {:.2} tokens each\n  UP: ${:.4} → ${:.2} USDC\n  DOWN: ${:.4} → ${:.2} USDC\n  Total: {:.2} tokens, ${:.2} USDC\n",
            token_amount, up_price, up_amount_usdc, down_price, down_amount_usdc, token_amount * 2.0, up_amount_usdc + down_amount_usdc
        )
        .green()
        .bold()
    );

    // Execute both orders (IMO: sequential for now, could be parallel)
    let up_result = execute_buy_order(clob_client, up_token_id, "UP", up_amount_usdc, up_price).await;
    let down_result = execute_buy_order(clob_client, down_token_id, "DOWN", down_amount_usdc, down_price).await;

    let both_success = up_result.success && down_result.success; // Check if both succeeded

    if both_success {
        println!(
            "{}",
            format!(
                "\n╔════════════════════════════════════════════════════════════════╗\n║         🎉 ARBITRAGE TRADE COMPLETED SUCCESSFULLY! 🎉          ║\n╚════════════════════════════════════════════════════════════════╝\n  ✅ UP Order:\n     • Tokens: {:.2}\n     • Price: ${:.4}\n     • Amount: ${:.2} USDC\n  ✅ DOWN Order:\n     • Tokens: {:.2}\n     • Price: ${:.4}\n     • Amount: ${:.2} USDC\n  📊 Summary:\n     • Total Spent: ${:.2} USDC\n     • Status: Both orders executed successfully\n╔════════════════════════════════════════════════════════════════╗\n\n",
                up_result.tokens_bought.unwrap_or(0.0),
                up_result.price,
                up_result.amount,
                down_result.tokens_bought.unwrap_or(0.0),
                down_result.price,
                down_result.amount,
                up_result.amount + down_result.amount
            )
            .green()
            .bold()
        );
    } else {
        let error_msg = format!(
            "Arbitrage trade failed - UP: {}, DOWN: {}",
            up_result.error.as_ref().unwrap_or(&"Unknown".to_string()),
            down_result.error.as_ref().unwrap_or(&"Unknown".to_string())
        );
        log_error(&error_msg, Some("executeArbitrageTrade"));
    }

    Ok((up_result, down_result, both_success))
}

