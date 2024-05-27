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

// TODO: Error handling for all of the endpoints

pub async fn get_story_results(
    Path(name): Path<String>,
    Query(params): Query<ProbeQueryParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<StoryResult>> {
    debug!("Get story results called");

    let show_response = params.show_response.unwrap_or(false);
    let read_lock = state.story_results.read().unwrap();
    let results = read_lock.get(&name).unwrap();

    let mut cloned_results: Vec<StoryResult> = results.clone();
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

pub async fn stories(Extension(state): Extension<Arc<AppState>>) -> Json<Vec<ProbeResponse>> {
    debug!("Get stories called");

    let read_lock = state.story_results.read().unwrap();

    let mut stories: Vec<ProbeResponse> = vec![];

    for (key, value) in read_lock.iter() {
        let last = value.last().unwrap();
        let status = if last.success { "OK" } else { "FAILING" };

        stories.push(ProbeResponse {
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started,
        })
    }

    Json(stories)
}

pub async fn story_trigger(
    Path(name): Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<StoryResult> {
    debug!("Story trigger called");

    let story = &state
        .config
        .stories
        .iter()
        .find(|x| x.name == name)
        .unwrap();

    story.probe_and_store_result(state.clone()).await;

    let lock = state.story_results.read().unwrap();
    let story_results = lock.get(&name).unwrap();

    Json(story_results.last().unwrap().clone())
}
