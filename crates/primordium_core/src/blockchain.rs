use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[async_trait]
pub trait BlockchainProvider {
    async fn anchor_hash(&self, hash: &str) -> Result<String>;
}

pub struct OpenTimestampsProvider;

#[async_trait]
impl BlockchainProvider for OpenTimestampsProvider {
    async fn anchor_hash(&self, hash: &str) -> Result<String> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 1000;
        const REQUEST_TIMEOUT_SECS: u64 = 30;

        let client = reqwest::Client::new();
        if hash.is_empty() {
            return Err(anyhow::anyhow!("Hash cannot be empty"));
        }

        let digest_bytes = hex::decode(hash)?;
        let url = "https://alice.btc.calendar.opentimestamps.org/digest";

        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..MAX_RETRIES {
            let response = client
                .post(url)
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .body(digest_bytes.clone())
                .header("Content-Type", "application/octet-stream")
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(format!("OTS_SUCCESS_{}", &hash[..8]));
                    }

                    let status = resp.status();
                    let is_transient = status.is_server_error() || status == 429;

                    if !is_transient || attempt == MAX_RETRIES - 1 {
                        return Err(anyhow::anyhow!("OTS server returned error: {}", status));
                    }

                    last_error = Some(anyhow::anyhow!("OTS server returned error: {}", status));
                }
                Err(e) => {
                    let is_timeout = e.is_timeout() || e.is_connect();
                    if !is_timeout || attempt == MAX_RETRIES - 1 {
                        return Err(anyhow::anyhow!("OTS request failed: {}", e));
                    }

                    last_error = Some(anyhow::anyhow!("OTS request failed: {}", e));
                }
            }

            if attempt < MAX_RETRIES - 1 {
                let backoff_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown OTS error")))
    }
}

pub struct PlaceholderProvider {
    pub network: String,
}

#[async_trait]
impl BlockchainProvider for PlaceholderProvider {
    async fn anchor_hash(&self, _hash: &str) -> Result<String> {
        // Simulate a transaction on Polygon/Base
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(format!("0x{}_tx_{}", self.network.to_lowercase(), ts))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnchorRecord {
    pub hash: String,
    pub tx_id: String,
    pub timestamp: String,
    pub provider: String,
}
