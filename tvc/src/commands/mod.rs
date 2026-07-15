//! CLI commands.
//!
//! Each command module should contain:
//! - An `Args` struct deriving `clap::Args`
//! - A `run(args, config) -> anyhow::Result<()>` function

pub mod activity;
pub mod app;
pub mod app_status;
pub mod confirmation;
pub(crate) mod consensus;
pub mod deploy;
pub mod display;
pub mod keys;
pub mod login;
