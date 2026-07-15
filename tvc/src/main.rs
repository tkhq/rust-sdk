use tracing::debug;
use tvc::cli::Cli;

#[tokio::main]
async fn main() {
    tvc::logging::init();
    debug!(version = env!("CARGO_PKG_VERSION"), "starting tvc");

    if let Err(error) = Cli::run().await {
        if let Some(exit_error) = error.downcast_ref::<tvc::exit::ExitError>() {
            std::process::exit(exit_error.code());
        }

        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}
