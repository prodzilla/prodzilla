use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotification {
    pub message: String,
    pub probe_name: String,
    pub failure_timestamp: DateTime<Utc>,
    pub error_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackNotification {
    pub blocks: Vec<SlackBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackBlock {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<SlackTextBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<SlackTextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackTextBlock {
    pub r#type: String,
    pub text: String,
}
