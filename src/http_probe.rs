use crate::probe::Probe;
use crate::probe::ProbeResult;

static CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
    .build()
    .unwrap();

pub async fn check_endpoint(probe: &Probe) -> ProbeResult {
    return ProbeResult{success: false};
}