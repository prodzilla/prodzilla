use chrono::{DateTime, Utc};

use serde::{de, Deserialize, Deserializer, Serialize};
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
    #[serde(default)] // default to false
    pub sensitive: bool,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInputParameters {
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub timeout_seconds: Option<u64>,
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
    NotEquals,
    IsOneOf,
    Contains,
    NotContains,
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
    pub url: String,
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
    pub sensitive: bool,
}

impl ProbeResponse {
    pub fn truncated_body(&self, n: usize) -> String {
        self.body.chars().take(n).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub name: String,
    pub steps: Vec<Step>,
    pub schedule: ProbeScheduleParameters,
    pub alerts: Option<Vec<ProbeAlert>>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub with: Option<ProbeInputParameters>,
    pub expectations: Option<Vec<ProbeExpectation>>,
    #[serde(default)] // default to false
    pub sensitive: bool,
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
    pub sensitive: bool,
}

impl EndpointResult {
    pub fn to_probe_response(&self) -> ProbeResponse {
        ProbeResponse {
            timestamp_received: self.timestamp_response_received,
            status_code: self.status_code,
            body: self.body.clone(),
            sensitive: self.sensitive,
        }
    }
}

/// Monitor represents a unified configuration that can be either a single-step test (probe)
/// or a multi-step test (story). It must have either `steps` OR the root-level probe fields,
/// but not both.
#[derive(Debug, Clone, Serialize)]
pub struct Monitor {
    pub name: String,
    // Fields for single-step monitors (probe-like)
    pub url: Option<String>,
    pub http_method: Option<String>,
    pub with: Option<ProbeInputParameters>,
    pub expectations: Option<Vec<ProbeExpectation>>,
    #[serde(default)]
    pub sensitive: bool,
    // Fields for multi-step monitors (story-like)
    pub steps: Option<Vec<Step>>,
    // Common fields
    pub schedule: ProbeScheduleParameters,
    pub alerts: Option<Vec<ProbeAlert>>,
    pub tags: Option<HashMap<String, String>>,
}

impl Monitor {
    /// Returns true if this monitor is a multi-step monitor (story-like)
    pub fn is_multi_step(&self) -> bool {
        self.steps.is_some()
    }

    /// Returns true if this monitor is a single-step monitor (probe-like)
    pub fn is_single_step(&self) -> bool {
        self.url.is_some()
    }

    /// Converts this monitor to a Probe (for single-step monitors)
    pub fn to_probe(&self) -> Option<Probe> {
        if !self.is_single_step() {
            return None;
        }
        Some(Probe {
            name: self.name.clone(),
            url: self.url.clone()?,
            http_method: self.http_method.clone()?,
            with: self.with.clone(),
            expectations: self.expectations.clone(),
            schedule: self.schedule.clone(),
            alerts: self.alerts.clone(),
            sensitive: self.sensitive,
            tags: self.tags.clone(),
        })
    }

    /// Converts this monitor to a Story (for multi-step monitors)
    pub fn to_story(&self) -> Option<Story> {
        if !self.is_multi_step() {
            return None;
        }
        Some(Story {
            name: self.name.clone(),
            steps: self.steps.clone()?,
            schedule: self.schedule.clone(),
            alerts: self.alerts.clone(),
            tags: self.tags.clone(),
        })
    }

    /// Validates that step names are unique within this monitor
    fn validate_step_names(&self) -> Result<(), String> {
        if let Some(steps) = &self.steps {
            let mut seen = HashMap::new();
            for step in steps {
                if seen.insert(&step.name, ()).is_some() {
                    return Err(format!(
                        "Duplicate step name '{}' in monitor '{}'",
                        step.name, self.name
                    ));
                }
            }
        }
        Ok(())
    }
}

// Custom deserialization to validate the Monitor structure
impl<'de> Deserialize<'de> for Monitor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MonitorHelper {
            name: String,
            url: Option<String>,
            http_method: Option<String>,
            with: Option<ProbeInputParameters>,
            expectations: Option<Vec<ProbeExpectation>>,
            #[serde(default)]
            sensitive: bool,
            steps: Option<Vec<Step>>,
            schedule: ProbeScheduleParameters,
            alerts: Option<Vec<ProbeAlert>>,
            tags: Option<HashMap<String, String>>,
        }

        let helper = MonitorHelper::deserialize(deserializer)?;

        let has_steps = helper.steps.is_some() && !helper.steps.as_ref().unwrap().is_empty();
        let has_probe_fields = helper.url.is_some() || helper.http_method.is_some();

        // Validate mutual exclusivity
        if has_steps && has_probe_fields {
            return Err(de::Error::custom(format!(
                "Monitor '{}' cannot have both 'steps' and root-level probe fields (url, http_method). Use one or the other.",
                helper.name
            )));
        }

        // Validate that at least one is present
        if !has_steps && !has_probe_fields {
            return Err(de::Error::custom(format!(
                "Monitor '{}' must have either 'steps' or root-level probe fields (url, http_method)",
                helper.name
            )));
        }

        // For single-step monitors, require url and http_method
        if !has_steps {
            if helper.url.is_none() {
                return Err(de::Error::custom(format!(
                    "Monitor '{}' is missing required field 'url'",
                    helper.name
                )));
            }
            if helper.http_method.is_none() {
                return Err(de::Error::custom(format!(
                    "Monitor '{}' is missing required field 'http_method'",
                    helper.name
                )));
            }
        }

        // For multi-step monitors, ensure steps is non-empty
        if has_steps && helper.steps.as_ref().unwrap().is_empty() {
            return Err(de::Error::custom(format!(
                "Monitor '{}' has 'steps' field but it is empty",
                helper.name
            )));
        }

        let monitor = Monitor {
            name: helper.name,
            url: helper.url,
            http_method: helper.http_method,
            with: helper.with,
            expectations: helper.expectations,
            sensitive: helper.sensitive,
            steps: helper.steps,
            schedule: helper.schedule,
            alerts: helper.alerts,
            tags: helper.tags,
        };

        // Validate unique step names
        monitor.validate_step_names().map_err(de::Error::custom)?;

        Ok(monitor)
    }
}
