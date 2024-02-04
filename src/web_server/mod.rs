mod model;

use axum::{extract::{Path, Query}, routing::get, Extension, Json, Router};
use crate::probe::model::{ProbeResult, StoryResult};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::app_state::AppState;

use self::model::{StoryQueryParams, StoryResponse};

pub async fn start_axum_server(app_state: Arc<AppState>) {
    let app = Router::new()
        .route("/", get(root))
        .route("/probe_results", get(get_probe_results))
        .route("/stories", get(stories)) // Placeholder, can be unimplemented
        .route("/stories/:name/results", get(get_story_results))
        .route("/stories/:name/trigger", get(story_trigger)) // Placeholder, can be unimplemented
        
        .layer(Extension(app_state.clone()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    debug!("Application root called");
    "Roar!"
}

async fn get_probe_results(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<HashMap<String, Vec<ProbeResult>>> {
    debug!("Get probe results called");
    let read_lock = state.probe_results.read();
    Json(read_lock.unwrap().clone())
}

async fn get_story_results(
    Path(name): Path<String>,
    Query(params): Query<StoryQueryParams>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<StoryResult>> {
    
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

    return Json(cloned_results);
}


// TODO: Make probes output like this as well, then document in README
async fn stories(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<StoryResponse>> {
    debug!("Get stories called");
    let read_lock = state.story_results.read().unwrap();

    let mut stories: Vec<StoryResponse> = vec![];

    for (key, value) in read_lock.iter() {

        let last = value.last().unwrap();
        let status = if last.success { "OK" } else { "FAILING" };

        stories.push(StoryResponse{
            name: key.clone(),
            status: status.to_owned(),
            last_probed: last.timestamp_started
        })
    }

    return Json(stories);
}


async fn story_trigger(Path(name): Path<String>) -> &'static str {
    // Placeholder for /stories/{name}/trigger endpoint

    unimplemented!();
}