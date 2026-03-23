use crate::commands;
use clap::{Parser, Subcommand};

const AFTER_HELP: &str = "\
Environment:
  TURNKEY_ORGANIZATION_ID
  TURNKEY_API_PUBLIC_KEY
  TURNKEY_API_PRIVATE_KEY
  TURNKEY_PRIVATE_KEY_ID
  TURNKEY_API_BASE_URL

Config file:
  Set TURNKEY_AUTH_CONFIG_PATH to override the config file location.
  Otherwise auth uses ~/.config/turnkey/auth.toml.

SSH agent:
  auth ssh-agent --socket /tmp/auth.sock
  export SSH_AUTH_SOCK=/tmp/auth.sock
";

#[derive(Debug, Parser)]
#[command(
    about = "CLI for Turnkey backed auth workflows",
    long_about = None,
    after_help = AFTER_HELP
)]
/// Top-level CLI arguments for the `auth` binary.
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Parses CLI arguments and dispatches to the selected subcommand.
    pub async fn run() -> anyhow::Result<()> {
        let args = Self::parse();

        match args.command {
            Commands::Config(args) => commands::config::run(args).await,
            Commands::SshAgent(args) => commands::agent::run(args).await,
            Commands::GitSign(args) => commands::git_sign::run(args).await,
            Commands::PublicKey(args) => commands::public_key::run(args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Inspect and update persistent auth configuration.
    Config(commands::config::Args),
    /// Run a foreground SSH agent over a Unix socket.
    SshAgent(commands::agent::Args),
    /// Sign a payload using the Git SSH signer interface.
    GitSign(commands::git_sign::Args),
    /// Print the configured SSH public key.
    PublicKey(commands::public_key::Args),
}
