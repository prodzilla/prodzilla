use crate::errors::ExpectationFailedError;
use crate::probe::model::ExpectField;
use crate::probe::model::ExpectOperation;
use crate::probe::model::ProbeExpectation;
use regex::Regex;
use tracing::debug;

pub fn validate_response(
    step_name: &String,
    status_code: u32,
    body: String,
    expectations: &Option<Vec<ProbeExpectation>>,
) -> Result<(), ExpectationFailedError> {
    match expectations {
        Some(expect_back) => match validate_response_internal(expect_back, status_code, body) {
            Ok(_) => {
                debug!("Successful response for {}, as expected", step_name);
                Ok(())
            }
            Err(e) => {
                debug!("Successful response for {}, not as expected!", step_name);
                Err(e)
            }
        },
        None => {
            // TODO:
            // If we don't have any expectations, default to checking status is 200

            debug!(
                "Successfully probed {}, no expectation so success is true",
                step_name
            );
            Ok(())
        }
    }
}

pub fn validate_response_internal(
    expect: &Vec<ProbeExpectation>,
    status_code: u32,
    body: String,
) -> Result<(), ExpectationFailedError> {
    for expectation in expect {
        validate_expectation(expectation, status_code, &body)?;
    }

    Ok(())
}

fn expectation_met(operation: &ExpectOperation, expected: &String, received: &String) -> bool {
    match operation {
        ExpectOperation::Equals => expected == received,
        ExpectOperation::NotEquals => expected != received,
        ExpectOperation::Contains => received.contains(expected),
        ExpectOperation::NotContains => !received.contains(expected),
        ExpectOperation::IsOneOf => expected.split('|').any(|part| part == received),
        // TODO: This regex could probably be pre-compiled?
        ExpectOperation::Matches => Regex::new(expected).unwrap().is_match(received),
    }
}

fn validate_expectation(
    expect: &ProbeExpectation,
    status_code: u32,
    body: &String,
) -> Result<(), ExpectationFailedError> {
    let expected_value = &expect.value;
    let status_string = status_code.to_string();
    let received_value = match expect.field {
        ExpectField::Body => body,
        ExpectField::StatusCode => &status_string,
    };
    let success = expectation_met(&expect.operation, expected_value, received_value);
    if success {
        Ok(())
    } else {
        Err(ExpectationFailedError {
            expected: expect.value.clone(),
            body: body.clone(),
            operation: expect.operation.clone(),
            field: expect.field.clone(),
            status_code,
        })
    }
}

#[tokio::test]
async fn test_validate_expectations_equals() {
    let success_result = expectation_met(
        &ExpectOperation::Equals,
        &"Test".to_owned(),
        &"Test".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::Equals,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_not_equals() {
    let success_result = expectation_met(
        &ExpectOperation::NotEquals,
        &"Test".to_owned(),
        &"Test123".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::NotEquals,
        &"Test".to_owned(),
        &"Test".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_contains() {
    let success_result = expectation_met(
        &ExpectOperation::Contains,
        &"Test".to_owned(),
        &"Test123".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::Contains,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_not_contains() {
    let success_result = expectation_met(
        &ExpectOperation::NotContains,
        &"Test123".to_owned(),
        &"Test".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::NotContains,
        &"Test".to_owned(),
        &"Test123".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_isoneof() {
    let success_result = expectation_met(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Test".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::IsOneOf,
        &"Test|Yes|No".to_owned(),
        &"Yest".to_owned(),
    );
    assert!(!fail_result);
}

#[tokio::test]
async fn test_validate_expectations_matches() {
    let success_result = expectation_met(
        &ExpectOperation::Matches,
        &r#"^\d{5}$"#.to_owned(),
        &"12345".to_owned(),
    );
    assert!(success_result);

    let fail_result = expectation_met(
        &ExpectOperation::Matches,
        &r#"^\d{5}$"#.to_owned(),
        &"1234".to_owned(),
    );
    assert!(!fail_result);
}
