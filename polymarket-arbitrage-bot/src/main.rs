mod config;
mod services;
mod utils;

use crate::config::Env;
use crate::services::market_discovery::{find_15_min_market, CoinMarket};
use crate::services::price_monitor::{create_price_data, display_coin_details, PriceData, PriceMonitor};
use crate::services::websocket_client::MarketWebSocket;
use crate::utils::coin_selector::{display_coin_selection, get_available_coins};
use crate::utils::keyboard::{KeyboardHandler, KeyAction};
use crate::utils::logger::{clear_log_files, init_monitor_log, log_error};
use colored::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Main entry point (FYI: uses Tokio async runtime)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::load();
    
    // Print fancy banner (IMO: looks pro)
    println!("{}", "\n╔════════════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║     Polymarket Arbitrage Bot - 15-Minute Market Monitor       ║".cyan().bold());
    println!("{}", "╚════════════════════════════════════════════════════════════════╝\n".cyan().bold());

    // Clear logs on startup (BTW: keeps things clean)
    clear_log_files();
    init_monitor_log();
    println!("{}", "Log files cleared (monitor.log, error.log)\n".bright_black());

    // Step 1: User picks a coin via interactive menu (FYI: arrow keys + Enter)
    let selected_coin = select_coin().await?;
    
    println!(
        "{}",
        format!(
            "\n✓ Coin selected: {}\n  Bot will automatically switch to next market when current market closes.\n  Press Ctrl+C to stop.\n\n",
            selected_coin
        )
        .green()
        .bold()
    );

    // Step 2: Start continuous monitoring loop
    monitor_market_loop(&selected_coin, &env).await?;

    Ok(())
}

// Interactive coin selection menu (AFAIK: uses crossterm for key handling)
async fn select_coin() -> anyhow::Result<String> {
    let coins = get_available_coins();
    let mut selected_index = 0;
    let mut keyboard = KeyboardHandler::new();
    keyboard.enable()?; // Enable raw mode for arrow key detection

    loop {
        display_coin_selection(selected_index); // Render menu with current selection

        match keyboard.read_key()? {
            KeyAction::Up => {
                // Wrap around to bottom if at top (FYI: circular navigation)
                selected_index = (selected_index + coins.len() - 1) % coins.len();
            }
            KeyAction::Down => {
                // Wrap around to top if at bottom
                selected_index = (selected_index + 1) % coins.len();
            }
            KeyAction::Enter => {
                keyboard.disable()?; // Clean up before returning
                return Ok(coins[selected_index].to_string());
            }
            KeyAction::Exit => {
                keyboard.disable()?;
                std::process::exit(0); // Ctrl+C exit
            }
            _ => {} // Ignore other keys
        }
    }
}

// Main monitoring loop (FYI: auto-switches to next market when current closes)
async fn monitor_market_loop(coin: &str, env: &Env) -> anyhow::Result<()> {
    let mut ws: Option<Arc<MarketWebSocket>> = None; // WS connection (lazy init)
    let clob_client = Arc::new(Mutex::new(None::<Arc<services::create_clob_client::ClobClient>>)); // Trading client (lazy init)
    let monitor = Arc::new(Mutex::new(PriceMonitor::new())); // Price history tracker
    let recent_opportunities = Arc::new(Mutex::new(HashSet::new())); // Dedup tracker (prevents duplicate trades)
    let is_executing_trade = Arc::new(Mutex::new(false)); // Trade lock (prevents concurrent executions)

    loop {
        match discover_and_monitor(coin, &mut ws, &clob_client, &monitor, &recent_opportunities, &is_executing_trade, env).await {
            Ok(Some(market)) => {
                // Monitor until market closes (BTW: auto-finds next market after)
                while let Some(ref m) = market {
                    let end_date = chrono::DateTime::parse_from_rfc3339(&m.end_date)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc);
                    let now = chrono::Utc::now();
                    let time_until_end = (end_date - now).num_milliseconds();

                    if time_until_end <= 0 {
                        println!(
                            "{}",
                            format!(
                                "\n\n╔════════════════════════════════════════════════════════════════╗\n║                    MARKET CLOSED                                 ║\n╚════════════════════════════════════════════════════════════════╝\n  Market: {}\n  Coin: {}\n  End Time: {}\n  Status: Searching for next market...\n\n",
                                m.slug, coin, end_date.format("%Y-%m-%d %H:%M:%S UTC")
                            )
                            .yellow()
                            .bold()
                        );
                        break;
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            }
            Ok(None) => {
                println!("{}", "Waiting 10 seconds before retrying...\n".yellow());
                sleep(Duration::from_secs(10)).await;
            }
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

async fn discover_and_monitor(
    coin: &str,
    ws: &mut Option<Arc<MarketWebSocket>>,
    clob_client: &Arc<Mutex<Option<Arc<services::create_clob_client::ClobClient>>>>,
    monitor: &Arc<Mutex<PriceMonitor>>,
    recent_opportunities: &Arc<Mutex<HashSet<String>>>,
    is_executing_trade: &Arc<Mutex<bool>>,
    env: &Env,
) -> anyhow::Result<Option<Arc<CoinMarket>>> {
    println!("{}", format!("\n🔍 Discovering market for {}...\n", coin).cyan());

    // Initialize ClobClient if needed (FYI: lazy init, only creates once)
    {
        let mut client_guard = clob_client.lock().await;
        if client_guard.is_none() {
            println!("{}", "Initializing ClobClient for trading...\n".bright_black());
            match services::create_clob_client::create_clob_client(env).await {
                Ok(client) => {
                    *client_guard = Some(Arc::new(client));
                    println!("{}", "✓ ClobClient initialized\n".green());
                }
                Err(e) => {
                    // NGL: trading disabled but detection still works
                    println!("{}", format!("⚠️  Warning: Failed to initialize ClobClient: {}\n", e).yellow());
                    println!("{}", "Arbitrage detection will work, but automatic trading is disabled.\n".yellow());
                }
            }
        }
    }

    // Discover active 15-min market (AFAIK: checks current/next/prev windows)
    let market = match find_15_min_market(coin).await? {
        Some(m) => Arc::new(m),
        None => {
            println!("{}", format!("⚠️  No active market found for {}. Will retry in 10 seconds...\n", coin).yellow());
            return Ok(None);
        }
    };

    println!("{}", format!("✓ Market found: {}\n", market.slug).green());

    // Initialize WebSocket if needed (FYI: runs in background task with auto-reconnect)
    if ws.is_none() {
        println!("{}", "Initializing WebSocket connection...\n".bright_black());
        let ws_client = Arc::new(MarketWebSocket::new(env.clob_ws_url.clone()));
        
        // Start WebSocket in background (BTW: auto-reconnects on disconnect)
        let ws_clone = ws_client.clone();
        tokio::spawn(async move {
            if let Err(e) = ws_clone.run(true).await {
                eprintln!("WebSocket error: {}", e);
            }
        });

        sleep(Duration::from_secs(1)).await; // Give WS time to connect
        *ws = Some(ws_client);
    }

    let ws_ref = ws.as_ref().unwrap();

    // Set up orderbook callback (IMO: this is where the magic happens)
    let monitor_clone = monitor.clone();
    let clob_client_clone = clob_client.clone();
    let recent_opps_clone = recent_opportunities.clone();
    let is_executing_clone = is_executing_trade.clone();
    let market_clone = market.clone();
    let coin_str = coin.to_string();
    let env_clone = env.clone();
    let ws_ref_clone = ws_ref.clone();

    ws_ref.on_book(Arc::new(move |snapshot| {
        let market = market_clone.clone();
        let coin = coin_str.clone();
        let monitor = monitor_clone.clone();
        let clob_client = clob_client_clone.clone();
        let recent_opps = recent_opps_clone.clone();
        let is_executing = is_executing_clone.clone();
        let env = env_clone.clone();
        let ws_ref = ws_ref_clone.clone();

        tokio::spawn(async move {
            // Check if market has closed (FYI: stops trading if closed)
            let end_date = chrono::DateTime::parse_from_rfc3339(&market.end_date)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            let now = chrono::Utc::now();
            let time_until_end = (end_date - now).num_milliseconds();

            if time_until_end <= 0 {
                // Market closed, stop trading (BTW: only show message once)
                if !*is_executing.lock().await {
                    let mins = (-time_until_end) / 60000;
                    println!(
                        "{}",
                        format!(
                            "\n⏰ MARKET CLOSED - {}\n   Market ended {} minute(s) ago\n   Trading has been stopped. Press Ctrl+C to exit.\n",
                            coin, mins
                        )
                        .red()
                        .bold()
                    );
                    *is_executing.lock().await = true; // Lock to prevent new trades
                }
                return;
            }

            // Determine if this is UP or DOWN token (AFAIK: we need both for arb calc)
            let is_up_token = snapshot.asset_id == market.up_token_id;
            let is_down_token = snapshot.asset_id == market.down_token_id;

            if !is_up_token && !is_down_token {
                return; // Not our tokens, ignore
            }

            // Get both orderbooks (FYI: need both UP and DOWN for arbitrage calc)
            let up_snapshot = ws_ref.get_orderbook(&market.up_token_id);
            let down_snapshot = ws_ref.get_orderbook(&market.down_token_id);

            if let (Some(up_snap), Some(down_snap)) = (up_snapshot, down_snapshot) {
                let price_data = create_price_data(&coin, Some(&up_snap), Some(&down_snap), &env);

                // Warn if market is closing soon
                if time_until_end > 0 && time_until_end < 60000 {
                    let secs = time_until_end / 1000;
                    println!(
                        "{}",
                        format!(
                            "\n⚠️  MARKET CLOSING SOON - {}\n   {} seconds remaining. Last chance to trade!\n",
                            coin, secs
                        )
                        .yellow()
                        .bold()
                    );
                }

                // Arbitrage detection (IMO: this is the core logic)
                if price_data.ask_sum < env.arbitrage_threshold {
                    let mut monitor_guard = monitor.lock().await;
                    monitor_guard.record_arbitrage(&coin, &price_data); // Log detection

                    let spread = (env.arbitrage_threshold - price_data.ask_sum) * 100.0;
                    let timestamp = chrono::Utc::now().format("%H:%M:%S EST");
                    println!(
                        "{}",
                        format!(
                            "\n⚡ [{}] ARBITRAGE DETECTED - {}\n   UP_ASK: {:.4} + DOWN_ASK: {:.4} = {:.4}\n   Spread: {:.2}%\n",
                            timestamp, coin, price_data.up_ask, price_data.down_ask, price_data.ask_sum, spread
                        )
                        .green()
                        .bold()
                    );

                    // Create opportunity key for dedup (FYI: prevents duplicate trades on same prices)
                    let opportunity_key = format!("{:.4}_{:.4}", price_data.up_ask, price_data.down_ask);
                    let is_market_open = time_until_end > 5000; // Need at least 5s remaining

                    let client_guard = clob_client.lock().await;
                    if let Some(ref client) = *client_guard {
                        drop(client_guard); // Release lock before async ops (BTW: prevents deadlock)
                        
                        let mut is_exec = is_executing.lock().await;
                        let mut opps = recent_opps.lock().await;
                        
                        // Check if we can trade (FYI: must not be executing, not duplicate, market open)
                        if !*is_exec && !opps.contains(&opportunity_key) && is_market_open {
                            *is_exec = true; // Lock to prevent concurrent trades
                            opps.insert(opportunity_key.clone());
                            
                            // Keep only last 50 opps in memory (AFAIK: prevents memory bloat)
                            if opps.len() > 50 {
                                let first_key = opps.iter().next().cloned();
                                if let Some(key) = first_key {
                                    opps.remove(&key);
                                }
                            }

                            drop(is_exec);
                            drop(opps);

                            // Execute trade in background (IMO: don't block price updates)
                            let client_clone = client.clone();
                            let market_clone = market.clone();
                            let is_exec_clone = is_executing.clone();
                            
                            tokio::spawn(async move {
                                let _ = services::arbitrage_executor::execute_arbitrage_trade(
                                    &client_clone,
                                    &market_clone.up_token_id,
                                    &market_clone.down_token_id,
                                    price_data.up_ask,
                                    price_data.down_ask,
                                    price_data.up_bid,
                                    price_data.down_bid,
                                    &env,
                                ).await;
                                
                                *is_exec_clone.lock().await = false; // Release lock when done
                            });
                        } else {
                            *is_exec = false; // Release lock if we can't trade
                        }
                    }
                }

                let mut monitor_guard = monitor.lock().await;
                monitor_guard.add_to_history(&coin, price_data.clone(), &env);
                
                // Display updated view
                display_coin_details(&coin, &price_data, &market, &monitor_guard, &env);
            }
        }));
    }));

    // Subscribe to both tokens
    ws_ref.subscribe(vec![market.up_token_id.clone(), market.down_token_id.clone()]).await?;

    sleep(Duration::from_secs(2)).await;

    Ok(Some(market))
}

