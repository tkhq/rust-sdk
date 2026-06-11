//! CLI parsing and dispatch.

use crate::commands;
use clap::{ArgAction, Parser, Subcommand, builder::BoolishValueParser};
use tracing::debug;

const LONG_ABOUT: &str = "\
CLI for building with Turnkey Verifiable Cloud.

Some commands accept multiple configuration input types.
Configuration values are resolved in this order, highest priority first:
  1. Command-line flag (e.g. --app-id)
  2. Environment variable (e.g. TVC_APP_ID)
  3. Config file value (--config-file)
  4. Built-in default

Special rules (exceptions to the order above):
  --pivot-args replaces the config file's list entirely (does not append)
  Debug-mode flags (--dangerous-deploy-debug-mode and
    --dangerous-enable-debug-mode-deployments) are opt-in only: the flag
    or its env var can turn debug mode ON, but its absence never turns OFF
    a config file that enables it. To disable debug mode, set it false in
    the config file (or omit it) and do not pass the flag.

Authentication:
  Local: run `tvc login` once; commands then read ~/.config/turnkey/.
  CI:    set TVC_ORG_ID, TVC_API_KEY_PUBLIC, and TVC_API_KEY_PRIVATE
         to authenticate without files. Env vars take precedence over local
         config files. Setting some but not all three required vars will error.

Interactive behavior:
  By default, commands may prompt when stdin is a TTY. Use --non-interactive
  or set TVC_NON_INTERACTIVE=true to disable prompts and fail fast instead.";

/// CLI command parsing and dispatch.
#[derive(Debug, Parser)]
#[command(about = "CLI for building with Turnkey Verifiable Cloud", long_about = LONG_ABOUT)]
pub struct Cli {
    /// Disable interactive prompts and fail fast when required values are missing.
    #[arg(
        long,
        global = true,
        env = "TVC_NON_INTERACTIVE",
        action = ArgAction::SetTrue,
        value_parser = BoolishValueParser::new()
    )]
    non_interactive: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Run the CLI.
    pub async fn run() -> anyhow::Result<()> {
        let args = Cli::parse();
        debug!(
            command = args.command.name(),
            non_interactive = args.non_interactive,
            "dispatching"
        );

        let non_interactive = args.non_interactive;

        match args.command {
            Commands::Deploy { command } => match command {
                DeployCommands::Approve(args) => {
                    commands::deploy::approve::run(args, non_interactive).await
                }
                DeployCommands::GetStatus(args) => commands::deploy::get_status::run(args).await,
                DeployCommands::ProvisioningDetails(args) => {
                    commands::deploy::provisioning_details::run(args).await
                }
                DeployCommands::PostShare(args) => commands::deploy::post_share::run(args).await,
                DeployCommands::Status(args) => commands::deploy::status::run(args).await,
                DeployCommands::Create(args) => {
                    commands::deploy::create::run(args, non_interactive).await
                }
                DeployCommands::Init(args) => {
                    commands::deploy::init::run(args, non_interactive).await
                }
                DeployCommands::Delete(args) => commands::deploy::delete::run(args).await,
                DeployCommands::Restore(args) => commands::deploy::restore::run(args).await,
            },
            Commands::App { command } => match command {
                AppCommands::Status(args) => commands::app::status::run(args).await,
                AppCommands::List(args) => commands::app::list::run(args).await,
                AppCommands::Create(args) => {
                    commands::app::create::run(args, non_interactive).await
                }
                AppCommands::Init(args) => commands::app::init::run(args, non_interactive).await,
                AppCommands::SetLiveDeploy(args) => commands::app::set_live_deploy::run(args).await,
                AppCommands::Delete(args) => commands::app::delete::run(args).await,
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
            Commands::Login(args) => commands::login::run(args, non_interactive).await,
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

impl Commands {
    fn name(&self) -> &'static str {
        match self {
            Commands::Login(_) => "login",
            Commands::Deploy { command } => command.name(),
            Commands::App { command } => command.name(),
            Commands::Keys { command } => command.name(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum DeployCommands {
    /// Approve a deployment manifest.
    Approve(commands::deploy::approve::Args),
    /// Get live runtime status for a deployment from the cluster.
    GetStatus(commands::deploy::get_status::Args),
    /// Get provisioning details for a deployment.
    ProvisioningDetails(commands::deploy::provisioning_details::Args),
    /// Post a re-encrypted quorum key share for a deployment.
    PostShare(commands::deploy::post_share::Args),
    /// Get the status of a deployment.
    Status(commands::deploy::status::Args),
    /// Create a new deployment from a config file.
    #[command(
        long_about = commands::deploy::create::LONG_ABOUT,
        after_help = commands::deploy::PORT_GUIDANCE
    )]
    Create(commands::deploy::create::Args),
    /// Generate a template deployment configuration file.
    Init(commands::deploy::init::Args),
    /// Delete a deployment by marking it for deletion.
    Delete(commands::deploy::delete::Args),
    /// Restore a deleted deployment.
    Restore(commands::deploy::restore::Args),
}

impl DeployCommands {
    fn name(&self) -> &'static str {
        match self {
            DeployCommands::Approve(_) => "deploy approve",
            DeployCommands::GetStatus(_) => "deploy get-status",
            DeployCommands::ProvisioningDetails(_) => "deploy provisioning-details",
            DeployCommands::PostShare(_) => "deploy post-share",
            DeployCommands::Status(_) => "deploy status",
            DeployCommands::Create(_) => "deploy create",
            DeployCommands::Init(_) => "deploy init",
            DeployCommands::Delete(_) => "deploy delete",
            DeployCommands::Restore(_) => "deploy restore",
        }
    }
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
    /// Set the live deployment for an app.
    SetLiveDeploy(commands::app::set_live_deploy::Args),
    /// Delete an app and all of its deployments.
    Delete(commands::app::delete::Args),
}

#[derive(Debug, Subcommand)]
enum KeysCommands {
    /// Generate and shamir-split a quorum key, encrypting each share to an operator key.
    GenerateQuorumKey(commands::keys::generate_quorum_key::Args),
    /// Generate a template quorum key configuration file.
    InitQuorumKey(commands::keys::init_quorum_key::Args),
    /// Re-encrypt a share for enclave provisioning.
    ReEncryptShare(commands::keys::re_encrypt_share::Args),
}

impl AppCommands {
    fn name(&self) -> &'static str {
        match self {
            AppCommands::Status(_) => "app status",
            AppCommands::List(_) => "app list",
            AppCommands::Create(_) => "app create",
            AppCommands::Init(_) => "app init",
            AppCommands::SetLiveDeploy(_) => "app set-live-deploy",
            AppCommands::Delete(_) => "app delete",
        }
    }
}

impl KeysCommands {
    fn name(&self) -> &'static str {
        match self {
            KeysCommands::GenerateQuorumKey(_) => "keys generate-quorum-key",
            KeysCommands::InitQuorumKey(_) => "keys init-quorum-key",
            KeysCommands::ReEncryptShare(_) => "keys re-encrypt-share",
        }
    }
}
