use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder, RequestBuilder};
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
    // When we have other alert types, add them in some kind of switch here

    let route_provider: String = alert_router(alert).await?;
    let status_code: u16;

    if route_provider == "any" {
        let mut request: RequestBuilder = CLIENT.post(&alert.url);

        let request_body: WebhookNotification = WebhookNotification {
            message: "Probe failed.".to_owned(),
            probe_name: probe_name.clone(),
            failure_timestamp,
        };

        let json: String = serde_json::to_string(&request_body).map_to_send_err()?;
        request = request.body(json);

        let alert_response = request
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await
            .map_to_send_err()?;

        status_code = alert_response.status().as_u16();
    } else if route_provider == "discord" {
        status_code = send_alert_discord(alert, probe_name.clone(), failure_timestamp).await?;
    } else {
        error!("Unknown route provider: {}", route_provider);
        return Ok(());
    }

    if status_code != 200 && status_code != 204 {
        error!(
            "Failed to send webhook alert. Response status code {}",
            status_code
        );
    } else {
        info!(
            "Sent webhook alert. Response status code {}",
            status_code
        );
    }

    Ok(())
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
