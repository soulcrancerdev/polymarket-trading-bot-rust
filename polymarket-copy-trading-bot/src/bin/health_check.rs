use anyhow::Result;
use polymarket_copy_rust::{
    get_usdc_balance, perform_health_check, utils::theme::colors, Db, EnvConfig, Logger,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    println!();
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        colors::ACCENT
    );
    println!("     POLYMARKET BOT — HEALTH CHECK");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        colors::RESET
    );
    println!();

    let config = EnvConfig::from_env().await?;
    let db_ok = Db::connect(&config.mongo_uri).await.is_ok();
    let balance = get_usdc_balance(
        &config.rpc_url,
        &config.usdc_contract_address,
        &config.proxy_wallet,
    )
    .await;
    let polymarket_ok = polymarket_copy_rust::fetch_data(
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
        "Degraded — fix issues below"
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

    if health.healthy {
        println!();
        println!(
            "{} Ready to run: make run{}",
            colors::SUCCESS,
            colors::RESET
        );
        println!();
    } else {
        println!();
        println!(
            "{} Fix the issues above, then run make health-check again.{}",
            colors::WARN,
            colors::RESET
        );
        println!();
        std::process::exit(1);
    }

    Ok(())
}
