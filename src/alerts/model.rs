use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotification {
    pub message: String,
    pub probe_name: String,
    pub failure_timestamp: DateTime<Utc>,
}