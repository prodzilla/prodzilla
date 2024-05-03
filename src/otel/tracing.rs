use std::time::Duration;

use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::resource::{EnvResourceDetector, ResourceDetector};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_sdk::trace::{BatchConfig, Tracer};

pub fn create_tracer() -> Tracer{
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(
                    Resource::default().merge(
                        &EnvResourceDetector::new().detect(Duration::from_secs(3))
                    )
                )
        )
        .with_batch_config(BatchConfig::default())
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(runtime::Tokio)
        .unwrap()
}

