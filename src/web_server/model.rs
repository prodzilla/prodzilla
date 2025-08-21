use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::probe::model::ProbeResult;

#[derive(Deserialize)]
pub struct ProbeQueryParams {
    pub show_response: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResponse {
    pub name: String,
    pub status: String,
    pub last_probed: DateTime<Utc>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct BulkTriggerRequest {
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct BulkTriggerResponse {
    pub triggered_count: usize,
    pub results: Vec<ProbeResult>,
}

