mod config;
mod probe;


use axum::{
    routing::{get},Router,
};

use crate::config::load_config;

const PRODZILLA_YAML: &str = "prodzilla.yml";

#[tokio::main]
async fn main() {

    let config = load_config(PRODZILLA_YAML);

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

// todo
// - load config in
// - do we need tracing?