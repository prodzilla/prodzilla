use std::sync::Arc;

use chrono::Utc;
use tracing::error;
use tracing::info;

use crate::alerts::outbound_webhook::alert_if_failure;
use crate::app_state;
use crate::probe::expectations;
use crate::probe::model::StepResult;

use super::expectations::validate_response;
use super::http_probe::call_endpoint;
use super::model::ProbeResponse;
use super::model::ProbeResult;
use super::model::ProbeScheduleParameters;
use super::model::Story;
use super::model::Probe;
use super::model::StoryResult;
use std::collections::HashMap;
use crate::AppState;

pub trait Monitorable {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>);
    fn get_name(&self) -> String;
    fn get_schedule(&self) -> &ProbeScheduleParameters;
}

// TODO: Step / Probe can be the same object
// Reduce nesting in this code?

impl Monitorable for Story {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {

        let story_state: HashMap<String,String> = HashMap::new();
        let mut step_results: Vec<StepResult> = vec![];
        let timestamp_started = Utc::now();

        for step in &self.steps {
            // TODO: Overwrite any variables in text

            let call_endpoint_result = call_endpoint(&step.http_method, &step.url, &step.with).await;
            
            match call_endpoint_result {
                Ok(endpoint_result) => {
                    let expectations_result = validate_response(&step.name, &endpoint_result, &step.expectations);

                    step_results.push(StepResult{
                        step_name: step.name.clone(),
                        timestamp_started: endpoint_result.timestamp_request_started,
                        success: expectations_result,
                        response: Some(endpoint_result.to_probe_response())
                    });

                    if !expectations_result {
                        break;
                    }

                    // TODO: Add Variables to State
                    
                },
                Err(e) => {
                    error!("Error calling endpoint: {}", e);
                    step_results.push(StepResult {
                        step_name: step.name.clone(),
                        success: false,
                        timestamp_started: Utc::now(),
                        response: None
                    });
                    break;
                }
            };
        }

        let last_step = step_results.last().unwrap();

        let story_result = StoryResult{
            story_name: self.name.clone(),
            timestamp_started: timestamp_started,
            success: last_step.success,
            step_results: step_results
        };

        app_state.add_story_result(self.name.clone(), story_result);
        
        // Alert if needed

    }

    fn get_name(&self) -> String {
        return self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule
    }
}

impl Monitorable for Probe {
    async fn probe_and_store_result(&self, app_state: Arc<AppState>) {
        
        let call_endpoint_result = call_endpoint(&self.http_method, &self.url, &self.with).await;

        let probe_result;

        match call_endpoint_result {
            Ok(endpoint_result) => {
                let expectations_result = validate_response(&self.name, &endpoint_result, &self.expectations);

                probe_result = ProbeResult{
                    probe_name: self.name.clone(),
                    timestamp_started: endpoint_result.timestamp_request_started,
                    success: expectations_result,
                    response: Some(endpoint_result.to_probe_response())
                };
            },
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

        let send_alert_result = alert_if_failure(self, &probe_result).await;
        if let Err(e) = send_alert_result {
            error!("Error sending out alert: {}", e);
        }

        app_state.add_probe_result(self.name.clone(), probe_result);

    }

    fn get_name(&self) -> String {
        return self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule
    }
}