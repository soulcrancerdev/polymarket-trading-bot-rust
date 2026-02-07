const CTF_CONTRACT: &str = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";
const POLYMARKET_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const POLYGON_CHAIN_ID: u64 = 137;

use alloy::network::TransactionBuilder;
use alloy::providers::Provider;
use alloy::signers::Signer;
use anyhow::Result;
use polymarket_copy_rust::{utils::theme::colors, EnvConfig, Logger};
use std::str::FromStr;

async fn ctf_is_approved_for_all(
    rpc_url: &str,
    ctf_contract: &str,
    account: &str,
    operator: &str,
) -> Result<bool> {
    let account_trimmed = account.trim().trim_start_matches("0x").to_lowercase();
    let operator_trimmed = operator.trim().trim_start_matches("0x").to_lowercase();
    let data = format!(
        "0xe985e9c5{:0>64}{:0>64}",
        account_trimmed, operator_trimmed
    );
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": ctf_contract, "data": data}, "latest"],
        "id": 1
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(rpc_url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    let result = json
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))?;
    let hex = result.trim_start_matches("0x");
    if hex.is_empty() {
        return Ok(false);
    }
    let value = u8::from_str_radix(&hex[hex.len().saturating_sub(2)..], 16).unwrap_or(0);
    Ok(value != 0)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = EnvConfig::from_env().await?;

    let signer =
        alloy::signers::local::PrivateKeySigner::from_str(&format!("0x{}", config.private_key))?
            .with_chain_id(Some(POLYGON_CHAIN_ID));
    let eoa = format!("0x{:x}", signer.address());

    println!();
    println!(
        "{} Setting CTF token allowance for Polymarket{}",
        colors::ACCENT,
        colors::RESET
    );
    println!();
    println!("  Wallet (EOA): {}", Logger::format_address(&eoa));
    println!("  CTF:          {}", CTF_CONTRACT);
    println!("  Exchange:     {}", POLYMARKET_EXCHANGE);
    println!();

    println!(
        "{} Checking current approval status...{}",
        colors::ACCENT,
        colors::RESET
    );
    let is_approved =
        ctf_is_approved_for_all(&config.rpc_url, CTF_CONTRACT, &eoa, POLYMARKET_EXCHANGE).await?;

    if is_approved {
        println!(
            "{} Tokens already approved. You can sell positions.{}",
            colors::SUCCESS,
            colors::RESET
        );
        println!();
        return Ok(());
    }

    println!(
        "{} Tokens not approved. Sending setApprovalForAll tx...{}",
        colors::WARN,
        colors::RESET
    );
    println!();

    let exchange_trim = POLYMARKET_EXCHANGE.trim_start_matches("0x").to_lowercase();
    let calldata_hex = format!("0xa22cb465{:0>64}{:0>64}", exchange_trim, "1");
    let calldata_str = calldata_hex.trim_start_matches("0x");
    let data_bytes: Vec<u8> = (0..calldata_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&calldata_str[i..i + 2], 16).unwrap_or(0))
        .collect();

    let to_addr = CTF_CONTRACT.parse::<alloy::primitives::Address>()?;
    let tx = alloy::rpc::types::TransactionRequest::default()
        .with_to(to_addr)
        .with_gas_limit(100_000)
        .with_input(alloy::primitives::Bytes::from(data_bytes));

    let url: url::Url = config.rpc_url.parse()?;
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(signer)
        .with_chain_id(POLYGON_CHAIN_ID)
        .connect_http(url);

    let pending = provider.send_transaction(tx).await?;
    let tx_hash = *pending.tx_hash();
    println!("  Tx sent: 0x{:x}", tx_hash);
    println!("  Waiting for confirmation...");
    let receipt = pending.get_receipt().await?;
    if receipt.status() {
        println!(
            "{} Success. Tokens approved.{}",
            colors::SUCCESS,
            colors::RESET
        );
        println!("  https://polygonscan.com/tx/0x{:x}", tx_hash);
        println!();

        println!("{} Verifying approval...{}", colors::ACCENT, colors::RESET);
        let verified =
            ctf_is_approved_for_all(&config.rpc_url, CTF_CONTRACT, &eoa, POLYMARKET_EXCHANGE)
                .await?;
        if verified {
            println!(
                "{} Verification: Approval confirmed on-chain{}",
                colors::SUCCESS,
                colors::RESET
            );
            println!(
                "{} You can now sell your positions.{}",
                colors::SUCCESS,
                colors::RESET
            );
        } else {
            println!(
                "{} Warning: Approval verification failed.{}",
                colors::WARN,
                colors::RESET
            );
        }
    } else {
        eprintln!("{} Transaction reverted.{}", colors::ERROR, colors::RESET);
        std::process::exit(1);
    }
    println!();
    Ok(())
}
