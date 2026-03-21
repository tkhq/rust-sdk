use crate::commands;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(about = "CLI for Turnkey backed auth workflows", long_about = None)]
pub struct Cli {
    #[command(flatten)]
    global: GlobalArgs,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, clap::Args)]
pub struct GlobalArgs {
    /// Output results as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Enable verbose output (HTTP requests, timing).
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

impl Cli {
    pub async fn run() -> anyhow::Result<()> {
        let args = Self::parse();

        if args.global.verbose {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("auth=debug")),
                )
                .with_target(false)
                .with_writer(std::io::stderr)
                .init();
        }

        match args.command {
            Command::Config(ref cmd_args) => commands::config::run(cmd_args, &args.global).await,
            Command::GitSign(cmd_args) => commands::git_sign::run(cmd_args).await,
            Command::PublicKey(ref cmd_args) => {
                commands::public_key::run(cmd_args, &args.global).await
            }
            Command::Whoami(ref cmd_args) => commands::whoami::run(cmd_args, &args.global).await,
            Command::Completion(cmd_args) => {
                let mut cmd = Self::command();
                clap_complete::generate(cmd_args.shell, &mut cmd, "auth", &mut std::io::stdout());
                Ok(())
            }
            Command::Introspect => {
                let cmd = Self::command();
                let schema = build_command_schema(&cmd);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&schema)
                        .expect("command schema should be serializable")
                );
                Ok(())
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect and update persistent auth configuration.
    Config(commands::config::Args),
    /// Sign a payload using the Git SSH signer interface.
    GitSign(commands::git_sign::Args),
    /// Print the configured SSH public key.
    PublicKey(commands::public_key::Args),
    /// Verify credentials and display the current identity.
    Whoami(commands::whoami::Args),
    /// Generate shell completions.
    Completion(CompletionArgs),
    /// Print all commands and flags as JSON (machine-readable introspection).
    Introspect,
}

#[derive(Debug, clap::Args)]
struct CompletionArgs {
    /// Shell to generate completions for.
    #[arg(short, long)]
    shell: Shell,
}

#[derive(Debug, Serialize)]
struct CommandSchema {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    flags: Vec<FlagSchema>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    commands: Vec<CommandSchema>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    subcommands: Vec<CommandSchema>,
}

#[derive(Debug, Serialize)]
struct FlagSchema {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    required: bool,
}

fn build_command_schema(cmd: &clap::Command) -> CommandSchema {
    CommandSchema {
        name: cmd.get_name().to_string(),
        description: cmd.get_about().map(|about| about.to_string()),
        flags: command_flags(cmd),
        commands: cmd.get_subcommands().map(command_schema).collect(),
        subcommands: Vec::new(),
    }
}

fn command_schema(cmd: &clap::Command) -> CommandSchema {
    CommandSchema {
        name: cmd.get_name().to_string(),
        description: cmd.get_about().map(|about| about.to_string()),
        flags: command_flags(cmd),
        commands: Vec::new(),
        subcommands: cmd.get_subcommands().map(command_schema).collect(),
    }
}

fn command_flags(cmd: &clap::Command) -> Vec<FlagSchema> {
    cmd.get_arguments()
        .filter(|arg| arg.get_id() != "help" && arg.get_id() != "version")
        .map(|arg| FlagSchema {
            name: arg.get_id().as_str().to_owned(),
            long: arg.get_long().map(|long| format!("--{long}")),
            short: arg.get_short().map(|short| format!("-{short}")),
            help: arg.get_help().map(|help| help.to_string()),
            required: arg.is_required_set(),
        })
        .collect()
}
