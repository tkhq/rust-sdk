//! CLI commands.
//!
//! Each command module should contain:
//! - An `Args` struct deriving `clap::Args`
//! - A `run(args) -> anyhow::Result<()>` function

pub mod approve_manifest;
