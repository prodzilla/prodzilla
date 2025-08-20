mod model;
mod probes;
mod prometheus_metrics;
mod stories;

use crate::web_server::{
    probes::{get_probe_results, probe_trigger, probes},
    stories::{get_story_results, stories, story_trigger},
};
use axum::{routing::get, Extension, Router};
use std::{env, sync::Arc};
use tracing::{debug, info};

use crate::app_state::AppState;

pub async fn start_axum_server(app_state: Arc<AppState>) {
    let app = Router::new()
        .route("/", get(root))
        .route("/probes", get(probes))
        .route("/probes/:name/results", get(get_probe_results))
        .route("/probes/:name/trigger", get(probe_trigger))
        .route("/stories", get(stories))
        .route("/stories/:name/results", get(get_story_results))
        .route("/stories/:name/trigger", get(story_trigger))
        .layer(Extension(app_state.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

pub async fn start_prometheus_server(registry: Arc<prometheus::Registry>) {
    let host = match env::var("OTEL_EXPORTER_PROMETHEUS_HOST") {
        Ok(host) => host,
        Err(_) => "localhost".to_owned(),
    };
    let port = match env::var("OTEL_EXPORTER_PROMETHEUS_PORT") {
        Ok(port) => port,
        Err(_) => "9464".to_owned(),
    };
    let app = Router::new()
        .route("/metrics", get(prometheus_metrics::metrics_handler))
        .layer(Extension(registry));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    info!(
        "Serving Prometheus metrics on {}/metrics",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    debug!("Application root called");
    "Roar!"
}
