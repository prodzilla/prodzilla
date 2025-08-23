mod model;
mod probes;
mod prometheus_metrics;
mod stories;

use crate::web_server::{
    probes::{bulk_probe_trigger, get_probe_results, probe_trigger, probes},
    stories::{bulk_story_trigger, get_story_results, stories, story_trigger},
};
use axum::{response::{Html, IntoResponse}, routing::{get, post}, Extension, Router};
use std::{env, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{debug, info};

use crate::app_state::AppState;

pub async fn start_axum_server(app_state: Arc<AppState>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/probes", get(probes))
        .route("/probes/:name/results", get(get_probe_results))
        .route("/probes/:name/trigger", get(probe_trigger))
        .route("/probes/bulk/trigger", post(bulk_probe_trigger))
        .route("/stories", get(stories))
        .route("/stories/:name/results", get(get_story_results))
        .route("/stories/:name/trigger", get(story_trigger))
        .route("/stories/bulk/trigger", post(bulk_story_trigger))
        .route("/ui", get(serve_ui))
        .nest_service("/ui/static", ServeDir::new("src/web_ui/dist/static"))
        .fallback_service(axum::routing::get(serve_ui_fallback))
        .layer(Extension(app_state.clone()))
        .layer(cors);

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

async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("../web_ui/dist/index.html"))
}

async fn serve_ui_fallback(uri: axum::http::Uri) -> axum::response::Response {
    let path = uri.path();
    if path.starts_with("/ui/") && !path.starts_with("/ui/static/") {
        Html(include_str!("../web_ui/dist/index.html")).into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}
