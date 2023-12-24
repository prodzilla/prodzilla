pub struct Config {
    #[serde(default)]
    pub probes: Vec<Probe>,
}

pub async fn load_config<P: Into<PathBuf>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let path = path.into();
    let config = tokio::fs::read_to_string(path).await?;
    let config: Config = serde_yaml::from_str(&config)?;
    Ok(config)
}