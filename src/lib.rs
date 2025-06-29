pub mod args;
pub mod client;
pub mod config;
pub mod token;

use anyhow::Result;

use crate::{
    args::Cli,
    client::{print_token_or_refresh, refresh_token, run_device_login},
    config::load_profile,
    token::clear_token,
};

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let args = Cli::parse();
    let profile = args.profile.as_deref().unwrap_or("default");
    let config = load_profile(profile)?;

    if args.clear_token {
        clear_token(profile)?;
        return Ok(());
    }

    if args.print_token {
        return print_token_or_refresh(&config, profile).await;
    }

    if args.refresh {
        return refresh_token(&config, profile).await;
    }

    if args.login {
        return run_device_login(&config, profile).await;
    }

    match refresh_token(&config, profile).await {
        Ok(_) => Ok(()),
        Err(_) => run_device_login(&config, profile).await,
    }
}
