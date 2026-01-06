use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use std::sync::Arc;
use tracing::debug;

use crate::{
    app_state::AppState,
    probe::{model::StoryResult, probe_logic::Monitorable},
};

use super::model::{ProbeQueryParams, ProbeResponse};

/// Get a list of all monitors with their status
pub async fn monitors(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<ProbeResponse>> {
    debug!("Get monitors called");

    let mut monitors: Vec<ProbeResponse> = vec![];

    // Collect from probe results (single-step monitors)
    let probe_lock = state.probe_results.read().unwrap();
    for (key, value) in probe_lock.iter() {
        if let Some(last) = value.last() {
            let status = if last.success { "OK" } else { "FAILING" };
            monitors.push(ProbeResponse {
                name: key.clone(),
                status: status.to_owned(),
                last_probed: last.timestamp_started,
            });
        }
    }

    // Collect from story results (multi-step monitors)
    let story_lock = state.story_results.read().unwrap();
    for (key, value) in story_lock.iter() {
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

    // Check if it's a probe result (single-step monitor)
    {
        let probe_lock = state.probe_results.read().unwrap();
        if let Some(results) = probe_lock.get(&name) {
            let mut cloned_results = results.clone();
            cloned_results.reverse();

            if !show_response {
                for result in &mut cloned_results {
                    result.response = None;
                }
            }

            return Json(serde_json::to_value(cloned_results).unwrap());
        }
    }

    // Check if it's a story result (multi-step monitor)
    {
        let story_lock = state.story_results.read().unwrap();
        if let Some(results) = story_lock.get(&name) {
            let mut cloned_results: Vec<StoryResult> = results.clone();
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
            if monitor.is_single_step() {
                if let Some(probe) = monitor.to_probe() {
                    probe.probe_and_store_result(state.clone()).await;

                    let lock = state.probe_results.read().unwrap();
                    if let Some(probe_results) = lock.get(&name) {
                        return Json(
                            serde_json::to_value(probe_results.last().unwrap().clone()).unwrap(),
                        );
                    }
                }
            } else if let Some(story) = monitor.to_story() {
                story.probe_and_store_result(state.clone()).await;

                let lock = state.story_results.read().unwrap();
                if let Some(story_results) = lock.get(&name) {
                    return Json(
                        serde_json::to_value(story_results.last().unwrap().clone()).unwrap(),
                    );
                }
            }
        }
    }

    // Monitor not found
    Json(serde_json::Value::Null)
}
