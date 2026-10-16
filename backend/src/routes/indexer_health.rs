use crate::db::rocks::seed_all;
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;

async fn indexer_health(State(st): State<AppState>) -> Json<Value> {
    Json(seed_all(&st.rocks, "indexer_health"))
}

inventory::submit! { RouteReg { mount: |r: Router<AppState>| r.route("/api-indexer/health", get(indexer_health)) } }
