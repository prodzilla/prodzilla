use std::time::Duration;

use crate::alerts::model::WebhookNotification;
use crate::errors::MapToSendError;
use crate::probe::model::ProbeAlert;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use tracing::info;

use super::model::SlackNotification;

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla Alert/1.0")
        .build()
        .unwrap();
}

pub async fn alert_if_failure(
    success: bool,
    probe_name: &str,
    failure_timestamp: DateTime<Utc>,
    alerts: &Option<Vec<ProbeAlert>>,
    trace_id: &Option<String>,
) -> Result<(), Vec<Box<dyn std::error::Error + Send>>> {
    if success {
        return Ok(());
    }
    let mut errors = Vec::new();
    if let Some(alerts_vec) = alerts {
        for alert in alerts_vec {
            if let Err(e) = send_alert(
                alert,
                probe_name.to_owned(),
                failure_timestamp,
                trace_id.clone(),
            )
            .await
            {
                errors.extend(e);
            }
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub async fn send_generic_webhook(
    url: &String,
    body: String,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let mut request = CLIENT.post(url);
    request = request.body(body);

    let alert_response = request
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_to_send_err()?;
    info!(
        "Sent webhook alert. Response status code {}",
        alert_response.status().to_owned()
    );

    Ok(())
}

pub async fn send_webhook_alert(
    url: &String,
    probe_name: String,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let request_body = WebhookNotification {
        message: "Probe failed.".to_owned(),
        probe_name,
        failure_timestamp,
        trace_id,
    };

    let json = serde_json::to_string(&request_body).map_to_send_err()?;
    send_generic_webhook(url, json).await
}

pub async fn send_slack_alert(
    webhook_url: &String,
    probe_name: String,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let request_body = SlackNotification {
        text: format!(
            "Probe {} failed at {}. Trace ID: {}",
            probe_name,
            failure_timestamp,
            trace_id.unwrap_or_else(|| "N/A".to_owned())
        ),
    };
    let json = serde_json::to_string(&request_body).map_to_send_err()?;
    send_generic_webhook(webhook_url, json).await
}

pub async fn send_alert(
    alert: &ProbeAlert,
    probe_name: String,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Vec<Box<dyn std::error::Error + Send>>> {
    // When we have other alert types, add them in some kind of switch here
    let mut errors = Vec::new();
    if let Some(url) = &alert.url {
        if let Err(e) =
            send_webhook_alert(url, probe_name.clone(), failure_timestamp, trace_id.clone()).await
        {
            errors.push(e);
        };
    }
    if let Some(url) = &alert.slack_webhook {
        if let Err(e) = send_slack_alert(url, probe_name, failure_timestamp, trace_id).await {
            errors.push(e);
        };
    }
    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod webhook_tests {

    use crate::alerts::outbound_webhook::alert_if_failure;
    use crate::probe::model::ProbeAlert;

    use chrono::Utc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_failure_gets_alerted() {
        let mock_server = MockServer::start().await;

        let alert_url = "/alert-test";

        Mock::given(method("POST"))
            .and(path(alert_url))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let probe_name = "Some Flow".to_owned();
        let alerts = Some(vec![ProbeAlert {
            url: Some(format!("{}{}", mock_server.uri(), alert_url.to_owned())),
            slack_webhook: None,
        }]);
        let failure_timestamp = Utc::now();

        let alert_result =
            alert_if_failure(false, &probe_name, failure_timestamp, &alerts, &None).await;

        assert!(alert_result.is_ok());
    }
}
