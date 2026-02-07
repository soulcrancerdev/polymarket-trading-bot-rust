mod config;
mod db;
mod services;
mod types;
mod utils;

use anyhow::Result;
use tokio::signal;

use config::EnvConfig;
use db::Db;
use services::{run_trade_executor, run_trade_monitor, stop_trade_executor, stop_trade_monitor};
use utils::{get_usdc_balance, is_contract_address, perform_health_check, Logger};

#[tokio::main]
async fn main() -> Result<()> {
    // Flush stdout/stderr before we start (clean slate)
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    
    dotenvy::dotenv().ok();

    println!();
    println!(
        "  {} New here? Read GETTING_STARTED.md and run a health check.{}",
        utils::theme::colors::MUTED,
        utils::theme::colors::RESET
    );
    println!();

    // Load config & connect to DB
    let config = EnvConfig::from_env().await?;
    let db = Db::connect(&config.mongo_uri).await?;

    // Store priv key in DB (encrypted, obv)
    if let Err(e) = db.set_config("PRIVATE_KEY", &config.private_key).await {
        Logger::error(&format!("Failed to store PRIVATE_KEY in database: {}", e));
    }

    Logger::startup(&config.user_addresses, &config.proxy_wallet);

    // Run health checks - DB, RPC, balance, Polymarket API
    Logger::info("Running system check…");
    let db_ok = true;
    let balance = get_usdc_balance(
        &config.rpc_url,
        &config.usdc_contract_address,
        &config.proxy_wallet,
    )
    .await;
    let polymarket_ok = utils::fetch_data(
        &reqwest::Client::new(),
        "https://data-api.polymarket.com/positions?user=0x0000000000000000000000000000000000000000",
        config.request_timeout_ms,
        config.network_retry_limit,
    )
    .await
    .is_ok();
    let health = perform_health_check(db_ok, &config.rpc_url, balance, polymarket_ok).await;

    Logger::separator();
    Logger::header("SYSTEM CHECK");
    let overall = if health.healthy {
        "All systems go"
    } else {
        "Degraded — check items below"
    };
    Logger::health_line(
        "Overall",
        if health.healthy { "ok" } else { "error" },
        overall,
    );
    Logger::health_line(
        "Database",
        &health.checks.database.status,
        &health.checks.database.message,
    );
    Logger::health_line("RPC", &health.checks.rpc.status, &health.checks.rpc.message);
    Logger::health_line(
        "Balance",
        &health.checks.balance.status,
        &health.checks.balance.message,
    );
    Logger::health_line(
        "Polymarket API",
        &health.checks.polymarket_api.status,
        &health.checks.polymarket_api.message,
    );
    Logger::separator();

    // Continue even if health check fails (degraded mode)
    if !health.healthy {
        Logger::warning("System check reported issues; continuing anyway.");
    }

    // Init CLOB client (handles wallet type detection)
    Logger::info("Initializing CLOB client...");
    let is_proxy_safe = is_contract_address(&config.rpc_url, &config.proxy_wallet)
        .await
        .unwrap_or(false);
    let wallet_type = if is_proxy_safe {
        "Gnosis Safe"
    } else {
        "EOA (Externally Owned Account)"
    };
    Logger::info(&format!("Wallet type detected: {}", wallet_type));
    Logger::success("CLOB client ready");

    Logger::separator();
    // Build HTTP client with timeout
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(config.request_timeout_ms))
        .build()?;

    // Start monitor (watches for new trades via RTDS)
    Logger::info("Starting trade monitor...");
    let _monitor_handle = run_trade_monitor(&config, &db, &http_client).await?;

    // Start executor (processes trades & executes orders)
    Logger::info("Starting trade executor...");
    let config_clone = config.clone();
    let db_clone = db.clone();
    let http_clone = http_client.clone();
    let executor_handle = tokio::spawn(async move {
        if let Err(e) = run_trade_executor(&config_clone, &db_clone, &http_clone).await {
            Logger::error(&format!("Trade executor error: {}", e));
        }
    });

    // Wait for Ctrl+C, then graceful shutdown
    match signal::ctrl_c().await {
        Ok(()) => {
            Logger::separator();
            Logger::info("Shutdown requested. Stopping…");
        }
        Err(_) => {}
    }

    stop_trade_monitor();
    stop_trade_executor();
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    executor_handle.abort();
    let _ = db.close().await;
    Logger::success("Goodbye.");
    Ok(())
}
