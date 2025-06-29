use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub tenant_id: String,
    pub scope: String,
}

pub type ConfigMap = HashMap<String, AppConfig>;

pub fn config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Could not determine config directory")?
        .join("msoauth/config.toml"))
}

pub fn load_profile(profile: &str) -> Result<AppConfig> {
    let path = config_path()?;

    if !path.exists() {
        anyhow::bail!(
            "Config file not found at {}\n\n\
            Please create it with the following format:\n\n\
            [default]\n\
            client_id = \"...\"\n\
            client_secret = \"...\"\n\
            tenant_id = \"...\"\n\
            scope = \"https://graph.microsoft.com/.default\"",
            path.display()
        );
    }

    let config_data = fs::read_to_string(&path)?;
    let profiles: ConfigMap = toml::from_str(&config_data)
        .with_context(|| format!("Failed to parse config from {}", path.display()))?;

    profiles
        .get(profile)
        .cloned()
        .with_context(|| format!("Auth profile `{profile}` not found in config"))
}
