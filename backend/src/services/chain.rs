// Simulated chain state for the Intertrain wallet's custom L1.
// All data is deterministic / synthetic — no real chain interaction.
// Covers: chain_info, validator_set, account_get, asset_list, token_list,
//         token_evm_address, transaction_broadcast, transaction_status,
//         token_operation_prepare, token_operation_broadcast, token_operation_status,
//         mna_reserve_status, mna_quote, mna_swap_prepare, mna_swap_broadcast,
//         mna_swap_status, sol_mna_quote, devnet_faucet.

use crate::db::rocks;
use rocksdb::DB;
use serde_json::{json, Value};

// ── helpers ───────────────────────────────────────────────────────
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn sim_hash(prefix: &str) -> String {
    format!("{prefix}_{:016x}", now_ms())
}

fn sim_bytes(len: usize) -> String {
    // deterministic hex bytes for signing_bytes / session tokens
    let ts = now_ms();
    let mut out = String::with_capacity(len * 2);
    for i in 0..len {
        out.push_str(&format!("{:02x}", ((ts.wrapping_add(i as u64)) & 0xff) as u8));
    }
    out
}

// ── chain_info ────────────────────────────────────────────────────
pub fn chain_info(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "chain_info") {
        return seed;
    }
    json!({
        "chain_id": "intertrain-1",
        "latest_height": 1_200_000 + (now_ms() / 6000) % 10_000,
        "latest_time": chrono_like_now(),
        "native_asset": {
            "id": "worldstreet:MNA:native",
            "symbol": "WSK",
            "name": "WorldStreet Kash",
            "decimals": 6
        },
        "evm_chain_id": 4683,
        "validator_count": 21,
        "epoch": 600
    })
}

// ── validator_set ─────────────────────────────────────────────────
pub fn validator_set(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "validator_set") {
        return seed;
    }
    let mut vals = Vec::new();
    for i in 1..=21 {
        vals.push(json!({
            "address": format!("wsk1validator{i:03}xxxxxxxxxxxxxxxxxxxxxxxxx"),
            "voting_power": 1_000_000 + i * 100_000,
            "moniker": format!("validator-{i}"),
            "commission": "0.05",
            "status": "active"
        }));
    }
    json!({ "height": 1_200_000, "validators": vals })
}

// ── account_get ───────────────────────────────────────────────────
pub fn account_get(db: &DB, params: &Value) -> Value {
    let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("wsk1unknown");

    // Check if we stored a mutable balance (after faucet / transfers)
    let key = format!("account:{address}");
    if let Some(acct) = rocks::get_json(db, &key) {
        return acct;
    }

    if let Some(seed) = seed_rpc(db, "account_get") {
        return seed;
    }

    json!({
        "address": address,
        "nonce": 0,
        "balance": "0",
        "assets": []
    })
}

// ── asset_list ────────────────────────────────────────────────────
pub fn asset_list(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "asset_list") {
        return seed;
    }
    json!({
        "assets": [
            { "id": "worldstreet:MNA:native", "symbol": "WSK", "name": "WorldStreet Kash", "decimals": 6, "enabled": true, "kind": "native" },
            { "id": "worldstreet:WUSDC:bridged", "symbol": "WUSDC", "name": "Wrapped USDC", "decimals": 6, "enabled": true, "kind": "bridged" },
            { "id": "worldstreet:WETH:bridged", "symbol": "WETH", "name": "Wrapped ETH", "decimals": 18, "enabled": true, "kind": "bridged" },
            { "id": "worldstreet:WSOL:bridged", "symbol": "WSOL", "name": "Wrapped SOL", "decimals": 9, "enabled": true, "kind": "bridged" }
        ]
    })
}

// ── token_list ────────────────────────────────────────────────────
pub fn token_list(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "token_list") {
        return seed;
    }
    json!({ "tokens": [] })
}

// ── token_evm_address ─────────────────────────────────────────────
pub fn token_evm_address(_db: &DB, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");
    // Deterministic EVM address derived from the token hash
    let evm = format!("0x{:0>40}", &hash.replace('-', "").chars().take(40).collect::<String>());
    json!({ "token_id": hash, "evm_address": evm })
}

// ── transaction_broadcast ─────────────────────────────────────────
pub fn transaction_broadcast(db: &DB, params: &Value) -> Value {
    let hash = sim_hash("tx");
    // Persist the pending transaction in RocksDB
    let record = json!({
        "hash": hash,
        "status": "confirmed",
        "height": 1_200_000 + (now_ms() / 6000) % 10_000,
        "from": params.get("transaction").and_then(|t| t.get("from")).cloned().unwrap_or(json!("")),
        "to": params.get("transaction").and_then(|t| t.get("to")).cloned().unwrap_or(json!("")),
        "amount": params.get("transaction").and_then(|t| t.get("amount")).cloned().unwrap_or(json!("0")),
        "fee": "500",
        "note": "Simulated — no real chain interaction."
    });
    rocks::put_json(db, &format!("tx:{hash}"), &record).ok();
    json!({ "hash": hash })
}

// ── transaction_status ────────────────────────────────────────────
pub fn transaction_status(db: &DB, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(tx) = rocks::get_json(db, &format!("tx:{hash}")) {
        return tx;
    }
    if let Some(seed) = seed_rpc(db, "transaction_status") {
        return seed;
    }
    json!({ "hash": hash, "status": "not_found" })
}

// ── token_operation_prepare ───────────────────────────────────────
pub fn token_operation_prepare(_db: &DB, params: &Value) -> Value {
    let signing_bytes = sim_bytes(64);
    json!({
        "signing_bytes": signing_bytes,
        "unsigned": params.get("unsigned").cloned().unwrap_or(json!({})),
        "fee": "1000",
        "nonce": now_ms() % 100_000
    })
}

// ── token_operation_broadcast ─────────────────────────────────────
pub fn token_operation_broadcast(db: &DB, params: &Value) -> Value {
    let op_id = sim_hash("tokop");
    let token_id = params
        .get("operation")
        .and_then(|o| o.get("token_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let record = json!({
        "operation_id": op_id,
        "token_id": token_id,
        "status": "confirmed",
        "height": 1_200_000 + (now_ms() / 6000) % 10_000,
        "note": "Simulated token operation."
    });
    rocks::put_json(db, &format!("tokop:{op_id}"), &record).ok();
    json!({ "operation_id": op_id, "token_id": token_id })
}

// ── token_operation_status ────────────────────────────────────────
pub fn token_operation_status(db: &DB, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(op) = rocks::get_json(db, &format!("tokop:{hash}")) {
        return op;
    }
    if let Some(seed) = seed_rpc(db, "token_operation_status") {
        return seed;
    }
    json!({ "operation_id": hash, "status": "not_found" })
}

// ── mna_reserve_status ────────────────────────────────────────────
pub fn mna_reserve_status(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "mna_reserve_status") {
        return seed;
    }
    json!({
        "total_supply_mna": "1000000000000",
        "total_collateral_usdc": "1000000000000",
        "collateralization_ratio": "1.0",
        "mint_rate": "1.0",
        "redeem_rate": "1.0",
        "status": "active"
    })
}

// ── mna_quote ─────────────────────────────────────────────────────
pub fn mna_quote(_db: &DB, params: &Value) -> Value {
    let amount_str = params.get("amount_usdc").and_then(|v| v.as_str()).unwrap_or("0");
    let amount: u64 = amount_str.parse().unwrap_or(0);
    // 1:1 peg for simulation
    json!({
        "amount_usdc": amount_str,
        "amount_mna": amount.to_string(),
        "rate": "1.0",
        "fee": "0",
        "expires_at": now_ms() + 30_000
    })
}

// ── mna_swap_prepare ──────────────────────────────────────────────
pub fn mna_swap_prepare(_db: &DB, params: &Value) -> Value {
    let signing_bytes = sim_bytes(64);
    json!({
        "signing_bytes": signing_bytes,
        "unsigned": params.get("unsigned").cloned().unwrap_or(json!({})),
        "fee": "500",
        "nonce": now_ms() % 100_000
    })
}

// ── mna_swap_broadcast ────────────────────────────────────────────
pub fn mna_swap_broadcast(db: &DB, params: &Value) -> Value {
    let op_id = sim_hash("swap");
    let record = json!({
        "operation_id": op_id,
        "status": "confirmed",
        "kind": params.get("operation").and_then(|o| o.get("kind")).cloned().unwrap_or(json!("MintMna")),
        "height": 1_200_000 + (now_ms() / 6000) % 10_000,
        "note": "Simulated MNA swap."
    });
    rocks::put_json(db, &format!("swap:{op_id}"), &record).ok();
    json!({ "operation_id": op_id })
}

// ── mna_swap_status ───────────────────────────────────────────────
pub fn mna_swap_status(db: &DB, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(op) = rocks::get_json(db, &format!("swap:{hash}")) {
        return op;
    }
    if let Some(seed) = seed_rpc(db, "mna_swap_status") {
        return seed;
    }
    json!({ "operation_id": hash, "status": "not_found" })
}

// ── sol_mna_quote ─────────────────────────────────────────────────
pub fn sol_mna_quote(_db: &DB, params: &Value) -> Value {
    let lamports_str = params.get("amount_lamports").and_then(|v| v.as_str()).unwrap_or("0");
    let lamports: u64 = lamports_str.parse().unwrap_or(0);
    // Simulated SOL -> WSK oracle: 1 SOL ~ 150 WSK (at 6 decimals each side)
    let mna = lamports * 150;
    json!({
        "amount_lamports": lamports_str,
        "amount_mna": mna.to_string(),
        "sol_price_usd": "150.00",
        "mna_price_usd": "1.00",
        "rate": "150.0",
        "fee_lamports": "5000",
        "expires_at": now_ms() + 30_000
    })
}

// ── devnet_faucet ─────────────────────────────────────────────────
pub fn devnet_faucet(db: &DB, params: &Value) -> Value {
    let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("wsk1unknown");
    let drip: u64 = 10_000_000; // 10 WSK in microWSK
    let hash = sim_hash("faucet");

    // Credit the account in RocksDB
    let key = format!("account:{address}");
    let mut acct = rocks::get_json(db, &key).unwrap_or_else(|| json!({
        "address": address,
        "nonce": 0,
        "balance": "0",
        "assets": []
    }));
    let cur: u64 = acct["balance"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0);
    acct["balance"] = json!((cur + drip).to_string());
    rocks::put_json(db, &key, &acct).ok();

    json!({
        "hash": hash,
        "amount": drip.to_string(),
        "address": address,
        "note": "Devnet faucet — simulated drip."
    })
}

// ── bridge_status (served via chain, used by rpc dispatcher) ──────
pub fn bridge_status(db: &DB, _params: &Value) -> Value {
    if let Some(seed) = seed_rpc(db, "bridge_status") {
        return seed;
    }
    json!({
        "lanes": [
            { "name": "ethereum", "chain": "Ethereum Sepolia", "asset": "USDC", "status": "active", "confirmations_required": 12 },
            { "name": "arbitrum", "chain": "Arbitrum One", "asset": "WSK", "status": "active", "confirmations_required": 1 },
            { "name": "solana", "chain": "Solana devnet", "asset": "WSOL", "status": "active", "confirmations_required": 32 },
            { "name": "solana_usdc", "chain": "Solana devnet", "asset": "USDC", "status": "active", "confirmations_required": 32 }
        ]
    })
}

// ── internal helpers ──────────────────────────────────────────────

/// Try to load a seed value from `seed:rpc` -> <method>.
fn seed_rpc(db: &DB, method: &str) -> Option<Value> {
    let rpc_seed = rocks::seed_all(db, "rpc");
    rpc_seed.get(method).cloned()
}

/// ISO-8601-ish timestamp for chain_info (no chrono crate needed).
fn chrono_like_now() -> String {
    let secs = now_ms() / 1000;
    // Rough UTC: good enough for display. Real deploys would use chrono.
    format!("2025-01-01T00:00:00Z+{secs}s")
}

/// Dispatch an RPC method to the correct chain function.
/// Returns None if the method is not a chain/wallet method.
pub fn dispatch(db: &DB, method: &str, params: &Value) -> Option<Value> {
    Some(match method {
        "chain_info"                => chain_info(db, params),
        "validator_set"             => validator_set(db, params),
        "account_get"               => account_get(db, params),
        "asset_list"                => asset_list(db, params),
        "bridge_status"             => bridge_status(db, params),
        "mna_reserve_status"        => mna_reserve_status(db, params),
        "transaction_broadcast"     => transaction_broadcast(db, params),
        "transaction_status"        => transaction_status(db, params),
        "token_list"                => token_list(db, params),
        "token_evm_address"         => token_evm_address(db, params),
        "token_operation_prepare"   => token_operation_prepare(db, params),
        "token_operation_broadcast" => token_operation_broadcast(db, params),
        "token_operation_status"    => token_operation_status(db, params),
        "mna_quote"                 => mna_quote(db, params),
        "mna_swap_prepare"          => mna_swap_prepare(db, params),
        "mna_swap_broadcast"        => mna_swap_broadcast(db, params),
        "mna_swap_status"           => mna_swap_status(db, params),
        "sol_mna_quote"             => sol_mna_quote(db, params),
        "devnet_faucet"             => devnet_faucet(db, params),
        _ => return None,
    })
}
