//! CLI parsing and dispatch.

use crate::commands;
use clap::{Parser, Subcommand};

/// Global configuration available to all commands.
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub api_base_url: String,
}

/// CLI command parsing and dispatch.
#[derive(Debug, Parser)]
#[command(about = "CLI for building with Turnkey Verifiable Cloud", long_about = None)]
pub struct Cli {
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
            api_base_url: args.api_base_url,
        };

        match args.command {
            Commands::Deploy { command } => match command {
                DeployCommands::Approve(args) => {
                    commands::deploy::approve::run(args, &config).await
                }
                DeployCommands::Status(args) => commands::deploy::status::run(args, &config).await,
                DeployCommands::Create(args) => commands::deploy::create::run(args, &config).await,
            },
            Commands::App { command } => match command {
                AppCommands::List(args) => commands::app::list::run(args, &config).await,
                AppCommands::Create(args) => commands::app::create::run(args, &config).await,
                AppCommands::Init(args) => commands::app::init::run(args, &config).await,
            },
            Commands::Login(args) => commands::login::run(args, &config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Authenticate with Turnkey and set up local credentials.
    Login(commands::login::Args),
    /// Manage deployments.
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },
    /// Manage applications.
    App {
        #[command(subcommand)]
        command: AppCommands,
    },
}

#[derive(Debug, Subcommand)]
enum DeployCommands {
    /// Approve a deployment manifest.
    Approve(commands::deploy::approve::Args),
    /// Get the status of a deployment.
    Status(commands::deploy::status::Args),
    /// Create a new deployment.
    Create(commands::deploy::create::Args),
}

#[derive(Debug, Subcommand)]
enum AppCommands {
    /// List applications.
    List(commands::app::list::Args),
    /// Create a new application from a config file.
    Create(commands::app::create::Args),
    /// Generate a template app configuration file.
    Init(commands::app::init::Args),
}
