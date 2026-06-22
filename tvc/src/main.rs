use std::process::ExitCode;
use tracing::debug;
use tvc::cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    tvc::logging::init();
    debug!(version = env!("CARGO_PKG_VERSION"), "starting tvc");

    Cli::run().await
}
