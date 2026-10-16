mod branding;
mod config;
mod db;
mod routes;
mod services;
mod state;
mod static_files;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Arc::new(config::Config::load()?);

    // Embedded KV (always available)
    let rocks = db::rocks::open(&config)?;
    let seeded = db::seed::load_into_rocks(&rocks);
    tracing::info!("rocksdb at {} ({seeded} seed collection(s))", config.rocksdb_path);

    // Analytical store (optional)
    let clickhouse = db::clickhouse::connect(&config).await;

    let http = services::http::client();

    // Detect SPA strategy from the cloned site.json, if present.
    let spa = std::fs::read_to_string(format!("{}/site.json", config.public_dir))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("strategy").and_then(|x| x.as_str()).map(|s| s == "spa"))
        .unwrap_or(false);

    let st = state::AppState {
        config: config.clone(),
        rocks,
        clickhouse,
        http,
        spa,
    };

    let app = routes::mount_all(Router::new())
        .fallback(static_files::fallback)
        .layer(TraceLayer::new_for_http())
        .with_state(st);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(
        "▶ {} on http://localhost:{}  (data mode {:?})",
        config.branding.site_name,
        config.port,
        config.data_mode
    );
    axum::serve(listener, app).await?;
    Ok(())
}
