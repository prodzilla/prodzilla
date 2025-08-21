use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use futures::future::join_all;
use std::sync::Arc;
use tracing::debug;

use crate::{
    app_state::AppState,
    probe::{model::{ProbeResult, StoryResult}, probe_logic::Monitorable},
};

use super::model::{ProbeQueryParams, ProbeResponse, BulkTriggerRequest, BulkTriggerResponse};

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

        // Find the corresponding story config to get tags
        let story_config = state.config.stories.iter().find(|s| s.name == *key);
        let tags = story_config.and_then(|s| s.tags.clone());

        stories.push(ProbeResponse {
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started,
            tags,
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

pub async fn bulk_story_trigger(
    Extension(state): Extension<Arc<AppState>>,
    Json(request): Json<BulkTriggerRequest>,
) -> Json<BulkTriggerResponse> {
    debug!("Bulk story trigger called with tags: {:?}", request.tags);

    // Filter stories based on tags
    let stories_to_trigger: Vec<_> = if request.tags.is_empty() {
        // Trigger all stories if no tags specified
        state.config.stories.iter().collect()
    } else {
        // Filter stories that have all of the requested tags
        state.config.stories
            .iter()
            .filter(|story| {
                if let Some(story_tags) = &story.tags {
                    request.tags.iter().all(|(key, value)| {
                        story_tags.get(key).map_or(false, |v| v == value)
                    })
                } else {
                    false
                }
            })
            .collect()
    };

    // Execute stories in parallel
    let trigger_futures: Vec<_> = stories_to_trigger
        .iter()
        .map(|story| {
            let story_name = story.name.clone();
            let state_clone = state.clone();
            async move {
                story.probe_and_store_result(state_clone.clone()).await;
                let lock = state_clone.story_results.read().unwrap();
                let story_result = lock.get(&story_name).unwrap().last().unwrap().clone();
                
                // Convert StoryResult to ProbeResult for unified response format
                ProbeResult {
                    probe_name: story_result.story_name,
                    timestamp_started: story_result.timestamp_started,
                    success: story_result.success,
                    error_message: None,
                    response: None,
                    trace_id: None,
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
