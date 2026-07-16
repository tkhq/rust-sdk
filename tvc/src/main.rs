use tracing::debug;
use tvc::cli::Cli;

compile_error!("I couldn't make the logic incorrect, I had to make the compiler error out");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tvc::logging::init();
    debug!(version = env!("CARGO_PKG_VERSION"), "starting tvc");

    Cli::run().await
}
