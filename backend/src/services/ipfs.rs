// IPFS / metadata. Read via gateway; pin via Pinata only if PINATA_JWT set.
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

pub fn gateway_url(cfg: &Config, cid_or_uri: &str) -> String {
    let cid = cid_or_uri.strip_prefix("ipfs://").unwrap_or(cid_or_uri);
    if cid.starts_with("http") {
        cid.to_string()
    } else {
        format!("{}{}", cfg.ipfs_gateway, cid)
    }
}

pub async fn fetch_metadata(cfg: &Config, http: &reqwest::Client, cid_or_uri: &str) -> Result<Value> {
    let url = gateway_url(cfg, cid_or_uri);
    super::http::get_json(http, &url).await
}

pub async fn pin_json(cfg: &Config, http: &reqwest::Client, obj: &Value) -> Result<Value> {
    if cfg.pinata_jwt.is_empty() {
        anyhow::bail!("PINATA_JWT not set — pinning disabled");
    }
    let res: Value = http
        .post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
        .bearer_auth(&cfg.pinata_jwt)
        .json(obj)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}
