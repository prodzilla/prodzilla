use opentelemetry::global;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        MeterProviderBuilder, PeriodicReader, SdkMeterProvider,
    },
    runtime,
};
use tracing::debug;
use std::env;

use super::resource;


pub fn create_meter_provider() -> Option<SdkMeterProvider> {
    let reader = match env::var("OTEL_METRICS_EXPORTER") {
        Ok(exporter_type) if exporter_type == "otlp" => {
            debug!("Using OTLP metrics exporter");
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .build_metrics_exporter(
                    Box::new(DefaultAggregationSelector::new()),
                    Box::new(DefaultTemporalitySelector::new()),
                )
                .unwrap();
            PeriodicReader::builder(exporter, runtime::Tokio)
                .build()
        }
        Ok(exporter_type) if exporter_type == "stdout" => {
            debug!("Using stdout metrics exporter");
            let exporter = opentelemetry_stdout::MetricsExporter::default();
            PeriodicReader::builder(exporter, runtime::Tokio)
                .build()
        }
        _ => {
            debug!("No metrics exporter configured");
            return None;
        }
    };
    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource())
        .with_reader(reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    Some(meter_provider)
}

