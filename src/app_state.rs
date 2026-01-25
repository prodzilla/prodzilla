use std::{collections::HashMap, sync::RwLock};

use crate::{config::Config, monitor::model::MonitorResult, otel::metrics::Metrics};

// Limits the number of results we store per monitor. Once we go over this amount we remove the earliest.
const RESULT_LIMIT: usize = 100;

pub struct AppState {
    pub monitor_results: RwLock<HashMap<String, Vec<MonitorResult>>>,
    pub config: Config,
    pub metrics: Metrics,
}

impl AppState {
    pub fn new(config: Config) -> AppState {
        AppState {
            monitor_results: RwLock::new(HashMap::new()),
            config,
            metrics: Metrics::new(),
        }
    }

    pub fn add_monitor_result(&self, monitor_name: String, result: MonitorResult) {
        let mut write_lock = self.monitor_results.write().unwrap();

        let results = write_lock.entry(monitor_name).or_default();
        results.push(result);

        // Ensure only the latest 100 elements are kept
        while results.len() > RESULT_LIMIT {
            results.remove(0);
        }
    }
}
