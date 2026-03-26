//! CLI parsing and dispatch.

use crate::commands;
use clap::{Args as ClapArgs, Parser, Subcommand};
use std::io::IsTerminal;

/// Global options available to all commands.
#[derive(Debug, Clone, ClapArgs)]
pub struct GlobalOpts {
    /// Output results as JSON.
    #[arg(long, global = true, env = "TVC_JSON")]
    pub json: bool,

    /// Disable all interactive prompts. Fails if input is required.
    #[arg(long, global = true, env = "TVC_NO_INPUT")]
    pub no_input: bool,

    /// Suppress non-essential output.
    #[arg(long, short, global = true)]
    pub quiet: bool,
}

impl GlobalOpts {
    /// Returns true if interactive prompts should be suppressed.
    /// This is true when --no-input is explicitly set OR when stdin is not a terminal.
    pub fn is_no_input(&self) -> bool {
        self.no_input || !std::io::stdin().is_terminal()
    }
}

/// CLI command parsing and dispatch.
#[derive(Debug, Parser)]
#[command(about = "CLI for building with Turnkey Verifiable Cloud", long_about = None)]
pub struct Cli {
    #[command(flatten)]
    global: GlobalOpts,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Run the CLI.
    pub async fn run() -> anyhow::Result<()> {
        let args = Cli::parse();
        let global = args.global;

        match args.command {
            Commands::Deploy { command } => match command {
                DeployCommands::Approve(cmd_args) => {
                    commands::deploy::approve::run(cmd_args, &global).await
                }
                DeployCommands::Status(cmd_args) => {
                    commands::deploy::status::run(cmd_args, &global).await
                }
                DeployCommands::Create(cmd_args) => {
                    commands::deploy::create::run(cmd_args, &global).await
                }
                DeployCommands::Init(cmd_args) => {
                    commands::deploy::init::run(cmd_args, &global).await
                }
            },
            Commands::App { command } => match command {
                AppCommands::List(cmd_args) => commands::app::list::run(cmd_args, &global).await,
                AppCommands::Create(cmd_args) => {
                    commands::app::create::run(cmd_args, &global).await
                }
                AppCommands::Init(cmd_args) => commands::app::init::run(cmd_args, &global).await,
            },
            Commands::Login(cmd_args) => commands::login::run(cmd_args, &global).await,
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
