use std::sync::Arc;

use tokio::time::Instant;
use tracing::info;

use crate::monitor::model::Monitor;
use crate::monitor::monitor_logic::Monitorable;
use crate::AppState;

pub fn schedule_monitors(monitors: &[Monitor], app_state: Arc<AppState>) {
    for monitor in monitors {
        let monitor_clone = monitor.clone();
        let task_state = app_state.clone();
        tokio::spawn(async move {
            monitoring_loop(&monitor_clone, task_state).await;
        });
    }
}

pub async fn monitoring_loop<T: Monitorable>(monitorable: &T, app_state: Arc<AppState>) {
    info!("Started monitoring {}", monitorable.get_name());

    let schedule = monitorable.get_schedule();

    let mut next_run_time =
        Instant::now() + std::time::Duration::from_secs(schedule.initial_delay as u64);

    loop {
        let now = Instant::now();
        if now < next_run_time {
            tokio::time::sleep(next_run_time - now).await;
        }

        next_run_time += std::time::Duration::from_secs(schedule.interval as u64);

        monitorable.probe_and_store_result(app_state.clone()).await;
    }
}

#[cfg(test)]
mod schedule_tests {

    use crate::config::Config;
    use crate::monitor::schedule::schedule_monitors;
    use crate::test_utils::probe_test_utils::{
        probe_get_with_expected_status, probe_get_with_expected_status_and_alert,
    };
    use crate::AppState;
    use std::sync::Arc;
    use std::time::Duration;
    use std::vec;

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

        let monitor = probe_get_with_expected_status_and_alert(
            StatusCode::OK,
            format!("{}{}", mock_server.uri(), probe_url.to_owned()),
            "".to_owned(),
            format!("{}{}", mock_server.uri(), alert_url.to_owned()),
        );

        let config = Config {
            monitors: vec![monitor],
        };

        let app_state = Arc::new(AppState::new(config));

        schedule_monitors(&app_state.config.monitors, app_state.clone());

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

        let monitor = probe_get_with_expected_status(
            StatusCode::OK,
            format!("{}{}", mock_server.uri(), probe_url.to_owned()),
            "".to_owned(),
        );

        let config = Config {
            monitors: vec![monitor],
        };

        let app_state = Arc::new(AppState::new(config));

        schedule_monitors(&app_state.config.monitors, app_state.clone());

        // As delay and interval are 0, we'd expect that within 15 seconds our probe has been hit twice
        // One for first probe, then 10s timeout on request, then second probe
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        // If we don't fail here it means our .expect() has succeded
    }
}
