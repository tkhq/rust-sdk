//! CLI commands.
//!
//! Each command module should contain:
//! - An `Args` struct deriving `clap::Args`
//! - A `run(args, config) -> anyhow::Result<()>` function

pub mod app;
pub mod deploy;
pub mod login;
