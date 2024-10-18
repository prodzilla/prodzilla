use std::sync::Arc;

use chrono::Utc;
use opentelemetry::global;
use opentelemetry::trace;
use opentelemetry::trace::FutureExt;
use opentelemetry::trace::Status;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::trace::Tracer;
use opentelemetry::Context;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions as semconv;
use tracing::error;
use tracing::info;

use crate::alerts::outbound_webhook::alert_if_failure;
use crate::otel::metrics::MonitorStatus;
use crate::probe::model::StepResult;
use crate::probe::variables::substitute_input_parameters;
use crate::probe::variables::substitute_variables;
use crate::probe::variables::StepVariables;
use crate::probe::variables::StoryVariables;

use super::expectations::validate_response;
use super::http_probe::call_endpoint;
use super::model::Probe;
use super::model::ProbeResult;
use super::model::ProbeScheduleParameters;
use super::model::Story;
use super::model::StoryResult;
use crate::AppState;

pub trait Monitorable {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>);
    fn get_name(&self) -> String;
    fn get_schedule(&self) -> &ProbeScheduleParameters;
}

fn time_since(timestamp: &chrono::DateTime<Utc>) -> u64 {
    Utc::now()
        .signed_duration_since(*timestamp)
        .num_milliseconds() as u64
}

// TODOs here: Step / Probe can be the same object
// The timestamps are a little disorganised
// Reduce nested code
// Kill all the .clone() - I think the source of truth is the StepResult values?

impl Monitorable for Story {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        let story_attributes = [
            KeyValue::new("name", self.name.clone()),
            KeyValue::new("type", "story"),
        ]
        .into_iter()
        .chain(self.tags.iter().flat_map(|tags| {
            tags.iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
        }))
        .collect::<Vec<_>>();
        app_state.metrics.runs.add(1, &story_attributes);
        let mut story_variables = StoryVariables::new();
        let mut step_results: Vec<StepResult> = vec![];
        let timestamp_started = Utc::now();

        let tracer = global::tracer("probe_logic");
        let root_span = tracer.start(self.name.clone());
        let root_cx = Context::default().with_span(root_span);
        for step in &self.steps {
            let step_started = Utc::now();
            let step_tags = [
                KeyValue::new("name", step.name.clone()),
                KeyValue::new("story_name", self.name.clone()),
                KeyValue::new("type", "step"),
            ]
            .into_iter()
            .chain(self.tags.iter().flat_map(|tags| {
                tags.iter()
                    .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            }))
            .collect::<Vec<_>>();

            app_state.metrics.runs.add(1, &step_tags);
            let step_span = tracer.start_with_context(step.name.clone(), &root_cx);
            let step_cx = root_cx.with_span(step_span);

            let url = substitute_variables(&step.url, &story_variables);
            let input_parameters = substitute_input_parameters(&step.with, &story_variables);

            let call_endpoint_result =
                call_endpoint(&step.http_method, &url, &input_parameters, step.sensitive)
                    .with_context(step_cx.clone())
                    .await;

            match call_endpoint_result {
                Ok(endpoint_result) => {
                    app_state
                        .metrics
                        .http_status_code
                        .record(endpoint_result.status_code.into(), &step_tags);
                    let probe_response = endpoint_result.to_probe_response();
                    let span = step_cx.span();
                    span.set_attribute(opentelemetry::KeyValue::new(
                        semconv::trace::HTTP_RESPONSE_STATUS_CODE,
                        endpoint_result.status_code.to_string(),
                    ));
                    let expectations_result = validate_response(
                        &step.name,
                        endpoint_result.status_code,
                        endpoint_result.body,
                        &step.expectations,
                    );
                    let mut monitor_status = MonitorStatus::Ok.as_u64();
                    if let Err(err) = expectations_result.as_ref() {
                        span.record_error(&err);
                        span.set_status(Status::Error {
                            description: "Expectation failed".into(),
                        });
                        app_state
                            .metrics
                            .duration
                            .record(time_since(&step_started), &step_tags);
                        app_state.metrics.errors.add(1, &step_tags);
                        monitor_status = MonitorStatus::Error.as_u64();
                    }
                    app_state
                        .metrics
                        .status
                        .record(monitor_status, &story_attributes);

                    let step_result = StepResult {
                        step_name: step.name.clone(),
                        timestamp_started: endpoint_result.timestamp_request_started,
                        success: expectations_result.is_ok(),
                        error_message: expectations_result.as_ref().err().map(|e| e.to_string()),
                        response: Some(probe_response),
                        trace_id: Some(endpoint_result.trace_id),
                        span_id: Some(endpoint_result.span_id),
                    };
                    step_results.push(step_result);

                    if expectations_result.is_err() {
                        break;
                    }

                    // Add 0 to ensure this is exported with value 0, so e.g. rate
                    // queries in promql don't miss the step from 0 -> 1
                    app_state.metrics.errors.add(0, &step_tags);
                    step_cx.span().set_status(Status::Ok);
                    let step_variables = StepVariables {
                        response_body: step_results.last().unwrap().response.clone().unwrap().body,
                    };
                    story_variables
                        .steps
                        .insert(step.name.clone(), step_variables);
                    app_state
                        .metrics
                        .duration
                        .record(time_since(&timestamp_started), &step_tags);
                }
                Err(e) => {
                    error!("Error calling endpoint: {}", e);
                    app_state.metrics.http_status_code.record(0, &step_tags);
                    trace::get_active_span(|span| {
                        span.record_error(&*e);
                    });
                    step_results.push(StepResult {
                        step_name: step.name.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        timestamp_started: Utc::now(),
                        response: None,
                        trace_id: None,
                        span_id: None,
                    });
                    app_state
                        .metrics
                        .duration
                        .record(time_since(&timestamp_started), &step_tags);
                    break;
                }
            };
        }
        let last_step = step_results.last().unwrap();
        let story_success = last_step.success;
        if !story_success {
            app_state.metrics.errors.add(1, &story_attributes);
        } else {
            app_state.metrics.errors.add(0, &story_attributes);
        }
        app_state
            .metrics
            .duration
            .record(time_since(&timestamp_started), &story_attributes);

        info!(
            "Finished scheduled story {}, success: {}",
            &self.name, story_success
        );

        let send_alert_result = alert_if_failure(
            story_success,
            last_step.error_message.as_deref(),
            last_step.response.as_ref(),
            &self.name,
            timestamp_started,
            &self.alerts,
            &last_step.trace_id,
        )
        .await;
        if let Err(e) = send_alert_result {
            for error in e {
                error!("Error sending out alert: {}", error);
            }
        }
        let story_result = StoryResult {
            story_name: self.name.clone(),
            timestamp_started,
            success: story_success,
            step_results,
        };

        app_state.add_story_result(self.name.clone(), story_result);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        &self.schedule
    }
}

impl Monitorable for Probe {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        let probe_attributes = [
            KeyValue::new("name", self.name.clone()),
            KeyValue::new("type", "probe"),
        ]
        .into_iter()
        .chain(self.tags.iter().flat_map(|tags| {
            tags.iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
        }))
        .collect::<Vec<_>>();
        app_state.metrics.runs.add(1, &probe_attributes);

        let root_span = global::tracer("probe_logic").start(self.name.clone());

        let root_cx = Context::default().with_span(root_span);
        let call_endpoint_result =
            call_endpoint(&self.http_method, &self.url, &self.with, self.sensitive)
                .with_context(root_cx.clone())
                .await;

        let probe_result = match call_endpoint_result {
            Ok(endpoint_result) => {
                app_state
                    .metrics
                    .http_status_code
                    .record(endpoint_result.status_code.into(), &probe_attributes);
                let probe_response = endpoint_result.to_probe_response();
                let expectations_result = validate_response(
                    &self.name,
                    endpoint_result.status_code,
                    endpoint_result.body,
                    &self.expectations,
                );

                if let Err(err) = expectations_result.as_ref() {
                    root_cx.span().record_error(&err);
                }

                let mut monitor_status = MonitorStatus::Ok.as_u64();
                if expectations_result.is_err() {
                    monitor_status = MonitorStatus::Error.as_u64();
                }
                app_state
                    .metrics
                    .status
                    .record(monitor_status, &probe_attributes);

                ProbeResult {
                    probe_name: self.name.clone(),
                    timestamp_started: endpoint_result.timestamp_request_started,
                    success: expectations_result.is_ok(),
                    error_message: expectations_result.err().map(|e| e.to_string()),
                    response: Some(probe_response),
                    trace_id: Some(endpoint_result.trace_id),
                }
            }
            Err(e) => {
                app_state
                    .metrics
                    .http_status_code
                    .record(0, &probe_attributes);
                app_state
                    .metrics
                    .status
                    .record(MonitorStatus::Error.as_u64(), &probe_attributes);
                error!("Error calling endpoint: {}", e);
                root_cx.span().record_error(&*e);
                ProbeResult {
                    success: false,
                    probe_name: self.name.clone(),
                    timestamp_started: Utc::now(),
                    error_message: Some(e.to_string()),
                    response: None,
                    trace_id: None,
                }
            }
        };

        if probe_result.success {
            app_state.metrics.errors.add(0, &probe_attributes);
            root_cx.span().set_status(Status::Ok);
        } else {
            app_state.metrics.errors.add(1, &probe_attributes);
            root_cx.span().set_status(Status::Error {
                description: "Expectation failed".into(),
            });
        }
        let timestamp = probe_result.timestamp_started;

        app_state
            .metrics
            .duration
            .record(time_since(&timestamp), &probe_attributes);

        info!(
            "Finished scheduled probe {}, success: {}",
            &self.name, probe_result.success,
        );

        let send_alert_result = alert_if_failure(
            probe_result.success,
            probe_result.error_message.as_deref(),
            probe_result.response.as_ref(),
            &self.name,
            timestamp,
            &self.alerts,
            &probe_result.trace_id,
        )
        .await;
        if let Err(e) = send_alert_result {
            for error in e {
                error!("Error sending out alert: {}", error);
            }
        }
        app_state.add_probe_result(self.name.clone(), probe_result);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        &self.schedule
    }
}

#[cfg(test)]
mod probe_logic_tests {

    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::app_state::AppState;
    use crate::config::Config;
    use crate::probe::model::{
        ExpectField, ExpectOperation, ProbeAlert, ProbeExpectation, ProbeInputParameters,
        ProbeScheduleParameters, Step, Story,
    };
    use crate::probe::probe_logic::Monitorable;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_story_success() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let story_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config {
            probes: vec![],
            stories: vec![],
        }));

        Mock::given(method("GET"))
            .and(path(step1_path))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(step2_path))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let story = Story {
            name: story_name.to_owned(),
            steps: vec![
                Step {
                    name: "Step 1".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step1_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: None,
                    sensitive: false,
                },
                Step {
                    name: "Step 2".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step2_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: None,
                    sensitive: false,
                },
            ],
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            tags: None,
            alerts: None,
        };

        story.probe_and_store_result(app_state.clone()).await;

        let story_result_map = app_state.story_results.read().unwrap();
        let results = &story_result_map[story_name];
        assert_eq!(1, results.len());
        let story_result = &results[0];
        assert!(story_result.success);
        assert_eq!(2, story_result.step_results.len());
    }

    #[tokio::test]
    async fn test_story_second_step_fails() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let alert_path = "/alert-test";
        let story_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config {
            probes: vec![],
            stories: vec![],
        }));

        Mock::given(method("GET"))
            .and(path(step1_path))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path(alert_path))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let story = Story {
            name: story_name.to_owned(),
            steps: vec![
                Step {
                    name: "Step 1".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step1_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: None,
                    sensitive: false,
                },
                Step {
                    name: "Step 2".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step2_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: Some(vec![ProbeExpectation {
                        field: ExpectField::StatusCode,
                        operation: ExpectOperation::Equals,
                        value: "200".to_owned(),
                    }]),
                    sensitive: false,
                },
            ],
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: Some(vec![ProbeAlert {
                url: format!("{}{}", mock_server.uri(), alert_path.to_owned()),
            }]),
            tags: None,
        };

        story.probe_and_store_result(app_state.clone()).await;

        let story_result_map = app_state.story_results.read().unwrap();
        let results = &story_result_map[story_name];
        assert_eq!(1, results.len());
        let story_result = &results[0];
        assert!(!story_result.success);
        assert_eq!(2, story_result.step_results.len());
    }

    #[tokio::test]
    async fn test_story_passes_all_variables() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step1_response_body_str = r#"{
            "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
            "path": "value"
        }"#;

        let step2_path = "/${{steps.step1.response.body.path}}/test2";
        let step2_constructed_path = "/value/test2";
        let step2_headers = HashMap::from([(
            "Authorization".to_owned(),
            "Bearer ${{steps.step1.response.body.token}}".to_owned(),
        )]);
        let step2_body_str = r#"{"uuid": "${{generate.uuid}}"}"#;

        let story_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config {
            probes: vec![],
            stories: vec![],
        }));

        Mock::given(method("GET"))
            .and(path(step1_path))
            .respond_with(ResponseTemplate::new(200).set_body_string(step1_response_body_str))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path(step2_constructed_path))
            .and(header("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let story = Story {
            name: story_name.to_owned(),
            steps: vec![
                Step {
                    name: "step1".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step1_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: None,
                    sensitive: false,
                },
                Step {
                    name: "Step 2".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step2_path.to_owned()),
                    with: Some(ProbeInputParameters {
                        headers: Some(step2_headers),
                        body: Some(step2_body_str.to_owned()),
                        timeout_seconds: None,
                    }),
                    http_method: "POST".to_owned(),
                    expectations: Some(vec![ProbeExpectation {
                        field: ExpectField::StatusCode,
                        operation: ExpectOperation::Equals,
                        value: "200".to_owned(),
                    }]),
                    sensitive: false,
                },
            ],
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
        };

        story.probe_and_store_result(app_state.clone()).await;

        let story_result_map = app_state.story_results.read().unwrap();
        let results = &story_result_map[story_name];
        assert_eq!(1, results.len());
        let story_result = &results[0];
        assert!(story_result.success);
        assert_eq!(2, story_result.step_results.len());
    }
}
