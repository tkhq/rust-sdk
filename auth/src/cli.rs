use crate::commands;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "CLI for Turnkey backed auth workflows", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub async fn run() -> anyhow::Result<()> {
        let args = Self::parse();

        match args.command {
            Commands::Config(args) => commands::config::run(args).await,
            Commands::GitSign(args) => commands::git_sign::run(args).await,
            Commands::PublicKey(args) => commands::public_key::run(args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Inspect and update persistent auth configuration.
    Config(commands::config::Args),
    /// Sign a payload using the Git SSH signer interface.
    GitSign(commands::git_sign::Args),
    /// Print the configured SSH public key.
    PublicKey(commands::public_key::Args),
}
