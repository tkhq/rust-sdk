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
                DeployCommands::GetStatus(args) => commands::deploy::get_status::run(args).await,
                DeployCommands::ProvisioningDetails(args) => {
                    commands::deploy::provisioning_details::run(args).await
                }
                DeployCommands::Status(args) => commands::deploy::status::run(args).await,
                DeployCommands::Create(args) => commands::deploy::create::run(args).await,
                DeployCommands::Init(args) => commands::deploy::init::run(args).await,
            },
            Commands::App { command } => match command {
                AppCommands::Status(args) => commands::app::status::run(args).await,
                AppCommands::List(args) => commands::app::list::run(args).await,
                AppCommands::Create(args) => commands::app::create::run(args).await,
                AppCommands::Init(args) => commands::app::init::run(args).await,
            },
            Commands::Keys { command } => match command {
                KeysCommands::GenerateQuorumKey(args) => {
                    commands::keys::generate_quorum_key::run(args).await
                }
                KeysCommands::InitQuorumKey(args) => {
                    commands::keys::init_quorum_key::run(args).await
                }
                KeysCommands::ReEncryptShare(args) => {
                    commands::keys::re_encrypt_share::run(args).await
                }
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
    /// Manage cryptographic keys.
    Keys {
        #[command(subcommand)]
        command: KeysCommands,
    },
}

#[derive(Debug, Subcommand)]
enum DeployCommands {
    /// Approve a deployment manifest.
    Approve(commands::deploy::approve::Args),
    /// Get live runtime status for a deployment from the cluster.
    GetStatus(commands::deploy::get_status::Args),
    /// Get provisioning details for a deployment.
    ProvisioningDetails(commands::deploy::provisioning_details::Args),
    /// Get the status of a deployment.
    Status(commands::deploy::status::Args),
    /// Create a new deployment from a config file.
    Create(commands::deploy::create::Args),
    /// Generate a template deployment configuration file.
    Init(commands::deploy::init::Args),
}

#[derive(Debug, Subcommand)]
enum AppCommands {
    /// Get live runtime status for an app from the cluster.
    Status(commands::app::status::Args),
    /// List applications.
    List(commands::app::list::Args),
    /// Create a new application from a config file.
    Create(commands::app::create::Args),
    /// Generate a template app configuration file.
    Init(commands::app::init::Args),
}

#[derive(Debug, Subcommand)]
enum KeysCommands {
    /// Generate and encrypt a quorum key from a config file.
    GenerateQuorumKey(commands::keys::generate_quorum_key::Args),
    /// Generate a template quorum key configuration file.
    InitQuorumKey(commands::keys::init_quorum_key::Args),
    /// Re-encrypt a share for enclave provisioning.
    ReEncryptShare(commands::keys::re_encrypt_share::Args),
}
