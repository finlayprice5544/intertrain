use crate::config::Config;
use std::sync::Arc;

/// Shared application state. Cheap to clone (Arc / client handles).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub rocks: Arc<rocksdb::DB>,
    /// None when ClickHouse is unreachable — routes fall back to RocksDB seed.
    pub clickhouse: Option<clickhouse::Client>,
    pub http: reqwest::Client,
    /// Whether the cloned frontend uses SPA (missing path -> index.html).
    pub spa: bool,
}
