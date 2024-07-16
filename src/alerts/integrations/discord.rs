use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tracing::{info, error};
use reqwest::{Client, ClientBuilder, RequestBuilder};

// crate imports
use crate::alerts::model::WebhookNotification;
use crate::errors::MapToSendError;
use crate::probe::model::ProbeAlert;

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: Client = ClientBuilder::new()
        .user_agent("Prodzilla Alert/1.1")
        .build()
        .expect("Failed to build reqwest client");
}


pub async fn send_alert_discord(
    alert: &ProbeAlert,
    probe_name: String,
    failure_timestamp: DateTime<Utc>,
) -> Result<u16, Box<dyn Error + Send>> {
    let client: Client = Client::new();
    let webhook_url: String = alert.url.clone();

    let formatted_log_message: String = format!(
        "```{} | Probe failed to return status code 200 \n Probe Name: {} \n Failure Timestamp: {}```",
        probe_name, probe_name, failure_timestamp
    );

    let alert_response = client.post(&webhook_url)
        .body(json!({
            "content": formatted_log_message
        }).to_string())
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_to_send_err()?;

    let status_code = alert_response.status().as_u16();

    if alert_response.status().is_success() {
        info!("Alert sent successfully");
    } else {
        error!("Failed to send alert: {:?}", alert_response.text().await);
    }

    Ok(status_code)
}