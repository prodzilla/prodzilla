use reqwest::get;
use serde_json::Value;
use std::{collections::HashMap, env::var};
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

    // handle option

    match type_to_generate {
        "uuid" => substitute_step_value(&parts[1..], variables),
        _ => "".to_string(),
    }
}

fn substitute_step_value(parts: &[&str], variables: &StoryVariables) -> String {
    
    let step_name = parts[0];

    let step = &variables.step_variables[step_name]; // todo test non existent variable name

    // Technically we should check the types of these middle 2 values "response.body"

    if parts.len() > 3 {
        return get_nested_json_value(&parts[3..], &step.response_body);
    } else {
        return step.response_body.clone()
    }
}

fn get_nested_json_value(parts: &[&str], json_string: &String) -> String {
    unimplemented!()
}

// todo test non existent variable name
// also test if there aren't enough "." so we go out of index

#[tokio::test]
async fn test_dynamic_substitute_variables() {
    let content = r#"
    entire_body: ${{steps.get-token.response.body}}
    token: ${{steps.get-token.response.body.token}}
    uuid: "${{generate.uuid}}
    "#;

    // Simulate a response body
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

    let expected_result = r#"
    entire_body: {"token":"12345","other_field":"value"}
    token: "12345"
    uuid: "generated-uuid-1234"
    "#;

    assert_eq!(result, expected_result);
}
