//! Logging setup for the `tvc` binary.

use crate::terminal::TerminalText;
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::{
    FmtContext,
    format::{self, FormatEvent, FormatFields, Writer},
};
use tracing_subscriber::registry::LookupSpan;

struct TerminalFormat<F>(F);

impl<S, N, F> FormatEvent<S, N> for TerminalFormat<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    F: FormatEvent<S, N>,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut text = String::new();
        self.0.format_event(ctx, Writer::new(&mut text), event)?;
        write!(writer, "{}", TerminalText(text))
    }
}

/// Initialize process-wide diagnostic logging.
///
/// Logging is disabled by default to keep normal CLI output unchanged. Set
/// `RUST_LOG`, for example `RUST_LOG=tvc=debug`, to enable structured debug
/// diagnostics on stderr.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"));

    tracing_subscriber::fmt()
        .with_ansi(false)
        .event_format(TerminalFormat(
            format::format()
                .compact()
                .with_ansi(false)
                .with_target(true),
        ))
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_fields_and_messages_cannot_overwrite_terminal_text() {
        let output = tempfile::NamedTempFile::new().unwrap();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .event_format(TerminalFormat(
                format::format()
                    .compact()
                    .without_time()
                    .with_level(false)
                    .with_target(false),
            ))
            .with_writer(output.reopen().unwrap())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(name = %"name\r\u{202e}", "message\x08\u{9b}2J");
        });
        assert_eq!(
            std::fs::read(output.path()).unwrap(),
            concat!(r"message\x08\u{9b}2J name=name\r\u{202e}", "\n").as_bytes()
        );
    }
}
