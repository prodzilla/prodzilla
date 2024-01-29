use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

pub struct StoryVariables {
    pub step_variables: HashMap<String, StepVariables>
}

pub struct StepVariables {
    pub response_body: String
}

fn dynamic_substitute_variables(content: &str, variables: &StoryVariables) -> String {
    let re = Regex::new(r"\$\{\{(.*?)\}\}").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
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
    unimplemented!()
}

fn substitute_step_value(parts: &[&str], variables: &StoryVariables) -> String {
    
    // Use the first index to find the right step
    let key = parts.join(".");


    // If needed do the finnicky json thing - I think the old code was better for that
    steps_data.get(&key)
        .map(|value| {
            if parts.len() > 1 {
                value.get(parts.last().unwrap()).map_or("".to_string(), |v| v.to_string())
            } else {
                value.to_string()
            }
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn test_dynamic_substitute_variables() {
    let yaml_content = r#"
    entire_body: ${{steps.get-token.response.body}}
    token: "${{steps.get-token.response.body.token}}"
    uuid: "${{generate.uuid}}"
    "#;

    // Simulate a response body
    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;
    let body: serde_json::Value = serde_json::from_str(body_str).unwrap();

    // Simulate steps data
    let steps_data = HashMap::from([
        ("get-token.response.body".to_string(), body),
    ]);

    // Simulate UUID generation (for testing, using a fixed value)
    let uuid = "generated-uuid-1234";

    let result = dynamic_substitute_variables(yaml_content, &steps_data, uuid);

    let expected_result = r#"
    entire_body: {"token":"12345","other_field":"value"}
    token: "12345"
    uuid: "generated-uuid-1234"
    "#;

    assert_eq!(result, expected_result);
}
