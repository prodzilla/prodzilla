use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder};
use std::error::Error;
use std::time::Duration;
use tracing::{error, info};

// crate imports
use crate::alerts::integrations::alert_router;
use crate::alerts::integrations::discord::send_alert_discord;
use crate::alerts::model::WebhookNotification;
use crate::errors::MapToSendError;
use crate::probe::model::ProbeAlert;

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: Client = ClientBuilder::new()
        .user_agent("Prodzilla Alert/1.0")
        .build()
        .expect("Failed to build reqwest client");
}



pub async fn alert_if_failure(
    success: bool,
    probe_name: &String,
    failure_timestamp: DateTime<Utc>,
    alerts: &Option<Vec<ProbeAlert>>,
) -> Result<(), Box<dyn Error + Send>> {
    if success {
        return Ok(());
    }

    if let Some(alerts_vec) = alerts {
        for alert in alerts_vec {
            send_alert(alert, probe_name.clone(), failure_timestamp).await?;
        }
    }

    return Ok(());
}

pub async fn send_alert(
    alert: &ProbeAlert,
    probe_name: String,
    failure_timestamp: DateTime<Utc>,
) -> Result<(), Box<dyn Error + Send>> {
    let route_provider: String = alert_router(alert).await?;

    let status_code = match route_provider.as_str() {
        "any" => send_generic_alert(alert, &probe_name, failure_timestamp).await?,
        "discord" => send_alert_discord(alert, probe_name.clone(), failure_timestamp).await?,
        _ => {
            error!("Unknown route provider: {}", route_provider);
            return Ok(());
        }
    };

    log_alert_status(status_code);
    Ok(())
}


async fn send_generic_alert(
    alert: &ProbeAlert,
    probe_name: &str,
    failure_timestamp: DateTime<Utc>,
) -> Result<u16, Box<dyn Error + Send>> {
    let request_body: WebhookNotification = WebhookNotification {
        message: "Probe failed.".to_owned(),
        probe_name: probe_name.to_owned(),
        failure_timestamp,
    };

    let json: String = serde_json::to_string(&request_body).map_to_send_err()?;
    let alert_response = CLIENT
        .post(&alert.url)
        .body(json)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .send()
        .await
        .map_to_send_err()?;

    Ok(alert_response.status().as_u16())
}

// discord success webhooks can return 204 so we need to log that as well
fn log_alert_status(status_code: u16) {
    if status_code != 200 && status_code != 204 {
        error!("Failed to send webhook alert. Response status code {}", status_code);
    } else {
        info!("Sent webhook alert. Response status code {}", status_code);
    }
}

#[cfg(test)]
mod webhook_tests {

    use crate::alerts::outbound_webhook::alert_if_failure;
    use crate::probe::model::ProbeAlert;

    use chrono::prelude::DateTime;
    use chrono::Utc;
    use std::error::Error;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_failure_gets_alerted() {
        let mock_server: MockServer = MockServer::start().await;

        let alert_url: &str = "/alert-test";

        Mock::given(method("POST"))
            .and(path(alert_url))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let probe_name: String = "Some Flow".to_owned();
        let alerts: Option<Vec<ProbeAlert>> = Some(vec![ProbeAlert {
            url: format!("{}{}", mock_server.uri(), alert_url.to_owned()),
        }]);
        let failure_timestamp: DateTime<Utc> = Utc::now();

        let alert_result: Result<(), Box<dyn Error + Send>> =
            alert_if_failure(false, &probe_name, failure_timestamp, &alerts).await;

        assert!(alert_result.is_ok());
    }
}
