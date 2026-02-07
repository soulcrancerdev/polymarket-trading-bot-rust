use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub checks: HealthChecks,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct HealthChecks {
    pub database: CheckResult,
    pub rpc: CheckResult,
    pub balance: BalanceCheckResult,
    pub polymarket_api: CheckResult,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceCheckResult {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<f64>,
}

pub async fn perform_health_check(
    db_ok: bool,
    rpc_url: &str,
    balance: Result<f64, anyhow::Error>,
    polymarket_ok: bool,
) -> HealthCheckResult {
    let db_status = if db_ok { "ok" } else { "error" };
    let db_msg = if db_ok { "Connected" } else { "Not connected" };

    let (rpc_status, rpc_msg) = match check_rpc(rpc_url).await {
        Ok(()) => ("ok".to_string(), "RPC endpoint responding".to_string()),
        Err(e) => ("error".to_string(), format!("RPC check failed: {}", e)),
    };

    let (balance_status, balance_msg, balance_val) = match balance {
        Ok(b) if b > 0.0 => {
            if b < 10.0 {
                ("warning", format!("Low balance: ${:.2}", b), Some(b))
            } else {
                ("ok", format!("Balance: ${:.2}", b), Some(b))
            }
        }
        Ok(_) => ("error", "Zero balance".into(), None),
        Err(e) => ("error", format!("Balance check failed: {}", e), None),
    };

    let pm_status = if polymarket_ok { "ok" } else { "error" };
    let pm_msg = if polymarket_ok {
        "API responding"
    } else {
        "API check failed"
    };

    let healthy =
        db_status == "ok" && rpc_status == "ok" && balance_status != "error" && pm_status == "ok";

    HealthCheckResult {
        healthy,
        checks: HealthChecks {
            database: CheckResult {
                status: db_status.to_string(),
                message: db_msg.to_string(),
            },
            rpc: CheckResult {
                status: rpc_status,
                message: rpc_msg,
            },
            balance: BalanceCheckResult {
                status: balance_status.to_string(),
                message: balance_msg,
                balance: balance_val,
            },
            polymarket_api: CheckResult {
                status: pm_status.to_string(),
                message: pm_msg.to_string(),
            },
        },
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

async fn check_rpc(rpc_url: &str) -> Result<()> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(rpc_url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    if json.get("result").is_some() {
        Ok(())
    } else {
        anyhow::bail!("Invalid RPC response")
    }
}
