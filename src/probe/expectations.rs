use crate::probe::model::ExpectField;
use crate::probe::model::ExpectOperation;
use crate::probe::model::ProbeExpectation;
use tracing::debug;

use super::model::EndpointResult;

pub fn validate_response(
    step_name: &String,
    endpoint_result: &EndpointResult, 
    expectations: &Option<Vec<ProbeExpectation>>,
) -> bool {

    match expectations {
        Some(expect_back) => {
            let validation_result = validate_response_internal(expect_back, endpoint_result.status_code, &endpoint_result.body);
            if validation_result {
                debug!("Successful response for {}, as expected", step_name);
            } else {
                debug!("Successful response for {}, not as expected!", step_name);
            }
            validation_result
        }
        None => {

            // TODO:
            // If we don't have any expectations, default to checking status is 200

            debug!(
                "Successfully probed {}, no expectation so success is true",
                step_name
            );
            true
        }
    }
}

pub fn validate_response_internal(
    expect: &Vec<ProbeExpectation>,
    status_code: u32,
    body: &String,
) -> bool {
    let status_string = status_code.to_string();

    for expectation in expect {
        let expectation_result = match expectation.field {
            ExpectField::Body => {
                validate_expectation(&expectation.operation, &expectation.value, body)
            }
            ExpectField::StatusCode => {
                validate_expectation(
                    &expectation.operation,
                    &expectation.value,
                    &status_string,
                )
            }
        };

        if !expectation_result {
            return false;
        }
    }

    true
}

fn validate_expectation(
    operation: &ExpectOperation,
    expected_value: &String,
    value: &String,
) -> bool {
    match operation {
        ExpectOperation::Equals => {
            value == expected_value
        }
        ExpectOperation::Contains => {
            value.contains(expected_value)
        }
        ExpectOperation::IsOneOf => {
            let parts = expected_value.split('|');
            for part in parts {
                if value == part {
                    return true;
                }
            }
            false
        }
    }
}

#[tokio::test]
async fn test_validate_expectations_equals() {
    let success_result = validate_expectation(
        &ExpectOperation::Equals,
        &"Test".to_owned(),
        &"Test".to_owned(),
    );
    assert!(success_result);

    let fail_result = validate_expectation(
        &ExpectOperation::Equals,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_contains() {
    let success_result = validate_expectation(
        &ExpectOperation::Contains,
        &"Test".to_owned(),
        &"Test123".to_owned(),
    );
    assert!(success_result);

    let fail_result = validate_expectation(
        &ExpectOperation::Contains,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_isoneof() {
    let success_result = validate_expectation(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Test".to_owned(),
    );
    assert!(success_result);

    let fail_result = validate_expectation(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Yest".to_owned(),
    );
    assert!(!fail_result);
}
