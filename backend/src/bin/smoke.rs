// `cargo run --bin smoke` — comprehensive smoke test against a RUNNING server.
// Tests all endpoints: GET routes, POST RPC with all 21 methods, and static pages.
// Exits non-zero on failures. Start the server first: cargo run --bin generated-site

use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8787".into());
    let base = format!("http://localhost:{port}");
    let http = reqwest::Client::new();

    let mut pass = 0u32;
    let mut fail = 0u32;
    println!("=== Smoke test -> {base} ===\n");

    // ── Helper closures ────────────────────────────────────────────

    // GET request, expect < 500.
    let get = |url: String, label: String| {
        let http = http.clone();
        async move {
            match http.get(&url).send().await {
                Ok(r) if r.status().as_u16() < 500 => {
                    println!("  ok  GET {label} -> {}", r.status());
                    true
                }
                Ok(r) => {
                    println!("  ERR GET {label} -> {}", r.status());
                    false
                }
                Err(e) => {
                    println!("  ERR GET {label} -> {e}");
                    false
                }
            }
        }
    };

    // POST JSON-RPC request.
    let rpc_call = |path: &str, method: &str, params: Value| {
        let http = http.clone();
        let url = format!("{base}{path}");
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        let label = format!("{path} [{method}]");
        async move {
            match http
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
            {
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();
                    let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

                    // A 200 with either a "result" or "error" key is a valid JSON-RPC response.
                    let has_result = parsed.get("result").is_some();
                    let has_error = parsed.get("error").is_some();

                    if status.as_u16() < 500 && (has_result || has_error) {
                        let summary = if has_result {
                            let r = &parsed["result"];
                            let s = serde_json::to_string(r).unwrap_or_default();
                            if s.len() > 80 { format!("{}...", &s[..77]) } else { s }
                        } else {
                            format!("error: {}", parsed["error"])
                        };
                        println!("  ok  POST {label} -> {status} | {summary}");
                        true
                    } else {
                        let short = if text.len() > 120 { format!("{}...", &text[..117]) } else { text };
                        println!("  ERR POST {label} -> {status} | {short}");
                        false
                    }
                }
                Err(e) => {
                    println!("  ERR POST {label} -> {e}");
                    false
                }
            }
        }
    };

    // ── 1. GET endpoints ────────────────────────────────────────────
    println!("── GET endpoints ──");

    let get_checks = vec![
        ("/healthz", "healthz"),
        ("/metrics", "metrics"),
        ("/api/_health", "api/_health"),
        ("/api/_branding", "api/_branding"),
        ("/api-indexer/health", "api-indexer/health"),
        ("/api-indexer/tokens", "api-indexer/tokens"),
        ("/", "root page"),
        ("/wallet/", "wallet page"),
    ];

    for (path, label) in &get_checks {
        let url = format!("{base}{path}");
        if get(url, label.to_string()).await {
            pass += 1;
        } else {
            fail += 1;
        }
    }

    // ── 2. POST /api/rpc — all 21 RPC methods ──────────────────────
    println!("\n── POST /api/rpc (devnet) — 21 RPC methods ──");

    let rpc_tests: Vec<(&str, Value)> = vec![
        // Stateless / seed methods
        ("chain_info", json!({})),
        ("validator_set", json!({})),
        ("bridge_status", json!({})),
        ("token_list", json!({})),
        ("asset_list", json!({})),
        ("mna_reserve_status", json!({})),

        // Parameterized reads
        ("account_get", json!({ "address": "wsk1testaddress123456789012345678901234" })),
        ("token_evm_address", json!({ "hash": "abc123def456" })),

        // Stateful writes
        ("transaction_broadcast", json!({
            "transaction": {
                "from": "wsk1sender",
                "to": "wsk1receiver",
                "amount": "1000000"
            }
        })),
        ("transaction_status", json!({ "hash": "tx_placeholder" })),

        // Token operations
        ("token_operation_prepare", json!({
            "unsigned": {
                "type": "Transfer",
                "token_id": "test_token_1",
                "from": "wsk1sender",
                "to": "wsk1receiver",
                "amount": "500"
            }
        })),
        ("token_operation_broadcast", json!({
            "operation": {
                "token_id": "test_token_1",
                "signed_bytes": "deadbeef"
            }
        })),
        ("token_operation_status", json!({ "hash": "tokop_placeholder" })),

        // Auth
        ("auth_challenge", json!({
            "address": "wsk1testaddress123456789012345678901234",
            "domain": "localhost"
        })),
        ("auth_verify", json!({
            "address": "wsk1testaddress123456789012345678901234",
            "nonce": "placeholder_nonce",
            "public_key": "aabbccdd",
            "signature": "deadbeef"
        })),

        // MNA swap (USDC peg)
        ("mna_quote", json!({ "amount_usdc": "1000000" })),
        ("mna_swap_prepare", json!({
            "unsigned": { "kind": "MintMna", "amount": "1000000" }
        })),
        ("mna_swap_broadcast", json!({
            "operation": { "kind": "MintMna", "signed_bytes": "deadbeef" }
        })),
        ("mna_swap_status", json!({ "hash": "swap_placeholder" })),

        // SOL bridge quote
        ("sol_mna_quote", json!({ "amount_lamports": "1000000000" })),

        // Devnet faucet
        ("devnet_faucet", json!({ "address": "wsk1testaddress123456789012345678901234" })),
    ];

    for (method, params) in &rpc_tests {
        if rpc_call("/api/rpc", method, params.clone()).await {
            pass += 1;
        } else {
            fail += 1;
        }
    }

    // ── 3. POST /api-mainnet/rpc — verify mainnet endpoint works ────
    println!("\n── POST /api-mainnet/rpc (mainnet) — sample methods ──");

    let mainnet_tests: Vec<(&str, Value)> = vec![
        ("chain_info", json!({})),
        ("account_get", json!({ "address": "wsk1mainnettest" })),
        ("mna_reserve_status", json!({})),
    ];

    for (method, params) in &mainnet_tests {
        if rpc_call("/api-mainnet/rpc", method, params.clone()).await {
            pass += 1;
        } else {
            fail += 1;
        }
    }

    // ── Summary ────────────────────────────────────────────────────
    println!("\n=== {pass} passed, {fail} failed ===");
    if fail > 0 {
        std::process::exit(1);
    }
}
