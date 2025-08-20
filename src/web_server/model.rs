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
