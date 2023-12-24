use crate::probe::Probe;
use crate::probe::ProbeResult;
use lazy_static::lazy_static;
use reqwest::RequestBuilder;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla")
        .build()
        .unwrap();
}

pub async fn check_endpoint(probe: &Probe) -> ProbeResult {

    let mut request = CLIENT.request(method, self.url.clone());
    // add body
    CLIENT.request(method, url)

    CLIENT.execute(request)

    println!("Fake probed {}", probe.name);
    return ProbeResult{success: false};
}