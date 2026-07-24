//! User-facing output primitives for TVC.

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::ValueEnum;
use reqwest::StatusCode;
use serde::{Serialize, Serializer};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, IsTerminal, Stderr, Stdout, Write};
use turnkey_client::TurnkeyClientError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum MessageFormat {
    Human,
    Json,
}

impl MessageFormat {
    pub fn is_json(self) -> bool {
        matches!(self, MessageFormat::Json)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

pub trait Message: Serialize {
    fn reason(&self) -> &'static str;

    fn human_message(&self) -> String;

    fn to_json_string(&self) -> String {
        #[derive(Serialize)]
        struct WithReason<'a, S: Serialize + ?Sized> {
            reason: &'a str,
            #[serde(flatten)]
            msg: &'a S,
        }

        serde_json::to_string(&WithReason {
            reason: self.reason(),
            msg: self,
        })
        .expect("serializing TVC output message should not fail")
    }
}

pub struct Shell<Out = Stdout, Err = Stderr> {
    stdout: Out,
    stderr: Err,
    color: ColorChoice,
    use_color: bool,
    message_format: MessageFormat,
}

impl Shell {
    pub fn standard(message_format: MessageFormat, color: ColorChoice) -> Self {
        let use_color = match color {
            ColorChoice::Auto => io::stderr().is_terminal(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        };

        Self {
            stdout: io::stdout(),
            stderr: io::stderr(),
            color,
            use_color,
            message_format,
        }
    }
}

impl<W, W2> Shell<W, W2> {
    pub fn message_format(&self) -> MessageFormat {
        self.message_format
    }

    pub fn color(&self) -> ColorChoice {
        self.color
    }

    fn style(&self, color: AnsiColor) -> Style {
        if self.use_color {
            Style::new().bold().fg_color(Some(Color::Ansi(color)))
        } else {
            Style::new()
        }
    }
}

impl<W: Write, W2: Write> Shell<W, W2> {
    /// Emit a machine-consumable message: one JSON line in JSON mode, or the
    /// message's `human_message()` in human mode.
    ///
    /// An empty `human_message()` means the outcome is machine-only; human
    /// mode prints nothing (JSON mode still emits the message).
    pub fn emit<M: Message>(&mut self, message: &M) -> Result<()> {
        match self.message_format {
            MessageFormat::Human => {
                let text = message.human_message();
                if text.is_empty() {
                    return Ok(());
                }
                self.human().line(text)
            }
            MessageFormat::Json => {
                writeln!(self.stdout, "{}", message.to_json_string())?;
                Ok(())
            }
        }
    }

    /// Human-only presentation writers.
    ///
    /// Every method on the returned [`Human`] handle writes only when the
    /// message format is [`MessageFormat::Human`] and is a silent no-op
    /// otherwise, so it must never carry machine-readable output — use
    /// [`Shell::emit`] for that.
    pub fn human(&mut self) -> Human<'_, W, W2> {
        Human(self)
    }
}

/// Human-only presentation writers over a borrowed [`Shell`].
///
/// Every method here writes only in [`MessageFormat::Human`] and
/// is a silent no-op otherwise, so it is meant for
/// human-facing output (progress, prompts, spacing) — never for
/// machine-readable JSON output, which must go through [`Shell::emit`].
pub struct Human<'a, W: Write, W2: Write>(&'a mut Shell<W, W2>);

impl<W: Write, W2: Write> Human<'_, W, W2> {
    pub fn line(&mut self, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            writeln!(self.0.stdout, "{message}")?;
        }
        Ok(())
    }

    pub fn blank_line(&mut self) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            writeln!(self.0.stdout)?;
        }
        Ok(())
    }

    pub fn status(&mut self, label: &str, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            let style = self.0.style(AnsiColor::Green);
            writeln!(self.0.stderr, "{style}{label}{style:#}: {message}")?;
        }
        Ok(())
    }

    pub fn warn(&mut self, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            let style = self.0.style(AnsiColor::Yellow);
            writeln!(self.0.stderr, "{style}warning{style:#}: {message}")?;
        }
        Ok(())
    }

    pub fn err_line(&mut self, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            writeln!(self.0.stderr, "{message}")?;
        }
        Ok(())
    }

    pub fn error(&mut self, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            let style = self.0.style(AnsiColor::Red);
            writeln!(self.0.stderr, "{style}error{style:#}: {message}")?;
        }
        Ok(())
    }

    pub fn print(&mut self, message: impl Display) -> Result<()> {
        if matches!(self.0.message_format, MessageFormat::Human) {
            write!(self.0.stdout, "{message}")?;
            self.0.stdout.flush()?;
        }
        Ok(())
    }
}

/// Bundles the `Shell` with cross-cutting CLI flags
pub struct Ctx<W, W2> {
    shell: Shell<W, W2>,
    non_interactive: bool,
}

pub type StdCtx = Ctx<Stdout, Stderr>;

impl<W: Write, W2: Write> Ctx<W, W2> {
    /// `non_interactive` is the raw `--non-interactive` flag; JSON output mode
    /// always forces non-interactive regardless of the flag, since a piped
    /// consumer can't answer prompts.
    pub fn new(shell: Shell<W, W2>, non_interactive: bool) -> Self {
        let non_interactive = non_interactive || shell.message_format().is_json();
        Self {
            shell,
            non_interactive,
        }
    }

    pub fn shell(&mut self) -> &mut Shell<W, W2> {
        &mut self.shell
    }

    pub fn is_non_interactive(&self) -> bool {
        self.non_interactive
    }
}

/// Writes a formatted line through TVC's output shell.
///
/// This presentation output is suppressed in JSON mode. Use `Shell::emit` for
/// structured command output.
#[macro_export]
macro_rules! shell_println {
    ($ctx:expr $(,)?) => {
        $ctx.shell().human().blank_line()
    };
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.shell().human().line(format_args!($($arg)*))
    };
}

/// Writes formatted text through TVC's output shell without a trailing newline.
///
/// This presentation output is suppressed in JSON mode. Use `Shell::emit` for
/// structured command output.
#[macro_export]
macro_rules! shell_print {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.shell().human().print(format_args!($($arg)*))
    };
}

/// Writes a formatted line to stderr through TVC's output shell.
///
/// This presentation output is suppressed in JSON mode. Use `Shell::emit` for
/// structured command output.
#[macro_export]
macro_rules! shell_eprintln {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.shell().human().err_line(format_args!($($arg)*))
    };
}

#[derive(Debug)]
pub struct MissingRequiredInput {
    message: String,
}

impl MissingRequiredInput {
    pub fn new(flag_hint: &str) -> Self {
        Self {
            message: format!(
                "{flag_hint} is required in non-interactive mode \
                 (set {flag_hint} or run in a TTY without --non-interactive \
                 / TVC_NON_INTERACTIVE=true)"
            ),
        }
    }
}

impl Display for MissingRequiredInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for MissingRequiredInput {}

/// A resource lookup that returned successfully but found nothing — e.g. an API
/// `Ok` response whose optional payload is `None`. Downcast in
/// [`ErrorMessage::from_error`] to classify these as [`ErrorCode::NotFound`],
/// alongside HTTP 404s.
#[derive(Debug)]
pub struct NotFound {
    resource: &'static str,
    id: String,
}

impl NotFound {
    pub fn new(resource: &'static str, id: impl Into<String>) -> Self {
        Self {
            resource,
            id: id.into(),
        }
    }
}

impl Display for NotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { resource, id } = self;
        write!(f, "{resource} not found: {id}")
    }
}

impl Error for NotFound {}

/// The stable, machine-readable classification of a runtime error, carried in
/// the `code` field of a `command-error` (or `missing-required-input`) message.
///
/// `code` is the taxonomy axis; the message `reason` stays stable
/// (`command-error` for all runtime errors) so the outcome registry is
/// unaffected. Every variant serializes to its snake_case [`ErrorCode::as_str`]
/// name, and that mapping is the single source of truth for the taxonomy —
/// exercised by the uniqueness/snake_case registry test below.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// A required value was absent in non-interactive mode.
    MissingRequiredInput,
    /// Bad flags/args — a clap parse failure (emitted from `cli.rs`).
    UsageError,
    /// Semantic validation failure in command code.
    InvalidInput,
    /// HTTP 401/403.
    Unauthorized,
    /// HTTP 404, or an OK response with an empty resource.
    NotFound,
    /// Any other non-success HTTP status, or a failed/unexpected activity.
    ApiError,
    /// An activity needs more approvals.
    ApprovalRequired,
    /// A connect/timeout/DNS failure — the request never reached the server.
    NetworkError,
    /// Fallback for everything else.
    CommandError,
}

impl ErrorCode {
    /// The snake_case wire name. Single source of truth for the taxonomy.
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorCode::MissingRequiredInput => "missing_required_input",
            ErrorCode::UsageError => "usage_error",
            ErrorCode::InvalidInput => "invalid_input",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::NotFound => "not_found",
            ErrorCode::ApiError => "api_error",
            ErrorCode::ApprovalRequired => "approval_required",
            ErrorCode::NetworkError => "network_error",
            ErrorCode::CommandError => "command_error",
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip)]
    reason: &'static str,
    code: ErrorCode,
    #[serde(rename = "httpStatus", skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
    message: String,
}

impl ErrorMessage {
    /// The message `reason` for every runtime error. `code` carries the finer
    /// classification so the outcome `reason` registry stays unchanged.
    const RUNTIME_REASON: &'static str = "command-error";
    const MISSING_INPUT_REASON: &'static str = "missing-required-input";

    /// Build an emitted error from an [`anyhow::Error`].
    ///
    /// The message is the FULL anyhow chain (`{error:#}`), not just the top
    /// context layer. The `code` (and, when known, `httpStatus`) is derived by
    /// walking the cause chain for the first typed error we recognize.
    pub fn from_error(error: &anyhow::Error) -> Self {
        // Preserve the historical special case first: missing required input
        // keeps its own dedicated `reason`.
        if error.downcast_ref::<MissingRequiredInput>().is_some() {
            return Self {
                reason: Self::MISSING_INPUT_REASON,
                code: ErrorCode::MissingRequiredInput,
                http_status: None,
                message: format!("{error:#}"),
            };
        }

        let (code, http_status) = classify(error);
        Self {
            reason: Self::RUNTIME_REASON,
            code,
            http_status,
            message: format!("{error:#}"),
        }
    }
}

/// Walk the cause chain and classify the first typed error we recognize into a
/// `(code, http_status)` pair. Falls back to `command_error` with no status.
fn classify(error: &anyhow::Error) -> (ErrorCode, Option<u16>) {
    for cause in error.chain() {
        if cause.downcast_ref::<NotFound>().is_some() {
            return (ErrorCode::NotFound, None);
        }
        if let Some(client_error) = cause.downcast_ref::<TurnkeyClientError>() {
            return classify_client_error(client_error);
        }
    }
    (ErrorCode::CommandError, None)
}

/// Map a [`TurnkeyClientError`] to its taxonomy code and optional HTTP status.
fn classify_client_error(error: &TurnkeyClientError) -> (ErrorCode, Option<u16>) {
    match error {
        TurnkeyClientError::UnexpectedHttpStatus(status, _) => {
            let code = match StatusCode::from_u16(*status) {
                Ok(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => ErrorCode::Unauthorized,
                Ok(StatusCode::NOT_FOUND) => ErrorCode::NotFound,
                _ => ErrorCode::ApiError,
            };
            (code, Some(*status))
        }
        // A connect/timeout/DNS failure means the request never reached the
        // server — classify as a network error rather than an API error.
        TurnkeyClientError::Http(reqwest_error)
            if reqwest_error.is_connect()
                || reqwest_error.is_timeout()
                || reqwest_error.is_request() =>
        {
            (ErrorCode::NetworkError, None)
        }
        TurnkeyClientError::ActivityRequiresApproval(_) => (ErrorCode::ApprovalRequired, None),
        TurnkeyClientError::ActivityFailed(_)
        | TurnkeyClientError::UnexpectedActivityStatus(_)
        | TurnkeyClientError::UnexpectedInnerActivityResult(_)
        | TurnkeyClientError::MissingActivity
        | TurnkeyClientError::MissingResult
        | TurnkeyClientError::MissingInnerResult => (ErrorCode::ApiError, None),
        _ => (ErrorCode::CommandError, None),
    }
}

impl Message for ErrorMessage {
    fn reason(&self) -> &'static str {
        self.reason
    }

    fn human_message(&self) -> String {
        self.message.clone()
    }
}

#[cfg(test)]
pub use tests::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::io::Empty;

    pub type TestShell = Shell<Vec<u8>, Vec<u8>>;
    pub type EmptyShell = Shell<Empty, Empty>;

    impl<W: Default, W2: Default> Default for Shell<W, W2> {
        fn default() -> Self {
            Self {
                stdout: Default::default(),
                stderr: Default::default(),
                color: ColorChoice::Never,
                use_color: false,
                message_format: MessageFormat::Human,
            }
        }
    }

    impl TestShell {
        pub fn with_json_formatter() -> Self {
            Self {
                message_format: MessageFormat::Json,
                ..Default::default()
            }
        }

        pub fn with_human_formatter() -> Self {
            Self {
                message_format: MessageFormat::Human,
                ..Default::default()
            }
        }

        pub fn into_stdout(self) -> Vec<u8> {
            self.stdout
        }

        pub fn into_stderr(self) -> Vec<u8> {
            self.stderr
        }
    }

    #[derive(Serialize)]
    struct TestMessage {
        value: &'static str,
    }

    impl Message for TestMessage {
        fn reason(&self) -> &'static str {
            "test-message"
        }

        fn human_message(&self) -> String {
            format!("value: {}", self.value)
        }
    }

    #[test]
    fn shell_emit_json_flattens_reason_into_message() {
        let mut shell = TestShell::with_json_formatter();

        shell.emit(&TestMessage { value: "ok" }).unwrap();

        assert_eq!(
            shell.into_stdout(),
            concat!(r#"{"reason":"test-message","value":"ok"}"#, "\n").as_bytes()
        );
    }

    #[test]
    fn shell_emit_human_uses_human_message() {
        let mut shell = TestShell::with_human_formatter();

        shell.emit(&TestMessage { value: "ok" }).unwrap();

        assert_eq!(shell.into_stdout(), "value: ok\n".as_bytes());
    }

    #[derive(Serialize)]
    struct MachineOnlyMessage {
        value: &'static str,
    }

    impl Message for MachineOnlyMessage {
        fn reason(&self) -> &'static str {
            "machine-only-message"
        }

        fn human_message(&self) -> String {
            String::new()
        }
    }

    #[test]
    fn shell_emit_human_skips_empty_human_message() {
        let mut shell = TestShell::with_human_formatter();

        shell.emit(&MachineOnlyMessage { value: "ok" }).unwrap();

        let output = String::from_utf8(shell.into_stdout()).unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn shell_emit_json_still_emits_message_with_empty_human_message() {
        let mut shell = TestShell::with_json_formatter();

        shell.emit(&MachineOnlyMessage { value: "ok" }).unwrap();

        let output = String::from_utf8(shell.into_stdout()).unwrap();
        assert_eq!(
            output,
            concat!(r#"{"reason":"machine-only-message","value":"ok"}"#, "\n")
        );
    }

    // --- Error classification (Part A / F1) ---

    use anyhow::anyhow;
    use serde_json::Value;

    /// Emit `error` through a JSON `TestShell` and parse the single NDJSON line.
    fn emit_error_json(error: &anyhow::Error) -> Value {
        let mut shell = TestShell::with_json_formatter();
        shell.emit(&ErrorMessage::from_error(error)).unwrap();
        let line = String::from_utf8(shell.into_stdout()).unwrap();
        // Exactly one NDJSON object, newline-terminated.
        assert_eq!(line.matches('\n').count(), 1, "expected one NDJSON line");
        serde_json::from_str(line.trim_end()).expect("emitted line should be valid JSON")
    }

    fn client_error(error: TurnkeyClientError) -> anyhow::Error {
        anyhow::Error::new(error)
    }

    #[test]
    fn missing_required_input_keeps_its_reason_and_code() {
        let error = anyhow::Error::new(MissingRequiredInput::new("--app-id"))
            .context("resolving required inputs");
        let json = emit_error_json(&error);

        assert_eq!(json["reason"], "missing-required-input");
        assert_eq!(json["code"], "missing_required_input");
        assert!(json.get("httpStatus").is_none());
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("resolving required inputs"));
        assert!(message.contains("--app-id is required in non-interactive mode"));
    }

    #[test]
    fn unexpected_http_404_is_not_found_with_status_and_full_chain() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            404,
            r#"{"message":"missing deployment"}"#.to_string(),
        ))
        .context("failed to fetch deployment abc-123");
        let json = emit_error_json(&error);

        assert_eq!(json["reason"], "command-error");
        assert_eq!(json["code"], "not_found");
        assert_eq!(json["httpStatus"], 404);
        let message = json["message"].as_str().unwrap();
        // Full chain: both the context layer AND the underlying body.
        assert!(message.contains("failed to fetch deployment abc-123"));
        assert!(message.contains("404"));
        assert!(message.contains("missing deployment"));
    }

    #[test]
    fn empty_response_not_found_maps_to_not_found_without_status() {
        let error = anyhow::Error::new(NotFound::new("deployment", "abc-123"))
            .context("failed to fetch deployment abc-123");
        let json = emit_error_json(&error);

        assert_eq!(json["reason"], "command-error");
        assert_eq!(json["code"], "not_found");
        assert!(json.get("httpStatus").is_none());
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("failed to fetch deployment abc-123"));
        assert!(message.contains("deployment not found: abc-123"));
    }

    #[test]
    fn http_401_and_403_map_to_unauthorized() {
        for status in [401u16, 403] {
            let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
                status,
                "denied".to_string(),
            ));
            let json = emit_error_json(&error);
            assert_eq!(json["code"], "unauthorized", "status {status}");
            assert_eq!(json["httpStatus"], status);
        }
    }

    #[test]
    fn other_http_status_maps_to_api_error() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            500,
            "boom".to_string(),
        ));
        let json = emit_error_json(&error);
        assert_eq!(json["code"], "api_error");
        assert_eq!(json["httpStatus"], 500);
    }

    #[test]
    fn activity_failed_maps_to_api_error_without_status() {
        let error = client_error(TurnkeyClientError::ActivityFailed(None));
        let json = emit_error_json(&error);
        assert_eq!(json["code"], "api_error");
        assert!(json.get("httpStatus").is_none());
    }

    #[test]
    fn activity_requires_approval_maps_to_approval_required() {
        let error = client_error(TurnkeyClientError::ActivityRequiresApproval(
            "act-1".to_string(),
        ));
        let json = emit_error_json(&error);
        assert_eq!(json["code"], "approval_required");
        assert!(json.get("httpStatus").is_none());
    }

    #[test]
    fn unrecognized_error_falls_back_to_command_error() {
        let error = anyhow!("some other failure").context("while doing a thing");
        let json = emit_error_json(&error);
        assert_eq!(json["reason"], "command-error");
        assert_eq!(json["code"], "command_error");
        assert!(json.get("httpStatus").is_none());
    }

    #[test]
    fn message_renders_full_anyhow_chain() {
        // Two context layers stacked on a base error — all three must appear,
        // proving `{:#}` (alternate) rendering rather than only the top layer.
        let error = anyhow!("base failure")
            .context("middle context")
            .context("top context");
        let json = emit_error_json(&error);
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("top context"));
        assert!(message.contains("middle context"));
        assert!(message.contains("base failure"));
    }

    #[test]
    fn ctx_reflects_explicit_non_interactive_flag() {
        let ctx = Ctx::new(TestShell::with_human_formatter(), true);

        assert!(ctx.is_non_interactive());
    }

    #[test]
    fn ctx_forces_non_interactive_in_json_mode_regardless_of_flag() {
        let ctx = Ctx::new(TestShell::with_json_formatter(), false);

        assert!(ctx.is_non_interactive());
    }

    #[test]
    fn ctx_is_interactive_when_flag_unset_and_format_is_human() {
        let ctx = Ctx::new(TestShell::with_human_formatter(), false);

        assert!(!ctx.is_non_interactive());
    }
}
