use std::collections::HashMap;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub with: Option<ProbeInputParameters>,
    pub expectations: Option<Vec<ProbeExpectation>>,
    pub schedule: ProbeScheduleParameters,
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
    Contains
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectField {
    Body,
    StatusCode
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeScheduleParameters {
    pub initial_delay: u32,
    pub interval: u32,
}

// datetime, statusCode, result
pub struct ProbeResult{
    pub success: bool,
}

