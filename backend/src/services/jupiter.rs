// Jupiter — Solana swap price & quote. Lite endpoints need no key.
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

pub async fn price(cfg: &Config, http: &reqwest::Client, ids: &str) -> Result<Value> {
    let url = format!("{}/price/v2?ids={}", cfg.jupiter_base, ids);
    super::http::get_json(http, &url).await
}

pub async fn quote(
    cfg: &Config,
    http: &reqwest::Client,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    slippage_bps: u32,
) -> Result<Value> {
    let url = format!(
        "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        cfg.jupiter_base, input_mint, output_mint, amount, slippage_bps
    );
    super::http::get_json(http, &url).await
}
