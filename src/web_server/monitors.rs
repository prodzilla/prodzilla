use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use std::sync::Arc;
use tracing::debug;

use crate::{app_state::AppState, monitor::monitor_logic::Monitorable};

use super::model::{ProbeQueryParams, ProbeResponse};

/// Get a list of all monitors with their status
pub async fn monitors(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<ProbeResponse>> {
    debug!("Get monitors called");

    let mut monitors: Vec<ProbeResponse> = vec![];

    let monitor_lock = state.monitor_results.read().unwrap();
    for (key, value) in monitor_lock.iter() {
        if let Some(last) = value.last() {
            let status = if last.success { "OK" } else { "FAILING" };
            monitors.push(ProbeResponse {
                name: key.clone(),
                status: status.to_owned(),
                last_probed: last.timestamp_started,
            });
        }
    }

    Json(monitors)
}

/// Get results for a specific monitor
pub async fn get_monitor_results(
    Path(name): Path<String>,
    Query(params): Query<ProbeQueryParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    debug!("Get monitor results called for: {}", name);

    let show_response = params.show_response.unwrap_or(false);

    let monitor_lock = state.monitor_results.read().unwrap();
    if let Some(results) = monitor_lock.get(&name) {
        let mut cloned_results = results.clone();
        cloned_results.reverse();

        if !show_response {
            for result in &mut cloned_results {
                for step_result in &mut result.step_results {
                    step_result.response = None;
                }
            }
        }

        return Json(serde_json::to_value(cloned_results).unwrap());
    }

    // Monitor not found
    Json(serde_json::Value::Null)
}

/// Trigger a specific monitor
pub async fn monitor_trigger(
    Path(name): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    debug!("Monitor trigger called for: {}", name);

    // Find the monitor in the config
    for monitor in &state.config.monitors {
        if monitor.name == name {
            monitor.probe_and_store_result(state.clone()).await;

            let lock = state.monitor_results.read().unwrap();
            if let Some(results) = lock.get(&name) {
                return Json(serde_json::to_value(results.last().unwrap().clone()).unwrap());
            }
        }
    }

    // Monitor not found
    Json(serde_json::Value::Null)
}
