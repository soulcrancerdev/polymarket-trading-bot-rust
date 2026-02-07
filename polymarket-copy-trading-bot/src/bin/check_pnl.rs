use anyhow::Result;
use polymarket_copy_rust::{fetch_data, EnvConfig, Logger};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
struct Activity {
    #[serde(rename = "proxyWallet")]
    proxy_wallet: String,
    timestamp: i64,
    #[serde(rename = "conditionId")]
    condition_id: String,
    #[serde(rename = "type")]
    activity_type: String,
    size: f64,
    #[serde(rename = "usdcSize")]
    usdc_size: f64,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    price: f64,
    asset: String,
    side: String,
    title: Option<String>,
    slug: Option<String>,
    outcome: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Position {
    asset: String,
    #[serde(rename = "conditionId")]
    condition_id: String,
    size: f64,
    #[serde(rename = "avgPrice")]
    avg_price: f64,
    #[serde(rename = "initialValue")]
    initial_value: f64,
    #[serde(rename = "currentValue")]
    current_value: f64,
    #[serde(rename = "cashPnl")]
    cash_pnl: f64,
    #[serde(rename = "percentPnl")]
    percent_pnl: f64,
    #[serde(rename = "totalBought")]
    total_bought: f64,
    #[serde(rename = "realizedPnl")]
    realized_pnl: f64,
    #[serde(rename = "percentRealizedPnl")]
    percent_realized_pnl: f64,
    #[serde(rename = "curPrice")]
    cur_price: f64,
    redeemable: Option<bool>,
    title: Option<String>,
    slug: Option<String>,
    outcome: Option<String>,
}

#[derive(Default)]
struct TradeGroup {
    buys: Vec<Activity>,
    sells: Vec<Activity>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = EnvConfig::from_env().await?;
    let proxy_wallet = &config.proxy_wallet;

    println!();
    println!("🔍 Detailed P&L discrepancy check\n");
    println!("Wallet: {}\n", proxy_wallet);
    Logger::separator();
    println!();

    let client = reqwest::Client::new();

    println!("📊 Fetching data from Polymarket API...\n");

    let positions_url = format!(
        "https://data-api.polymarket.com/positions?user={}",
        proxy_wallet
    );
    let positions: Vec<Position> = fetch_data(
        &client,
        &positions_url,
        config.request_timeout_ms,
        config.network_retry_limit,
    )
    .await?
    .as_array()
    .ok_or_else(|| anyhow::anyhow!("Expected array of positions"))?
    .iter()
    .filter_map(|v| serde_json::from_value(v.clone()).ok())
    .collect();

    println!("Fetched positions: {}\n", positions.len());

    let open_positions: Vec<Position> = positions.iter().filter(|p| p.size > 0.0).cloned().collect();
    let closed_positions: Vec<Position> = positions.iter().filter(|p| p.size == 0.0).cloned().collect();

    println!("• Open: {}", open_positions.len());
    println!("• Closed: {}\n", closed_positions.len());

    Logger::separator();
    println!();

    println!("📈 OPEN POSITIONS:\n");
    let mut total_open_value = 0.0;
    let mut total_open_initial = 0.0;
    let mut total_unrealized_pnl = 0.0;
    let mut total_open_realized = 0.0;

    for (idx, pos) in open_positions.iter().enumerate() {
        total_open_value += pos.current_value;
        total_open_initial += pos.initial_value;
        total_unrealized_pnl += pos.cash_pnl;
        total_open_realized += pos.realized_pnl;

        println!("{}. {} - {}", 
            idx + 1,
            pos.title.as_deref().unwrap_or("Unknown"),
            pos.outcome.as_deref().unwrap_or("N/A")
        );
        println!("   Size: {:.2} @ ${:.3}", pos.size, pos.avg_price);
        println!("   Current Value: ${:.2}", pos.current_value);
        println!("   Initial Value: ${:.2}", pos.initial_value);
        println!("   Unrealized P&L: ${:.2} ({:.2}%)", pos.cash_pnl, pos.percent_pnl);
        println!("   Realized P&L: ${:.2}", pos.realized_pnl);
        println!();
    }

    println!("   TOTAL for open:");
    println!("   • Current value: ${:.2}", total_open_value);
    println!("   • Initial value: ${:.2}", total_open_initial);
    println!("   • Unrealized P&L: ${:.2}", total_unrealized_pnl);
    println!("   • Realized P&L: ${:.2}\n", total_open_realized);

    Logger::separator();
    println!();

    println!("✅ CLOSED POSITIONS:\n");
    let mut total_closed_realized = 0.0;
    let mut total_closed_initial = 0.0;

    if !closed_positions.is_empty() {
        for (idx, pos) in closed_positions.iter().enumerate() {
            total_closed_realized += pos.realized_pnl;
            total_closed_initial += pos.initial_value;

            println!("{}. {} - {}",
                idx + 1,
                pos.title.as_deref().unwrap_or("Unknown"),
                pos.outcome.as_deref().unwrap_or("N/A")
            );
            println!("   Initial Value: ${:.2}", pos.initial_value);
            println!("   Realized P&L: ${:.2}", pos.realized_pnl);
            println!("   % P&L: {:.2}%", pos.percent_realized_pnl);
            println!();
        }

        println!("   TOTAL for closed:");
        println!("   • Initial investments: ${:.2}", total_closed_initial);
        println!("   • Realized P&L: ${:.2}\n", total_closed_realized);
    } else {
        println!("   ❌ No closed positions found in API\n");
    }

    Logger::separator();
    println!();

    println!("📊 OVERALL STATISTICS:\n");
    let total_realized = total_open_realized + total_closed_realized;

    println!("   • Open positions - Realized P&L: ${:.2}", total_open_realized);
    println!("   • Closed positions - Realized P&L: ${:.2}", total_closed_realized);
    println!("   • Unrealized P&L: ${:.2}", total_unrealized_pnl);
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   💰 TOTAL REALIZED PROFIT: ${:.2}\n", total_realized);

    Logger::separator();
    println!();

    println!("🔎 CHECK THROUGH TRADE HISTORY:\n");
    let activity_url = format!(
        "https://data-api.polymarket.com/activity?user={}&type=TRADE",
        proxy_wallet
    );
    let activities: Vec<Activity> = fetch_data(
        &client,
        &activity_url,
        config.request_timeout_ms,
        config.network_retry_limit,
    )
    .await?
    .as_array()
    .ok_or_else(|| anyhow::anyhow!("Expected array of activities"))?
    .iter()
    .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();

    let mut market_trades: HashMap<String, TradeGroup> = HashMap::new();

    for trade in &activities {
        let key = format!("{}:{}", trade.condition_id, trade.asset);
        let group = market_trades.entry(key).or_insert_with(TradeGroup::default);
        if trade.side == "BUY" {
            group.buys.push((*trade).clone());
        } else {
            group.sells.push((*trade).clone());
        }
    }

    println!("   Found markets with activity: {}\n", market_trades.len());

    let mut calculated_realized_pnl = 0.0;
    let mut markets_with_profit = 0;

    for (key, trades) in &market_trades {
        let total_bought: f64 = trades.buys.iter().map(|t| t.usdc_size).sum();
        let total_sold: f64 = trades.sells.iter().map(|t| t.usdc_size).sum();
        let pnl = total_sold - total_bought;

        if pnl.abs() > 0.01 {
            let market = trades.buys.first().or_else(|| trades.sells.first()).unwrap();
            println!("   {}", market.title.as_deref().unwrap_or("Unknown"));
            println!("   • Bought: ${:.2}", total_bought);
            println!("   • Sold: ${:.2}", total_sold);
            println!("   • P&L: ${:.2}", pnl);
            println!();

            if total_sold > 0.0 {
                calculated_realized_pnl += pnl;
                markets_with_profit += 1;
            }
        }
    }

    println!("   💰 Calculated realized profit: ${:.2}", calculated_realized_pnl);
    println!("   📊 Markets with closed profit: {}\n", markets_with_profit);

    Logger::separator();
    println!();

    println!("💡 CONCLUSIONS:\n");
    println!("   1. API returns realized profit: ${:.2}", total_realized);
    println!("   2. Calculated from trade history: ${:.2}", calculated_realized_pnl);
    println!("   3. Polymarket UI shows: ~$12.02\n");

    if (total_realized - calculated_realized_pnl).abs() > 1.0 {
        println!("   ⚠️  DISCREPANCY DETECTED!\n");
        println!("   Possible reasons:");
        println!("   • API only counts partially closed positions");
        println!("   • UI includes unrealized partial sales");
        println!("   • Data synchronization delay between UI and API");
        println!("   • Different P&L calculation methodology\n");
    }

    println!("   📈 Why chart shows $0.00:");
    println!("   • Amount too small ($2-12) for visualization");
    println!("   • Timeline doesn't start from $0");
    println!("   • Chart requires at least several data points");
    println!("   • UI update delay (can be 1-24 hours)\n");

    println!("   🔧 Recommendations:");
    println!("   1. Wait 24 hours for full update");
    println!("   2. Close more positions to increase realized profit");
    println!("   3. Try clearing browser cache");
    println!("   4. Check in incognito mode\n");

    Logger::separator();
    println!();

    Ok(())
}
