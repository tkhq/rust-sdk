//! User-facing output primitives for TVC.

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::ValueEnum;
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, IsTerminal, Write};

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

pub struct Shell<W: Write = Box<dyn Write + Send>> {
    stdout: W,
    stderr: W,
    color: ColorChoice,
    use_color: bool,
    message_format: MessageFormat,
}

impl Shell<Box<dyn Write + Send>> {
    pub fn standard(message_format: MessageFormat, color: ColorChoice) -> Self {
        let use_color = match color {
            ColorChoice::Auto => io::stderr().is_terminal(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        };

        Self {
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            color,
            use_color,
            message_format,
        }
    }
}

impl<W: Write> Shell<W> {
    pub fn message_format(&self) -> MessageFormat {
        self.message_format
    }

    pub fn emit<M: Message>(&mut self, message: &M) -> Result<()> {
        match self.message_format {
            MessageFormat::Human => self.human().line(message.human_message()),
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
    pub fn human(&mut self) -> Human<'_, W> {
        Human(self)
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

/// Human-only presentation writers over a borrowed [`Shell`].
///
/// Every method here writes only in [`MessageFormat::Human`] and
/// is a silent no-op otherwise, so it is meant for
/// human-facing output (progress, prompts, spacing) — never for
/// machine-readable JSON output, which must go through [`Shell::emit`].
pub struct Human<'a, W: Write>(&'a mut Shell<W>);

impl<W: Write> Human<'_, W> {
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
pub struct Ctx<W: Write = Box<dyn Write + Send>> {
    shell: Shell<W>,
    non_interactive: bool,
}

impl<W: Write> Ctx<W> {
    /// `non_interactive` is the raw `--non-interactive` flag; JSON output mode
    /// always forces non-interactive regardless of the flag, since a piped
    /// consumer can't answer prompts.
    pub fn new(shell: Shell<W>, non_interactive: bool) -> Self {
        let non_interactive = non_interactive || shell.message_format().is_json();
        Self {
            shell,
            non_interactive,
        }
    }

    pub fn shell(&mut self) -> &mut Shell<W> {
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
    const CODE: &'static str = "missing_required_input";
    const REASON: &'static str = "missing-required-input";

    pub fn new(flag_hint: &str) -> Self {
        Self {
            message: format!(
                "{flag_hint} is required in non-interactive mode \
                 (set {flag_hint} or run in a TTY without --non-interactive \
                 / TVC_NON_INTERACTIVE=true)"
            ),
        }
    }

    fn to_error_message(&self) -> ErrorMessage {
        ErrorMessage {
            reason: Self::REASON,
            code: Self::CODE,
            message: self.message.clone(),
        }
    }
}

impl Display for MissingRequiredInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for MissingRequiredInput {}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip)]
    reason: &'static str,
    code: &'static str,
    message: String,
}

impl ErrorMessage {
    pub fn from_error(error: &anyhow::Error) -> Self {
        if let Some(missing) = error.downcast_ref::<MissingRequiredInput>() {
            return missing.to_error_message();
        }

        Self {
            reason: "command-error",
            code: "command_error",
            message: error.to_string(),
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
mod tests {
    use super::*;
    use serde::Serialize;

    impl Shell<Vec<u8>> {
        pub fn from_write(stdout: Vec<u8>, message_format: MessageFormat) -> Self {
            Self {
                stdout,
                stderr: Vec::new(),
                color: ColorChoice::Never,
                use_color: false,
                message_format,
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
        let mut shell = Shell::from_write(Vec::new(), MessageFormat::Json);

        shell.emit(&TestMessage { value: "ok" }).unwrap();

        let output = String::from_utf8(shell.into_stdout()).unwrap();
        assert_eq!(output, "{\"reason\":\"test-message\",\"value\":\"ok\"}\n");
    }

    #[test]
    fn shell_emit_human_uses_human_message() {
        let mut shell = Shell::from_write(Vec::new(), MessageFormat::Human);

        shell.emit(&TestMessage { value: "ok" }).unwrap();

        let output = String::from_utf8(shell.into_stdout()).unwrap();
        assert_eq!(output, "value: ok\n");
    }

    #[test]
    fn ctx_reflects_explicit_non_interactive_flag() {
        let shell = Shell::from_write(Vec::new(), MessageFormat::Human);
        let ctx = Ctx::new(shell, true);

        assert!(ctx.is_non_interactive());
    }

    #[test]
    fn ctx_forces_non_interactive_in_json_mode_regardless_of_flag() {
        let shell = Shell::from_write(Vec::new(), MessageFormat::Json);
        let ctx = Ctx::new(shell, false);

        assert!(ctx.is_non_interactive());
    }

    #[test]
    fn ctx_is_interactive_when_flag_unset_and_format_is_human() {
        let shell = Shell::from_write(Vec::new(), MessageFormat::Human);
        let ctx = Ctx::new(shell, false);

        assert!(!ctx.is_non_interactive());
    }
}
