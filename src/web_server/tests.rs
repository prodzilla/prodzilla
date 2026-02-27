use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use std::sync::Arc;
use tracing::debug;

use crate::{
    app_state::AppState,
    probe::{model::TestResult, probe_logic::Monitorable},
};

use super::model::{QueryParams, TestSummary};

// TODO: Error handling for all of the endpoints

pub async fn get_test_results(
    Path(name): Path<String>,
    Query(params): Query<QueryParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<TestResult>> {
    debug!("Get test results called");

    let show_response = params.show_response.unwrap_or(false);
    let read_lock = state.test_results.read().unwrap();
    let results = read_lock.get(&name).unwrap();

    let mut cloned_results: Vec<TestResult> = results.clone();
    cloned_results.reverse();

    if !show_response {
        for result in &mut cloned_results {
            for step_result in &mut result.step_results {
                step_result.response = None;
            }
        }
    }

    Json(cloned_results)
}

pub async fn tests(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<TestSummary>> {
    debug!("Get tests called");

    let read_lock = state.test_results.read().unwrap();

    let mut tests: Vec<TestSummary> = vec![];

    for (key, value) in read_lock.iter() {
        let last = value.last().unwrap();
        let status = if last.success { "OK" } else { "FAILING" };

        tests.push(TestSummary {
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started,
        })
    }

    Json(tests)
}

pub async fn test_trigger(
    Path(name): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<TestResult> {
    debug!("Test trigger called");

    let test = &state.config.tests.iter().find(|x| x.name == name).unwrap();

    test.probe_and_store_result(state.clone()).await;

    let lock = state.test_results.read().unwrap();
    let test_results = lock.get(&name).unwrap();

    Json(test_results.last().unwrap().clone())
}
