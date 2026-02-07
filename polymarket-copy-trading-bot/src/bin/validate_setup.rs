use anyhow::Result;
use polymarket_copy_rust::{utils::theme::colors, EnvConfig, Logger};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    match EnvConfig::from_env().await {
        Ok(config) => {
            println!(
                "{} Configuration OK: {} trader(s), vault {}",
                colors::SUCCESS,
                config.user_addresses.len(),
                Logger::format_address(&config.proxy_wallet)
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("{} Configuration error: {}", colors::ERROR, e);
            Err(e)
        }
    }
}
