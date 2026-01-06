use std::{collections::HashSet, path::PathBuf};

use serde::{de, Deserialize, Deserializer, Serialize};
use tracing::warn;

use crate::probe::model::{Monitor, Probe, Story};

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    #[serde(default)]
    pub monitors: Vec<Monitor>,
    // Keep these for backward compatibility during transition
    #[serde(default, skip_serializing)]
    pub probes: Vec<Probe>,
    #[serde(default, skip_serializing)]
    pub stories: Vec<Story>,
}

// Custom deserialization to validate unique monitor names
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConfigHelper {
            #[serde(default)]
            monitors: Vec<Monitor>,
            #[serde(default)]
            probes: Vec<Probe>,
            #[serde(default)]
            stories: Vec<Story>,
        }

        let helper = ConfigHelper::deserialize(deserializer)?;

        // Validate unique monitor names
        let mut seen_names = HashSet::new();
        for monitor in &helper.monitors {
            if !seen_names.insert(&monitor.name) {
                return Err(de::Error::custom(format!(
                    "Duplicate monitor name: '{}'",
                    monitor.name
                )));
            }
        }

        Ok(Config {
            monitors: helper.monitors,
            probes: helper.probes,
            stories: helper.stories,
        })
    }
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
        assert_eq!(2, config.monitors.len(), "Monitors length should be 2");
        // Verify one is single-step and one is multi-step
        let single_step_count = config
            .monitors
            .iter()
            .filter(|m| m.is_single_step())
            .count();
        let multi_step_count = config.monitors.iter().filter(|m| m.is_multi_step()).count();
        assert_eq!(1, single_step_count, "Should have 1 single-step monitor");
        assert_eq!(1, multi_step_count, "Should have 1 multi-step monitor");
    }

    #[tokio::test]
    async fn test_monitor_validation_requires_steps_or_url() {
        let yaml = r#"
monitors:
  - name: invalid-monitor
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: Result<super::Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_monitor_validation_rejects_both_steps_and_url() {
        let yaml = r#"
monitors:
  - name: invalid-monitor
    url: https://example.com
    http_method: GET
    steps:
      - name: step1
        url: https://example.com
        http_method: GET
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: Result<super::Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_monitor_validation_duplicate_names() {
        let yaml = r#"
monitors:
  - name: duplicate
    url: https://example.com
    http_method: GET
    schedule:
      initial_delay: 0
      interval: 60
  - name: duplicate
    url: https://example2.com
    http_method: GET
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: Result<super::Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_monitor_validation_duplicate_step_names() {
        let yaml = r#"
monitors:
  - name: test-monitor
    steps:
      - name: duplicate
        url: https://example.com
        http_method: GET
      - name: duplicate
        url: https://example2.com
        http_method: GET
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: Result<super::Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_monitor_validation_empty_steps() {
        let yaml = r#"
monitors:
  - name: test-monitor
    steps: []
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: Result<super::Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err());
    }

    #[tokio::test]
    async fn test_monitor_single_step_conversion() {
        let yaml = r#"
monitors:
  - name: test-probe
    url: https://example.com
    http_method: GET
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: super::Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, config.monitors.len());
        let monitor = &config.monitors[0];
        assert!(monitor.is_single_step());
        assert!(!monitor.is_multi_step());
        let probe = monitor.to_probe();
        assert!(probe.is_some());
        assert_eq!("test-probe", probe.unwrap().name);
    }

    #[tokio::test]
    async fn test_monitor_multi_step_conversion() {
        let yaml = r#"
monitors:
  - name: test-story
    steps:
      - name: step1
        url: https://example.com
        http_method: GET
      - name: step2
        url: https://example2.com
        http_method: POST
    schedule:
      initial_delay: 0
      interval: 60
"#;
        let config: super::Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(1, config.monitors.len());
        let monitor = &config.monitors[0];
        assert!(monitor.is_multi_step());
        assert!(!monitor.is_single_step());
        let story = monitor.to_story();
        assert!(story.is_some());
        let story = story.unwrap();
        assert_eq!("test-story", story.name);
        assert_eq!(2, story.steps.len());
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
