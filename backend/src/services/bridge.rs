// Simulated bridge lane status and events for the Intertrain wallet.
// Covers Ethereum USDC, Arbitrum WSK, Solana SOL, and Solana USDC lanes.
// All data is synthetic — no real cross-chain interaction.

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

fn sim_hash(prefix: &str) -> String {
    format!("{prefix}_{:016x}", now_ms())
}

/// Bridge lane configuration and live status.
/// Already partially handled by chain::bridge_status, but this module provides
/// richer detail for the bridge UI: contract addresses, recent events, and volumes.
pub fn lane_detail(db: &DB, params: &Value) -> Value {
    let lane = params.get("lane").and_then(|v| v.as_str()).unwrap_or("ethereum");

    // Try seed data first
    if let Some(seed) = rocks::get_json(db, &format!("seed:bridge_{lane}")) {
        return seed;
    }

    match lane {
        "ethereum" => json!({
            "lane": "ethereum",
            "chain": "Ethereum Sepolia",
            "asset": "USDC",
            "status": "active",
            "bridge_contract": "0x981fA35883e60Fbe229D449334B3507997af84E1",
            "escrow_contract": "0xab4056bCb0369897d6D5Ca1A13f670f76C75ef3e",
            "confirmations_required": 12,
            "total_deposited": "500000000000",
            "total_withdrawn": "200000000000",
            "recent_events": []
        }),
        "arbitrum" => json!({
            "lane": "arbitrum",
            "chain": "Arbitrum One",
            "asset": "WSK",
            "status": "active",
            "bridge_contract": "0x0729F81ACc0948089B0BAcc0685c461F8F54F23B",
            "confirmations_required": 1,
            "total_deposited": "100000000000",
            "total_withdrawn": "50000000000",
            "recent_events": []
        }),
        "solana" => json!({
            "lane": "solana",
            "chain": "Solana devnet",
            "asset": "WSOL",
            "status": "active",
            "program_id": "FyAuUc2pPkz1nt2vR27R6NfE3Lgb4Z69sjUoPSeU7PCw",
            "pda_vault": "H9q1t5qw2gbwD9RoUSJp4wNiqWMAyr4zmwz53NmM9bG8",
            "confirmations_required": 32,
            "total_deposited": "50000000000",
            "total_withdrawn": "20000000000",
            "recent_events": []
        }),
        "solana_usdc" => json!({
            "lane": "solana_usdc",
            "chain": "Solana devnet",
            "asset": "USDC",
            "status": "active",
            "program_id": "FyAuUc2pPkz1nt2vR27R6NfE3Lgb4Z69sjUoPSeU7PCw",
            "usdc_mint": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "confirmations_required": 32,
            "total_deposited": "30000000000",
            "total_withdrawn": "10000000000",
            "recent_events": []
        }),
        _ => json!({ "error": "unknown_lane", "lane": lane }),
    }
}

/// Simulate a bridge deposit event. Returns an event ID and marks it pending.
pub fn simulate_deposit(db: &DB, params: &Value) -> Value {
    let lane = params.get("lane").and_then(|v| v.as_str()).unwrap_or("ethereum");
    let dest = params.get("dest_address").and_then(|v| v.as_str()).unwrap_or("wsk1unknown");
    let amount = params.get("amount").and_then(|v| v.as_str()).unwrap_or("0");
    let event_id = sim_hash("bridge");

    let event = json!({
        "event_id": event_id,
        "lane": lane,
        "direction": "deposit",
        "dest_address": dest,
        "amount": amount,
        "status": "confirmed",
        "confirmations": 12,
        "created_at": now_ms(),
        "note": "Simulated bridge deposit."
    });
    rocks::put_json(db, &format!("bridge_event:{event_id}"), &event).ok();

    json!({ "event_id": event_id, "status": "confirmed" })
}

/// Look up a bridge event by ID.
pub fn event_status(db: &DB, params: &Value) -> Value {
    let event_id = params.get("event_id").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(ev) = rocks::get_json(db, &format!("bridge_event:{event_id}")) {
        return ev;
    }
    json!({ "event_id": event_id, "status": "not_found" })
}

/// Dispatch a bridge-related method. Returns None if not a bridge method.
/// Note: bridge_status itself is handled in chain.rs (it is an RPC method);
/// these are auxiliary endpoints for the bridge UI.
pub fn dispatch(db: &DB, method: &str, params: &Value) -> Option<Value> {
    Some(match method {
        "bridge_lane_detail"    => lane_detail(db, params),
        "bridge_deposit"        => simulate_deposit(db, params),
        "bridge_event_status"   => event_status(db, params),
        _ => return None,
    })
}
