mod alerts;
mod app_state;
mod config;
mod errors;
mod probe;
mod web_server;
mod otel;

use clap::Parser;
use probe::schedule::schedule_probes;
use probe::schedule::schedule_stories;
use tracing_opentelemetry::OpenTelemetryLayer;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use web_server::start_axum_server;
use tracing_subscriber::prelude::*;

use crate::{app_state::AppState, config::load_config};

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Test definition file to execute
    #[arg(short, long, default_value = PRODZILLA_YAML)]
    file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    init_tracing();

    let config = load_config(args.file).await?;

    let app_state = Arc::new(AppState::new(config));

    start_monitoring(app_state.clone()).await?;

    start_axum_server(app_state.clone()).await;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(otel::tracing::create_tracer()))
        .init();
}

async fn start_monitoring(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {    
    schedule_probes(&app_state.config.probes, app_state.clone());
    schedule_stories(&app_state.config.stories, app_state.clone());
    Ok(())
}

#[cfg(test)]
mod test_utils;
