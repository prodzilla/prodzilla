use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

fn substitute_variables(yaml_content: &str, variables: &HashMap<String, Value>) -> String {
    let re = Regex::new(r"\$\{\{(.*?)\}\}").unwrap();
    re.replace_all(yaml_content, |caps: &regex::Captures| {
        let placeholder = &caps[1];
        let parts: Vec<&str> = placeholder.split('.').collect();

        // Identify the base key (e.g., "steps.get-token.body")
        let base_key = parts.iter().take(4).cloned().collect::<Vec<&str>>().join(".");

        if let Some(base_value) = variables.get(&base_key) {
            if parts.len() > 4 {
                // Navigate through JSON for additional fields
                parts[3..].iter().fold(Some(base_value), |acc, part| {
                    acc.and_then(|v| v.get(part))
                })
                .map(|v| json_value_to_string(v))
                .unwrap_or_else(|| "".to_string())
            } else {
                // Use the entire JSON object
                let test = serde_json::to_string(base_value).unwrap_or_default();
                test
            }
        } else {
            "".to_string()
        }
    }).to_string()
}

use serde_json::{self};

fn json_value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

#[tokio::test]
async fn test_substitute_variables() {
    let yaml_content = r#"{"token":"${{steps.get-token.body.token}}","entire_body":${{steps.get-token.body}}}"#;

    let body_str = r#"{
        "token": "12345",
        "other_field": "value"
    }"#;

    // Parse the JSON string into a serde_json::Value
    let body: serde_json::Value = serde_json::from_str(body_str).unwrap();
    let test = serde_json::to_string(&body).unwrap();
    print!("{}", test);

    let variables = HashMap::from([
        ("steps.get-token.response.body".to_string(), body),
    ]);

    let result = substitute_variables(yaml_content, &variables);

    let expected_result = r#"{"token":"12345","entire_body":{"other_field":"value","token":"12345"}}"#;

    assert_eq!(result, expected_result);
}