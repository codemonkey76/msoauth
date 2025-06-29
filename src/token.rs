use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

pub fn token_path(profile: &str) -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Could not determine config directory")?
        .join(format!("msoauth/{profile}.json")))
}

pub fn save_token(token: &TokenResponse, profile: &str) -> Result<()> {
    let path = token_path(profile)?;
    let json = serde_json::to_string_pretty(token)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn read_token(profile: &str) -> Result<TokenResponse> {
    let path = token_path(profile)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read token file: {}", path.display()))?;
    let token: TokenResponse = serde_json::from_str(&content)?;
    Ok(token)
}

pub fn token_valid(token: &TokenResponse) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    token.expires_at.is_some_and(|exp| exp > now + 300)
}

pub fn clear_token(profile: &str) -> Result<()> {
    let path = token_path(profile)?;
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("failed to remove token file: {}", path.display()))?;
    }

    Ok(())
}
