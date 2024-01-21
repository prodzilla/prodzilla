use std::sync::Arc;

use tracing::error;

use crate::alerts::outbound_webhook::alert_if_failure;

use super::expectations::check_expectations;
use super::http_probe::call_endpoint;
use super::http_probe::check_endpoint;
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
        println!("Performing check on a Story");
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

        let endpoint_result = match call_endpoint_result {
            Ok(val) => val,
            Err(e) => {
                error!("Error calling endpoint: {}", e);
                // add probe result
                return;
            }
        };

        let expectations_result = check_expectations(&self.name, &endpoint_result, &self.expectations);

        if !expectations_result {
            let send_alert_result = alert_if_failure(self, &probe_result).await;
            if let Err(e) = send_alert_result {
                error!("Error sending out alert: {}", e);
            }
        }
    }

    fn get_name(&self) -> String {
        return self.name.clone()
    }
    fn get_schedule(&self) -> &ProbeScheduleParameters {
        return &self.schedule
    }
}