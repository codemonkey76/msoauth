use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "msoauth", version, about)]
pub struct Cli {
    /// Print current access token (reresh if needed)
    #[arg(long)]
    pub print_token: bool,

    /// Force a token refresh
    #[arg(long)]
    pub refresh: bool,

    /// Start device login flow
    #[arg(long)]
    pub login: bool,

    /// Delete saves token file
    #[arg(long)]
    pub clear_token: bool,

    /// Profile name (default if not specified)
    #[arg(long, default_value = "default")]
    pub profile: Option<String>,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
