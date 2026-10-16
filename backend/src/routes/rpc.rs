use crate::db::rocks::seed_all;
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde_json::{json, Value};

async fn rpc(State(st): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let method = body["method"].as_str().unwrap_or("");
    let id = body.get("id").cloned().unwrap_or_else(|| json!(1));

    let seed = seed_all(&st.rocks, "rpc");
    match seed.get(method) {
        Some(result) => Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })),
        None => Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "method not found" }
        })),
    }
}

inventory::submit! { RouteReg { mount: |r: Router<AppState>| r.route("/api-mainnet/rpc", post(rpc)) } }
