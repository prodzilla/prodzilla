use axum::{http::StatusCode, response::IntoResponse, Extension};

use prometheus::Encoder;
use std::sync::Arc;
use tracing::error;

pub async fn metrics_handler(
    Extension(registry): Extension<Arc<prometheus::Registry>>,
) -> Result<impl IntoResponse, StatusCode> {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();
    let mut result = Vec::new();
    if let Err(err) = encoder.encode(&metric_families, &mut result) {
        error!("Failed to encode Prometheus metrics: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let response = match String::from_utf8(result) {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to convert Prometheus metrics to string: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok((StatusCode::OK, response))
}
