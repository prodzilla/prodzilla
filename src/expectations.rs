use crate::probe::ProbeExpectation;
use crate::probe::ExpectField;
use crate::probe::ExpectOperation;
use reqwest::Error;
use reqwest::Response;

// todo be explicit about what failed

pub async fn validate_response(expect: &Vec<ProbeExpectation>, response: Response) -> Result<bool, Box<dyn std::error::Error>> {
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
    for expectation in expect {
        let expectation_result: bool;
        match expectation.field {
            ExpectField::Body => {
                expectation_result = false;
            }
            ExpectField::StatusCode => {
                if let Some(status_code) = error.status() {
                    let status_code_str: String = status_code.as_str().into();
                    expectation_result = validate_expectation(&expectation.operation, &expectation.value, &status_code_str);
                } else {
                    // This might mean some error on our side??
                    println!("Error on request, got no status code.");
                    expectation_result = false;
                }
            }
        }

        if !expectation_result {
            return false
        }
    }
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