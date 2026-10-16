// Embedded RocksDB: KV + state + cache + seed collections.
use crate::config::Config;
use anyhow::Result;
use rocksdb::{Options, DB};
use serde_json::Value;
use std::sync::Arc;

pub fn open(cfg: &Config) -> Result<Arc<DB>> {
    std::fs::create_dir_all(&cfg.rocksdb_path).ok();
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, &cfg.rocksdb_path)?;
    Ok(Arc::new(db))
}

pub fn get_json(db: &DB, key: &str) -> Option<Value> {
    db.get(key.as_bytes())
        .ok()
        .flatten()
        .and_then(|b| serde_json::from_slice(&b).ok())
}

pub fn put_json(db: &DB, key: &str, value: &Value) -> Result<()> {
    db.put(key.as_bytes(), serde_json::to_vec(value)?)?;
    Ok(())
}

/// A seed collection, stored under `seed:<name>`. Returns JSON null if absent.
pub fn seed_all(db: &DB, name: &str) -> Value {
    get_json(db, &format!("seed:{name}")).unwrap_or(Value::Null)
}

/// Find one row in a seed array collection by a string field match.
pub fn seed_find(db: &DB, name: &str, field: &str, value: &str) -> Option<Value> {
    match seed_all(db, name) {
        Value::Array(rows) => rows.into_iter().find(|r| {
            r.get(field).and_then(|v| v.as_str()) == Some(value)
        }),
        _ => None,
    }
}
