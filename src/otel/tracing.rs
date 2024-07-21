use std::env;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;

use opentelemetry_sdk::trace::TracerProvider;
use tracing::debug;

use super::{create_otlp_export_config, resource};

pub fn create_tracer() {
    let provider = match env::var("OTEL_TRACES_EXPORTER") {
        Ok(exporter) if exporter == "otlp" => {
            let export_config = create_otlp_export_config();
            let span_exporter = match export_config.protocol {
                opentelemetry_otlp::Protocol::Grpc => {
                    debug!("Using OTLP gRPC exporter");
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_export_config(export_config)
                        .build_span_exporter()
                        .unwrap()
                }
                _ => {
                    debug!("Using OTLP HTTP exporter");
                    opentelemetry_otlp::new_exporter()
                        .http()
                        .with_protocol(export_config.protocol)
                        .with_endpoint(format!("{}/v1/traces", export_config.endpoint))
                        .build_span_exporter()
                        .unwrap()
                }
            };
            TracerProvider::builder()
                .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
                .with_config(opentelemetry_sdk::trace::Config::default().with_resource(resource()))
                .build()
        }
        Ok(exporter) if exporter == "stdout" => TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build(),
        _ => TracerProvider::default(),
    };
    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
}
