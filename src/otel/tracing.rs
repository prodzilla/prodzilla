use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;

// Needs to be called to enable trace ids
pub fn init_otel_tracing() {
    let span_exporter = match opentelemetry_otlp::new_exporter()
        .tonic()
        .build_span_exporter() {
        Ok(exporter) => exporter,
        Err(why) => {
            panic!("Failed to create OTLP exporter: {:?}", why);
        }
    };
    let provider = TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
        .build();
    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
}
