use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkTriggerResponse {
    pub triggered_count: usize,
    pub results: Vec<TriggerResult>,
}

#[derive(Debug, Serialize)]
pub struct TriggerResult {
    pub name: String,
    pub success: bool,
    pub triggered_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
