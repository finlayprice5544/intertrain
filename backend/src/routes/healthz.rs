// GET /healthz — lightweight health probe for Railway / load balancers.
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/healthz", get(healthz))
    }
}
