use axum::{routing::get, Extension, Json, Router};
use crate::probe::model::ProbeResult;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::app_state::AppState;

pub async fn start_axum_server(app_state: Arc<AppState>) -> shuttle_axum::ShuttleAxum {
    let app = Router::new()
        .route("/", get(root))
        .route("/probe_results", get(get_probe_results))
        .layer(Extension(app_state.clone()));

    return Ok(app.into())
}

async fn root() -> &'static str {
    debug!("Application root called");
    "Roar!"
}

async fn get_probe_results(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<HashMap<String, Vec<ProbeResult>>> {
    debug!("Get probe results called");
    let read_lock = state.probe_results.read();
    Json(read_lock.unwrap().clone())
}
