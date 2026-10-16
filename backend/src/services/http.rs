use anyhow::Result;
use serde_json::Value;
use std::time::Duration;

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("generated-site/0.1")
        .build()
        .expect("reqwest client")
}

pub async fn get_json(http: &reqwest::Client, url: &str) -> Result<Value> {
    Ok(http.get(url).send().await?.error_for_status()?.json().await?)
}
