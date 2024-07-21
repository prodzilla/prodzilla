use std::str::FromStr;
use std::time::Duration;

use crate::errors::MapToSendError;
use chrono::Utc;
use lazy_static::lazy_static;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions::trace as semconv;

use opentelemetry::trace::FutureExt;
use opentelemetry::trace::Span;
use opentelemetry::trace::SpanId;
use opentelemetry::trace::TraceId;

use reqwest::header::HeaderMap;
use reqwest::RequestBuilder;

use super::model::EndpointResult;
use super::model::ProbeInputParameters;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use opentelemetry::{global, trace::Tracer};

const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 10;

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
    sensitive: bool,
) -> Result<EndpointResult, Box<dyn std::error::Error + Send>> {
    let timestamp_start = Utc::now();
    let (otel_headers, cx, span_id, trace_id) =
        get_otel_headers(format!("{} {}", http_method, url));

    let request = build_request(http_method, url, input_parameters, otel_headers)?;
    let request_timeout = Duration::from_secs(
        input_parameters
            .as_ref()
            .and_then(|params| params.timeout_seconds)
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS),
    );
    let response = request
        .timeout(request_timeout)
        .send()
        .with_context(cx.clone())
        .await
        .map_to_send_err()?;

    let timestamp_response = Utc::now();

    let result = EndpointResult {
        timestamp_request_started: timestamp_start,
        timestamp_response_received: timestamp_response,
        status_code: response.status().as_u16() as u32,
        body: response.text().await.map_to_send_err()?,
        sensitive,
        trace_id: trace_id.to_string(),
        span_id: span_id.to_string(),
    };
    let span = cx.span();
    span.set_attributes(vec![
        KeyValue::new(semconv::HTTP_METHOD, http_method.to_owned()),
        KeyValue::new(semconv::HTTP_URL, url.clone()),
    ]);
    span.set_attribute(KeyValue::new(
        semconv::HTTP_STATUS_CODE,
        result.status_code.to_string(),
    ));
    if !sensitive {
        span.add_event(
            "response",
            vec![KeyValue::new(
                "body",
                result.body.chars().take(500).collect::<String>(),
            )],
        )
    }

    Ok(result)
}

fn get_otel_headers(span_name: String) -> (HeaderMap, Context, SpanId, TraceId) {
    let span = global::tracer("http_probe").start(span_name);
    let span_id = span.span_context().span_id();
    let trace_id = span.span_context().trace_id();
    let cx = Context::current_with_span(span);

    let mut headers = HeaderMap::new();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut opentelemetry_http::HeaderInjector(&mut headers));
    });

    (headers, cx, span_id, trace_id)
}

fn build_request(
    http_method: &str,
    url: &String,
    input_parameters: &Option<ProbeInputParameters>,
    otel_headers: HeaderMap,
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

    use std::env;
    use std::time::Duration;

    use crate::otel;
    use crate::probe::expectations::validate_response;
    use crate::probe::http_probe::call_endpoint;
    use crate::test_utils::probe_test_utils::{
        probe_get_with_expected_status, probe_get_with_timeout_and_expected_status,
        probe_post_with_expected_body,
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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with, false)
            .await
            .unwrap();
        let check_expectations_result = validate_response(
            &probe.name,
            endpoint_result.status_code,
            endpoint_result.body,
            &probe.expectations,
        );

        assert!(check_expectations_result.is_ok());
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
        let endpoint_result =
            call_endpoint(&probe.http_method, &probe.url, &probe.with, false).await;

        assert!(endpoint_result.is_err());
    }

    #[tokio::test]
    async fn test_request_timeout_configuration() {
        let mock_server = MockServer::start().await;

        let body = "test body";

        Mock::given(method("GET"))
            .and(path("/five_second_response"))
            .respond_with(ResponseTemplate::new(404).set_delay(Duration::from_secs(5)))
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_timeout_and_expected_status(
            StatusCode::NOT_FOUND,
            format!("{}/five_second_response", mock_server.uri()),
            body.to_string(),
            Some(1), // Timeout is 1 second, reduced from default of 10
        );
        let endpoint_result =
            call_endpoint(&probe.http_method, &probe.url, &probe.with, false).await;

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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with, false)
            .await
            .unwrap();
        let check_expectations_result = validate_response(
            &probe.name,
            endpoint_result.status_code,
            endpoint_result.body,
            &probe.expectations,
        );

        assert!(check_expectations_result.is_ok());
    }

    #[tokio::test]
    async fn test_requests_post_200_with_body() {
        // necessary for trace propagation
        env::set_var("OTEL_TRACES_EXPORTER", "otlp");
        otel::tracing::create_tracer();
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
        let endpoint_result = call_endpoint(&probe.http_method, &probe.url, &probe.with, false)
            .await
            .unwrap();
        let check_expectations_result = validate_response(
            &probe.name,
            endpoint_result.status_code,
            endpoint_result.body,
            &probe.expectations,
        );

        assert!(check_expectations_result.is_ok());
    }
}
