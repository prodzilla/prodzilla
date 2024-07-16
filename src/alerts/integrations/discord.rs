use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder, Response};
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tracing::{error, info};

// crate imports
use crate::errors::MapToSendError;
use crate::probe::model::ProbeAlert;

const REQUEST_TIMEOUT_SECS: u64 = 30;
const CONTENT_TYPE: &str = "application/json";

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

    let content: String = format!(
        "```{} | Probe failed to return status code 200 \n Probe Name: {} \n Failure Timestamp: {}```",
        probe_name, probe_name, failure_timestamp
    );

    let alert_response: Response = client
        .post(&webhook_url)
        .body(
            json!({
                "content": content
            })
            .to_string(),
        )
        .header("Content-Type", CONTENT_TYPE)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_to_send_err()?;

    let status_code: u16 = alert_response.status().as_u16();

    if alert_response.status().is_success() {
        info!("Alert sent successfully");
    } else {
        error!("Failed to send alert: {:?}", alert_response.text().await);
    }

    Ok(status_code)
}
