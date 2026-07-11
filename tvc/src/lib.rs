// Keep synchronized with the corresponding lint policy in the other TVC crate root.
// TVC has separate library and binary crates, so this attribute must appear in both.
#![deny(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "direct print macros bypass TVC's output protocol; use shell_println!, shell_print!, or shell_eprintln! for presentation output, Shell::emit for structured output, and tracing for diagnostics"
)]

/// Components of CLI for building with Turnkey Verifiable Cloud.
pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub(crate) mod local_operator_key;
pub mod logging;
pub mod outcome;
pub mod output;
pub mod pair;
pub mod prompts;
pub(crate) mod provisioning;
pub mod pull_secret;
pub(crate) mod quorum_key_metadata;
pub mod util;
