use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
    let config = tokio::fs::read_to_string(path).await?;
    let config: Config = serde_yaml::from_str(&config)?;
    Ok(config)
}

#[cfg(test)]
mod config_tests {
    use crate::{config::load_config, PRODZILLA_YAML};

    #[tokio::test]
    async fn test_app_yaml_can_load() {
        let config_result = load_config(PRODZILLA_YAML).await;

        // Assert that the config is successfully loaded
        assert!(config_result.is_ok(), "Failed to load config");

        // Borrow the config for subsequent operations
        let config = config_result.as_ref().unwrap();

        // Perform multiple tests using borrowed references
        assert_eq!(1, config.probes.len(), "Probes length should be 2");
        assert_eq!(1, config.stories.len(), "Stories length should be 1"); 
    }
}
