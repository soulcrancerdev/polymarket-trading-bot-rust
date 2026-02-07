use dotenv::dotenv;
use std::env;

// Config struct for env vars (FYI: all optional fields can be None if not set)
#[derive(Debug, Clone)]
pub struct Env {
    pub clob_http_url: String, // CLOB HTTP API endpoint
    pub clob_ws_url: String, // WebSocket endpoint for orderbook updates
    pub private_key: Option<String>, // Wallet private key (required for trading)
    pub usdc_contract_address: Option<String>, // USDC contract addr on Polygon
    pub proxy_wallet: Option<String>, // Proxy wallet (Gnosis Safe or EOA)
    pub rpc_url: String, // Polygon RPC endpoint
    pub arbitrage_amount_usdc: f64, // USDC amount per token side
    pub token_amount: f64, // Fixed token qty to buy
    pub arbitrage_threshold: f64, // Threshold for arb detection (usually 1.0)
}

impl Env {
    // Load env vars from .env file (AFAIK: falls back to defaults if missing)
    pub fn load() -> Self {
        dotenv().ok(); // Load .env, ignore errors if file doesn't exist

        Self {
            clob_http_url: env::var("CLOB_HTTP_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
            clob_ws_url: env::var("CLOB_WS_URL")
                .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string()),
            private_key: env::var("PRIVATE_KEY").ok(),
            usdc_contract_address: env::var("USDC_CONTRACT_ADDRESS").ok(),
            proxy_wallet: env::var("PROXY_WALLET").ok(),
            rpc_url: env::var("RPC_URL")
                .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),
            arbitrage_amount_usdc: env::var("ARBITRAGE_AMOUNT_USDC")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()
                .unwrap_or(1.0),
            token_amount: env::var("TOKEN_AMOUNT")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .unwrap_or(5.0),
            arbitrage_threshold: env::var("ARBITRAGE_THRESHOLD")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()
                .unwrap_or(1.0),
        }
    }
}

