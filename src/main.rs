mod config;
mod probe;
mod expectations;
mod http_probe;
mod schedule;
mod alert_webhook;
mod errors;

use axum::{
    routing::{get},Router,
};
use schedule::schedule_probes;

use crate::config::load_config;

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    start_monitoring().await?;

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn start_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(PRODZILLA_YAML).await?;
    schedule_probes(config.probes);
    Ok(())
}

async fn root() -> &'static str {
    "Roar!"
}

#[cfg(test)]
mod test_utils;


// TODO:
// - consolidate logging into one place, only get 1 log when errors etc
// - adjustable timeouts and retries on probe and alert calls
// - test what happens if alert call fails - make sure it continues to probe
// - update readme with expectations format, remove prodzilla-future
// - do we need tracing?
// - validation of config fields / use enums for http GET 
