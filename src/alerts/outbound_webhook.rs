use std::time::Duration;

use crate::errors::MapToSendError;
use crate::probe::model::ProbeAlert;
use crate::{alerts::model::WebhookNotification, probe::model::ProbeResponse};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use tracing::{info, warn};

use super::model::{SlackBlock, SlackNotification, SlackTextBlock};

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla Alert/1.0")
        .build()
        .unwrap();
}

pub async fn alert_if_failure(
    success: bool,
    error: Option<&str>,
    probe_response: Option<&ProbeResponse>,
    probe_name: &str,
    failure_timestamp: DateTime<Utc>,
    alerts: &Option<Vec<ProbeAlert>>,
    trace_id: &Option<String>,
) -> Result<(), Vec<Box<dyn std::error::Error + Send>>> {
    if success {
        return Ok(());
    }
    let error_message = error.unwrap_or("No error message");
    let truncated_body = probe_response.map(|r| r.truncated_body(500));
    warn!(
        "Probe {probe_name} failed at {failure_timestamp} with trace ID {}. Status code: {}. Error: {error_message}. Body: {}",
        trace_id.as_ref().unwrap_or(&"N/A".to_owned()),
        probe_response.map_or("N/A".to_owned(), |r| r.status_code.to_string()),
        truncated_body.unwrap_or("N/A".to_owned()),
    );
    let mut errors = Vec::new();
    if let Some(alerts_vec) = alerts {
        for alert in alerts_vec {
            if let Err(e) = send_alert(
                alert,
                probe_name.to_owned(),
                probe_response,
                error_message,
                failure_timestamp,
                trace_id.clone(),
            )
            .await
            {
                errors.push(e);
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
    probe_response: Option<&ProbeResponse>,
    error_message: &str,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let request_body = WebhookNotification {
        message: "Probe failed.".to_owned(),
        probe_name,
        error_message: error_message.to_owned(),
        failure_timestamp,
        trace_id,
        body: probe_response.map(|r| r.truncated_body(500)),
        status_code: probe_response.map(|r| r.status_code),
    };

    let json = serde_json::to_string(&request_body).map_to_send_err()?;
    send_generic_webhook(url, json).await
}

pub async fn send_slack_alert(
    webhook_url: &String,
    probe_name: String,
    probe_response: Option<&ProbeResponse>,
    error_message: &str,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    // Uses Slack's Block Kit UI to make the message prettier
    let mut blocks = vec![
        SlackBlock {
            r#type: "header".to_owned(),
            text: Some(SlackTextBlock {
                r#type: "plain_text".to_owned(),
                text: format!("\"{}\" failed.", probe_name),
            }),
            elements: None,
        },
        SlackBlock {
            r#type: "section".to_owned(),
            text: Some(SlackTextBlock {
                r#type: "mrkdwn".to_owned(),
                text: format!("Error message:\n\n> {}", error_message),
            }),
            elements: None,
        },
    ];

    if let Some(response) = probe_response {
        blocks.extend([
            SlackBlock {
                r#type: "divider".to_owned(),
                elements: None,
                text: None,
            },
            SlackBlock {
                r#type: "section".to_owned(),
                elements: None,
                text: Some(SlackTextBlock {
                    r#type: "mrkdwn".to_owned(),
                    text: format!(
                        "Received status code *{}* and (truncated) body:\n```\n{}\n```",
                        response.status_code,
                        response.body.chars().take(500).collect::<String>()
                    ),
                }),
            },
        ])
    }

    blocks.push(SlackBlock {
        r#type: "context".to_owned(),
        elements: Some(vec![
            SlackTextBlock {
                r#type: "mrkdwn".to_owned(),
                text: format!("Time: *{}*", failure_timestamp),
            },
            SlackTextBlock {
                r#type: "mrkdwn".to_owned(),
                text: format!("Trace ID: *{}*", trace_id.unwrap_or("N/A".to_owned())),
            },
        ]),
        text: None,
    });
    let request_body = SlackNotification { blocks };
    let json = serde_json::to_string(&request_body).map_to_send_err()?;
    println!("{}", json);
    send_generic_webhook(webhook_url, json).await
}

pub async fn send_alert(
    alert: &ProbeAlert,
    probe_name: String,
    probe_response: Option<&ProbeResponse>,
    error_message: &str,
    failure_timestamp: DateTime<Utc>,
    trace_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let domain = alert.url.split('/').nth(2).unwrap_or("");
    match domain {
        "hooks.slack.com" => {
            send_slack_alert(
                &alert.url,
                probe_name.clone(),
                probe_response,
                error_message,
                failure_timestamp,
                trace_id.clone(),
            )
            .await
        }
        _ => {
            send_webhook_alert(
                &alert.url,
                probe_name.clone(),
                probe_response,
                error_message,
                failure_timestamp,
                trace_id.clone(),
            )
            .await
        }
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
            url: format!("{}{}", mock_server.uri(), alert_url.to_owned()),
        }]);
        let failure_timestamp = Utc::now();

        let alert_result = alert_if_failure(
            false,
            Some("Test error"),
            None,
            &probe_name,
            failure_timestamp,
            &alerts,
            &None,
        )
        .await;

        assert!(alert_result.is_ok());
    }
}
