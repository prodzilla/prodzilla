use std::str::FromStr;
use std::time::Duration;

use crate::errors::MapToSendError;
use crate::expectations::validate_response;
use crate::probe::Probe;
use crate::probe::ProbeResponse;
use crate::probe::ProbeResult;
use chrono::Utc;
use lazy_static::lazy_static;
use reqwest::RequestBuilder;
use tracing::debug;
use tracing::error;

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla Probe/1.0")
        .build()
        .unwrap();
}

pub async fn check_endpoint(
    probe: &Probe,
) -> Result<ProbeResult, Box<dyn std::error::Error + Send>> {
    let timestamp_start = Utc::now();

    let request = build_request(probe)?;
    let response = request
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await;

    let timestamp_response = Utc::now();

    match response {
        Ok(res) => {
            let status_code = res.status();
            let body = res.text().await.map_to_send_err()?;
            let probe_response = ProbeResponse {
                timestamp: timestamp_response,
                status_code: status_code.as_u16() as u32,
                body: body.clone(),
            };

            let validation_result: bool;

            match &probe.expectations {
                Some(expect_back) => {
                    validation_result = validate_response(&expect_back, status_code, &body);
                    if validation_result {
                        debug!("Successful response for {}, as expected", &probe.name);
                    } else {
                        debug!("Successful response for {}, not as expected!", &probe.name);
                    }
                }
                None => {
                    debug!(
                        "Successfully probed {}, no expectation so success is true",
                        &probe.name
                    );
                    validation_result = true;
                }
            }

            return Ok(ProbeResult {
                probe_name: probe.name.clone(),
                success: validation_result,
                response: Some(probe_response),
                timestamp_started: timestamp_start,
            });
        }
        Err(e) => {
            error!("Error whilst executing probe: {}", e);
            return Ok(ProbeResult {
                probe_name: probe.name.clone(),
                success: false,
                response: None,
                timestamp_started: timestamp_start,
            });
        }
    }
}

fn build_request(probe: &Probe) -> Result<RequestBuilder, Box<dyn std::error::Error + Send>> {
    let method = reqwest::Method::from_str(&probe.http_method).map_to_send_err()?;

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

#[cfg(test)]
mod http_tests {

    use std::time::Duration;

    use crate::http_probe::check_endpoint;
    use crate::test_utils::test_utils::{
        probe_get_with_expected_status, probe_post_with_expected_body,
    };

    use reqwest::StatusCode;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_requests_get_200() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_expected_status(
            StatusCode::OK,
            format!("{}/test", mock_server.uri()),
            "".to_owned(),
        );
        let probe_result = check_endpoint(&probe).await;

        assert_eq!(probe_result.unwrap().success, true);
    }

    #[tokio::test]
    async fn test_requests_get_timeout() {
        let mock_server = MockServer::start().await;

        let body = "test body";

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(404).set_delay(Duration::from_secs(30)))
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_expected_status(
            StatusCode::NOT_FOUND,
            format!("{}/test", mock_server.uri()),
            body.to_string(),
        );
        let probe_result = check_endpoint(&probe).await;

        assert_eq!(probe_result.unwrap().success, false);
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

        let probe = probe_get_with_expected_status(
            StatusCode::NOT_FOUND,
            format!("{}/test", mock_server.uri()),
            body.to_string(),
        );
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

        let probe = probe_post_with_expected_body(
            expected_body.to_owned(),
            format!("{}/test", mock_server.uri()),
            request_body.to_owned(),
        );
        let probe_result = check_endpoint(&probe).await;

        assert_eq!(probe_result.unwrap().success, true);
    }
}
