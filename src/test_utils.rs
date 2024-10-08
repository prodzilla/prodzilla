#[cfg(test)]
pub mod probe_test_utils {
    use std::collections::HashMap;

    use reqwest::StatusCode;

    use crate::probe::model::{
        ExpectField, ExpectOperation, Probe, ProbeAlert, ProbeExpectation, ProbeInputParameters,
        ProbeScheduleParameters,
    };

    pub fn probe_get_with_timeout_and_expected_status(
        status_code: StatusCode,
        url: String,
        body: String,
        timeout_seconds: Option<u64>,
    ) -> Probe {
        Probe {
            name: "Test probe".to_string(),
            url,
            http_method: "GET".to_string(),
            with: Some(ProbeInputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds,
            }),
            expectations: Some(vec![ProbeExpectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
        }
    }

    pub fn probe_get_with_expected_status(
        status_code: StatusCode,
        url: String,
        body: String,
    ) -> Probe {
        Probe {
            name: "Test probe".to_string(),
            url,
            http_method: "GET".to_string(),
            with: Some(ProbeInputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![ProbeExpectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
        }
    }

    pub fn probe_get_with_expected_status_and_alert(
        status_code: StatusCode,
        url: String,
        body: String,
        alert_url: String,
    ) -> Probe {
        Probe {
            name: "Test probe".to_string(),
            url,
            http_method: "GET".to_string(),
            with: Some(ProbeInputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![ProbeExpectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: Some(vec![ProbeAlert { url: alert_url }]),
            tags: None,
            sensitive: false,
        }
    }

    pub fn probe_post_with_expected_body(
        expected_body: String,
        url: String,
        body: String,
    ) -> Probe {
        Probe {
            name: "Test probe".to_string(),
            url,
            http_method: "POST".to_string(),
            with: Some(ProbeInputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![
                ProbeExpectation {
                    field: ExpectField::StatusCode,
                    operation: ExpectOperation::Equals,
                    value: "200".to_owned(),
                },
                ProbeExpectation {
                    field: ExpectField::Body,
                    operation: ExpectOperation::Equals,
                    value: expected_body,
                },
            ]),
            schedule: ProbeScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
        }
    }
}
