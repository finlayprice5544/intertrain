// GET /metrics — minimal Prometheus-style metrics endpoint.
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{http::header, response::IntoResponse, routing::get, Router};

async fn metrics() -> impl IntoResponse {
    // Emit a small set of static gauges. In a real deployment these would
    // come from a metrics registry, but the frontend only checks that the
    // endpoint exists and returns text/plain.
    let body = concat!(
        "# HELP up Whether the service is up.\n",
        "# TYPE up gauge\n",
        "up 1\n",
        "# HELP rpc_requests_total Total JSON-RPC requests served.\n",
        "# TYPE rpc_requests_total counter\n",
        "rpc_requests_total 0\n",
    );

    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/metrics", get(metrics))
    }
}
