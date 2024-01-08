use std::{sync::RwLock, collections::HashMap};

use crate::probe::ProbeResult;

// Limits the number of results we store per probe. Once we go over this amount we remove the earliest.
const PROBE_RESULT_LIMIT: usize = 100;

pub struct AppState {
    pub probe_results: RwLock<HashMap<String, Vec<ProbeResult>>>,
}

impl AppState {
    
    pub fn new() -> AppState {
        return AppState {
            probe_results: RwLock::new(HashMap::new()),
        };
    }

    pub fn add_probe_result(&self, probe_name: String, result: ProbeResult) {
        let mut write_lock = self.probe_results.write().unwrap();

        let results = write_lock.entry(probe_name).or_insert_with(Vec::new);
        results.push(result);

        // Ensure only the latest 100 elements are kept
        while results.len() > PROBE_RESULT_LIMIT {
            results.remove(0);
        }
    }
}