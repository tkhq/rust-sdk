//! CLI parsing and dispatch.

use crate::commands;
use clap::{Parser, Subcommand};

/// CLI command parsing and dispatch.
#[derive(Debug, Parser)]
#[command(about = "CLI for building with Turnkey Verifiable Cloud", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Run the CLI.
    pub async fn run() -> anyhow::Result<()> {
        let args = Cli::parse();

        match args.command {
            Commands::Deploy { command } => match command {
                DeployCommands::Approve(args) => commands::deploy::approve::run(args).await,
                DeployCommands::Status(args) => commands::deploy::status::run(args).await,
                DeployCommands::Create(args) => commands::deploy::create::run(args).await,
                DeployCommands::Init(args) => commands::deploy::init::run(args).await,
            },
            Commands::App { command } => match command {
                AppCommands::List(args) => commands::app::list::run(args).await,
                AppCommands::Create(args) => commands::app::create::run(args).await,
                AppCommands::Init(args) => commands::app::init::run(args).await,
            },
            Commands::Login(args) => commands::login::run(args).await,
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
    /// Create a new deployment from a config file.
    Create(commands::deploy::create::Args),
    /// Generate a template deployment configuration file.
    Init(commands::deploy::init::Args),
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
