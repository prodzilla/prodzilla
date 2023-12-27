use std::str::FromStr;

use crate::probe::Probe;
use crate::probe::ProbeExpectParameters;
use crate::probe::ProbeResult;
use lazy_static::lazy_static;
use reqwest::Error;
use reqwest::RequestBuilder;
use reqwest::Response;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla")
        .build()
        .unwrap();
}

pub async fn check_endpoint(probe: &Probe) -> Result<ProbeResult, Box<dyn std::error::Error>> {

    let method = reqwest::Method::from_str(&probe.http_method)?;

    let mut request = CLIENT.request(method, &probe.url);

    let response = request.send().await;

    // todo add headers and body

    match response {
        Ok(res) => {
            println!("successful response whilst pinging {}", &probe.url);
            if !probe.expect_back.is_some() {
                return Ok(ProbeResult{success: true});
            } else {
                return Ok(ProbeResult{success: true});
            }

        }
        Err(e) => {
            println!("Error whilst pinging {}", &probe.url);
            match probe.expect_back {
                Some(expect_back) => {
                    let result = validate_error_response(&expect_back, e);
                    return Ok(ProbeResult{success: true});
                }
                None => {
                    return Ok(ProbeResult{success: true});
                }
            }
        }
    }
}

fn validate_response(expect: &ProbeExpectParameters, response: &Response) -> bool {
    // todo be explicit about what failed
    return false;
}

fn validate_error_response(expect: &ProbeExpectParameters, error: &Error) -> bool {
    // todo be explicit about what failed
    return false;
}

