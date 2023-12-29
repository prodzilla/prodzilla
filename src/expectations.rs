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

pub async fn validate_response(expect: &Vec<ProbeExpectation>, response: &Response) -> Result<bool, Box<dyn std::error::Error>> {
    // todo be explicit about what failed
    let body = response.text().await?;
    let status_code = response.status();

    for expectation in expect {
        let expectation_result: bool;
        match expectation.field {
            ExpectField::Body => {
                expectation_result = 
            }
            ExpectField::StatusCode => {}
        }

        if (!expectation_result) {
            return Ok(false)
        }
    }

    return Ok(true);
}

pub fn validate_error_response(expect: &Vec<ProbeExpectation>, error: &Error) -> bool {
    // todo be explicit about what failed
    error.
    return false;
}

fn validate_expectation(operation: ExpectOperation, expectedValue: String, value: String) {
    match operation {
        ExpectOperation::Equals =>
        ExpectOperation::Contains =>
        ExpectOperation::IsOneOf =>
    }

}