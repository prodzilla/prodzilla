use std::sync::Arc;

use chrono::Utc;
use tracing::error;
use tracing::info;

use crate::alerts::outbound_webhook::alert_if_failure;
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

// TODOs here: Step / Probe can be the same object
// The timestamps are a little disorganised
// Reduce nested code
// Kill all the .clone() - I think the source of truth is the StepResult values?

impl Monitorable for Story {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        let mut story_variables = StoryVariables::new();
        let mut step_results: Vec<StepResult> = vec![];
        let timestamp_started = Utc::now();

        for step in &self.steps {
            
            let url = substitute_variables(&step.url, &story_variables);
            let input_parameters = substitute_input_parameters(&step.with, &story_variables);

            let call_endpoint_result =
                call_endpoint(&step.http_method, &url, &input_parameters).await;

            match call_endpoint_result {
                Ok(endpoint_result) => {
                    let expectations_result =
                        validate_response(&step.name, &endpoint_result, &step.expectations);

                    let step_result = StepResult {
                        step_name: step.name.clone(),
                        timestamp_started: endpoint_result.timestamp_request_started,
                        success: expectations_result,
                        response: Some(endpoint_result.to_probe_response()),
                    };
                    step_results.push(step_result);

                    if !expectations_result {
                        break;
                    }

                    let step_variables = StepVariables{
                        response_body: step_results.last().unwrap().response.clone().unwrap().body
                    };
                    story_variables.steps.insert(step.name.clone(), step_variables);
                }
                Err(e) => {
                    error!("Error calling endpoint: {}", e);
                    step_results.push(StepResult {
                        step_name: step.name.clone(),
                        success: false,
                        timestamp_started: Utc::now(),
                        response: None,
                    });
                    break;
                }
            };
        }

        let story_success = step_results.last().unwrap().success;

        let story_result = StoryResult {
            story_name: self.name.clone(),
            timestamp_started: timestamp_started,
            success: story_success,
            step_results: step_results,
        };

        app_state.add_story_result(self.name.clone(), story_result);

        info!(
            "Finished scheduled story {}, success: {}",
            &self.name, story_success
        );

        let send_alert_result =
            alert_if_failure(story_success, &self.name, timestamp_started, &self.alerts).await;
        if let Err(e) = send_alert_result {
            error!("Error sending out alert: {}", e);
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule;
    }
}

impl Monitorable for Probe {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        let call_endpoint_result = call_endpoint(&self.http_method, &self.url, &self.with).await;

        let probe_result;

        match call_endpoint_result {
            Ok(endpoint_result) => {
                let expectations_result =
                    validate_response(&self.name, &endpoint_result, &self.expectations);

                probe_result = ProbeResult {
                    probe_name: self.name.clone(),
                    timestamp_started: endpoint_result.timestamp_request_started,
                    success: expectations_result,
                    response: Some(endpoint_result.to_probe_response()),
                };
            }
            Err(e) => {
                error!("Error calling endpoint: {}", e);
                probe_result = ProbeResult {
                    probe_name: self.name.clone(),
                    timestamp_started: Utc::now(),
                    success: false,
                    response: None,
                };
            }
        };

        let success = probe_result.success;
        let timestamp = probe_result.timestamp_started;

        app_state.add_probe_result(self.name.clone(), probe_result);

        info!(
            "Finished scheduled probe {}, success: {}",
            &self.name, success
        );

        let send_alert_result =
            alert_if_failure(success, &self.name, timestamp, &self.alerts).await;
        if let Err(e) = send_alert_result {
            error!("Error sending out alert: {}", e);
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule;
    }
}

#[cfg(test)]
mod probe_logic_tests {

    use std::sync::Arc;

    use crate::app_state::AppState;
    use crate::probe::model::{ExpectField, ExpectOperation, ProbeAlert, ProbeExpectation, ProbeScheduleParameters, Step, Story};
    use crate::probe::probe_logic::Monitorable;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_story_success() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let story_name = "User Flow";
        let app_state = Arc::new(AppState::new());

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
                },
                Step {
                    name: "Step 2".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step2_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: None,
                },
            ],
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
        };

        story.probe_and_store_result(app_state.clone()).await;

        let story_result_map = app_state.story_results.read().unwrap();
        let results = &story_result_map[story_name];
        assert_eq!(1, results.len());
        let story_result = &results[0];
        assert_eq!(true, story_result.success);
        assert_eq!(2, story_result.step_results.len());
    }

    #[tokio::test]
    async fn test_story_second_step_fails() {
        let mock_server = MockServer::start().await;
        let step1_path = "/test1";
        let step2_path = "/test2";
        let alert_path = "/alert-test";
        let story_name = "User Flow";
        let app_state = Arc::new(AppState::new());

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
                },
                Step {
                    name: "Step 2".to_owned(),
                    url: format!("{}{}", mock_server.uri(), step2_path.to_owned()),
                    with: None,
                    http_method: "GET".to_owned(),
                    expectations: Some(
                        vec![
                            ProbeExpectation{
                                field: ExpectField::StatusCode,
                                operation: ExpectOperation::Equals,
                                value: "200".to_owned(),
                            }
                        ]
                    ),
                },
            ],
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: Some(vec![ProbeAlert {
                url: format!("{}{}", mock_server.uri(), alert_path.to_owned()),
            }]),
        };

        story.probe_and_store_result(app_state.clone()).await;

        let story_result_map = app_state.story_results.read().unwrap();
        let results = &story_result_map[story_name];
        assert_eq!(1, results.len());
        let story_result = &results[0];
        assert_eq!(false, story_result.success);
        assert_eq!(2, story_result.step_results.len());

    }

    // let alert_url = "/alert-test";

    //     Mock::given(method("POST"))
    //         .and(path(alert_url))
    //         .respond_with(ResponseTemplate::new(200))
    //         .expect(1)
    //         .mount(&mock_server)
    //         .await;

    //     let probe_name = "Some Flow".to_owned();
    //     let alerts = Some(vec![ProbeAlert {
    //         url: format!("{}{}", mock_server.uri(), alert_url.to_owned()),
    //     }]);
    //     let failure_timestamp = Utc::now();

    //     let alert_result = alert_if_failure(false, &probe_name, failure_timestamp, &alerts).await;

    //     assert!(alert_result.is_ok());
}
