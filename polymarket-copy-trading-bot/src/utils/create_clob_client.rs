use anyhow::Result;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer as _;
use polymarket_client_sdk::clob::Client as ClobClient;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::POLYGON;
use std::str::FromStr;
use crate::config::EnvConfig;
use crate::utils::{is_contract_address, Logger};

// Init CLOB client & signer - handles both EOA & Gnosis Safe wallets
pub async fn create_clob_client(config: &EnvConfig) -> Result<(ClobClient<Authenticated<Normal>>, PrivateKeySigner)> {
    let chain_id = POLYGON;
    let host = &config.clob_http_url;
    
    // Parse priv key & set chain ID (Polygon mainnet)
    let signer = PrivateKeySigner::from_str(&format!("0x{}", config.private_key))
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?
        .with_chain_id(Some(chain_id));
    
    // Check if wallet is a contract (Gnosis Safe) vs EOA
    let is_proxy_safe = is_contract_address(&config.rpc_url, &config.proxy_wallet).await?;
    
    // Default to EOA, switch to Gnosis if detected
    let mut wallet_type = "EOA (Externally Owned Account)";
    if is_proxy_safe {
        wallet_type = "Gnosis Safe";
    }
    
    Logger::info(&format!("Wallet type detected: {}", wallet_type));
    
    // Auth with CLOB API using appropriate sig type
    let clob_client = ClobClient::new(host, Default::default())?;
    Ok((clob_client, signer))
}

