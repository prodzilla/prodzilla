use serde_json::Value;
use uuid::Uuid;
use tracing::error;
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

pub struct StoryVariables {
    pub step_variables: HashMap<String, StepVariables>
}

pub struct StepVariables {
    pub response_body: String
}

lazy_static! {
    static ref SUB_REGEX: Regex = Regex::new(r"\$\{\{(.*?)\}\}").unwrap();
}

// This could return an error in future - for now it will just fill an empty string
fn substitute_variables(content: &str, variables: &StoryVariables) -> String {
    SUB_REGEX.replace_all(content, |caps: &regex::Captures| {
        let placeholder = &caps[1];
        let parts: Vec<&str> = placeholder.split('.').collect();

        match parts[0] {
            "steps" => substitute_step_value(&parts[1..], variables),
            "generate" => get_generated_value(parts.get(1)),
            _ => "".to_string(),
        }
    }).to_string()
}

fn get_generated_value(type_to_generate: Option<&&str>) -> String {
    match type_to_generate {
        Some(&"uuid") => Uuid::new_v4().to_string(),
        _ => "".to_string(),
    }
}

fn substitute_step_value(parts: &[&str], variables: &StoryVariables) -> String {
    let step_name = parts[0];

    match variables.step_variables.get(step_name) {
        Some(step) => {
            // TODO: We should check .response and .body
            if parts.len() > 3 {
                get_nested_json_value(&parts[3..], &step.response_body)
            } else {
                step.response_body.clone()
            }
        },
        None => {
            error!("Error: Step name '{}' not found.", step_name);
            "".to_string()
        }
    }
}


fn get_nested_json_value(parts: &[&str], json_string: &String) -> String {
    let json_value: Value = match serde_json::from_str(json_string) {
        Ok(val) => val,
        Err(_) => {
            error!("Error parsing json response: {}", json_string);
            return "".to_string()
        },
    };

    let mut current_value = &json_value;
    for part in parts {
        current_value = match current_value.get(part) {
            Some(value) => value,
            None => {
                error!("Error finding value in json payload: {}", part);
                return "".to_string()
            },
        };
    }

    serde_json::to_string(current_value).unwrap_or_else(|_| "".to_string())
}


// todo test non existent variable name
// also test if there aren't enough "." so we go out of index

#[tokio::test]
async fn test_substitute_several_variables() {
    let content = r#"
    entire_body: ${{steps.get-token.response.body}}
    token: ${{steps.get-token.response.body.token}}
    uuid: "${{generate.uuid}}"
    "#;

    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    let variables = StoryVariables {
        step_variables: HashMap::from([
            ("get-token".to_string(), StepVariables{
                response_body: body_str.to_string()
            })
        ])
    };

    let result = substitute_variables(content, &variables);
    assert!(result.contains(r#""other_field": "value""#));
    assert!(result.contains(r#""token": "12345""#));
}

#[tokio::test]
async fn test_substitute_variable_doesnt_exist_in_json() {
    let content = r#"field: ${{steps.get-token.response.body.invalid}}"#;

    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    // Simulate variables from story
    let variables = StoryVariables {
        step_variables: HashMap::from([
            ("get-token".to_string(), StepVariables{
                response_body: body_str.to_string()
            })
        ])
    };

    let result = substitute_variables(content, &variables);
    assert_eq!("field: ".to_owned(), result);
}

#[tokio::test]
async fn test_substitute_variable_step_doesnt_exist() {
    let content = r#"field: ${{steps.get-token.response.body.invalid}}"#;

    // Simulate variables from story
    let variables = StoryVariables {
        step_variables: HashMap::new()
    };

    let result = substitute_variables(content, &variables);
    assert_eq!("field: ".to_owned(), result);
}