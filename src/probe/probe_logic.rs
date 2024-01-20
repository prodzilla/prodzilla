use std::sync::Arc;

use tracing::error;

use crate::alerts::outbound_webhook::alert_if_failure;

use super::http_probe::check_endpoint;
use super::model::ProbeScheduleParameters;
use super::model::Story;
use super::model::Probe;
use crate::AppState;

pub trait Monitorable {
    async fn probe(&self, app_state: Arc<AppState>);
    fn get_name(&self) -> String;
    fn get_schedule(&self) -> &ProbeScheduleParameters;
}

impl Monitorable for Story {
    async fn probe(&self, app_state: Arc<AppState>) {
        // Implementation for Story
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
        // Implementation for Probe
        let check_endpoint_result = check_endpoint(self).await;

        match check_endpoint_result {
            Ok(probe_result) => {
                app_state.add_probe_result(self.name.clone(), probe_result.clone());

                let send_alert_result = alert_if_failure(self, &probe_result).await;
                if let Err(e) = send_alert_result {
                    error!("Error sending out alert: {}", e);
                }
            }
            Err(e) => {
                error!("Error constructing probe: {}", e);
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