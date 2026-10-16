// Built-in meta endpoints.
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

async fn health(State(st): State<AppState>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "dataMode": format!("{:?}", st.config.data_mode),
        "clickhouse": st.clickhouse.is_some(),
    }))
}

async fn branding(State(st): State<AppState>) -> Json<Value> {
    Json(crate::branding::payload(&st.config))
}

inventory::submit! { RouteReg { mount: |r: Router<AppState>| r.route("/api/_health", get(health)) } }
inventory::submit! { RouteReg { mount: |r: Router<AppState>| r.route("/api/_branding", get(branding)) } }
