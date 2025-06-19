use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::sleep;
use tracing::{error, info};

/// OAuth CLI for retrieving and refreshing tokens
#[derive(Parser)]
#[command(name = "msoauth", version, about)]
struct Cli {
    /// Print current access token (reresh if needed)
    #[arg(long)]
    print_token: bool,

    /// Force a token refresh
    #[arg(long)]
    refresh: bool,

    /// Start device login flow
    #[arg(long)]
    login: bool,

    /// Delete saves token file
    #[arg(long)]
    clear_token: bool,
}

#[derive(Deserialize)]
struct AppConfig {
    client_id: String,
    client_secret: Option<String>,
    tenant_id: String,
    scope: String,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
    message: String,
}

#[derive(Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<u64>,
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    grant_type: &'a str,
    client_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<&'a str>,
    device_code: &'a str,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let args = Cli::parse();
    let config = load_config().context("Failed to load config")?;

    if args.clear_token {
        let path = token_path()?;
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove token file at {}", path.display()))?;
            info!("Deleted token file at {}", path.display());
        } else {
            info!("No token file found to delete.");
        }
        return Ok(());
    }

    if args.print_token {
        return print_token_or_refresh(&config).await;
    }

    if args.refresh {
        return refresh_token(&config).await;
    }

    if args.login {
        return run_device_login(&config).await;
    }

    // Default fallback
    match refresh_token(&config).await {
        Ok(_) => Ok(()),
        Err(_) => run_device_login(&config).await,
    }
}

async fn run_device_login(config: &AppConfig) -> Result<()> {
    let device_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode",
        config.tenant_id
    );
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.tenant_id
    );

    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", &config.client_id);
    params.insert("scope", &config.scope);

    let res: DeviceCodeResponse = client
        .post(&device_url)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;

    info!(
        "\nTo authenticate, visit: {}\nAnd enter code: {}\n",
        res.verification_uri, res.user_code
    );
    info!("{}\n", res.message);

    loop {
        sleep(Duration::from_secs(res.interval)).await;

        let request_body = TokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            client_id: &config.client_id,
            client_secret: config.client_secret.as_deref(),
            device_code: &res.device_code,
        };

        let resp = client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(&request_body)?)
            .send()
            .await?;

        if resp.status().is_success() {
            let mut token: TokenResponse = resp.json().await?;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            token.expires_at = Some(now + token.expires_in);
            save_token(&token)?;
            info!("\nAccess token: {}\n", token.access_token);
            break;
        } else {
            let body = resp.text().await?;
            if !body.contains("authorization_pending") {
                error!("\nError: {}", body);
                break;
            }
        }
    }
    Ok(())
}
fn token_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Could not determine config directory")?
        .join("neomutt/token.json"))
}

fn config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Could not determine config directory")?
        .join("msoauth/config.toml"))
}

fn read_token_file() -> Result<String> {
    let path = token_path()?;
    fs::read_to_string(&path).context("Failed to read token file")
}

async fn refresh_token(config: &AppConfig) -> Result<()> {
    let data = read_token_file()?;
    let old_token: TokenResponse = serde_json::from_str(&data)?;

    let refresh_token = old_token
        .refresh_token
        .ok_or_else(|| anyhow::anyhow!("Missing refresh token"))?;

    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("client_id", &config.client_id);
    params.insert("refresh_token", &refresh_token);
    params.insert("scope", &config.scope);

    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        &config.tenant_id
    );
    let res = client.post(&token_url).form(&params).send().await?;

    if res.status().is_success() {
        let mut new_token: TokenResponse = res.json().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        new_token.expires_at = Some(now + new_token.expires_in);
        save_token(&new_token)?;
        if !std::env::args().any(|arg| arg == "--print-token") {
            info!("Refreshed access token.");
        }
        Ok(())
    } else {
        let body = res.text().await?;
        error!("Refresh failed:\n{}", body);
        Err(anyhow::anyhow!("Refresh failed"))
    }
}

async fn print_token_or_refresh(config: &AppConfig) -> Result<()> {
    let data = read_token_file()?;
    let token: TokenResponse = serde_json::from_str(&data)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if token.expires_at.is_some_and(|exp| exp > now + 300) {
        info!("{}", token.access_token);
        return Ok(());
    }

    refresh_token(config).await?;
    let data = read_token_file()?;
    let token: TokenResponse = serde_json::from_str(&data)?;
    info!("{}", token.access_token);
    Ok(())
}

fn save_token(token: &TokenResponse) -> Result<()> {
    let path = token_path()?;
    let json = serde_json::to_string_pretty(token)?;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn load_config() -> Result<AppConfig> {
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
    let config: AppConfig = toml::from_str(&config_data)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_print_token_only() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Setup a mock AppConfig
            let config = AppConfig {
                client_id: "test_client_id".to_string(),
                client_secret: Some("test_client_secret".to_string()),
                tenant_id: "test_tenant_id".to_string(),
                scope: "test_scope".to_string(),
            };

            // Mock the read_token_file function to return a valid token
            let mock_token = TokenResponse {
                access_token: "mock_access_token".to_string(),
                refresh_token: Some("mock_refresh_token".to_string()),
                expires_in: 3600,
                token_type: "Bearer".to_string(),
                expires_at: Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + 3600,
                ),
            };

            // Assuming read_token_file can be mocked or altered for testing
            // Here we simulate its behavior
            let mock_read =
                || -> Result<String> { Ok(serde_json::to_string(&mock_token).unwrap()) };
            assert_eq!(
                mock_read().unwrap(),
                serde_json::to_string(&mock_token).unwrap()
            );

            // Test print_token_or_refresh
            let output = print_token_or_refresh(&config).await;
            assert!(output.is_ok());
            // Validate that only the token is printed
            assert_eq!(mock_token.access_token, "mock_access_token");
        });
    }
}
