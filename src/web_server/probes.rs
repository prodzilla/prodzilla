use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use chrono::Utc;
use futures::future::join_all;
use std::sync::Arc;
use tracing::debug;

use crate::{
    app_state::AppState,
    probe::{model::ProbeResult, probe_logic::Monitorable},
};

use super::model::{ProbeQueryParams, ProbeResponse, BulkTriggerRequest, BulkTriggerResponse, TriggerResult};

pub async fn get_probe_results(
    Path(name): Path<String>,
    Query(params): Query<ProbeQueryParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<ProbeResult>> {
    debug!("Get probe results called");

    let show_response = params.show_response.unwrap_or(false);
    let read_lock = state.probe_results.read().unwrap();
    let results = read_lock.get(&name).unwrap();

    let mut cloned_results: Vec<ProbeResult> = results.clone();
    cloned_results.reverse();

    if !show_response {
        for result in &mut cloned_results {
            result.response = None;
        }
    }

    Json(cloned_results)
}

pub async fn probes(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<ProbeResponse>> {
    debug!("Get probes called");

    let read_lock = state.probe_results.read().unwrap();

    let mut probes: Vec<ProbeResponse> = vec![];

    for (key, value) in read_lock.iter() {
        let last = value.last().unwrap();
        let status = if last.success { "OK" } else { "FAILING" };

        // Find the corresponding probe config to get tags
        let probe_config = state.config.probes.iter().find(|p| p.name == *key);
        let tags = probe_config.and_then(|p| p.tags.clone());

        probes.push(ProbeResponse {
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started,
            tags,
        })
    }

    Json(probes)
}

pub async fn probe_trigger(
    Path(name): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ProbeResult> {
    debug!("Probe trigger called");

    let probe = &state.config.probes.iter().find(|x| x.name == name).unwrap();

    probe.probe_and_store_result(state.clone()).await;

    let lock = state.probe_results.read().unwrap();
    let probe_results = lock.get(&name).unwrap();

    Json(probe_results.last().unwrap().clone())
}

pub async fn bulk_probe_trigger(
    Json(request): Json<BulkTriggerRequest>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<BulkTriggerResponse> {
    debug!("Bulk probe trigger called with tags: {:?}", request.tags);

    // Filter probes based on tags
    let probes_to_trigger: Vec<_> = if request.tags.is_empty() {
        // Trigger all probes if no tags specified
        state.config.probes.iter().collect()
    } else {
        // Parse requested tags into key:value pairs
        let requested_tags: Vec<(String, String)> = request.tags
            .iter()
            .filter_map(|tag| {
                let parts: Vec<&str> = tag.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        // Filter probes that have any of the requested tags
        state.config.probes
            .iter()
            .filter(|probe| {
                if let Some(probe_tags) = &probe.tags {
                    requested_tags.iter().any(|(key, value)| {
                        probe_tags.get(key).map_or(false, |v| v == value)
                    })
                } else {
                    false
                }
            })
            .collect()
    };

    // Execute probes in parallel
    let trigger_futures: Vec<_> = probes_to_trigger
        .iter()
        .map(|probe| {
            let probe_name = probe.name.clone();
            let state_clone = state.clone();
            async move {
                let triggered_at = Utc::now();
                match probe.probe_and_store_result(state_clone).await {
                    Ok(_) => TriggerResult {
                        name: probe_name,
                        success: true,
                        triggered_at,
                        error_message: None,
                    },
                    Err(e) => TriggerResult {
                        name: probe_name,
                        success: false,
                        triggered_at,
                        error_message: Some(e.to_string()),
                    },
                }
            }
        })
        .collect();

    let results = join_all(trigger_futures).await;
    let triggered_count = results.len();

    Json(BulkTriggerResponse {
        triggered_count,
        results,
    })
}
