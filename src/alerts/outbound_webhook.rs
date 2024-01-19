use std::time::Duration;

use crate::errors::MapToSendError;
use crate::probe::{Probe, ProbeResult};
use lazy_static::lazy_static;

const REQUEST_TIMEOUT_SECS: u64 = 10;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .user_agent("Prodzilla Alert/1.0")
        .build()
        .unwrap();
}

pub async fn alert_if_failure(
    probe: &Probe,
    probe_result: &ProbeResult,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    if probe_result.success {
        return Ok(());
    }

    if let Some(alerts) = &probe.alerts {
        for alert in alerts {
            let mut request = CLIENT.post(&alert.url);
            let json = serde_json::to_string(probe_result).map_to_send_err()?;
            request = request.body(json);

            let alert_response = request
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .send()
                .await
                .map_to_send_err()?;
            println!(
                "Sent webhook alert. Response status code {}",
                alert_response.status().to_owned()
            );
        }
    }

    return Ok(());
}

#[cfg(test)]
mod webhook_tests {

    use crate::alert_webhook::alert_if_failure;
    use crate::probe::{ProbeResponse, ProbeResult};
    use crate::test_utils::test_utils::probe_get_with_expected_status_and_alert;

    use chrono::Utc;
    use reqwest::StatusCode;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_failure_gets_alerted() {
        let mock_server = MockServer::start().await;

        let alert_url = "/alert-test";

        Mock::given(method("POST"))
            .and(path(alert_url))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_expected_status_and_alert(
            StatusCode::OK,
            "".to_owned(),
            "".to_owned(),
            format!("{}{}", mock_server.uri(), alert_url.to_owned()),
        );

        let probe_result = ProbeResult {
            probe_name: probe.name.clone(),
            timestamp_started: Utc::now(),
            success: false,
            response: Some(ProbeResponse {
                timestamp: Utc::now(),
                status_code: 200,
                body: "Some unexpected body".to_owned(),
            }),
        };

        let alert_result = alert_if_failure(&probe, &probe_result).await;

        assert!(alert_result.is_ok());
    }
}
