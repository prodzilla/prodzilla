use tokio::time::Instant;

use crate::alert_webhook::alert_if_failure;
use crate::http_probe::check_endpoint;
use crate::probe::Probe;

pub fn schedule_probes(probes: Vec<Probe>) {
    for probe in probes {
        let probe_clone = probe.clone();
        tokio::spawn(async move {
            probing_loop(&probe_clone).await;
        });
    }
}

pub async fn probing_loop(probe: &Probe) {
    let mut next_run_time =
        Instant::now() + std::time::Duration::from_secs(probe.schedule.initial_delay as u64);

    loop {
        let now = Instant::now();
        if now < next_run_time {
            tokio::time::sleep(next_run_time - now).await;
        }

        next_run_time += std::time::Duration::from_secs(probe.schedule.interval as u64);

        let check_endpoint_result = check_endpoint(probe).await;

        match check_endpoint_result {
            Ok(probe_result) => {
                let send_alert_result = alert_if_failure(probe, &probe_result).await;
                if let Err(e) = send_alert_result {
                    println!("Error sending out alert: {}", e);
                }
            }
            Err(e) => {
                println!("Error constructing probe: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod schedule_tests {

    use std::time::Duration;
    use crate::schedule::schedule_probes;
    use crate::test_utils::test_utils::{probe_get_with_expected_status_and_alert, probe_get_with_expected_status};

    use reqwest::StatusCode;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_loop_continues_when_alert_fails() {
        let mock_server = MockServer::start().await;

        let alert_url = "/alert-test";
        let probe_url = "/probe-test";

        // Set probe to return 404 - which should trigger an alert
        Mock::given(method("GET"))
            .and(path(probe_url))
            .respond_with(ResponseTemplate::new(404))
            .expect(2)
            .mount(&mock_server)
            .await;

        // Set alert to timeout
        Mock::given(method("POST"))
            .and(path(alert_url))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(11)))
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_expected_status_and_alert(
            StatusCode::OK,
            format!("{}{}", mock_server.uri(), probe_url.to_owned()),
            "".to_owned(),
            format!("{}{}", mock_server.uri(), alert_url.to_owned()),
        );

        schedule_probes(vec![probe]);

        // As delay and interval are 0, we'd expect that within 15 seconds our probe has been hit twice
        // One for first probe, then 10s timeout on request, then second probe
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        // If we don't fail here it means our .expect() has succeded
    }

    #[tokio::test]
    async fn test_loop_continues_when_probe_fails() {
        let mock_server = MockServer::start().await;

        let probe_url = "/probe-test";

        Mock::given(method("GET"))
            .and(path(probe_url))
            .respond_with(ResponseTemplate::new(404).set_delay(Duration::from_secs(11)))
            .expect(2)
            .mount(&mock_server)
            .await;

        let probe = probe_get_with_expected_status(
            StatusCode::OK,
            format!("{}{}", mock_server.uri(), probe_url.to_owned()),
            "".to_owned(),
        );
        
        schedule_probes(vec![probe]);

        // As delay and interval are 0, we'd expect that within 15 seconds our probe has been hit twice
        // One for first probe, then 10s timeout on request, then second probe
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        // If we don't fail here it means our .expect() has succeded
    }
}
