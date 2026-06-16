//! Authentication management commands.

use crate::passkey::{PasskeyTransport, UnsupportedCeremony};
use anyhow::{Result, bail};
use clap::{Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Manage passkey authenticators.
    Passkey(PasskeyArgs),
}

#[derive(Debug, ClapArgs)]
pub struct PasskeyArgs {
    #[command(subcommand)]
    pub command: PasskeyCommands,
}

#[derive(Debug, Subcommand)]
pub enum PasskeyCommands {
    /// Register a new passkey authenticator for the active Turnkey user.
    Register(RegisterArgs),
    /// List passkey authenticators for the active Turnkey user.
    List(ListArgs),
    /// Remove a passkey authenticator by ID.
    Remove(RemoveArgs),
}

#[derive(Debug, ClapArgs)]
pub struct RegisterArgs {
    /// Human-readable passkey label in Turnkey.
    #[arg(long)]
    pub name: String,
    /// WebAuthn transport to use for user presence.
    #[arg(long, value_enum, default_value_t = PasskeyTransport::Auto)]
    pub passkey_transport: PasskeyTransport,
}

#[derive(Debug, ClapArgs)]
pub struct ListArgs {}

#[derive(Debug, ClapArgs)]
pub struct RemoveArgs {
    /// Authenticator ID to remove.
    pub id: String,
}

pub async fn run(args: Args, is_non_interactive: bool) -> Result<()> {
    match args.command {
        AuthCommands::Passkey(args) => run_passkey(args, is_non_interactive).await,
    }
}

async fn run_passkey(args: PasskeyArgs, is_non_interactive: bool) -> Result<()> {
    match args.command {
        PasskeyCommands::Register(args) => register(args, is_non_interactive).await,
        PasskeyCommands::List(_) => list().await,
        PasskeyCommands::Remove(args) => remove(args).await,
    }
}

async fn register(args: RegisterArgs, is_non_interactive: bool) -> Result<()> {
    if is_non_interactive {
        bail!("passkey registration requires an interactive terminal; remove --non-interactive");
    }

    let _ceremony = UnsupportedCeremony::new(args.passkey_transport);
    bail!(
        "passkey registration for '{}' is not available in this build; native/browser WebAuthn ceremony support is required",
        args.name
    )
}

async fn list() -> Result<()> {
    bail!("passkey listing is not available until Turnkey authenticator query wiring is added")
}

async fn remove(args: RemoveArgs) -> Result<()> {
    bail!(
        "passkey removal for '{}' is not available until Turnkey delete_authenticators wiring is added",
        args.id
    )
}
