use anyhow::Result;
use polymarket_copy_rust::{fetch_data, EnvConfig};
use serde::Deserialize;

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
    market: Option<String>,
    slug: Option<String>,
    outcome: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = EnvConfig::from_env().await?;
    let wallet = &config.proxy_wallet;

    let client = reqwest::Client::new();
    let url = format!(
        "https://data-api.polymarket.com/activity?user={}&type=TRADE",
        wallet
    );
    let activities: Vec<Activity> = fetch_data(
        &client,
        &url,
        config.request_timeout_ms,
        config.network_retry_limit,
    )
    .await?
    .as_array()
    .ok_or_else(|| anyhow::anyhow!("Expected array of activities"))?
    .iter()
    .filter_map(|v| serde_json::from_value(v.clone()).ok())
    .collect();

    if activities.is_empty() {
        println!("No trade data available");
        return Ok(());
    }

    let redemption_end_time = chrono::DateTime::parse_from_rfc3339("2025-10-31T18:14:16Z")
        .unwrap()
        .timestamp();

    println!("═══════════════════════════════════════════════════════════════");
    println!("📋 CLOSED POSITIONS (Redeemed October 31, 2025 at 18:00-18:14)");
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("💰 TOTAL RECEIVED FROM REDEMPTION: $66.37 USDC\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("🛒 PURCHASES AFTER REDEMPTION (after 18:14 UTC October 31)");
    println!("═══════════════════════════════════════════════════════════════\n");

    let trades_after_redemption: Vec<&Activity> = activities
        .iter()
        .filter(|t| t.timestamp > redemption_end_time && t.side == "BUY")
        .collect();

    if trades_after_redemption.is_empty() {
        println!("✅ No purchases after redemption!\n");
        println!("This means funds should be in the balance.");
        return Ok(());
    }

    let mut total_spent = 0.0;

    for (i, trade) in trades_after_redemption.iter().enumerate() {
        let date = chrono::DateTime::from_timestamp(trade.timestamp, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");
        let value = trade.usdc_size;
        total_spent += value;

        println!("{}. 🟢 BOUGHT: {}", 
            i + 1,
            trade.title.as_deref()
                .or_else(|| trade.market.as_deref())
                .unwrap_or("Unknown")
        );
        println!("   💸 Spent: ${:.2}", value);
        println!("   📊 Size: {:.2} tokens @ ${:.4}", trade.size, trade.price);
        println!("   📅 Date: {}", date);
        let tx_hash = &trade.transaction_hash;
        println!(
            "   🔗 TX: https://polygonscan.com/tx/{}...\n",
            &tx_hash[..tx_hash.len().min(20)]
        );
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("📊 TOTAL PURCHASES AFTER REDEMPTION:");
    println!("   Number of trades: {}", trades_after_redemption.len());
    println!("   💸 SPENT: ${:.2} USDC", total_spent);
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("💡 EXPLANATION OF WHERE THE MONEY WENT:\n");
    println!("   ✅ Received from redemption: +$66.37");
    println!("   ❌ Spent on new purchases: -${:.2}", total_spent);
    println!("   📊 Balance change: ${:.2}", 66.37 - total_spent);
    println!("\n═══════════════════════════════════════════════════════════════\n");

    println!("💵 RECENT SALES:\n");
    let recent_sells: Vec<&Activity> = activities
        .iter()
        .filter(|t| t.side == "SELL")
        .take(10)
        .collect();

    let mut total_sold = 0.0;
    for (i, trade) in recent_sells.iter().enumerate() {
        let date = chrono::DateTime::from_timestamp(trade.timestamp, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S");
        let value = trade.usdc_size;
        total_sold += value;

        println!("{}. 🔴 SOLD: {}",
            i + 1,
            trade.title.as_deref()
                .or_else(|| trade.market.as_deref())
                .unwrap_or("Unknown")
        );
        println!("   💰 Received: ${:.2}", value);
        println!("   📅 Date: {}\n", date);
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("💵 Sold in recent trades: ${:.2}", total_sold);
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}
