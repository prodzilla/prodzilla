use opentelemetry::{
    global,
    metrics::{Counter, Gauge, Histogram, Unit},
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector, MetricReader},
        MeterProviderBuilder, PeriodicReader, SdkMeterProvider,
    },
    runtime,
};

use std::{env, sync::Arc};
use tracing::debug;

use crate::otel::create_otlp_export_config;

use super::resource;

fn build_meter_provider<T>(reader: T) -> SdkMeterProvider
where
    T: MetricReader,
{
    MeterProviderBuilder::default()
        .with_resource(resource())
        .with_reader(reader)
        .build()
}

pub struct MetricsState {
    pub meter: Option<SdkMeterProvider>,
    pub registry: Option<Arc<prometheus::Registry>>,
}

pub fn initialize() -> MetricsState {
    let (meter_provider, prometheus_registry) = match env::var("OTEL_METRICS_EXPORTER") {
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
            let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
            (build_meter_provider(reader), None)
        }
        Ok(exporter_type) if exporter_type == "stdout" => {
            debug!("Using stdout metrics exporter");
            let exporter = opentelemetry_stdout::MetricsExporter::default();
            let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
            (build_meter_provider(reader), None)
        }
        Ok(exporter_type) if exporter_type == "prometheus" => {
            debug!("Using Prometheus metrics exporter");
            let registry = prometheus::Registry::new();
            let reader = opentelemetry_prometheus::exporter()
                .with_registry(registry.clone())
                .build()
                .unwrap();
            (build_meter_provider(reader), Some(Arc::new(registry)))
        }
        _ => {
            debug!("No metrics exporter configured");
            return MetricsState {
                meter: None,
                registry: None,
            };
        }
    };

    global::set_meter_provider(meter_provider.clone());

    MetricsState {
        meter: Some(meter_provider),
        registry: prometheus_registry,
    }
}

pub struct Metrics {
    pub duration: Histogram<u64>,
    pub runs: Counter<u64>,
    pub errors: Counter<u64>,
    pub status: Gauge<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum MonitorStatus {
    Ok = 0,
    Error = 1,
}

impl MonitorStatus {
    pub fn as_u64(&self) -> u64 {
        *self as u64
    }
}

impl Metrics {
    pub fn new() -> Metrics {
        let meter = opentelemetry::global::meter("prodzilla");
        Metrics {
            duration: meter
                .u64_histogram("duration")
                .with_unit(Unit::new("ms"))
                .with_description("request duration histogram in milliseconds")
                .init(),
            runs: meter
                .u64_counter("runs")
                .with_description("the total count of runs by monitor (story/probe)")
                .init(),
            errors: meter
                .u64_counter("errors")
                .with_description("the total number of errors by monitor (story/probe)")
                .init(),
            status: meter
                .u64_gauge("status")
                .with_description("the current status of each monitor OK = 0 Error = 1")
                .init(),
        }
    }
}
