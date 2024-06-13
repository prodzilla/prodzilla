use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::probe::model::Probe;
use crate::probe::model::Story;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub probes: Vec<Probe>,
    #[serde(default)]
    pub stories: Vec<Story>,
}

pub async fn load_config<P: Into<PathBuf>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let path = path.into();
    let config = match tokio::fs::read_to_string(path.clone()).await {
        Ok(content) => content,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("Config file not found: {:?}", path)
        }
        Err(e) => {
            panic!("Failed to read config file: {:?}, err {}", path, e)
        }
    };
    let config = replace_env_vars(&config);
    let config: Config = serde_yaml::from_str(&config)?;
    Ok(config)
}

pub fn replace_env_vars(content: &str) -> String {
    let re = regex::Regex::new(r"\$\{\{\s*env\.(.*?)\s*\}\}").unwrap();
    let replaced = re.replace_all(content, |caps: &regex::Captures| {
        let var_name = &caps[1];
        // panics on missing enivronment variables, probably desirable?
        match std::env::var(var_name) {
            Ok(val) => val,
            Err(_) => {
                warn!(
                    "Environment variable {} not found, defaulting to empty string.",
                    var_name
                );
                "".to_string()
            }
        }
    });
    replaced.to_string()
}

#[cfg(test)]
mod config_tests {
    use crate::{config::load_config, PRODZILLA_YAML};
    use std::env;

    #[tokio::test]
    async fn test_app_yaml_can_load() {
        let config_result = load_config(PRODZILLA_YAML).await;

        // Assert that the config is successfully loaded
        assert!(config_result.is_ok(), "Failed to load config");

        // Borrow the config for subsequent operations
        let config = config_result.as_ref().unwrap();

        // Perform multiple tests using borrowed references
        assert_eq!(1, config.probes.len(), "Probes length should be 1");
        assert_eq!(1, config.stories.len(), "Stories length should be 1");
    }

    #[tokio::test]
    async fn test_env_substitution() {
        env::set_var("TEST_ENV_VAR", "test_value");
        let content = "Environment variable ${{ env.TEST_ENV_VAR }} should be replaced even with varying whitespace ${{env.TEST_ENV_VAR}}${{ env.TEST_ENV_VAR}}  ${{env.TEST_ENV_VAR }}${{ env.TEST_ENV_VAR     }}, missing ${{ env.MISSING_VAR }} should be empty";
        let replaced = super::replace_env_vars(content);
        assert_eq!(
            "Environment variable test_value should be replaced even with varying whitespace test_valuetest_value  test_valuetest_value, missing  should be empty",
            replaced
        );
    }
}
