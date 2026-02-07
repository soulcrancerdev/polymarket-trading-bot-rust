mod create_clob_client;
mod fetch;
mod health;
mod logger;
mod post_order;
mod spinner;
pub mod theme;

pub use create_clob_client::create_clob_client;
pub use fetch::fetch_data;
pub use health::perform_health_check;
pub use logger::{Logger, TradeDetails};
pub use post_order::post_order;
pub use spinner::Spinner;

pub async fn is_contract_address(rpc_url: &str, address: &str) -> anyhow::Result<bool> {
    let addr_trimmed = address.trim().trim_start_matches("0x");
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getCode",
        "params": [format!("0x{}", addr_trimmed), "latest"],
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
    let code = result.trim_start_matches("0x");
    Ok(!code.is_empty() && code.chars().any(|c| c != '0'))
}

async fn get_erc20_decimals(rpc_url: &str, contract: &str) -> anyhow::Result<u8> {
    let data = "0x313ce567";
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": contract, "data": data}, "latest"],
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
        return Ok(6);
    }
    let value = u8::from_str_radix(&hex[hex.len().saturating_sub(2)..], 16).unwrap_or(6);
    Ok(value)
}

pub async fn get_erc20_balance(
    rpc_url: &str,
    contract: &str,
    address: &str,
) -> anyhow::Result<(f64, u8)> {
    let addr_trimmed = address.trim().trim_start_matches("0x").to_lowercase();
    let addr_padded = format!("{:0>64}", addr_trimmed);
    let data = format!("0x70a08231{}", addr_padded);
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": contract, "data": data}, "latest"],
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
        let decimals = get_erc20_decimals(rpc_url, contract).await.unwrap_or(6);
        return Ok((0.0, decimals));
    }

    let decimals = get_erc20_decimals(rpc_url, contract).await.unwrap_or(6);
    let value_str = hex.trim_start_matches('0');
    if value_str.is_empty() {
        return Ok((0.0, decimals));
    }

    let trimmed_hex = hex.trim_start_matches('0');
    let is_max = trimmed_hex.len() == 64 && trimmed_hex.chars().all(|c| c == 'f' || c == 'F');
    if is_max {
        return Ok((f64::INFINITY, decimals));
    }

    let value = if hex.len() <= 32 {
        u128::from_str_radix(hex, 16).unwrap_or(0) as f64
    } else {
        let last_32 = &hex[hex.len().saturating_sub(32)..];
        let base = u128::from_str_radix(last_32, 16).unwrap_or(0) as f64;
        let higher_order_hex = &hex[..hex.len().saturating_sub(32)];
        if !higher_order_hex.is_empty() {
            let multiplier = 16_f64.powi(higher_order_hex.len() as i32);
            base + (u128::from_str_radix(higher_order_hex, 16).unwrap_or(0) as f64 * multiplier)
        } else {
            base
        }
    };

    let divisor = 10_f64.powi(decimals as i32);
    Ok((value / divisor, decimals))
}

pub async fn get_erc20_allowance(
    rpc_url: &str,
    contract: &str,
    owner: &str,
    spender: &str,
) -> anyhow::Result<(f64, u8)> {
    let o = owner.trim().trim_start_matches("0x").to_lowercase();
    let s = spender.trim().trim_start_matches("0x").to_lowercase();
    let data = format!("0xdd62ed3e{:0>64}{:0>64}", o, s);
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": contract, "data": data}, "latest"],
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
        let decimals = get_erc20_decimals(rpc_url, contract).await.unwrap_or(6);
        return Ok((0.0, decimals));
    }

    let decimals = get_erc20_decimals(rpc_url, contract).await.unwrap_or(6);
    let value_str = hex.trim_start_matches('0');
    if value_str.is_empty() {
        return Ok((0.0, decimals));
    }

    let trimmed_hex = hex.trim_start_matches('0');
    let is_max = trimmed_hex.len() == 64 && trimmed_hex.chars().all(|c| c == 'f' || c == 'F');
    if is_max {
        return Ok((f64::INFINITY, decimals));
    }

    let value = if hex.len() <= 32 {
        u128::from_str_radix(hex, 16).unwrap_or(0) as f64
    } else {
        let last_32 = &hex[hex.len().saturating_sub(32)..];
        let base = u128::from_str_radix(last_32, 16).unwrap_or(0) as f64;
        let higher_order_hex = &hex[..hex.len().saturating_sub(32)];
        if !higher_order_hex.is_empty() {
            let multiplier = 16_f64.powi(higher_order_hex.len() as i32);
            base + (u128::from_str_radix(higher_order_hex, 16).unwrap_or(0) as f64 * multiplier)
        } else {
            base
        }
    };

    let divisor = 10_f64.powi(decimals as i32);
    Ok((value / divisor, decimals))
}

pub async fn get_usdc_balance(
    rpc_url: &str,
    usdc_contract: &str,
    address: &str,
) -> anyhow::Result<f64> {
    let (balance, _) = get_erc20_balance(rpc_url, usdc_contract, address).await?;
    Ok(balance)
}

pub async fn get_usdc_allowance(
    rpc_url: &str,
    usdc_contract: &str,
    owner: &str,
    spender: &str,
) -> anyhow::Result<f64> {
    let (allowance, _) = get_erc20_allowance(rpc_url, usdc_contract, owner, spender).await?;
    Ok(allowance)
}
