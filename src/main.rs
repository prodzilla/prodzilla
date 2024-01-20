mod alerts;
mod app_state;
mod config;
mod errors;
mod probe;
mod web_server;

use probe::schedule::schedule_probes;
use probe::schedule::schedule_stories;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use web_server::start_axum_server;

use crate::{app_state::AppState, config::load_config};

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let app_state = Arc::new(AppState::new());

    start_monitoring(app_state.clone()).await?;

    start_axum_server(app_state.clone()).await;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn start_monitoring(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(PRODZILLA_YAML).await?;
    schedule_probes(config.probes, app_state.clone());
    schedule_stories(config.stories, app_state);
    Ok(())
}

#[cfg(test)]
mod test_utils;
