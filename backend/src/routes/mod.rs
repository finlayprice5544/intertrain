// Route registry. Each route file submits a `RouteReg` via `inventory`;
// mount_all() collects them. To add an endpoint:
//   1. create src/routes/<name>.rs  (copy example_coins.rs)
//   2. add `pub mod <name>;` below
// It then auto-mounts — no central wiring to edit.
use crate::state::AppState;
use axum::Router;

pub struct RouteReg {
    pub mount: fn(Router<AppState>) -> Router<AppState>,
}
inventory::collect!(RouteReg);

pub fn mount_all(mut router: Router<AppState>) -> Router<AppState> {
    let mut count = 0;
    for reg in inventory::iter::<RouteReg> {
        router = (reg.mount)(router);
        count += 1;
    }
    tracing::info!("mounted {count} route group(s)");
    router
}

pub mod meta;
pub mod indexer_health;
pub mod indexer_tokens;
pub mod rpc;
pub mod healthz;
pub mod metrics;
