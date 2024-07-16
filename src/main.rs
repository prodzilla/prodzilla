mod alerts;
mod app_state;
mod config;
mod errors;
mod probe;
mod web_server;

use probe::http_probe::init_otel_tracing;
use probe::schedule::{schedule_probes, schedule_stories};
use std::error::Error;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use web_server::start_axum_server;

// crate imports
use crate::{app_state::AppState, config::load_config, config::Config};

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();

    let config: Config = load_config(PRODZILLA_YAML).await?;

    let app_state: Arc<AppState> = Arc::new(AppState::new(config));

    start_monitoring(app_state.clone()).await?;
    start_axum_server(app_state.clone()).await;

    Ok(())
}

fn init_tracing() {
    let filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn start_monitoring(app_state: Arc<AppState>) -> Result<(), Box<dyn Error>> {
    init_otel_tracing();

    schedule_probes(&app_state.config.probes, app_state.clone());
    schedule_stories(&app_state.config.stories, app_state.clone());

    Ok(())
}

#[cfg(test)]
mod test_utils;
