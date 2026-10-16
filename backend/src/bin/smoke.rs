// `cargo run --bin smoke` — hit /api/_health, /, and every endpoint in
// config/endpoints.json against a RUNNING server. Exits non-zero on failures.
// Start the server first (another terminal):  cargo run
use serde_json::Value;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8787".into());
    let base = format!("http://localhost:{port}");
    let http = reqwest::Client::new();

    let manifest: Value = std::fs::read_to_string("config/endpoints.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let endpoints = manifest.get("endpoints").and_then(|e| e.as_array()).cloned().unwrap_or_default();

    let mut pass = 0;
    let mut fail = 0;
    println!("smoke test -> {base}");

    let mut checks: Vec<(String, String)> =
        vec![("GET".into(), "/api/_health".into()), ("GET".into(), "/".into())];
    for e in &endpoints {
        let method = e.get("method").and_then(|m| m.as_str()).unwrap_or("GET").to_uppercase();
        if let Some(path) = e.get("path").and_then(|p| p.as_str()) {
            checks.push((method, path.to_string()));
        }
    }

    for (method, path) in checks {
        let url = format!("{base}{path}");
        let req = match method.as_str() {
            "POST" => http.post(&url),
            "PUT" => http.put(&url),
            "PATCH" => http.patch(&url),
            "DELETE" => http.delete(&url),
            _ => http.get(&url),
        };
        match req.send().await {
            Ok(r) if r.status().as_u16() < 500 => {
                println!("  ok  {method} {path} -> {}", r.status());
                pass += 1;
            }
            Ok(r) => {
                println!("  ERR {method} {path} -> {}", r.status());
                fail += 1;
            }
            Err(e) => {
                println!("  ERR {method} {path} -> {e}");
                fail += 1;
            }
        }
    }

    println!("\n{pass} passed, {fail} failed");
    if fail > 0 {
        std::process::exit(1);
    }
}
