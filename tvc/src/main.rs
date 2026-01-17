use tvc::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (silently ignore if missing)
    let _ = dotenvy::dotenv();

    Cli::run().await
}
