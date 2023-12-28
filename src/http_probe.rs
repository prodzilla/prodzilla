use std::collections::HashMap;
use std::str::FromStr;

use crate::probe::Probe;
use crate::probe::ProbeExpectation;
use crate::probe::ProbeInputParameters;
use crate::probe::ProbeResult;
use crate::probe::ProbeScheduleParameters;
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

    let request = build_request(probe)?;
    let response = request.send().await;

    // TODO: Fix this dirty block below

    match response {
        Ok(res) => {
            match &probe.expectations {
                Some(expect_back) => {
                    let validation_result = validate_response(&expect_back, &res);
                    if validation_result {
                        println!("Successful response for {}, as expected.", &probe.name);
                    } else {
                        println!("Successful response for {}, not as expected!", &probe.name);
                    }
                    return Ok(ProbeResult{success: validation_result});
                }
                None => {
                    println!("Successfully probed {}, no expectation so success is true.", &probe.name);
                    return Ok(ProbeResult{success: true});
                }
            }

        }
        Err(e) => {
            match &probe.expectations {
                Some(expect_back) => {
                    let validation_result = validate_error_response(&expect_back, &e);
                    println!("Error whilst executing probe {}, but as expected.", &probe.name);
                    return Ok(ProbeResult{success: validation_result});
                }
                None => {
                    println!("Error whilst executing probe {}, but no expectation so success is true.", &probe.name);
                    return Ok(ProbeResult{success: true});
                }
            }
        }
    }
}

fn build_request(probe: &Probe) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
    let method = reqwest::Method::from_str(&probe.http_method)?;

    let mut request = CLIENT.request(method, &probe.url);

    if let Some(probe_input_parameters) = &probe.with {
        if let Some(body) = &probe_input_parameters.body {
            request = request.body(body.clone());
        }
        if let Some(headers) = &probe_input_parameters.headers {
            for (key, value) in headers.clone().iter() {
                request = request.header(key, value);
            }
        }
    }

    return Ok(request);
}


fn validate_response(expect: &Vec<ProbeExpectation>, response: &Response) -> bool {
    // todo be explicit about what failed
    return false;
}

fn validate_error_response(expect: &Vec<ProbeExpectation>, error: &Error) -> bool {
    // todo be explicit about what failed
    return false;
}

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_string};

#[tokio::test]
async fn test_requests_hit_downstream_with_all_data() {
    let mock_server = MockServer::start().await;

    let body = "test body";

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(body_string("poo".to_string()))
        .respond_with(ResponseTemplate::new(200)) // You can change the status code here
        .mount(&mock_server)
        .await;

    let probe = Probe{
        name: "Test probe".to_string(),
        url: format!("{}/test", mock_server.uri()),
        http_method: "GET".to_string(),
        with: Some(ProbeInputParameters{
            body: Some(body.to_string()),
            headers: Some(HashMap::new())
        }),
        expectations: None,
        schedule: ProbeScheduleParameters{
            initial_delay: 0,
            interval: 0
        }
    };

    let probe_result = check_endpoint(&probe).await;

    assert_eq!(probe_result.unwrap().success, true);
}