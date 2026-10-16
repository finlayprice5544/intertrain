// ClickHouse: analytical / event / time-series data.
use crate::config::{ClickhouseCfg, Config};
use anyhow::Result;
use clickhouse::Client;
use serde_json::Value;

/// Build a client and ping it. Returns None if unreachable (app still boots).
pub async fn connect(cfg: &Config) -> Option<Client> {
    let client = Client::default()
        .with_url(&cfg.clickhouse.url)
        .with_database(&cfg.clickhouse.database)
        .with_user(&cfg.clickhouse.user)
        .with_password(&cfg.clickhouse.password);

    match client.query("SELECT 1").execute().await {
        Ok(_) => {
            tracing::info!("clickhouse connected at {}", cfg.clickhouse.url);
            Some(client)
        }
        Err(e) => {
            tracing::warn!("clickhouse unavailable ({e}); analytics routes fall back to seed");
            None
        }
    }
}

/// Generic query returning JSON rows (via ClickHouse's JSONEachRow over HTTP).
/// Lets generated routes run arbitrary SQL without defining Row structs.
pub async fn query_json(
    cfg: &ClickhouseCfg,
    http: &reqwest::Client,
    sql: &str,
) -> Result<Vec<Value>> {
    let url = format!("{}/?database={}", cfg.url.trim_end_matches('/'), cfg.database);
    let body = format!("{sql}\nFORMAT JSONEachRow");
    let mut req = http.post(&url).body(body);
    if !cfg.user.is_empty() {
        req = req.header("X-ClickHouse-User", &cfg.user);
    }
    if !cfg.password.is_empty() {
        req = req.header("X-ClickHouse-Key", &cfg.password);
    }
    let text = req.send().await?.error_for_status()?.text().await?;
    let rows = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect();
    Ok(rows)
}
