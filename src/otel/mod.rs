use std::time::Duration;

use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    resource::{EnvResourceDetector, ResourceDetector},
    Resource,
};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub(crate) mod metrics;
pub(crate) mod tracing;

pub fn resource() -> Resource {
    Resource::default().merge(&EnvResourceDetector::new().detect(Duration::from_secs(3)))
}

pub struct OtelGuard {
    meter_provider: Option<SdkMeterProvider>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(Err(err)) = self.meter_provider.as_ref().map(|mp| mp.shutdown()) {
            eprintln!("Failed to shutdown meter provider: {err:?}");
        }

        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn init() -> OtelGuard {
    let meter_provider = metrics::create_meter_provider();
    let metrics_layer = meter_provider.clone().map(MetricsLayer::new);
    tracing::create_tracer();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(metrics_layer)
        .init();

    OtelGuard { meter_provider }
}
