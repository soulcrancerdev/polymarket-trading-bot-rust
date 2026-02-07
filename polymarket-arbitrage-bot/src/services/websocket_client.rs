use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Single price level in orderbook (FYI: price + size)
#[derive(Debug, Clone)]
pub struct OrderbookLevel {
    pub price: f64,
    pub size: f64,
}

// Full orderbook snapshot (AFAIK: contains all bids/asks for a token)
#[derive(Debug, Clone)]
pub struct OrderbookSnapshot {
    pub asset_id: String, // Token ID
    pub market: String, // Market identifier
    pub timestamp: i64, // Unix timestamp
    pub bids: Vec<OrderbookLevel>, // Buy orders (sorted desc by price)
    pub asks: Vec<OrderbookLevel>, // Sell orders (sorted asc by price)
    pub hash: Option<String>, // Optional hash for validation
}

// Callback type for orderbook updates (BTW: Arc allows sharing across threads)
pub type BookCallback = Arc<dyn Fn(OrderbookSnapshot) + Send + Sync>;

pub struct MarketWebSocket {
    url: String,
    subscribed_assets: Arc<Mutex<Vec<String>>>,
    orderbooks: Arc<Mutex<HashMap<String, OrderbookSnapshot>>>,
    on_book_callback: Arc<Mutex<Option<BookCallback>>>,
    is_running: Arc<Mutex<bool>>,
}

impl MarketWebSocket {
    pub fn new(url: String) -> Self {
        Self {
            url,
            subscribed_assets: Arc::new(Mutex::new(Vec::new())),
            orderbooks: Arc::new(Mutex::new(HashMap::new())),
            on_book_callback: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    // Register callback for orderbook updates (FYI: called whenever we get new data)
    pub fn on_book<F>(&self, callback: F)
    where
        F: Fn(OrderbookSnapshot) + Send + Sync + 'static,
    {
        *self.on_book_callback.blocking_lock() = Some(Arc::new(callback));
    }

    // Get cached orderbook for asset (AFAIK: returns latest snapshot we received)
    pub fn get_orderbook(&self, asset_id: &str) -> Option<OrderbookSnapshot> {
        self.orderbooks.blocking_lock().get(asset_id).cloned()
    }

    // Parse orderbook from JSON (IMO: handles Polymarket's WS message format)
    fn parse_orderbook_snapshot(data: &serde_json::Value) -> Result<OrderbookSnapshot> {
        let mut bids: Vec<OrderbookLevel> = data
            .get("bids")
            .and_then(|v| v.as_array())
            .unwrap_or(&[])
            .iter()
            .filter_map(|b| {
                Some(OrderbookLevel {
                    price: b.get("price")?.as_str()?.parse().ok()?,
                    size: b.get("size")?.as_str()?.parse().ok()?,
                })
            })
            .collect();
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()); // Sort bids desc (best bid first)

        let mut asks: Vec<OrderbookLevel> = data
            .get("asks")
            .and_then(|v| v.as_array())
            .unwrap_or(&[])
            .iter()
            .filter_map(|a| {
                Some(OrderbookLevel {
                    price: a.get("price")?.as_str()?.parse().ok()?,
                    size: a.get("size")?.as_str()?.parse().ok()?,
                })
            })
            .collect();
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()); // Sort asks asc (best ask first)

        Ok(OrderbookSnapshot {
            asset_id: data
                .get("asset_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            market: data
                .get("market")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            timestamp: data
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            bids,
            asks,
            hash: data.get("hash").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }

    // Handle incoming WS message (FYI: can be single msg or array of msgs)
    async fn handle_message(&self, message: &str) -> Result<()> {
        let data: serde_json::Value = serde_json::from_str(message)?;

        // Handle both single msg and array formats (AFAIK: Polymarket sends both)
        let messages = if data.is_array() {
            data.as_array().unwrap().clone()
        } else {
            vec![data]
        };

        for msg in messages {
            let event_type = msg
                .get("event_type")
                .or_else(|| msg.get("type")) // Try both field names (BTW: API inconsistency)
                .and_then(|v| v.as_str());

            if event_type == Some("book") {
                let snapshot = Self::parse_orderbook_snapshot(&msg)?;
                let asset_id = snapshot.asset_id.clone();
                
                // Cache orderbook (IMO: allows quick lookups without WS roundtrip)
                {
                    let mut orderbooks = self.orderbooks.blocking_lock();
                    orderbooks.insert(asset_id.clone(), snapshot.clone());
                }

                // Call registered callback (FYI: triggers arbitrage detection)
                let callback_guard = self.on_book_callback.blocking_lock();
                if let Some(ref callback) = *callback_guard {
                    callback(snapshot.clone());
                }
            }
        }

        Ok(())
    }

    // Subscribe to asset orderbooks (FYI: stores IDs, actual sub happens in run loop)
    pub async fn subscribe(&self, asset_ids: Vec<String>) -> Result<()> {
        if asset_ids.is_empty() {
            return Err(anyhow!("No asset IDs provided"));
        }

        {
            let mut subscribed = self.subscribed_assets.blocking_lock();
            *subscribed = asset_ids.clone(); // Store for later subscription
        }

        // Subscription will be handled in the run loop (BTW: after WS connects)
        Ok(())
    }

    // Main WS loop with auto-reconnect (IMO: keeps connection alive)
    pub async fn run(&self, auto_reconnect: bool) -> Result<()> {
        *self.is_running.blocking_lock() = true;

        loop {
            if !*self.is_running.blocking_lock() {
                break; // Stop requested
            }

            // Connect to WS endpoint
            match self.connect().await {
                Ok((mut ws_stream, _)) => {
                    // Subscribe to assets (AFAIK: sends sub msg after connection)
                    {
                        let subscribed = self.subscribed_assets.blocking_lock();
                        if !subscribed.is_empty() {
                            let subscribe_msg = json!({
                                "assets_ids": subscribed.clone(),
                                "type": "MARKET"
                            });
                            let _ = ws_stream.send(Message::Text(subscribe_msg.to_string())).await;
                        }
                    }

                    // Handle incoming messages (FYI: processes orderbook updates)
                    while *self.is_running.blocking_lock() {
                        match ws_stream.next().await {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = self.handle_message(&text).await {
                                    eprintln!("Error handling message: {}", e);
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                let _ = ws_stream.send(Message::Pong(data)).await;
                            }
                            Some(Ok(Message::Close(_))) => {
                                break;
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }

            if !auto_reconnect || !*self.is_running.blocking_lock() {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    async fn connect(&self) -> Result<(tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::handshake::client::Response)> {
        let result = connect_async(&self.url).await?;
        Ok(result)
    }

    pub fn stop(&self) {
        *self.is_running.blocking_lock() = false;
    }
}

