use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use std::sync::Arc;
use tracing::debug;

use crate::{
    app_state::AppState,
    probe::{model::ProbeResult, probe_logic::Monitorable},
};

use super::model::{ProbeQueryParams, ProbeResponse};

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

        probes.push(ProbeResponse {
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started,
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
