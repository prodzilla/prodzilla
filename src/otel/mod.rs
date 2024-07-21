use std::{env, time::Duration};

use opentelemetry_otlp::{ExportConfig, Protocol};
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    resource::{EnvResourceDetector, ResourceDetector},
    Resource,
};
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
    tracing::create_tracer();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    OtelGuard { meter_provider }
}

fn create_otlp_export_config() -> ExportConfig {
    ExportConfig {
        endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string()),
        protocol: match env::var("OTEL_EXPORTER_OTLP_PROTOCOL") {
            Ok(protocol) if protocol == "http/protobuf" => Protocol::HttpBinary,
            Ok(protocol) if protocol == "http/json" => Protocol::HttpJson,
            _ => Protocol::Grpc,
        },
        timeout: Duration::from_secs(
            env::var("OTEL_EXPORTER_OTLP_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse::<u64>()
                .expect("OTEL_EXPORTER_OTLP_TIMEOUT must be a number"),
        ),
    }
}
