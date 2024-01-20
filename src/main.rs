mod alerts;
mod app_state;
mod config;
mod errors;
mod probe;

use axum::{routing::get, Extension, Json, Router};
use probe::{model::ProbeResult, schedule::schedule_stories};
use probe::schedule::schedule_probes;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use crate::{config::load_config, app_state::AppState};

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise logging, so we can use tracing::info! etc elsewhere
    init_tracing();

    let app_state = Arc::new(AppState::new());

    start_monitoring(app_state.clone()).await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/probe_results", get(get_probe_results))
        .layer(Extension(app_state.clone()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
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
    schedule_probes(config.probes, app_state.clone());
    schedule_stories(config.stories, app_state);
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
