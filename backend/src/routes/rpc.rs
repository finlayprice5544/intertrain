// JSON-RPC dispatch for both /api/rpc (devnet) and /api-mainnet/rpc (mainnet).
// Handles all 21 wallet RPC methods defined in endpoints.json.
//
// Strategy per method:
//   - Stateless/seed: load from seed:rpc.<method>, extract "result"
//   - Parameterized:  load seed template, merge caller-supplied params
//   - Stateful write: load seed template, generate deterministic fake hashes
//   - Auth:           generate dynamic challenge / return seed token
//   - Fallback:       check legacy seed:rpc collection (keyed by method name)
//   - Unknown:        JSON-RPC -32601

use crate::db::rocks::seed_all;
use crate::routes::RouteReg;
use crate::services;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Wrap a result value in a JSON-RPC 2.0 success envelope.
fn ok_envelope(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

/// JSON-RPC 2.0 error envelope.
fn err_envelope(id: &Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

/// Load a per-method seed file (seed:rpc.<method>) and extract its "result" field.
fn seed_result(db: &rocksdb::DB, method: &str) -> Option<Value> {
    let key = format!("rpc.{method}");
    let envelope = seed_all(db, &key);
    if envelope.is_null() {
        return None;
    }
    // The individual seed files store the full JSON-RPC envelope; extract result.
    envelope.get("result").cloned().or(Some(envelope))
}

/// Fallback: look in the flat seed:rpc collection (keyed by method name).
fn seed_rpc_legacy(db: &rocksdb::DB, method: &str) -> Option<Value> {
    let collection = seed_all(db, "rpc");
    collection.get(method).cloned()
}

/// Load seed for a method, trying per-file first then legacy collection.
fn load_seed(db: &rocksdb::DB, method: &str) -> Option<Value> {
    seed_result(db, method).or_else(|| seed_rpc_legacy(db, method))
}

/// Generate a deterministic 64-char hex "hash" from an input string.
/// Not cryptographic — just a reproducible placeholder.
fn fake_hash(input: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    for b in input.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3); // FNV prime
    }
    let h2 = h.wrapping_mul(0x517c_c1b7_2722_0a95);
    let h3 = h.wrapping_mul(0x6c62_272e_07bb_0142);
    let h4 = h2.wrapping_add(h3);
    format!("{h:016x}{h2:016x}{h3:016x}{h4:016x}")
}

/// Extract a string param from the "params" object, falling back to a default.
fn param_str<'a>(params: &'a Value, key: &str, default: &'a str) -> &'a str {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// per-method dispatch
// ---------------------------------------------------------------------------

fn dispatch(db: &rocksdb::DB, method: &str, params: &Value, id: &Value) -> Value {
    match method {
        // ── Stateless / seed ──────────────────────────────────────────
        "chain_info" | "validator_set" | "bridge_status" | "token_list"
        | "asset_list" | "mna_reserve_status" => {
            match load_seed(db, method) {
                Some(result) => ok_envelope(id, result),
                None => err_envelope(id, -32603, &format!("no seed data for {method}")),
            }
        }

        // ── account_get — echo address, merge seed balances ──────────
        "account_get" => {
            let addr = param_str(params, "address", "wsk1unknown");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("address".into(), json!(addr));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "address": addr,
                    "balance": "0",
                    "nonce": 0,
                    "assets": {}
                })),
            }
        }

        // ── transaction_broadcast — return a deterministic hash ──────
        "transaction_broadcast" => {
            let hash = fake_hash(&format!("tx_broadcast:{}", serde_json::to_string(params).unwrap_or_default()));
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("hash".into(), json!(hash));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({ "hash": hash })),
            }
        }

        // ── transaction_status — echo hash, return seed or confirmed ─
        "transaction_status" => {
            let hash = param_str(params, "hash", "");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("hash".into(), json!(hash));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "status": "confirmed",
                    "hash": hash,
                    "height": 2045380
                })),
            }
        }

        // ── token_evm_address — echo the token hash ─────────────────
        "token_evm_address" => {
            let hash = param_str(params, "hash", "");
            match load_seed(db, method) {
                Some(mut result) => {
                    if !hash.is_empty() {
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert("token_id".into(), json!(hash));
                        }
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "token_id": hash,
                    "evm_address": format!("0x{}", &fake_hash(hash)[..40]),
                    "symbol": "TKN",
                    "decimals": 6
                })),
            }
        }

        // ── token_operation_prepare — return seed signing_bytes ──────
        "token_operation_prepare" => {
            match load_seed(db, method) {
                Some(result) => ok_envelope(id, result),
                None => {
                    let hash = fake_hash(&format!("tok_op_prepare:{}", serde_json::to_string(params).unwrap_or_default()));
                    ok_envelope(id, json!({
                        "signing_bytes": "01096d696e7465727472616e2d31",
                        "hash": hash
                    }))
                }
            }
        }

        // ── token_operation_broadcast — return operation_id + token_id
        "token_operation_broadcast" => {
            let op_id = fake_hash(&format!("tok_op_broadcast:{}", serde_json::to_string(params).unwrap_or_default()));
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("operation_id".into(), json!(op_id));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "operation_id": op_id,
                    "token_id": "0000000000000000000000000000000000000000000000000000000000000000"
                })),
            }
        }

        // ── token_operation_status — echo hash, return confirmed ─────
        "token_operation_status" => {
            let hash = param_str(params, "hash", "");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("operation_id".into(), json!(hash));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "status": "confirmed",
                    "operation_id": hash,
                    "height": 2045385
                })),
            }
        }

        // ── auth_challenge — dynamic nonce, echo address+domain ──────
        "auth_challenge" => {
            let addr = param_str(params, "address", "wsk1unknown");
            let domain = param_str(params, "domain", "intertrain.online");
            let nonce = fake_hash(&format!("nonce:{}:{}:{}", addr, domain, id));
            let nonce_short = &nonce[..12];
            let message = format!(
                "Sign this message to prove you own {} on {} at 2026-10-16T12:00:00Z nonce:{}",
                addr, domain, nonce_short
            );
            ok_envelope(id, json!({
                "message": message,
                "nonce": nonce_short
            }))
        }

        // ── auth_verify — return seed session_token ──────────────────
        "auth_verify" => {
            match load_seed(db, method) {
                Some(result) => ok_envelope(id, result),
                None => ok_envelope(id, json!({
                    "session_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.placeholder"
                })),
            }
        }

        // ── mna_quote — echo amount, return seed rate ────────────────
        "mna_quote" => {
            let amount = param_str(params, "amount_usdc", "0");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("amount_usdc".into(), json!(amount));
                        // 1:1 rate means amount_mna == amount_usdc
                        obj.insert("amount_mna".into(), json!(amount));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "amount_usdc": amount,
                    "amount_mna": amount,
                    "rate": "1 USDC = 1 WSK",
                    "fee_bps": 0,
                    "direction": "MintMna"
                })),
            }
        }

        // ── sol_mna_quote — echo lamports, return seed rate ──────────
        "sol_mna_quote" => {
            let lamports = param_str(params, "amount_lamports", "0");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("amount_lamports".into(), json!(lamports));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "amount_lamports": lamports,
                    "amount_mna": "0",
                    "sol_price_usd": "145.00",
                    "fee_bps": 50,
                    "rate": "1 SOL ≈ 145 WSK"
                })),
            }
        }

        // ── mna_swap_prepare — return seed signing_bytes ─────────────
        "mna_swap_prepare" => {
            match load_seed(db, method) {
                Some(result) => ok_envelope(id, result),
                None => {
                    let hash = fake_hash(&format!("mna_swap_prepare:{}", serde_json::to_string(params).unwrap_or_default()));
                    ok_envelope(id, json!({
                        "signing_bytes": "02096d696e7465727472616e2d31",
                        "hash": hash
                    }))
                }
            }
        }

        // ── mna_swap_broadcast — return operation_id ─────────────────
        "mna_swap_broadcast" => {
            let op_id = fake_hash(&format!("mna_swap_broadcast:{}", serde_json::to_string(params).unwrap_or_default()));
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("operation_id".into(), json!(op_id));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({ "operation_id": op_id })),
            }
        }

        // ── mna_swap_status — echo hash, return confirmed ────────────
        "mna_swap_status" => {
            let hash = param_str(params, "hash", "");
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("operation_id".into(), json!(hash));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "status": "confirmed",
                    "operation_id": hash,
                    "height": 2045390
                })),
            }
        }

        // ── devnet_faucet — echo address, return fake tx hash ────────
        "devnet_faucet" => {
            let addr = param_str(params, "address", "wsk1unknown");
            let hash = fake_hash(&format!("faucet:{addr}"));
            match load_seed(db, method) {
                Some(mut result) => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("address".into(), json!(addr));
                        obj.insert("hash".into(), json!(hash));
                    }
                    ok_envelope(id, result)
                }
                None => ok_envelope(id, json!({
                    "address": addr,
                    "amount": "100000000",
                    "hash": hash,
                    "note": "100 WSK credited on devnet"
                })),
            }
        }

        // ── unknown method ───────────────────────────────────────────
        _ => err_envelope(id, -32601, "Method not found"),
    }
}

// ---------------------------------------------------------------------------
// handler (shared by both mount points)
// ---------------------------------------------------------------------------

async fn rpc_handler(State(st): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let method = body["method"].as_str().unwrap_or("");
    let id = body.get("id").cloned().unwrap_or_else(|| json!(1));
    let params = body.get("params").cloned().unwrap_or_else(|| json!({}));

    // Validate JSON-RPC envelope minimally.
    if method.is_empty() {
        return Json(err_envelope(&id, -32600, "Invalid request: missing method"));
    }

    // Try stateful service dispatchers first (they persist state in RocksDB:
    // faucet credits, transaction records, auth sessions, bridge events).
    if let Some(result) = services::chain::dispatch(&st.rocks, method, &params) {
        return Json(ok_envelope(&id, result));
    }
    if let Some(result) = services::auth::dispatch(&st.rocks, method, &params) {
        return Json(ok_envelope(&id, result));
    }
    if let Some(result) = services::bridge::dispatch(&st.rocks, method, &params) {
        return Json(ok_envelope(&id, result));
    }

    // Fall back to the seed-based dispatch for anything the services don't claim.
    Json(dispatch(&st.rocks, method, &params, &id))
}

// ---------------------------------------------------------------------------
// mount on both /api/rpc (devnet) and /api-mainnet/rpc (mainnet)
// ---------------------------------------------------------------------------

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| {
            r.route("/api/rpc", post(rpc_handler))
             .route("/api-mainnet/rpc", post(rpc_handler))
        }
    }
}
