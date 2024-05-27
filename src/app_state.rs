use std::{collections::HashMap, sync::RwLock};

use crate::{
    config::Config,
    probe::model::{ProbeResult, StoryResult},
};

// Limits the number of results we store per probe. Once we go over this amount we remove the earliest.
const PROBE_RESULT_LIMIT: usize = 100;

pub struct AppState {
    pub probe_results: RwLock<HashMap<String, Vec<ProbeResult>>>,
    pub story_results: RwLock<HashMap<String, Vec<StoryResult>>>,
    pub config: Config,
}

impl AppState {
    pub fn new(config: Config) -> AppState {
        AppState {
            probe_results: RwLock::new(HashMap::new()),
            story_results: RwLock::new(HashMap::new()),
            config,
        }
    }

    pub fn add_probe_result(&self, probe_name: String, result: ProbeResult) {
        let mut write_lock = self.probe_results.write().unwrap();

        let results = write_lock.entry(probe_name).or_default();
        results.push(result);

        // Ensure only the latest 100 elements are kept
        while results.len() > PROBE_RESULT_LIMIT {
            results.remove(0);
        }
    }

    pub fn add_story_result(&self, story_name: String, result: StoryResult) {
        let mut write_lock = self.story_results.write().unwrap();

        let results = write_lock.entry(story_name).or_default();
        results.push(result);

        // Ensure only the latest 100 elements are kept
        while results.len() > PROBE_RESULT_LIMIT {
            results.remove(0);
        }
    }
}
