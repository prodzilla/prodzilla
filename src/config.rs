use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::probe::Probe;

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
        let config = load_config(PRODZILLA_YAML).await;
        assert_eq!(2, config.unwrap().probes.len());
    }
}
