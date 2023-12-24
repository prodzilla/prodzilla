use crate::probe::Probe;
use crate::probe::ProbeResult;
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla")
        .build()
        .unwrap();
}

pub async fn check_endpoint(probe: &Probe) -> ProbeResult {
    println!("Fake probed {}", probe.name);
    return ProbeResult{success: false};
}