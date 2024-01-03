use crate::probe::Probe;
use crate::probe::ProbeExpectation;
use crate::probe::ProbeInputParameters;
use crate::probe::ProbeResult;
use crate::probe::ProbeScheduleParameters;
use crate::probe::ExpectField;
use crate::probe::ExpectOperation;
use lazy_static::lazy_static;
use reqwest::Error;
use reqwest::RequestBuilder;
use reqwest::Response;

pub async fn validate_response(expect: &Vec<ProbeExpectation>, response: Response) -> Result<bool, Box<dyn std::error::Error>> {
    // todo be explicit about what failed
    let status_code: String = response.status().as_str().into();
    let body = response.text().await?;
    for expectation in expect {
        let expectation_result: bool;
        match expectation.field {
            ExpectField::Body => {
                expectation_result = validate_expectation(&expectation.operation, &expectation.value, &body);
            }
            ExpectField::StatusCode => {
                expectation_result = validate_expectation(&expectation.operation, &expectation.value, &status_code);
            }
        }

        if !expectation_result {
            return Ok(false)
        }
    }

    return Ok(true);
}

pub fn validate_error_response(expect: &Vec<ProbeExpectation>, error: &Error) -> bool {
    // todo be explicit about what failed
    return false;
}

fn validate_expectation(operation: &ExpectOperation, expected_value: &String, value: &String) -> bool {
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

    let success_result = validate_expectation(&ExpectOperation::Equals, &"Test".to_owned(), &"Test".to_owned());
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(&ExpectOperation::Equals, &"Test123".to_owned(), &"Test".to_owned());
    assert_eq!(fail_result, false);
}

#[tokio::test]
async fn test_validate_expectations_contains() {

    let success_result = validate_expectation(&ExpectOperation::Contains, &"Test".to_owned(), &"Test123".to_owned());
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(&ExpectOperation::Contains, &"Test123".to_owned(), &"Test".to_owned());
    assert_eq!(fail_result, false);
}

#[tokio::test]
async fn test_validate_expectations_isoneof() {

    let success_result = validate_expectation(&ExpectOperation::IsOneOf, &"Test|Yes|No".to_owned(), &"Test".to_owned());
    assert_eq!(success_result, true);

    let fail_result = validate_expectation(&ExpectOperation::IsOneOf, &"Test|Yes|No".to_owned(), &"Yest".to_owned());
    assert_eq!(fail_result, false);
}