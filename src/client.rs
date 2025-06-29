use crate::config::AppConfig;
use crate::token::{TokenResponse, read_token, save_token, token_valid};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
    message: String,
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    grant_type: &'a str,
    client_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<&'a str>,
    device_code: &'a str,
}

pub async fn run_device_login(config: &AppConfig, profile: &str) -> Result<()> {
    let client = Client::new();
    let device_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode",
        config.tenant_id
    );
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.tenant_id
    );

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
            save_token(&token, profile)?;
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

pub async fn refresh_token(config: &AppConfig, profile: &str) -> Result<()> {
    let old_token = read_token(profile)?;

    let refresh_token = old_token
        .refresh_token
        .ok_or_else(|| anyhow::anyhow!("Missing refresh token"))?;

    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("client_id", &config.client_id);
    params.insert("refresh_token", &refresh_token);
    params.insert("scope", &config.scope);

    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        &config.tenant_id
    );
    let res = Client::new().post(&token_url).form(&params).send().await?;

    if res.status().is_success() {
        let mut new_token: TokenResponse = res.json().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        new_token.expires_at = Some(now + new_token.expires_in);
        save_token(&new_token, profile)?;
        Ok(())
    } else {
        let body = res.text().await?;
        error!("Refresh failed:\n{}", body);
        Err(anyhow::anyhow!("Refresh failed"))
    }
}
pub async fn print_token_or_refresh(config: &AppConfig, profile: &str) -> Result<()> {
    match read_token(profile) {
        Ok(token) if token_valid(&token) => {
            println!("{}", token.access_token);
            Ok(())
        }
        _ => {
            refresh_token(config, profile).await?;
            let token = read_token(profile)?;
            println!("{}", token.access_token);
            Ok(())
        }
    }
}
