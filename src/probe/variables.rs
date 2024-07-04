use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;

use super::model::ProbeInputParameters;

pub struct StoryVariables {
    pub steps: HashMap<String, StepVariables>,
}

impl StoryVariables {
    pub fn new() -> StoryVariables {
        StoryVariables {
            steps: HashMap::new(),
        }
    }
}

pub struct StepVariables {
    pub response_body: String,
}

lazy_static! {
    static ref SUB_REGEX: Regex = Regex::new(r"\$\{\{(.*?)\}\}").unwrap();
}

pub fn substitute_input_parameters(
    input_parameters: &Option<ProbeInputParameters>,
    variables: &StoryVariables,
) -> Option<ProbeInputParameters> {
    input_parameters.as_ref().map(|input| ProbeInputParameters {
        body: input
            .body
            .as_ref()
            .map(|body| substitute_variables(body, variables)),
        headers: input
            .headers
            .as_ref()
            .map(|headers| substitute_variables_in_headers(headers, variables)),
        timeout_seconds: input.timeout_seconds,
    })
}

pub fn substitute_variables_in_headers(
    headers: &HashMap<String, String>,
    variables: &StoryVariables,
) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(key, value)| {
            let substituted_key = substitute_variables(key, variables);
            let substituted_value = substitute_variables(value, variables);
            (substituted_key, substituted_value)
        })
        .collect()
}

// This could return an error in future - for now it fills an empty string
pub fn substitute_variables(content: &str, variables: &StoryVariables) -> String {
    SUB_REGEX
        .replace_all(content, |caps: &regex::Captures| {
            let placeholder = &caps[1].trim();
            let parts: Vec<&str> = placeholder.split('.').collect();

            match parts[0] {
                "steps" => substitute_step_value(&parts[1..], variables),
                "generate" => get_generated_value(parts.get(1)),
                _ => "".to_string(),
            }
        })
        .to_string()
}

fn get_generated_value(type_to_generate: Option<&&str>) -> String {
    match type_to_generate {
        Some(&"uuid") => Uuid::new_v4().to_string(),
        _ => "".to_string(),
    }
}

fn substitute_step_value(parts: &[&str], variables: &StoryVariables) -> String {
    let step_name = parts[0];

    match variables.steps.get(step_name) {
        Some(step) => {
            // TODO: We should check .response and .body
            if parts.len() > 3 {
                get_nested_json_value(&parts[3..], &step.response_body)
            } else {
                step.response_body.clone()
            }
        }
        None => {
            error!("Error: Step name '{}' not found.", step_name);
            "".to_string()
        }
    }
}

// TODO: We should be able to handle arrays
fn get_nested_json_value(parts: &[&str], json_string: &String) -> String {
    let json_value: Value = match serde_json::from_str(json_string) {
        Ok(val) => val,
        Err(_) => {
            error!("Error parsing json response: {}", json_string);
            return "".to_string();
        }
    };

    let mut current_value = &json_value;
    for part in parts {
        current_value = match current_value.get(part) {
            Some(value) => value,
            None => {
                error!("Error finding value in json payload: {}", part);
                return "".to_string();
            }
        };
    }

    json_value_to_string(current_value)
}

fn json_value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "".to_string()),
    }
}

#[tokio::test]
async fn test_substitute_several_variables() {
    let content = r#"
    entire_body: ${{steps.get-token.response.body}}
    token: ${{steps.get-token.response.body.token}}
    uuid: "${{generate.uuid}}"
    "#
    .to_owned();

    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    let variables = StoryVariables {
        steps: HashMap::from([(
            "get-token".to_string(),
            StepVariables {
                response_body: body_str.to_string(),
            },
        )]),
    };

    let result = substitute_variables(&content, &variables);
    assert!(result.contains(r#""other_field": "value""#));
    assert!(result.contains(r#""token": "12345""#));
}

#[tokio::test]
async fn test_substitute_input_parameters() {
    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    let variables = StoryVariables {
        steps: HashMap::from([(
            "get-token".to_string(),
            StepVariables {
                response_body: body_str.to_string(),
            },
        )]),
    };

    let input_parameters = Some(ProbeInputParameters {
        body: Some("entire_body: ${{steps.get-token.response.body}}".to_owned()),
        headers: Some(HashMap::from([(
            "Authorization".to_owned(),
            "Bearer ${{steps.get-token.response.body.token}}".to_owned(),
        )])),
        timeout_seconds: None,
    });

    let result = substitute_input_parameters(&input_parameters, &variables);
    assert_eq!(
        "Bearer 12345",
        result.unwrap().headers.unwrap()["Authorization"]
    );
}

#[tokio::test]
async fn test_substitute_input_parameters_empty() {
    let result = substitute_input_parameters(&None, &StoryVariables::new());
    assert!(result.is_none());
}

#[tokio::test]
async fn test_substitute_variable_doesnt_exist_in_json() {
    let content = r#"field: ${{steps.get-token.response.body.invalid}}"#.to_owned();

    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    let variables = StoryVariables {
        steps: HashMap::from([(
            "get-token".to_string(),
            StepVariables {
                response_body: body_str.to_string(),
            },
        )]),
    };

    let result = substitute_variables(&content, &variables);
    assert_eq!("field: ".to_owned(), result);
}

#[tokio::test]
async fn test_substitute_variable_step_doesnt_exist() {
    let content = r#"field: ${{steps.get-token.response.body.invalid}}"#.to_owned();

    let variables = StoryVariables {
        steps: HashMap::new(),
    };

    let result = substitute_variables(&content, &variables);
    assert_eq!("field: ".to_owned(), result);
}

// TODO test what happens with spaces in the ${{ steps.etc }}
