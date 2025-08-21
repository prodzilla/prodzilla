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
    probe::{model::StoryResult, probe_logic::Monitorable},
};

use super::model::{ProbeQueryParams, ProbeResponse, BulkTriggerRequest, BulkTriggerResponse, TriggerResult};

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
    Json(request): Json<BulkTriggerRequest>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<BulkTriggerResponse> {
    debug!("Bulk story trigger called with tags: {:?}", request.tags);

    // Filter stories based on tags
    let stories_to_trigger: Vec<_> = if request.tags.is_empty() {
        // Trigger all stories if no tags specified
        state.config.stories.iter().collect()
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

        // Filter stories that have any of the requested tags
        state.config.stories
            .iter()
            .filter(|story| {
                if let Some(story_tags) = &story.tags {
                    requested_tags.iter().any(|(key, value)| {
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
                let triggered_at = Utc::now();
                match story.probe_and_store_result(state_clone).await {
                    Ok(_) => TriggerResult {
                        name: story_name,
                        success: true,
                        triggered_at,
                        error_message: None,
                    },
                    Err(e) => TriggerResult {
                        name: story_name,
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
