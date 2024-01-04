use std::str::FromStr;

use crate::expectations::validate_error_response;
use crate::expectations::validate_response;
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

pub async fn check_endpoint(probe: &Probe) -> Result<ProbeResult, Box<dyn std::error::Error>> {

    let request = build_request(probe)?;
    let response = request.send().await;

    // TODO: Fix this dirty block below

    match response {
        Ok(res) => {
            match &probe.expectations {
                Some(expect_back) => {
                    let validation_result = validate_response(&expect_back, res).await?;
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

use reqwest::StatusCode;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_string};
use crate::http_probe::test_utils::probe_get_with_expected_status;

use self::test_utils::probe_post_with_expected_body;

#[tokio::test]
async fn test_requests_get_200() {
    let mock_server = MockServer::start().await;

    let body = "test body";

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(body_string(body.to_string()))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    
    let probe = probe_get_with_expected_status(StatusCode::OK, format!("{}/test", mock_server.uri()), body.to_string());
    let probe_result = check_endpoint(&probe).await;

    assert_eq!(probe_result.unwrap().success, true);
}

#[tokio::test]
async fn test_requests_get_404() {
    let mock_server = MockServer::start().await;

    let body = "test body";

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(body_string(body.to_string()))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let probe = probe_get_with_expected_status(StatusCode::NOT_FOUND, format!("{}/test", mock_server.uri()), body.to_string());
    let probe_result = check_endpoint(&probe).await;

    assert_eq!(probe_result.unwrap().success, true);
}

#[tokio::test]
async fn test_requests_post_200_with_body() {
    let mock_server = MockServer::start().await;

    let request_body = "request body";
    let expected_body = "{\"expected_body_field\":\"test\"}";

    Mock::given(method("POST"))
        .and(path("/test"))
        .and(body_string(request_body.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_string(expected_body.to_owned()))
        .mount(&mock_server)
        .await;

    
    let probe = probe_post_with_expected_body(expected_body.to_owned(), format!("{}/test", mock_server.uri()), request_body.to_owned());
    let probe_result = check_endpoint(&probe).await;

    assert_eq!(probe_result.unwrap().success, true);
}

#[cfg(test)]
mod test_utils {
    use std::collections::HashMap;

    use reqwest::StatusCode;

    use crate::probe::{Probe, ProbeInputParameters, ProbeExpectation, ExpectField, ExpectOperation, ProbeScheduleParameters};

    pub fn probe_get_with_expected_status(status_code: StatusCode, url: String, body: String) -> Probe {
        return Probe{
            name: "Test probe".to_string(),
            url: url,
            http_method: "GET".to_string(),
            with: Some(ProbeInputParameters{
                body: Some(body),
                headers: Some(HashMap::new())
            }),
            expectations: Some(vec![ProbeExpectation{
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into()
            }]),
            schedule: ProbeScheduleParameters{
                initial_delay: 0,
                interval: 0
            }
        };
    }

    pub fn probe_post_with_expected_body(expected_body: String, url: String, body: String) -> Probe {
        return Probe{
            name: "Test probe".to_string(),
            url: url,
            http_method: "POST".to_string(),
            with: Some(ProbeInputParameters{
                body: Some(body),
                headers: Some(HashMap::new())
            }),
            expectations: Some(vec![ProbeExpectation{
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: "200".to_owned(),
            },ProbeExpectation{
                field: ExpectField::Body,
                operation: ExpectOperation::Equals,
                value: expected_body,
            }]),
            schedule: ProbeScheduleParameters{
                initial_delay: 0,
                interval: 0
            }
        };
    }
}

// todo: test what happens with different response codes, timeouts etc