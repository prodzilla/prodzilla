use crate::probe::model::ExpectField;
use crate::probe::model::ExpectOperation;
use crate::probe::model::ProbeExpectation;
use tracing::debug;

use super::model::EndpointResult;

pub fn check_expectations(
    step_name: &String,
    endpoint_result: &EndpointResult, 
    expectations: &Option<Vec<ProbeExpectation>>,
) -> bool {

    match expectations {
        Some(expect_back) => {
            let validation_result = validate_response(&expect_back, endpoint_result.status_code, &endpoint_result.body);
            if validation_result {
                debug!("Successful response for {}, as expected", step_name);
            } else {
                debug!("Successful response for {}, not as expected!", step_name);
            }
            return validation_result;
        }
        None => {
            debug!(
                "Successfully probed {}, no expectation so success is true",
                step_name
            );
            return true;
        }
    }
}

pub fn validate_response(
    expect: &Vec<ProbeExpectation>,
    status_code: u32,
    body: &String,
) -> bool {
    let status_string = status_code.to_string();

    for expectation in expect {
        let expectation_result: bool;
        match expectation.field {
            ExpectField::Body => {
                expectation_result =
                    validate_expectation(&expectation.operation, &expectation.value, &body);
            }
            ExpectField::StatusCode => {
                expectation_result = validate_expectation(
                    &expectation.operation,
                    &expectation.value,
                    &status_string,
                );
            }
        }

        if !expectation_result {
            return false;
        }
    }

    return true;
}

fn validate_expectation(
    operation: &ExpectOperation,
    expected_value: &String,
    value: &String,
) -> bool {
    match operation {
        ExpectOperation::Equals => {
            return value == expected_value;
        }
        ExpectOperation::Contains => {
            return value.contains(expected_value);
        }
        ExpectOperation::IsOneOf => {
            let parts = expected_value.split("|");
            for part in parts {
                if value == part {
                    return true;
                }
            }
            return false;
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
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(
        &ExpectOperation::Equals,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert_eq!(fail_result, false);
}

#[tokio::test]
async fn test_validate_expectations_contains() {
    let success_result = validate_expectation(
        &ExpectOperation::Contains,
        &"Test".to_owned(),
        &"Test123".to_owned(),
    );
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(
        &ExpectOperation::Contains,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert_eq!(fail_result, false);
}

#[tokio::test]
async fn test_validate_expectations_isoneof() {
    let success_result = validate_expectation(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Test".to_owned(),
    );
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Yest".to_owned(),
    );
    assert_eq!(fail_result, false);
}
