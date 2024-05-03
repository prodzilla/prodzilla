use std::time::Duration;

use opentelemetry_sdk::{
    resource::{EnvResourceDetector, ResourceDetector},
    Resource,
};

pub(crate) mod metrics;
pub(crate) mod tracing;

pub fn resource() -> Resource {
    Resource::default().merge(&EnvResourceDetector::new().detect(Duration::from_secs(3)))
}
