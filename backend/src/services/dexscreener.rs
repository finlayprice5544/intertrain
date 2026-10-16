// DexScreener — token pairs/price/liquidity. Public API, no key.
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

pub async fn token_pairs(cfg: &Config, http: &reqwest::Client, token: &str) -> Result<Value> {
    let url = format!("{}/latest/dex/tokens/{}", cfg.dexscreener_base, token);
    super::http::get_json(http, &url).await
}

/// Best USD price for a token (0.0 if unknown).
pub async fn price_usd(cfg: &Config, http: &reqwest::Client, token: &str) -> f64 {
    match token_pairs(cfg, http, token).await {
        Ok(v) => v
            .get("pairs")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("priceUsd"))
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}
