use std::env;

use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;

use opentelemetry_sdk::trace::TracerProvider;

use super::resource;

pub fn create_tracer() {
    let provider = match env::var("OTEL_TRACES_EXPORTER") {
        Ok(exporter) if exporter == "otlp" => {
            let span_exporter = match opentelemetry_otlp::new_exporter()
                .tonic()
                .build_span_exporter()
            {
                Ok(exporter) => exporter,
                Err(why) => {
                    panic!("Failed to create OTLP exporter: {:?}", why);
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
        _ => return,
    };
    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
}
