use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::time::sleep;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;

    if std::env::args().any(|arg| arg == "--print-token") {
        return print_token_only().map_err(Into::into);
    }

    if std::env::args().any(|arg| arg == "--refresh") {
        return refresh_token(&config).await;
    }

    let device_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode",
        config.tenant_id
    );
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.tenant_id
    );

    let client = reqwest::Client::new();

    // Step 1: Request device code
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

    println!(
        "\nTo authenticate, visit: {}\nAnd enter code: {}\n",
        res.verification_uri, res.user_code
    );
    println!("{}\n", res.message);

    // Step 2: Poll for token
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

        println!("REQUEST: \n{}", serde_urlencoded::to_string(&request_body)?);

        if resp.status().is_success() {
            let token: TokenResponse = resp.json().await?;
            save_token(&token)?;
            println!("\nAccess token: {}\n", token.access_token);
            break;
        } else {
            let body = resp.text().await?;
            if !body.contains("authorization_pending") {
                eprintln!("\nError: {}", body);
                break;
            }
        }
    }
    Ok(())
}

async fn refresh_token(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("~/.config/neomutt/token.json").expand();
    let path = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let data = std::fs::read_to_string(path)?;
    let old_token: TokenResponse = serde_json::from_str(&data)?;

    let refresh_token = old_token.refresh_token.ok_or("Missing refresh token")?;

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
        let new_token: TokenResponse = res.json().await?;
        save_token(&new_token)?;
        println!("Refreshed access token.");
    } else {
        let body = res.text().await?;
        eprintln!("Refresh failed:\n{}", body);
    }

    Ok(())
}

fn print_token_only() -> std::io::Result<()> {
    let path = PathBuf::from("~/.config/neomutt/token.json").expand();
    let path = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let data = fs::read_to_string(path)?;
    let token: TokenResponse = serde_json::from_str(&data)?;
    println!("{}", token.access_token);
    Ok(())
}

fn save_token(token: &TokenResponse) -> std::io::Result<()> {
    let path = PathBuf::from("~/.config/neomutt/token.json").expand();
    let json = serde_json::to_string_pretty(&token)?;
    let path = shellexpand::tilde(&path.to_string_lossy()).to_string();
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = shellexpand::tilde("~/.config/msoauth/config.toml").to_string();
    let config_data = fs::read_to_string(config_path)?;
    let config: AppConfig = toml::from_str(&config_data)?;
    Ok(config)
}

trait ExpandPath {
    fn expand(&self) -> PathBuf;
}

impl ExpandPath for PathBuf {
    fn expand(&self) -> PathBuf {
        PathBuf::from(shellexpand::tilde(&self.to_string_lossy()).to_string())
    }
}
