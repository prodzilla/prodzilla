mod alert_webhook;
mod app_state;
mod config;
mod errors;
mod expectations;
mod http_probe;
mod probe;
mod schedule;

use axum::{routing::get, Extension, Json, Router};
use probe::ProbeResult;
use schedule::schedule_probes;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::{config::load_config, app_state::AppState};

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Shuttle has its own tracing so turn this off
    //init_tracing();

    let app_state = Arc::new(AppState::new());

    start_monitoring(app_state.clone()).await
        .expect("Error in start_monitoring");

    let app = Router::new()
        .route("/", get(root))
        .route("/probe_results", get(get_probe_results))
        .layer(Extension(app_state.clone()));

    Ok(app.into())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

async fn start_monitoring(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(PRODZILLA_YAML).await?;
    schedule_probes(config.probes, app_state);
    Ok(())
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

#[cfg(test)]
mod test_utils;
