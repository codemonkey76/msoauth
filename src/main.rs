use anyhow::Result;
use msoauth::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
