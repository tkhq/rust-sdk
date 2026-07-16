// Keep synchronized with the corresponding lint policy in the other TVC crate root.
// TVC has separate library and binary crates, so this attribute must appear in both.
#![deny(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "direct print macros bypass TVC's output protocol; use shell_println!, shell_print!, or shell_eprintln! for presentation output, Shell::emit for structured output, and tracing for diagnostics"
)]

use std::process::ExitCode;
use tracing::debug;
use tvc::cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    tvc::logging::init();
    debug!(version = env!("CARGO_PKG_VERSION"), "starting tvc");

    Cli::run().await
}
