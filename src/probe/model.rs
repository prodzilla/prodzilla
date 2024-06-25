use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub with: Option<ProbeInputParameters>,
    pub expectations: Option<Vec<ProbeExpectation>>,
    pub schedule: ProbeScheduleParameters,
    pub alerts: Option<Vec<ProbeAlert>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInputParameters {
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeExpectation {
    pub field: ExpectField,
    pub operation: ExpectOperation,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectOperation {
    Equals,
    IsOneOf,
    Contains,
    Matches,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectField {
    Body,
    StatusCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeScheduleParameters {
    pub initial_delay: u32,
    pub interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeAlert {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slack_webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub probe_name: String,
    pub timestamp_started: DateTime<Utc>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ProbeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

// todo track application errors
// also track the request and response bodies that were sent now that variables exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResponse {
    pub timestamp_received: DateTime<Utc>,
    pub status_code: u32,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub name: String,
    pub steps: Vec<Step>,
    pub schedule: ProbeScheduleParameters,
    pub alerts: Option<Vec<ProbeAlert>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub with: Option<ProbeInputParameters>,
    pub expectations: Option<Vec<ProbeExpectation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResult {
    pub story_name: String,
    pub timestamp_started: DateTime<Utc>,
    pub success: bool,
    pub step_results: Vec<StepResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_name: String,
    pub timestamp_started: DateTime<Utc>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ProbeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
}

pub struct EndpointResult {
    pub timestamp_request_started: DateTime<Utc>,
    pub timestamp_response_received: DateTime<Utc>,
    pub status_code: u32,
    pub body: String,
    pub trace_id: String,
    pub span_id: String,
}

impl EndpointResult {
    pub fn to_probe_response(&self) -> ProbeResponse {
        ProbeResponse {
            timestamp_received: self.timestamp_response_received,
            status_code: self.status_code,
            body: self.body.clone(),
        }
    }
}
