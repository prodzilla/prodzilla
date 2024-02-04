use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct StoryQueryParams {
    pub show_response: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResponse {
    pub name: String,
    pub status: String,
    pub last_probed: DateTime<Utc>,
}