mod copy_strategy;

pub use copy_strategy::{
    calculate_order_size, get_trade_multiplier, parse_tiered_multipliers, CopyStrategy,
    CopyStrategyConfig,
};

use anyhow::{Context, Result};
use std::env;

pub fn is_valid_ethereum_address(addr: &str) -> bool {
    let s = addr.trim().trim_start_matches("0x");
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn parse_user_addresses(input: &str) -> Result<Vec<String>> {
    let trimmed = input.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let parsed: Vec<String> =
            serde_json::from_str(trimmed).context("Invalid JSON format for USER_ADDRESSES")?;
        let addresses: Vec<String> = parsed
            .into_iter()
            .map(|a| a.to_lowercase().trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();
        for addr in &addresses {
            if !is_valid_ethereum_address(addr) {
                anyhow::bail!("Invalid Ethereum address in USER_ADDRESSES: {}", addr);
            }
        }
        return Ok(addresses);
    }
    let addresses: Vec<String> = trimmed
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    for addr in &addresses {
        if !is_valid_ethereum_address(addr) {
            anyhow::bail!("Invalid Ethereum address in USER_ADDRESSES: {}", addr);
        }
    }
    Ok(addresses)
}

fn parse_copy_strategy_from_env() -> Result<CopyStrategyConfig> {
    let has_legacy = env::var("COPY_PERCENTAGE").is_ok() && env::var("COPY_STRATEGY").is_err();
    if has_legacy {
        let copy_pct: f64 = env::var("COPY_PERCENTAGE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10.0);
        let trade_mult: f64 = env::var("TRADE_MULTIPLIER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);
        let effective = copy_pct * trade_mult;
        let mut config = CopyStrategyConfig {
            strategy: CopyStrategy::Percentage,
            copy_size: effective,
            max_order_size_usd: env::var("MAX_ORDER_SIZE_USD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
            min_order_size_usd: env::var("MIN_ORDER_SIZE_USD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            max_position_size_usd: env::var("MAX_POSITION_SIZE_USD")
                .ok()
                .and_then(|v| v.parse().ok()),
            max_daily_volume_usd: env::var("MAX_DAILY_VOLUME_USD")
                .ok()
                .and_then(|v| v.parse().ok()),
            adaptive_min_percent: None,
            adaptive_max_percent: None,
            adaptive_threshold: None,
            tiered_multipliers: None,
            trade_multiplier: if (trade_mult - 1.0).abs() > 1e-9 {
                Some(trade_mult)
            } else {
                None
            },
        };
        if let Ok(tiers_str) = env::var("TIERED_MULTIPLIERS") {
            config.tiered_multipliers = Some(parse_tiered_multipliers(&tiers_str)?);
        }
        return Ok(config);
    }

    let strategy_str = env::var("COPY_STRATEGY")
        .unwrap_or_else(|_| "PERCENTAGE".into())
        .to_uppercase();
    let strategy = match strategy_str.as_str() {
        "FIXED" => CopyStrategy::Fixed,
        "ADAPTIVE" => CopyStrategy::Adaptive,
        _ => CopyStrategy::Percentage,
    };

    let mut config = CopyStrategyConfig {
        strategy,
        copy_size: env::var("COPY_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10.0),
        max_order_size_usd: env::var("MAX_ORDER_SIZE_USD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100.0),
        min_order_size_usd: env::var("MIN_ORDER_SIZE_USD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        max_position_size_usd: env::var("MAX_POSITION_SIZE_USD")
            .ok()
            .and_then(|v| v.parse().ok()),
        max_daily_volume_usd: env::var("MAX_DAILY_VOLUME_USD")
            .ok()
            .and_then(|v| v.parse().ok()),
        adaptive_min_percent: None,
        adaptive_max_percent: None,
        adaptive_threshold: None,
        tiered_multipliers: None,
        trade_multiplier: env::var("TRADE_MULTIPLIER")
            .ok()
            .and_then(|v| v.parse().ok())
            .and_then(|m: f64| {
                if (m - 1.0).abs() > 1e-9 {
                    Some(m)
                } else {
                    None
                }
            }),
    };

    if let Ok(tiers_str) = env::var("TIERED_MULTIPLIERS") {
        config.tiered_multipliers = Some(parse_tiered_multipliers(&tiers_str)?);
    }
    if strategy == CopyStrategy::Adaptive {
        config.adaptive_min_percent = Some(
            env::var("ADAPTIVE_MIN_PERCENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(config.copy_size),
        );
        config.adaptive_max_percent = Some(
            env::var("ADAPTIVE_MAX_PERCENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(config.copy_size),
        );
        config.adaptive_threshold = Some(
            env::var("ADAPTIVE_THRESHOLD_USD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500.0),
        );
    }
    Ok(config)
}

#[derive(Clone)]
pub struct EnvConfig {
    pub user_addresses: Vec<String>,
    pub proxy_wallet: String,
    pub private_key: String,
    pub clob_http_url: String,
    pub clob_ws_url: String,
    pub fetch_interval_secs: u64,
    pub too_old_timestamp_hours: i64,
    pub retry_limit: u32,
    pub copy_strategy_config: CopyStrategyConfig,
    pub request_timeout_ms: u64,
    pub network_retry_limit: u32,
    pub trade_aggregation_enabled: bool,
    pub trade_aggregation_window_seconds: u64,
    pub mongo_uri: String,
    pub rpc_url: String,
    pub usdc_contract_address: String,
}

impl EnvConfig {
    pub async fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let required = [
            "USER_ADDRESSES",
            "PROXY_WALLET",
            "PRIVATE_KEY",
            "CLOB_HTTP_URL",
            "CLOB_WS_URL",
            "RPC_URL",
            "USDC_CONTRACT_ADDRESS",
        ];
        for key in &required {
            if env::var(key).unwrap_or_default().trim().is_empty() {
                anyhow::bail!(
                    "Missing required env var: {}. Run setup or create .env (see .env.example)",
                    key
                );
            }
        }

        if let Ok(ref u) = env::var("USDC_CONTRACT_ADDRESS") {
            if !is_valid_ethereum_address(u) {
                anyhow::bail!("Invalid USDC_CONTRACT_ADDRESS: {}", u);
            }
        }
        if let Ok(ref p) = env::var("PROXY_WALLET") {
            if !is_valid_ethereum_address(p) {
                anyhow::bail!("Invalid PROXY_WALLET: {}", p);
            }
        }

        let user_addresses = parse_user_addresses(&env::var("USER_ADDRESSES")?)?;
        if user_addresses.is_empty() {
            anyhow::bail!("USER_ADDRESSES must contain at least one address");
        }

        let fetch_interval_secs: u64 = env::var("FETCH_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let too_old_timestamp_hours: i64 = env::var("TOO_OLD_TIMESTAMP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(24);
        let retry_limit: u32 = env::var("RETRY_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let request_timeout_ms: u64 = env::var("REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000);
        let network_retry_limit: u32 = env::var("NETWORK_RETRY_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let trade_aggregation_enabled = env::var("TRADE_AGGREGATION_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);
        let trade_aggregation_window_seconds: u64 = env::var("TRADE_AGGREGATION_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        let private_key = env::var("PRIVATE_KEY")?
            .trim()
            .trim_start_matches("0x")
            .to_string();

        let mongo_uri = env::var("MONGO_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017/polymarket_copytrading".into());

        Ok(Self {
            user_addresses,
            proxy_wallet: env::var("PROXY_WALLET")?.trim().to_string(),
            private_key,
            clob_http_url: env::var("CLOB_HTTP_URL")?
                .trim()
                .trim_end_matches('/')
                .to_string(),
            clob_ws_url: env::var("CLOB_WS_URL")?.trim().to_string(),
            fetch_interval_secs,
            too_old_timestamp_hours,
            retry_limit,
            copy_strategy_config: parse_copy_strategy_from_env()?,
            request_timeout_ms,
            network_retry_limit,
            trade_aggregation_enabled,
            trade_aggregation_window_seconds,
            mongo_uri,
            rpc_url: env::var("RPC_URL")?.trim().to_string(),
            usdc_contract_address: env::var("USDC_CONTRACT_ADDRESS")?.trim().to_string(),
        })
    }
}
