use std::collections::HashMap;

// as:
// call: 
// with: 
//     body:
//     headers:
// expectBack:
//     statusCode: 
//     body:
//     headers:
// schedule:
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub with: Option<ProbeInputParameters>,
    pub expect_back: Option<ProbeExpectParameters>,
    pub schedule: ProbeScheduleParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInputParameters {
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeExpectParameters {
    pub status_code: String,
    pub body: Option<String>,
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

