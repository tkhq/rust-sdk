//! CLI parsing and dispatch.

use crate::commands;
use crate::outcome::Outcome;
use crate::output::{ColorChoice, Ctx, ErrorMessage, Message, MessageFormat, Shell, StdCtx};
use clap::{ArgAction, Parser, Subcommand, builder::BoolishValueParser, error::ErrorKind};
use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;
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
    or set TVC_NON_INTERACTIVE=true to disable prompts and fail fast instead.

Output format:
    --message-format human (default) prints human-readable text. Use
    --message-format json to emit machine-readable output instead: one JSON
    object per line (newline-delimited JSON), each with a \"reason\" field
    identifying the message, including errors. JSON mode implies
    --non-interactive, so commands never prompt and fail fast on missing input.

    Errors emit reason \"command-error\" (or \"missing-required-input\") plus a
    \"code\" classifying the failure, an optional numeric \"httpStatus\", and a
    \"message\" carrying the full error chain. The \"code\" taxonomy is:
        missing_required_input  a required value was absent (non-interactive)
        usage_error             bad flags/args (argument parsing failed)
        invalid_input           semantic validation failed in the command
        unauthorized            HTTP 401/403
        not_found               HTTP 404, or a resource that resolved to empty
        api_error               other non-success HTTP status, or a failed or
                                unexpected activity
        approval_required       the activity needs more approvals
        network_error           connect/timeout/DNS: request never reached the
                                server
        command_error           fallback for everything else
    Exit codes are unchanged: 0 success, 1 runtime error, 2 usage error.";

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

    /// Format user-facing output.
    #[arg(long, global = true, value_enum, default_value_t = MessageFormat::Human)]
    message_format: MessageFormat,

    /// Control ANSI color in user-facing output.
    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Run the CLI.
    pub async fn run() -> ExitCode {
        let args = match Cli::try_parse() {
            Ok(args) => args,
            Err(error) => return handle_parse_error(error),
        };
        debug!(
            command = args.command.name(),
            non_interactive = args.non_interactive,
            message_format = ?args.message_format,
            color = ?args.color,
            "dispatching"
        );

        let shell = Shell::standard(args.message_format, args.color);
        let mut ctx = Ctx::new(shell, args.non_interactive);
        let result = args.command.run(&mut ctx).await;
        match result {
            Ok(outcome) => {
                // Fail open: the command itself already succeeded, so a failure
                // to deliver the outcome message should not flip the exit code.
                // Make a best-effort warning on stderr and still exit successfully.
                if let Err(emit_error) = ctx.shell().emit(&outcome) {
                    let mut stderr = std::io::stderr();
                    let _ = writeln!(stderr, "warning: failed to write CLI output: {emit_error}");
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                let shell = ctx.shell();
                let emit_result = if shell.message_format().is_json() {
                    shell.emit(&ErrorMessage::from_error(&error))
                } else {
                    // Render the FULL anyhow chain (alternate Display joins every
                    // context layer with ": "), not just the top layer.
                    shell.human().error(format!("{error:#}"))
                };
                if let Err(emit_error) = emit_result {
                    let mut stderr = std::io::stderr();
                    let _ = writeln!(stderr, "error: failed to write CLI error: {emit_error}");
                }
                ExitCode::FAILURE
            }
        }
    }
}

/// Exit code for a usage error (bad flags/args). Matches clap's default.
const USAGE_ERROR_EXIT_CODE: i32 = 2;

/// Handle a clap parse failure.
///
/// `--help`/`-h` and `--version` also surface as `Err`; those must always print
/// clap's own text (never JSON), so we defer to `error.exit()` for them. Real
/// parse failures are emitted as a single `command-error`/`usage_error` NDJSON
/// line on stdout when `--message-format json` was requested, otherwise handed
/// back to clap's default rendering via `error.exit()`.
fn handle_parse_error(error: clap::Error) -> ExitCode {
    match error.kind() {
        // Help/version output must never be JSON — let clap print and exit.
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => error.exit(),
        _ => {
            if argv_requests_json(std::env::args_os()) {
                // Emit clap's full text (including the "Usage:" line) with any
                // ANSI stripped so agents get pure NDJSON they can self-correct
                // from, regardless of terminal/color detection.
                let message = strip_ansi(&error.render().to_string());
                let error_message = ErrorMessage::usage_error(message);
                let mut stdout = std::io::stdout();
                // Best-effort: if writing fails we still exit with the usage code.
                let _ = writeln!(stdout, "{}", error_message.to_json_string());
                std::process::exit(USAGE_ERROR_EXIT_CODE)
            } else {
                error.exit()
            }
        }
    }
}

/// Heuristic scan of raw argv for `--message-format json`, used only on the
/// parse-failure path where clap could not give us the parsed flag value.
///
/// Matches both spellings: the two-token `--message-format json` and the
/// single-token `--message-format=json`. This is intentionally a heuristic: a
/// tolerable false positive is a stray `--message-format json` where `json`
/// actually belongs to a different flag's value. Getting an extra JSON error
/// line in that rare case is preferable to hiding usage errors from agents that
/// asked for JSON.
fn argv_requests_json(args: impl IntoIterator<Item = OsString>) -> bool {
    const FLAG: &str = "--message-format";
    let mut expect_value = false;
    for arg in args {
        let Some(arg) = arg.to_str() else {
            expect_value = false;
            continue;
        };
        if expect_value {
            return arg == "json";
        }
        if let Some(value) = arg.strip_prefix(&format!("{FLAG}=")) {
            if value == "json" {
                return true;
            }
        } else if arg == FLAG {
            expect_value = true;
        }
    }
    false
}

/// Remove ANSI escape sequences (CSI `\x1b[ ... m` etc.) from `input`.
///
/// clap's `.ansi()` rendering embeds styling escapes; strip them so the JSON
/// `message` is plain text regardless of terminal detection.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // CSI sequences are ESC '[' <params/intermediates> <final>, where
            // the final byte is in the range @-~ (0x40-0x7e). Skip the leading
            // '[' (also in that range) before scanning for the final byte.
            if chars.peek() == Some(&'[') {
                chars.next();
            }
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl Commands {
    async fn run(self, ctx: &mut StdCtx) -> anyhow::Result<Outcome> {
        match self {
            Commands::Deploy { command } => match command {
                DeployCommands::Approve(args) => commands::deploy::approve::run(ctx, args).await,
                DeployCommands::GetStatus(args) => {
                    commands::deploy::get_status::run(ctx, args).await
                }
                DeployCommands::ProvisioningDetails(args) => {
                    commands::deploy::provisioning_details::run(ctx, args).await
                }
                DeployCommands::PostShare(args) => {
                    commands::deploy::post_share::run(ctx, args).await
                }
                DeployCommands::Status(args) => commands::deploy::status::run(ctx, args).await,
                DeployCommands::Create(args) => commands::deploy::create::run(ctx, args).await,
                DeployCommands::Init(args) => commands::deploy::init::run(ctx, args).await,
                DeployCommands::DebugLogs(args) => {
                    commands::deploy::debug_logs::run(ctx, args).await
                }
                DeployCommands::Delete(args) => commands::deploy::delete::run(ctx, args).await,
                DeployCommands::Restore(args) => commands::deploy::restore::run(ctx, args).await,
            },
            Commands::App { command } => match command {
                AppCommands::Status(args) => commands::app::status::run(ctx, args).await,
                AppCommands::List(args) => commands::app::list::run(ctx, args).await,
                AppCommands::Create(args) => commands::app::create::run(ctx, args).await,
                AppCommands::Init(args) => commands::app::init::run(ctx, args).await,
                AppCommands::SetLiveDeploy(args) => {
                    commands::app::set_live_deploy::run(ctx, args).await
                }
                AppCommands::Delete(args) => commands::app::delete::run(ctx, args).await,
            },
            Commands::Keys { command } => match command {
                KeysCommands::CreateQuorumKey(args) => {
                    commands::keys::create_quorum_key::run(ctx, args).await
                }
                KeysCommands::GenerateLocalQuorumKey(args) => {
                    commands::keys::generate_local_quorum_key::run(ctx, args).await
                }
                KeysCommands::InitLocalQuorumKey(args) => {
                    commands::keys::init_local_quorum_key::run(ctx, args).await
                }
                KeysCommands::ReEncryptLocalShare(args) => {
                    commands::keys::re_encrypt_local_share::run(ctx, args).await
                }
            },
            Commands::Login(args) => commands::login::run(ctx, args).await,
            Commands::Operator { command } => match command {
                OperatorCommands::Create(args) => commands::operator::create::run(ctx, args).await,
            },
            Commands::Profile { command } => match command {
                ProfileCommands::Delete(delete_args) => {
                    commands::login::run_delete(ctx, delete_args).await
                }
            },
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Authenticate with Turnkey.
    Login(commands::login::Args),
    /// Manage hosted TVC operators.
    Operator {
        #[command(subcommand)]
        command: OperatorCommands,
    },
    /// Manage saved login profiles.
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
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
            Commands::Operator { command } => match command {
                OperatorCommands::Create(_) => "operator create",
            },
            Commands::Profile { command } => match command {
                ProfileCommands::Delete(_) => "profile delete",
            },
            Commands::Deploy { command } => command.name(),
            Commands::App { command } => command.name(),
            Commands::Keys { command } => command.name(),
        }
    }
}

#[derive(Debug, Subcommand)]
enum OperatorCommands {
    /// Create a hosted TVC operator and save it to the active organization.
    Create(commands::operator::create::Args),
}

#[derive(Debug, Subcommand)]
enum ProfileCommands {
    /// Permanently delete a saved login profile and its key files.
    Delete(commands::login::DeleteArgs),
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
    /// Generate a deployment configuration file
    #[command(long_about = commands::deploy::init::LONG_ABOUT)]
    Init(commands::deploy::init::Args),
    /// Fetch debug logs for a deployment.
    #[command(long_about = commands::deploy::debug_logs::LONG_ABOUT)]
    DebugLogs(commands::deploy::debug_logs::Args),
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
            DeployCommands::DebugLogs(_) => "deploy debug-logs",
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
    /// Create a hosted quorum key encrypted to hosted operator keys.
    CreateQuorumKey(commands::keys::create_quorum_key::Args),
    /// Generate and shamir-split a local quorum key, encrypting each share to an operator key.
    GenerateLocalQuorumKey(commands::keys::generate_local_quorum_key::Args),
    /// Generate a template local quorum key configuration file.
    InitLocalQuorumKey(commands::keys::init_local_quorum_key::Args),
    /// Re-encrypt a local share for enclave provisioning.
    ReEncryptLocalShare(commands::keys::re_encrypt_local_share::Args),
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
            KeysCommands::CreateQuorumKey(_) => "keys create-quorum-key",
            KeysCommands::GenerateLocalQuorumKey(_) => "keys generate-local-quorum-key",
            KeysCommands::InitLocalQuorumKey(_) => "keys init-local-quorum-key",
            KeysCommands::ReEncryptLocalShare(_) => "keys re-encrypt-local-share",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(tokens: &[&str]) -> Vec<OsString> {
        tokens.iter().map(OsString::from).collect()
    }

    #[test]
    fn argv_json_two_token_spelling_is_detected() {
        assert!(argv_requests_json(argv(&[
            "tvc",
            "deploy",
            "status",
            "--message-format",
            "json",
        ])));
    }

    #[test]
    fn argv_json_equals_spelling_is_detected() {
        assert!(argv_requests_json(argv(&[
            "tvc",
            "deploy",
            "status",
            "--message-format=json",
        ])));
    }

    #[test]
    fn argv_human_format_is_not_json() {
        assert!(!argv_requests_json(argv(&[
            "tvc",
            "deploy",
            "status",
            "--message-format",
            "human",
        ])));
        assert!(!argv_requests_json(argv(&[
            "tvc",
            "--message-format=human",
        ])));
    }

    #[test]
    fn argv_without_message_format_is_not_json() {
        assert!(!argv_requests_json(argv(&["tvc", "deploy", "status"])));
    }

    #[test]
    fn argv_tolerable_false_positive_when_value_belongs_to_another_flag() {
        // Documented, accepted heuristic limitation: a `json` token immediately
        // after a bare `--message-format` is treated as its value even if the
        // real intent was for `json` to belong elsewhere. The scan cannot know
        // the true arg grammar on the parse-failure path, so this counts as
        // JSON. An extra JSON error line is preferable to hiding a usage error
        // from a consumer that asked for JSON.
        assert!(argv_requests_json(argv(&[
            "tvc",
            "--message-format",
            "json",
            "--some-other-flag",
        ])));
    }

    #[test]
    fn strip_ansi_removes_escape_sequences() {
        let styled = "\x1b[1mUsage:\x1b[0m tvc \x1b[32m<COMMAND>\x1b[0m";
        assert_eq!(strip_ansi(styled), "Usage: tvc <COMMAND>");
    }
}
