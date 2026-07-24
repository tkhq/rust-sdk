//! User-facing output primitives for TVC.

use crate::errors::{Classification, ErrorCode, classify};
use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::ValueEnum;
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, IsTerminal, Stderr, Stdout, Write};

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

        let Classification { code, http_status } = classify(error);
        Self {
            reason: Self::RUNTIME_REASON,
            code,
            http_status,
            message: format!("{error:#}"),
        }
    }

    /// Build a `usage_error` message for a CLI argument-parsing failure. Emitted
    /// from `cli.rs` on the clap parse-failure path when JSON output was
    /// requested; `message` is clap's rendered (ANSI-stripped) error text.
    pub fn usage_error(message: String) -> Self {
        Self {
            reason: Self::RUNTIME_REASON,
            code: ErrorCode::UsageError,
            http_status: None,
            message,
        }
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

    // --- Error envelope wiring (Part A / F1) ---

    use crate::errors::NotFound;
    use anyhow::anyhow;
    use serde_json::Value;
    use turnkey_client::TurnkeyClientError;

    /// Emit `error` through a JSON `TestShell` and parse the single NDJSON line.
    fn emit_error_json(error: &anyhow::Error) -> Value {
        let mut shell = TestShell::with_json_formatter();
        shell.emit(&ErrorMessage::from_error(error)).unwrap();
        let line = String::from_utf8(shell.into_stdout()).unwrap();
        // Exactly one NDJSON object, newline-terminated.
        assert_eq!(line.matches('\n').count(), 1, "expected one NDJSON line");
        serde_json::from_str(line.trim_end()).expect("emitted line should be valid JSON")
    }

    // The `code` taxonomy and its classification are owned and unit-tested in
    // `crate::errors`. The tests below only assert the consumer wiring: that
    // `ErrorMessage::from_error` renders the full chain, serializes
    // `code`/`httpStatus` correctly, and preserves the `missing-required-input`
    // reason override.

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
        let error = anyhow::Error::new(TurnkeyClientError::UnexpectedHttpStatus(
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
    fn empty_response_not_found_serializes_without_http_status() {
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
