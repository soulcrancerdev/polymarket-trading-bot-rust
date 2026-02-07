use crate::config::env::Env;

// Supported coins for 15-min markets (FYI: these are the only ones we track)
pub const AVAILABLE_COINS: &[&str] = &["BTC", "ETH", "SOL", "XRP"];

// Maps coin ticker to Polymarket slug format (AFAIK: format is {coin}-updown-15m)
pub fn coin_slug(coin: &str) -> Option<&str> {
    match coin.to_uppercase().as_str() {
        "BTC" => Some("btc-updown-15m"),
        "ETH" => Some("eth-updown-15m"),
        "SOL" => Some("sol-updown-15m"),
        "XRP" => Some("xrp-updown-15m"),
        _ => None, // Invalid coin, return None
    }
}

// WebSocket endpoint for real-time orderbook data (WRT: Polymarket's CLOB WS API)
pub const WSS_MARKET_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
// Gamma API host for market discovery (BTW: this is Polymarket's market data API)
pub const GAMMA_API_HOST: &str = "https://gamma-api.polymarket.com";

// Trading constants (IMO: these defaults work well for most cases)
pub const TOKEN_AMOUNT: f64 = 5.0; // Fixed token qty per side (UP/DOWN)
pub const MIN_ORDER_SIZE_USD: f64 = 1.0; // Min order size in USD (Polymarket requirement)
pub const RENDER_THROTTLE_MS: u64 = 10; // UI update throttle (caps at ~100fps)

pub fn get_token_amount(env: &Env) -> f64 {
    env.token_amount
}

pub fn get_arbitrage_threshold(env: &Env) -> f64 {
    env.arbitrage_threshold
}

