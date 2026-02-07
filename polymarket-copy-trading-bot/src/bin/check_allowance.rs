const POLYMARKET_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const POLYMARKET_COLLATERAL: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const NATIVE_USDC_ADDRESS: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

use anyhow::Result;
use polymarket_copy_rust::{
    utils::{get_erc20_allowance, get_erc20_balance, theme::colors},
    EnvConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = EnvConfig::from_env().await?;

    println!();
    println!(
        "{} Checking USDC balance and allowance...{}",
        colors::ACCENT,
        colors::RESET
    );
    println!();

    let configured_usdc = config.usdc_contract_address.trim().to_lowercase();
    let polymarket_collateral_lower = POLYMARKET_COLLATERAL.trim().to_lowercase();
    let native_usdc_lower = NATIVE_USDC_ADDRESS.trim().to_lowercase();
    let uses_polymarket_collateral = configured_usdc == polymarket_collateral_lower;

    let (local_balance, local_decimals) = get_erc20_balance(
        &config.rpc_url,
        &config.usdc_contract_address,
        &config.proxy_wallet,
    )
    .await?;
    let (local_allowance, _) = get_erc20_allowance(
        &config.rpc_url,
        &config.usdc_contract_address,
        &config.proxy_wallet,
        POLYMARKET_EXCHANGE,
    )
    .await?;

    println!("  USDC Decimals: {}", local_decimals);
    println!();
    println!(
        "  Your USDC Balance ({}): {} {:.6} USDC {}",
        config.usdc_contract_address,
        colors::SUCCESS,
        local_balance,
        colors::RESET
    );
    println!(
        "  Current Allowance ({}): {} {:.6} USDC {}",
        config.usdc_contract_address,
        colors::SUCCESS,
        local_allowance,
        colors::RESET
    );
    println!("  Polymarket Exchange: {}", POLYMARKET_EXCHANGE);
    println!();

    if configured_usdc != native_usdc_lower {
        match get_erc20_balance(&config.rpc_url, NATIVE_USDC_ADDRESS, &config.proxy_wallet).await {
            Ok((native_balance, _)) if native_balance > 0.0 => {
                println!(
                    "{} Detected native USDC (Polygon PoS) balance:{}",
                    colors::ACCENT,
                    colors::RESET
                );
                println!(
                    "    {:.6} tokens at {}",
                    native_balance, NATIVE_USDC_ADDRESS
                );
                println!("    Polymarket does not recognize this token. Swap to USDC.e (0x2791...) to trade.");
                println!();
            }
            _ => {}
        }
    }

    let (polymarket_balance, polymarket_allowance, _polymarket_decimals) =
        if !uses_polymarket_collateral {
            let (balance, decimals) =
                get_erc20_balance(&config.rpc_url, POLYMARKET_COLLATERAL, &config.proxy_wallet)
                    .await?;
            let (allowance, _) = get_erc20_allowance(
                &config.rpc_url,
                POLYMARKET_COLLATERAL,
                &config.proxy_wallet,
                POLYMARKET_EXCHANGE,
            )
            .await?;
            (balance, allowance, decimals)
        } else {
            (local_balance, local_allowance, local_decimals)
        };

    if !uses_polymarket_collateral {
        println!(
            "{} Polymarket collateral token is USDC.e (bridged) at address{}",
            colors::WARN,
            colors::RESET
        );
        println!("    {}", POLYMARKET_COLLATERAL);
        println!(
            "{} Polymarket-tracked USDC balance: {:.6} USDC{}",
            colors::WARN,
            polymarket_balance,
            colors::RESET
        );
        println!(
            "{} Polymarket-tracked allowance: {:.6} USDC{}",
            colors::WARN,
            polymarket_allowance,
            colors::RESET
        );
        println!();
        println!("{} Swap native USDC to USDC.e or update your .env to point at the collateral token before trading.{}", colors::WARN, colors::RESET);
        println!();
    }

    if polymarket_allowance.is_infinite()
        || (polymarket_allowance >= polymarket_balance && polymarket_allowance > 0.0)
    {
        println!(
            "{} Allowance is already sufficient! No action needed.{}",
            colors::SUCCESS,
            colors::RESET
        );
        println!();
    } else {
        println!(
            "{} Allowance is insufficient or zero!{}",
            colors::WARN,
            colors::RESET
        );
        println!("  Use the Makefile command to set allowance: make set-token-allowance");
        println!();
    }

    Ok(())
}
