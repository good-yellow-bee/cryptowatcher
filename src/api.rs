use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerData {
    #[allow(dead_code)]
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_f64")]
    pub last_price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub price_change_percent: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub high_price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub low_price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub volume: f64,
}

fn deserialize_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub struct BinanceClient {
    client: reqwest::Client,
    base_url: String,
}

impl BinanceClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            base_url: "https://api.binance.com".to_string(),
        }
    }

    pub async fn get_ticker_24h(&self, symbol: &str) -> Result<TickerData> {
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.base_url, symbol);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("API error for {}: {}", symbol, resp.status()));
        }
        let data: TickerData = resp.json().await?;
        Ok(data)
    }

    pub async fn get_tickers(&self, symbols: &[String]) -> Vec<Result<TickerData>> {
        let mut results = Vec::new();
        for symbol in symbols {
            results.push(self.get_ticker_24h(symbol).await);
        }
        results
    }

    /// Fetch historical 15-minute klines (last `limit` periods)
    /// Returns Vec of (timestamp_ms, close_price)
    pub async fn get_klines(&self, symbol: &str, limit: u32) -> Result<Vec<(i64, f64)>> {
        let url = format!(
            "{}/api/v3/klines?symbol={}&interval=15m&limit={}",
            self.base_url, symbol, limit
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("API error for {}: {}", symbol, resp.status()));
        }
        let data: Vec<Vec<serde_json::Value>> = resp.json().await?;

        // Extract (open_time, close_price) from each kline
        let prices: Vec<(i64, f64)> = data
            .iter()
            .filter_map(|kline| {
                let ts = kline.get(0).and_then(|v| v.as_i64())?;
                let price = kline
                    .get(4)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())?;
                Some((ts, price))
            })
            .collect();

        Ok(prices)
    }
}

impl Default for BinanceClient {
    fn default() -> Self {
        Self::new()
    }
}
