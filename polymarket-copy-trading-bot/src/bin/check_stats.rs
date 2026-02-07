use anyhow::Result;
use polymarket_copy_rust::{fetch_data, get_usdc_balance, EnvConfig, Logger};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
    title: Option<String>,
    slug: Option<String>,
    outcome: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = EnvConfig::from_env().await?;
    let proxy_wallet = &config.proxy_wallet;

    println!();
    println!("🔍 Checking your wallet statistics on Polymarket\n");
    println!("Wallet: {}\n", proxy_wallet);
    Logger::separator();
    println!();

    let client = reqwest::Client::new();

    println!("💰 USDC BALANCE");
    let balance = get_usdc_balance(
        &config.rpc_url,
        &config.usdc_contract_address,
        proxy_wallet,
    )
    .await?;
    println!("   Available: ${:.2}\n", balance);

    println!("📊 OPEN POSITIONS");
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

    if !positions.is_empty() {
        println!("   Total positions: {}\n", positions.len());

        let total_value: f64 = positions.iter().map(|p| p.current_value).sum();
        let total_initial_value: f64 = positions.iter().map(|p| p.initial_value).sum();
        let total_unrealized_pnl: f64 = positions.iter().map(|p| p.cash_pnl).sum();
        let total_realized_pnl: f64 = positions.iter().map(|p| p.realized_pnl).sum();

        println!("   💵 Current value: ${:.2}", total_value);
        println!("   💵 Initial value: ${:.2}", total_initial_value);
        if total_initial_value > 0.0 {
            println!(
                "   📈 Unrealized P&L: ${:.2} ({:.2}%)",
                total_unrealized_pnl,
                (total_unrealized_pnl / total_initial_value) * 100.0
            );
        } else {
            println!("   📈 Unrealized P&L: ${:.2}", total_unrealized_pnl);
        }
        println!("   ✅ Realized P&L: ${:.2}\n", total_realized_pnl);

        println!("   🏆 Top-5 positions by profit:\n");
        let mut top_positions = positions.clone();
        top_positions.sort_by(|a, b| {
            b.percent_pnl
                .partial_cmp(&a.percent_pnl)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top_positions.truncate(5);

        for (idx, pos) in top_positions.iter().enumerate() {
            let pnl_sign = if pos.percent_pnl >= 0.0 { "📈" } else { "📉" };
            println!("   {}. {} {}", idx + 1, pnl_sign, pos.title.as_deref().unwrap_or("Unknown"));
            if let Some(ref outcome) = pos.outcome {
                println!("      {}", outcome);
            }
            println!(
                "      Size: {:.2} tokens @ ${:.3}",
                pos.size, pos.avg_price
            );
            println!(
                "      P&L: ${:.2} ({:.2}%)",
                pos.cash_pnl, pos.percent_pnl
            );
            println!("      Current price: ${:.3}", pos.cur_price);
            if let Some(ref slug) = pos.slug {
                println!("      📍 https://polymarket.com/event/{}", slug);
            }
            println!();
        }
    } else {
        println!("   ❌ No open positions found\n");
    }

    Logger::separator();
    println!();
    println!("📜 TRADE HISTORY (last 20)\n");
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

    if !activities.is_empty() {
        println!("   Total trades in API: {}\n", activities.len());

        let buy_trades: Vec<&Activity> = activities.iter().filter(|a| a.side == "BUY").collect();
        let sell_trades: Vec<&Activity> = activities.iter().filter(|a| a.side == "SELL").collect();
        let total_buy_volume: f64 = buy_trades.iter().map(|t| t.usdc_size).sum();
        let total_sell_volume: f64 = sell_trades.iter().map(|t| t.usdc_size).sum();

        println!("   📊 Trade statistics:");
        println!(
            "      • Buys: {} (volume: ${:.2})",
            buy_trades.len(),
            total_buy_volume
        );
        println!(
            "      • Sells: {} (volume: ${:.2})",
            sell_trades.len(),
            total_sell_volume
        );
        println!(
            "      • Total volume: ${:.2}\n",
            total_buy_volume + total_sell_volume
        );

        let recent_trades: Vec<&Activity> = activities.iter().take(20).collect();
        println!("   📝 Last 20 trades:\n");

        for (idx, trade) in recent_trades.iter().enumerate() {
            let date = chrono::DateTime::from_timestamp(trade.timestamp, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S");
            let side_icon = if trade.side == "BUY" { "🟢" } else { "🔴" };
            println!(
                "   {}. {} {} - {}",
                idx + 1,
                side_icon,
                trade.side,
                date
            );
            println!("      {}", trade.title.as_deref().unwrap_or("Unknown Market"));
            if let Some(ref outcome) = trade.outcome {
                println!("      {}", outcome);
            }
            println!(
                "      Volume: ${:.2} @ ${:.3}",
                trade.usdc_size, trade.price
            );
            let tx_hash = &trade.transaction_hash;
            println!(
                "      TX: {}...{}",
                &tx_hash[..tx_hash.len().min(10)],
                &tx_hash[tx_hash.len().saturating_sub(8)..]
            );
            println!("      🔗 https://polygonscan.com/tx/{}", tx_hash);
            println!();
        }
    } else {
        println!("   ❌ Trade history not found\n");
    }

    Logger::separator();
    println!();
    println!("❓ WHY NO P&L CHARTS ON POLYMARKET?\n");
    println!("   Profit/Loss charts on Polymarket only show REALIZED");
    println!("   profit (closed positions). This is why it shows $0.00:\n");

    if !positions.is_empty() {
        let total_realized_pnl: f64 = positions.iter().map(|p| p.realized_pnl).sum();
        let total_unrealized_pnl: f64 = positions.iter().map(|p| p.cash_pnl).sum();

        println!("   ✅ Realized P&L (closed positions):");
        println!("      → ${:.2} ← THIS is displayed on the chart\n", total_realized_pnl);

        println!("   📊 Unrealized P&L (open positions):");
        println!(
            "      → ${:.2} ← THIS is NOT displayed on the chart\n",
            total_unrealized_pnl
        );

        if total_realized_pnl == 0.0 {
            println!("   💡 Solution: To see charts, you need to:");
            println!("      1. Close several positions with profit");
            println!("      2. Wait 5-10 minutes for Polymarket API to update");
            println!("      3. P&L chart will start displaying data\n");
        }
    }

    Logger::separator();
    println!();
    println!("✅ Check completed!\n");
    println!("📱 Your profile: https://polymarket.com/profile/{}\n", proxy_wallet);

    Ok(())
}
