use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

fn is_network_error(error: &reqwest::Error) -> bool {
    error.is_timeout()
        || error.is_connect()
        || error.is_request()
        || error.status().is_none()
}

pub async fn fetch_data(
    client: &Client,
    url: &str,
    timeout_ms: u64,
    retry_limit: u32,
) -> Result<serde_json::Value> {
    let retry_delay_ms = 1000u64;
    
    for attempt in 1..=retry_limit {
        match client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(Duration::from_millis(timeout_ms))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                return Ok(resp.json().await?);
            }
            Ok(resp) => {
                if attempt == retry_limit {
                    return Err(anyhow::anyhow!("HTTP {} after {} attempts", resp.status(), retry_limit));
                }
            }
            Err(e) => {
                let is_last_attempt = attempt == retry_limit;
                
                if is_network_error(&e) && !is_last_attempt {
                    let delay_ms = retry_delay_ms * 2u64.pow(attempt - 1);
                    eprintln!(
                        "⚠️  Network error (attempt {}/{}), retrying in {:.1}s...",
                        attempt,
                        retry_limit,
                        delay_ms as f64 / 1000.0
                    );
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    continue;
                }
                
                if is_last_attempt && is_network_error(&e) {
                    eprintln!(
                        "❌ Network timeout after {} attempts - {}",
                        retry_limit,
                        e
                    );
                }
                return Err(e.into());
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed after {} attempts", retry_limit))
}
