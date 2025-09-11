use anyhow::Result;
use worldlycli::run;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(run().await?)
}
