//! CLI parsing and dispatch.

use crate::commands;
use clap::{Parser, Subcommand};

/// Global configuration available to all commands.
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub organization_id: Option<String>,
    pub api_base_url: String,
}

/// CLI command parsing and dispatch.
#[derive(Debug, Parser)]
#[command(about = "CLI for building with Turnkey Verifiable Cloud", long_about = None)]
pub struct Cli {
    /// Turnkey organization ID.
    #[arg(long, global = true, env = "TVC_ORGANIZATION_ID")]
    pub organization_id: Option<String>,

    /// API base URL.
    #[arg(
        long,
        global = true,
        env = "TVC_API_BASE_URL",
        default_value = "https://api.turnkey.com"
    )]
    pub api_base_url: String,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Run the CLI.
    pub async fn run() -> anyhow::Result<()> {
        let args = Cli::parse();

        let config = GlobalConfig {
            organization_id: args.organization_id,
            api_base_url: args.api_base_url,
        };

        match args.command {
            Commands::ApproveManifest(cmd_args) => {
                commands::approve_manifest::run(cmd_args, &config).await
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    ApproveManifest(commands::approve_manifest::Args),
}
