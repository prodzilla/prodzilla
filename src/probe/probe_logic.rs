use std::sync::Arc;

use chrono::Utc;
use tracing::error;
use tracing::info;

use crate::alerts::outbound_webhook::alert_if_failure;

use super::expectations::validate_response;
use super::http_probe::call_endpoint;
use super::model::ProbeResponse;
use super::model::ProbeResult;
use super::model::ProbeScheduleParameters;
use super::model::Story;
use super::model::Probe;
use std::collections::HashMap;
use crate::AppState;

pub trait Monitorable {
    async fn probe(&self, app_state: Arc<AppState>);
    fn get_name(&self) -> String;
    fn get_schedule(&self) -> &ProbeScheduleParameters;
}

impl Monitorable for Story {
    async fn probe(&self, _app_state: Arc<AppState>) {
        // Implementation for Story

        // set up hashmap of steps to json objects?
        let story_state: HashMap<String,String> = HashMap::new();

        for step in &self.steps {
            // Overwrite any variables in text

            // Execute Request


            // Check Expectations

            // Add Variables to State
        }
        // for each step

        // TODO: Implement stories, 
        // keep track of previous response bodies1
        info!("Performing check on a Story");
    }

    fn get_name(&self) -> String {
        return self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule
    }
}

impl Monitorable for Probe {
    async fn probe(&self, app_state: Arc<AppState>) {
        
        let call_endpoint_result = call_endpoint(&self.http_method, &self.url, &self.with).await;

        let probe_result;

        match call_endpoint_result {
            Ok(endpoint_result) => {
                let expectations_result = validate_response(&self.name, &endpoint_result, &self.expectations);

                probe_result = ProbeResult{
                    probe_name: self.name.clone(),
                    timestamp_started: endpoint_result.timestamp_request_started,
                    success: expectations_result,
                    response: Some(ProbeResponse {
                        timestamp: endpoint_result.timestamp_response_received,
                        status_code: endpoint_result.status_code,
                        body: endpoint_result.body,
                    })
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