// Solana JSON-RPC (read-only). Execution is SIMULATED — generated clones never
// move real funds. Implement real signing yourself only for your own token.
use crate::config::Config;
use anyhow::Result;
use serde_json::{json, Value};

async fn rpc(cfg: &Config, http: &reqwest::Client, method: &str, params: Value) -> Result<Value> {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let res: Value = http
        .post(&cfg.solana_rpc_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res.get("result").cloned().unwrap_or(Value::Null))
}

pub async fn sol_balance(cfg: &Config, http: &reqwest::Client, address: &str) -> Result<f64> {
    let r = rpc(cfg, http, "getBalance", json!([address])).await?;
    let lamports = r.get("value").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok(lamports as f64 / 1e9)
}

pub async fn token_supply(cfg: &Config, http: &reqwest::Client, mint: &str) -> Result<Value> {
    let r = rpc(cfg, http, "getTokenSupply", json!([mint])).await?;
    Ok(r.get("value").cloned().unwrap_or(Value::Null))
}

/// Simulated execution result so frontend flows complete without touching chain.
pub fn simulate_execution(kind: &str, params: Value) -> Value {
    json!({
        "simulated": true,
        "kind": kind,
        "params": params,
        "signature": format!("SIMULATED_{:x}", rand_suffix()),
        "note": "Execution is simulated in generated clones."
    })
}

fn rand_suffix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
}
