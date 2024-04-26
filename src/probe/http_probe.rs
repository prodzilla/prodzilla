use std::str::FromStr;
use std::time::Duration;

use crate::errors::MapToSendError;
use chrono::Utc;
use lazy_static::lazy_static;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use reqwest::header::HeaderMap;
use reqwest::RequestBuilder;

use super::model::EndpointResult;
use super::model::ProbeInputParameters;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use opentelemetry::{global, trace::Tracer};

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla Probe/1.0")
        .build()
        .unwrap();
}

pub async fn call_endpoint(
    http_method: &str,
    url: &String,
    input_parameters: &Option<ProbeInputParameters>,
) -> Result<EndpointResult, Box<dyn std::error::Error + Send>> {
    let timestamp_start = Utc::now();
    let (otel_headers, trace_id) = get_otel_headers();

    let request = build_request(http_method, url, input_parameters, otel_headers)?;
    let response = request
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_to_send_err()?;

    let timestamp_response = Utc::now();

    Ok(EndpointResult {
        timestamp_request_started: timestamp_start,
        timestamp_response_received: timestamp_response,
        status_code: response.status().as_u16() as u32,
        body: response.text().await.map_to_send_err()?,
        trace_id
    })
}

fn get_otel_headers() -> (HeaderMap, String) {
    
    let tracer = global::tracer("prodzilla_tracer");
    let span = tracer.start("prodzilla_call");
    let cx = Context::current_with_span(span);

    let mut headers = HeaderMap::new();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut opentelemetry_http::HeaderInjector(&mut headers));
    });

    let trace_id = cx.span().span_context().trace_id().to_string();

    (headers, trace_id)
}

// Needs to be called to enable trace ids
pub fn init_otel_tracing() {
    let provider = TracerProvider::default();
    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
}

fn build_request(
    http_method: &str,
    url: &String,
    input_parameters: &Option<ProbeInputParameters>,
    otel_headers: HeaderMap
) -> Result<RequestBuilder, Box<dyn std::error::Error + Send>> {
    let method = reqwest::Method::from_str(http_method).map_to_send_err()?;

    let mut request = CLIENT.request(method, url);
    request = request.headers(otel_headers);

    if let Some(probe_input_parameters) = input_parameters {
        if let Some(body) = &probe_input_parameters.body {
            request = request.body(body.clone());
        }
        if let Some(headers) = &probe_input_parameters.headers {
            for (key, value) in headers.clone().iter() {
                request = request.header(key, value);
            }
        }
    }

    Ok(request)
}

#[cfg(test)]
mod http_tests {

    use std::time::Duration;

    use crate::probe::expectations::validate_response;
    use crate::probe::http_probe::{call_endpoint, init_otel_tracing};
    use crate::test_utils::probe_test_utils::{
        probe_get_with_expected_status, probe_post_with_expected_body,
    };

    use reqwest::StatusCode;
    use wiremock::matchers::{body_string, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Note: These tests are a bit odd because they have been updated since a refactor

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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with)
            .await
            .unwrap();
        let check_expectations_result =
            validate_response(&probe.name, &endpoint_result, &probe.expectations);

        assert!(check_expectations_result);
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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with).await;

        assert!(endpoint_result.is_err());
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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with)
            .await
            .unwrap();
        let check_expectations_result =
            validate_response(&probe.name, &endpoint_result, &probe.expectations);

        assert!(check_expectations_result);
    }

    #[tokio::test]
    async fn test_requests_post_200_with_body() {
        init_otel_tracing();
        let mock_server = MockServer::start().await;

        let request_body = "request body";
        let expected_body = "{\"expected_body_field\":\"test\"}";

        Mock::given(method("POST"))
            .and(path("/test"))
            .and(body_string(request_body.to_string()))
            .and(header_exists("traceparent"))
            .respond_with(ResponseTemplate::new(200).set_body_string(expected_body.to_owned()))
            .expect(1)
            .mount(&mock_server)
            .await;

        let probe = probe_post_with_expected_body(
            expected_body.to_owned(),
            format!("{}/test", mock_server.uri()),
            request_body.to_owned(),
        );
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with)
            .await
            .unwrap();
        let check_expectations_result =
            validate_response(&probe.name, &endpoint_result, &probe.expectations);

        assert!(check_expectations_result);
    }
}
