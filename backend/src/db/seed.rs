// Load seed/*.json (one file = one collection) into RocksDB under `seed:<name>`.
use rocksdb::DB;
use std::path::Path;

pub fn load_into_rocks(db: &DB) -> usize {
    let dir = Path::new("seed");
    if !dir.exists() {
        return 0;
    }
    let mut n = 0;
    let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            tracing::warn!("seed {stem}: invalid JSON, skipped");
            continue;
        };
        if crate::db::rocks::put_json(db, &format!("seed:{stem}"), &value).is_ok() {
            n += 1;
        }
    }
    n
}
