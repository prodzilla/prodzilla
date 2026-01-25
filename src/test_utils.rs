#[cfg(test)]
pub mod probe_test_utils {
    use std::collections::HashMap;

    use reqwest::StatusCode;

    use crate::monitor::model::{
        Alert, ExpectField, ExpectOperation, Expectation, InputParameters, Monitor,
        ScheduleParameters,
    };

    pub fn probe_get_with_timeout_and_expected_status(
        status_code: StatusCode,
        url: String,
        body: String,
        timeout_seconds: Option<u64>,
    ) -> Monitor {
        Monitor {
            name: "Test probe".to_string(),
            url: Some(url),
            http_method: Some("GET".to_string()),
            with: Some(InputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds,
            }),
            expectations: Some(vec![Expectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
            steps: None,
        }
    }

    pub fn probe_get_with_expected_status(
        status_code: StatusCode,
        url: String,
        body: String,
    ) -> Monitor {
        Monitor {
            name: "Test probe".to_string(),
            url: Some(url),
            http_method: Some("GET".to_string()),
            with: Some(InputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![Expectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
            steps: None,
        }
    }

    pub fn probe_get_with_expected_status_and_alert(
        status_code: StatusCode,
        url: String,
        body: String,
        alert_url: String,
    ) -> Monitor {
        Monitor {
            name: "Test probe".to_string(),
            url: Some(url),
            http_method: Some("GET".to_string()),
            with: Some(InputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![Expectation {
                field: ExpectField::StatusCode,
                operation: ExpectOperation::Equals,
                value: status_code.as_str().into(),
            }]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: Some(vec![Alert { url: alert_url }]),
            tags: None,
            sensitive: false,
            steps: None,
        }
    }

    pub fn probe_post_with_expected_body(
        expected_body: String,
        url: String,
        body: String,
    ) -> Monitor {
        Monitor {
            name: "Test probe".to_string(),
            url: Some(url),
            http_method: Some("POST".to_string()),
            with: Some(InputParameters {
                body: Some(body),
                headers: Some(HashMap::new()),
                timeout_seconds: None,
            }),
            expectations: Some(vec![
                Expectation {
                    field: ExpectField::StatusCode,
                    operation: ExpectOperation::Equals,
                    value: "200".to_owned(),
                },
                Expectation {
                    field: ExpectField::Body,
                    operation: ExpectOperation::Equals,
                    value: expected_body,
                },
            ]),
            schedule: ScheduleParameters {
                initial_delay: 0,
                interval: 0,
            },
            alerts: None,
            tags: None,
            sensitive: false,
            steps: None,
        }
    }
}
