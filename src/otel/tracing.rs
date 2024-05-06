use std::env;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{BatchConfig, Tracer, TracerProvider};

use super::resource;

pub fn create_tracer() -> Option<Tracer> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    match env::var("OTEL_TRACES_EXPORTER") {
        Ok(exporter) if exporter == "otlp" => Some(
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_trace_config(
                    opentelemetry_sdk::trace::Config::default().with_resource(resource()),
                )
                .with_batch_config(BatchConfig::default())
                .with_exporter(opentelemetry_otlp::new_exporter().tonic())
                .install_batch(runtime::Tokio)
                .unwrap(),
        ),
        Ok(exporter) if exporter == "stdout" => {
            let provider = TracerProvider::builder()
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .build();
            Some(provider.tracer("prodzilla"))
        }
        _ => None,
    }
}
