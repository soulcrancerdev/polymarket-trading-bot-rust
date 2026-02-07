use crate::config::Env;
use crate::services::market_discovery::CoinMarket;
use crate::services::websocket_client::OrderbookSnapshot;
use crate::utils::logger::{log_monitor_data, MonitorData};
use chrono::{DateTime, Utc};
use colored::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PriceData {
    pub coin: String,
    pub up_bid: f64,
    pub up_ask: f64,
    pub down_bid: f64,
    pub down_ask: f64,
    pub bid_sum: f64,
    pub ask_sum: f64,
    pub spread: f64,
    pub has_arbitrage: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct ArbitrageDetection {
    pub timestamp: i64,
    pub up_ask: f64,
    pub down_ask: f64,
    pub ask_sum: f64,
    pub spread: f64,
    pub spread_percent: f64,
}

pub fn create_price_data(
    coin: &str,
    up_snapshot: Option<&OrderbookSnapshot>,
    down_snapshot: Option<&OrderbookSnapshot>,
    env: &Env,
) -> PriceData {
    let up_bid = up_snapshot
        .and_then(|s| s.bids.first())
        .map(|b| b.price)
        .unwrap_or(0.0);
    let up_ask = up_snapshot
        .and_then(|s| s.asks.first())
        .map(|a| a.price)
        .unwrap_or(0.0);
    let down_bid = down_snapshot
        .and_then(|s| s.bids.first())
        .map(|b| b.price)
        .unwrap_or(0.0);
    let down_ask = down_snapshot
        .and_then(|s| s.asks.first())
        .map(|a| a.price)
        .unwrap_or(0.0);

    let bid_sum = up_bid + down_bid;
    let ask_sum = up_ask + down_ask;
    let spread = env.arbitrage_threshold - ask_sum;
    let has_arbitrage = ask_sum < env.arbitrage_threshold && spread > 0.0;

    PriceData {
        coin: coin.to_string(),
        up_bid,
        up_ask,
        down_bid,
        down_ask,
        bid_sum,
        ask_sum,
        spread,
        has_arbitrage,
        timestamp: Utc::now().timestamp_millis(),
    }
}

pub struct PriceMonitor {
    price_history: HashMap<String, Vec<PriceData>>,
    arbitrage_history: HashMap<String, Vec<ArbitrageDetection>>,
}

impl PriceMonitor {
    pub fn new() -> Self {
        Self {
            price_history: HashMap::new(),
            arbitrage_history: HashMap::new(),
        }
    }

    pub fn record_arbitrage(&mut self, coin: &str, price_data: &PriceData) {
        let history = self.arbitrage_history.entry(coin.to_string()).or_insert_with(Vec::new);
        history.push(ArbitrageDetection {
            timestamp: price_data.timestamp,
            up_ask: price_data.up_ask,
            down_ask: price_data.down_ask,
            ask_sum: price_data.ask_sum,
            spread: price_data.spread,
            spread_percent: price_data.spread * 100.0,
        });
        if history.len() > 10 {
            history.remove(0);
        }
    }

    pub fn get_arbitrage_history(&self, coin: &str) -> Vec<ArbitrageDetection> {
        self.arbitrage_history.get(coin).cloned().unwrap_or_default()
    }

    pub fn clear_arbitrage_history(&mut self, coin: &str) {
        self.arbitrage_history.remove(coin);
    }

    pub fn add_to_history(&mut self, coin: &str, price_data: PriceData, env: &Env) {
        let history = self.price_history.entry(coin.to_string()).or_insert_with(Vec::new);
        history.push(price_data.clone());
        if history.len() > 10 {
            history.remove(0);
        }

        // Log to monitor.log
        let time_str = format_timestamp(price_data.timestamp);
        log_monitor_data(MonitorData {
            time: time_str,
            bid_up: price_data.up_bid,
            bid_down: price_data.down_bid,
            bid_sum: price_data.bid_sum,
            ask_up: price_data.up_ask,
            ask_down: price_data.down_ask,
            ask_sum: price_data.ask_sum,
        });
    }

    pub fn get_price_history(&self, coin: &str) -> Vec<PriceData> {
        self.price_history.get(coin).cloned().unwrap_or_default()
    }
}

fn format_timestamp(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp_millis(timestamp).unwrap_or_else(|| Utc::now());
    let est_offset = chrono::Duration::hours(-5);
    let est_time = dt + est_offset;
    est_time.format("%H:%M:%S EST").to_string()
}

pub fn display_coin_details(
    coin: &str,
    price_data: &PriceData,
    market: &CoinMarket,
    monitor: &PriceMonitor,
    env: &Env,
) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen

    // Market information header
    println!("{}", format!("{} - {}", coin, market.question).cyan().bold());
    println!("{}", format!("Market: {}", market.slug).bright_black());

    // End date and countdown
    let end_date = DateTime::parse_from_rfc3339(&market.end_date)
        .unwrap_or_else(|_| Utc::now().into())
        .with_timezone(&Utc);
    let now = Utc::now();
    let time_until_end = (end_date - now).num_milliseconds();
    let mins = time_until_end / 60000;
    let secs = (time_until_end % 60000) / 1000;

    let end_date_str = end_date.format("%m/%d/%Y %I:%M:%S %p").to_string();

    if time_until_end <= 0 {
        println!("{}", format!("Ends: {} (MARKET CLOSED)", end_date_str).red().bold());
    } else if time_until_end < 60000 {
        println!(
            "{}",
            format!("Ends: {} ({}s remaining - CLOSING SOON!)", end_date_str, secs)
                .yellow()
                .bold()
        );
    } else if time_until_end < 300000 {
        println!(
            "{}",
            format!("Ends: {} ({}m {}s remaining)", end_date_str, mins, secs).yellow()
        );
    } else {
        println!(
            "{}",
            format!("Ends: {} ({}m {}s remaining)", end_date_str, mins, secs).bright_black()
        );
    }
    println!();
    println!("{}", "All price data is being logged to monitor.log".bright_black());
    println!();

    // Display recent arbitrage detections
    let arb_history = monitor.get_arbitrage_history(coin);
    if !arb_history.is_empty() {
        println!("{}", "─".repeat(100).green().bold());
        println!("{}", "⚡ Recent Arbitrage Detections:".green().bold());
        println!("{}", "─".repeat(100).bright_black());

        let recent_arb: Vec<_> = arb_history.iter().rev().take(5).collect();
        for arb in recent_arb {
            let time_str = format_timestamp(arb.timestamp);
            println!(
                "{}",
                format!(
                    "{:12} | UP_ASK={:.4} + DOWN_ASK={:.4} = {:.4} | Spread: {:.4} ({:.2}%)",
                    time_str, arb.up_ask, arb.down_ask, arb.ask_sum, arb.spread, arb.spread_percent
                )
                .green()
                .bold()
            );
        }
        println!();
    }

    println!("{}", "Press Ctrl+C to exit".yellow());
}

