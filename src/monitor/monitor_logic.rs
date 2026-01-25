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
use crate::monitor::model::StepResult;
use crate::monitor::variables::substitute_input_parameters;
use crate::monitor::variables::substitute_variables;
use crate::monitor::variables::ExecutionContext;
use crate::monitor::variables::StepVariables;
use crate::otel::metrics::MonitorStatus;

use super::expectations::validate_response;
use super::http::call_endpoint;
use super::model::Monitor;
use super::model::MonitorResult;
use super::model::ScheduleParameters;
use crate::AppState;

pub trait Monitorable {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>);
    fn get_name(&self) -> String;
    fn get_schedule(&self) -> &ScheduleParameters;
}

fn time_since(timestamp: &chrono::DateTime<Utc>) -> u64 {
    Utc::now()
        .signed_duration_since(*timestamp)
        .num_milliseconds() as u64
}

impl Monitorable for Monitor {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        let monitor_attributes = [
            KeyValue::new("name", self.name.clone()),
            KeyValue::new("type", "monitor"),
        ]
        .into_iter()
        .chain(self.tags.iter().flat_map(|tags| {
            tags.iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
        }))
        .collect::<Vec<_>>();

        app_state.metrics.runs.add(1, &monitor_attributes);

        let mut execution_context = ExecutionContext::new();
        let mut step_results: Vec<StepResult> = vec![];
        let timestamp_started = Utc::now();

        let tracer = global::tracer("monitor_logic");
        let root_span = tracer.start(self.name.clone());
        let root_cx = Context::default().with_span(root_span);

        let steps = self.get_steps();
        let is_multi_step = self.is_multi_step();

        for step in &steps {
            let step_started = Utc::now();
            let mut step_tags = vec![
                KeyValue::new("name", step.name.clone()),
                KeyValue::new("type", "step"),
            ];

            // Only add monitor_name for multi-step monitors
            if is_multi_step {
                step_tags.push(KeyValue::new("monitor_name", self.name.clone()));
            }

            step_tags.extend(self.tags.iter().flat_map(|tags| {
                tags.iter()
                    .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            }));

            app_state.metrics.runs.add(1, &step_tags);
            let step_span = tracer.start_with_context(step.name.clone(), &root_cx);
            let step_cx = root_cx.with_span(step_span);

            let url = substitute_variables(&step.url, &execution_context);
            let input_parameters = substitute_input_parameters(&step.with, &execution_context);

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
                    let endpoint_response = endpoint_result.to_endpoint_response();
                    let span = step_cx.span();
                    span.set_attribute(opentelemetry::KeyValue::new(
                        semconv::trace::HTTP_RESPONSE_STATUS_CODE,
                        endpoint_result.status_code.to_string(),
                    ));
                    let expectations_result = validate_response(
                        &step.name,
                        endpoint_result.status_code,
                        endpoint_result.body.clone(),
                        &step.expectations,
                    );

                    let mut monitor_status = MonitorStatus::Ok.as_u64();
                    if let Err(err) = expectations_result.as_ref() {
                        span.record_error(&err);
                        span.set_status(Status::Error {
                            description: "Expectation failed".into(),
                        });
                        app_state.metrics.errors.add(1, &step_tags);
                        monitor_status = MonitorStatus::Error.as_u64();
                    } else {
                        // Add 0 to ensure this is exported with value 0, so e.g. rate
                        // queries in promql don't miss the step from 0 -> 1
                        app_state.metrics.errors.add(0, &step_tags);
                        step_cx.span().set_status(Status::Ok);
                    }

                    app_state
                        .metrics
                        .duration
                        .record(time_since(&step_started), &step_tags);

                    app_state
                        .metrics
                        .status
                        .record(monitor_status, &monitor_attributes);

                    let step_result = StepResult {
                        step_name: step.name.clone(),
                        timestamp_started: endpoint_result.timestamp_request_started,
                        success: expectations_result.is_ok(),
                        error_message: expectations_result.as_ref().err().map(|e| e.to_string()),
                        response: Some(endpoint_response.clone()),
                        trace_id: Some(endpoint_result.trace_id),
                        span_id: Some(endpoint_result.span_id),
                    };
                    step_results.push(step_result);

                    if expectations_result.is_err() {
                        break;
                    }

                    // Store step variables for multi-step monitors
                    if is_multi_step {
                        let step_variables = StepVariables {
                            response_body: endpoint_result.body,
                        };
                        execution_context
                            .steps
                            .insert(step.name.clone(), step_variables);
                    }
                }
                Err(e) => {
                    error!("Error calling endpoint: {}", e);
                    app_state.metrics.http_status_code.record(0, &step_tags);
                    app_state.metrics.errors.add(1, &step_tags);
                    app_state
                        .metrics
                        .duration
                        .record(time_since(&step_started), &step_tags);
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
                    break;
                }
            };
        }

        let last_step = step_results.last().unwrap();
        let monitor_success = last_step.success;

        if !monitor_success {
            app_state.metrics.errors.add(1, &monitor_attributes);
            root_cx.span().set_status(Status::Error {
                description: "Expectation failed".into(),
            });
        } else {
            app_state.metrics.errors.add(0, &monitor_attributes);
            root_cx.span().set_status(Status::Ok);
        }

        app_state
            .metrics
            .duration
            .record(time_since(&timestamp_started), &monitor_attributes);

        info!(
            "Finished scheduled monitor {}, success: {}",
            &self.name, monitor_success
        );

        let send_alert_result = alert_if_failure(
            monitor_success,
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

        let monitor_result = MonitorResult {
            monitor_name: self.name.clone(),
            timestamp_started,
            success: monitor_success,
            step_results,
        };

        app_state.add_monitor_result(self.name.clone(), monitor_result);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_schedule(&self) -> &ScheduleParameters {
        &self.schedule
    }
}

#[cfg(test)]
mod monitor_logic_tests {

    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::app_state::AppState;
    use crate::config::Config;
    use crate::monitor::model::{
        Alert, ExpectField, ExpectOperation, Expectation, InputParameters, Monitor,
        ScheduleParameters, Step,
    };
    use crate::monitor::monitor_logic::Monitorable;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_multi_step_monitor_success() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let monitor_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config { monitors: vec![] }));

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

        let monitor = Monitor {
            name: monitor_name.to_owned(),
            url: None,
            http_method: None,
            with: None,
            expectations: None,
            sensitive: false,
            steps: Some(vec![
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
            ]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            tags: None,
            alerts: None,
        };

        monitor.probe_and_store_result(app_state.clone()).await;

        let monitor_result_map = app_state.monitor_results.read().unwrap();
        let results = &monitor_result_map[monitor_name];
        assert_eq!(1, results.len());
        let monitor_result = &results[0];
        assert!(monitor_result.success);
        assert_eq!(2, monitor_result.step_results.len());
    }

    #[tokio::test]
    async fn test_multi_step_monitor_second_step_fails() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let alert_path = "/alert-test";
        let monitor_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config { monitors: vec![] }));

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

        let monitor = Monitor {
            name: monitor_name.to_owned(),
            url: None,
            http_method: None,
            with: None,
            expectations: None,
            sensitive: false,
            steps: Some(vec![
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
                    expectations: Some(vec![Expectation {
                        field: ExpectField::StatusCode,
                        operation: ExpectOperation::Equals,
                        value: "200".to_owned(),
                    }]),
                    sensitive: false,
                },
            ]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: Some(vec![Alert {
                url: format!("{}{}", mock_server.uri(), alert_path.to_owned()),
            }]),
            tags: None,
        };

        monitor.probe_and_store_result(app_state.clone()).await;

        let monitor_result_map = app_state.monitor_results.read().unwrap();
        let results = &monitor_result_map[monitor_name];
        assert_eq!(1, results.len());
        let monitor_result = &results[0];
        assert!(!monitor_result.success);
        assert_eq!(2, monitor_result.step_results.len());
    }

    #[tokio::test]
    async fn test_multi_step_monitor_passes_all_variables() {
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

        let monitor_name = "User Flow";
        let app_state = Arc::new(AppState::new(Config { monitors: vec![] }));

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

        let monitor = Monitor {
            name: monitor_name.to_owned(),
            url: None,
            http_method: None,
            with: None,
            expectations: None,
            sensitive: false,
            steps: Some(vec![
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
                    with: Some(InputParameters {
                        headers: Some(step2_headers),
                        body: Some(step2_body_str.to_owned()),
                        timeout_seconds: None,
                    }),
                    http_method: "POST".to_owned(),
                    expectations: Some(vec![Expectation {
                        field: ExpectField::StatusCode,
                        operation: ExpectOperation::Equals,
                        value: "200".to_owned(),
                    }]),
                    sensitive: false,
                },
            ]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
        };

        monitor.probe_and_store_result(app_state.clone()).await;

        let monitor_result_map = app_state.monitor_results.read().unwrap();
        let results = &monitor_result_map[monitor_name];
        assert_eq!(1, results.len());
        let monitor_result = &results[0];
        assert!(monitor_result.success);
        assert_eq!(2, monitor_result.step_results.len());
    }
}
