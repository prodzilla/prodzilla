use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        MeterProviderBuilder, PeriodicReader, SdkMeterProvider,
    },
    runtime,
};
use std::env;
use tracing::debug;

use crate::otel::create_otlp_export_config;

use super::resource;

pub fn create_meter_provider() -> Option<SdkMeterProvider> {
    let reader = match env::var("OTEL_METRICS_EXPORTER") {
        Ok(exporter_type) if exporter_type == "otlp" => {
            debug!("Using OTLP metrics exporter");
            let export_config = create_otlp_export_config();
            let exporter = match export_config.protocol {
                opentelemetry_otlp::Protocol::Grpc => {
                    debug!("Using OTLP gRPC exporter");
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_export_config(export_config)
                        .build_metrics_exporter(
                            Box::new(DefaultAggregationSelector::new()),
                            Box::new(DefaultTemporalitySelector::new()),
                        )
                        .unwrap()
                }
                _ => {
                    debug!("Using OTLP HTTP exporter");
                    match opentelemetry_otlp::new_exporter()
                        .http()
                        .with_protocol(export_config.protocol)
                        .with_endpoint(format!("{}/v1/metrics", export_config.endpoint))
                        // .with_export_config(export_config)
                        .build_metrics_exporter(
                            Box::new(DefaultAggregationSelector::new()),
                            Box::new(DefaultTemporalitySelector::new()),
                        ) {
                        Ok(exporter) => exporter,
                        Err(err) => {
                            panic!("Failed to create OTLP HTTP metrics exporter: {}", err);
                        }
                    }
                }
            };
            PeriodicReader::builder(exporter, runtime::Tokio).build()
        }
        Ok(exporter_type) if exporter_type == "stdout" => {
            debug!("Using stdout metrics exporter");
            let exporter = opentelemetry_stdout::MetricsExporter::default();
            PeriodicReader::builder(exporter, runtime::Tokio).build()
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

pub struct Metrics {
    pub duration: Histogram<u64>,
    pub runs: Counter<u64>,
    pub errors: Counter<u64>,
}

impl Metrics {
    pub fn new() -> Metrics {
        let meter = opentelemetry::global::meter("prodzilla");
        Metrics {
            duration: meter.u64_histogram("duration").init(),
            runs: meter.u64_counter("runs").init(),
            errors: meter.u64_counter("errors").init(),
        }
    }
}
