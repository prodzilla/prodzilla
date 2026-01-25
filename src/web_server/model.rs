use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MonitorQueryParams {
    pub show_response: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResponse {
    pub name: String,
    pub status: String,
    pub last_probed: DateTime<Utc>,
}
