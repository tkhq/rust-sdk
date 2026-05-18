use tvc::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tvc::logging::init();
    tracing::debug!(version = env!("CARGO_PKG_VERSION"), "starting tvc");

    Cli::run().await
}
