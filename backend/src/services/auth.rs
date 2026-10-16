// Simulated auth challenge / verify flow for wallet ownership proof.
// No real cryptographic verification — always succeeds with synthetic tokens.

use crate::db::rocks;
use rocksdb::DB;
use serde_json::{json, Value};

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn sim_hex(len: usize) -> String {
    let ts = now_ms();
    let mut out = String::with_capacity(len * 2);
    for i in 0..len {
        out.push_str(&format!("{:02x}", ((ts.wrapping_add(i as u64 * 7)) & 0xff) as u8));
    }
    out
}

/// Generate a one-time challenge for wallet ownership proof.
/// Stores the challenge in RocksDB so auth_verify can look it up.
pub fn auth_challenge(db: &DB, params: &Value) -> Value {
    let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("wsk1unknown");
    let domain = params.get("domain").and_then(|v| v.as_str()).unwrap_or("localhost");
    let nonce = sim_hex(16);

    let message = format!(
        "Sign this message to prove you own {address}.\n\nDomain: {domain}\nNonce: {nonce}\nIssued: {ts}",
        ts = now_ms()
    );

    let challenge = json!({
        "address": address,
        "domain": domain,
        "nonce": nonce,
        "message": message,
        "expires_at": now_ms() + 300_000  // 5 minutes
    });

    // Persist so verify can find it
    rocks::put_json(db, &format!("auth_challenge:{nonce}"), &challenge).ok();

    json!({
        "message": message,
        "nonce": nonce
    })
}

/// Verify a signed auth challenge. In simulation mode this always succeeds
/// (no real Ed25519 verification) and returns a session token.
pub fn auth_verify(db: &DB, params: &Value) -> Value {
    let nonce = params.get("nonce").and_then(|v| v.as_str()).unwrap_or("");
    let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("wsk1unknown");

    // Look up the challenge (optional — we succeed regardless for simulation)
    let _challenge = rocks::get_json(db, &format!("auth_challenge:{nonce}"));

    // Generate a session token
    let session_token = format!("sess_{}", sim_hex(32));

    // Store the session
    let session = json!({
        "address": address,
        "session_token": session_token,
        "created_at": now_ms(),
        "expires_at": now_ms() + 86_400_000  // 24 hours
    });
    rocks::put_json(db, &format!("session:{session_token}"), &session).ok();

    // Clean up the used challenge
    let _ = db.delete(format!("auth_challenge:{nonce}").as_bytes());

    json!({
        "session_token": session_token
    })
}

/// Dispatch an auth RPC method. Returns None if not an auth method.
pub fn dispatch(db: &DB, method: &str, params: &Value) -> Option<Value> {
    Some(match method {
        "auth_challenge" => auth_challenge(db, params),
        "auth_verify"    => auth_verify(db, params),
        _ => return None,
    })
}
