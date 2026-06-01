//! Output formatting: human-readable text vs machine-readable JSON.
//!
//! A single [`Emitter`], constructed once from the global `--format` flag and
//! threaded into every command's `run`, is the one place command output reaches
//! the terminal. Commands build a purpose-built [`Report`] struct and hand it to
//! [`Emitter::emit`]; the emitter decides text vs JSON. Progress chatter goes
//! through [`Emitter::progress`].
//!
//! # Security
//!
//! [`Report`] is a hand-authored allowlist, never a passthrough. Implement it
//! only on structs that enumerate exactly the fields safe to surface. Never
//! implement it on, or embed into a report, credential/config/domain types such
//! as `StoredApiKey` or `StoredQosOperatorKey` — both derive `Serialize` and
//! carry a `private_key`, so serializing them would leak secrets into JSON.

use clap::ValueEnum;
use serde::Serialize;
use std::fmt::Display;
use std::io::{self, Write};

/// Output format selected via the global `--format` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable text (default).
    #[default]
    Text,
    /// Machine-readable JSON: exactly one JSON document on stdout.
    Json,
}

/// A command result that can be rendered as text or serialized as JSON.
///
/// Implementors are purpose-built structs listing only the fields safe to
/// surface — see the module-level security note.
pub trait Report: Serialize {
    /// Render the human-readable text form. This should reproduce the command's
    /// historical CLI output so the text path stays byte-compatible.
    fn render_text(&self, w: &mut dyn Write) -> io::Result<()>;
}

/// Carries the selected output format and is the single sink for command
/// output: progress chatter, final results, and top-level errors.
#[derive(Debug, Clone, Copy)]
pub struct Emitter {
    format: OutputFormat,
}

impl Emitter {
    /// Build an emitter for the given format.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// The selected output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// True when emitting machine-readable JSON.
    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    /// Emit human-readable progress chatter (e.g. "Creating app...").
    ///
    /// In text mode this prints to stdout, matching the historical inline
    /// `println!`s. In JSON mode it is redirected to stderr so stdout remains a
    /// single JSON document a machine can parse.
    pub fn progress(&self, msg: impl Display) {
        match self.format {
            OutputFormat::Text => println!("{msg}"),
            OutputFormat::Json => eprintln!("{msg}"),
        }
    }

    /// Emit the final command result. The single stdout result sink.
    pub fn emit<T: Report>(&self, report: &T) -> anyhow::Result<()> {
        let stdout = io::stdout();
        let mut w = stdout.lock();
        match self.format {
            OutputFormat::Text => report.render_text(&mut w)?,
            OutputFormat::Json => {
                serde_json::to_writer_pretty(&mut w, report)?;
                writeln!(w)?;
            }
        }
        Ok(())
    }

    /// Render a top-level error. In JSON mode this emits `{"error": "..."}` to
    /// stdout so machine callers always get parseable output; in text mode it
    /// writes an `Error: ...` line to stderr (anyhow/clap convention).
    ///
    /// Best-effort: write failures here are ignored since we are already on the
    /// error path and about to exit non-zero.
    pub fn emit_error(&self, err: &anyhow::Error) {
        match self.format {
            OutputFormat::Text => eprintln!("Error: {err:#}"),
            OutputFormat::Json => {
                let body = serde_json::json!({ "error": format!("{err:#}") });
                let stdout = io::stdout();
                let mut w = stdout.lock();
                let _ = serde_json::to_writer_pretty(&mut w, &body);
                let _ = writeln!(w);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SampleReport {
        deployment_id: String,
        replica_count: u32,
    }

    impl Report for SampleReport {
        fn render_text(&self, w: &mut dyn Write) -> io::Result<()> {
            writeln!(w, "Deployment: {}", self.deployment_id)?;
            writeln!(w, "Replicas: {}", self.replica_count)
        }
    }

    /// Render a report to a byte buffer the way `emit` would, but without
    /// touching the process stdout (so unit tests stay hermetic).
    fn render(format: OutputFormat, report: &SampleReport) -> String {
        let mut buf = Vec::new();
        match format {
            OutputFormat::Text => report.render_text(&mut buf).unwrap(),
            OutputFormat::Json => {
                serde_json::to_writer_pretty(&mut buf, report).unwrap();
                writeln!(&mut buf).unwrap();
            }
        }
        String::from_utf8(buf).unwrap()
    }

    fn sample() -> SampleReport {
        SampleReport {
            deployment_id: "deploy-123".to_string(),
            replica_count: 3,
        }
    }

    #[test]
    fn text_render_matches_human_lines() {
        let out = render(OutputFormat::Text, &sample());
        assert_eq!(out, "Deployment: deploy-123\nReplicas: 3\n");
    }

    #[test]
    fn json_render_uses_camel_case_and_parses() {
        let out = render(OutputFormat::Json, &sample());
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["deploymentId"], "deploy-123");
        assert_eq!(value["replicaCount"], 3);
        // snake_case keys must not leak through.
        assert!(value.get("deployment_id").is_none());
    }

    #[test]
    fn default_format_is_text() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
        assert!(!Emitter::new(OutputFormat::Text).is_json());
        assert!(Emitter::new(OutputFormat::Json).is_json());
    }

    #[test]
    fn output_format_parses_from_lowercase_flag_values() {
        assert_eq!(
            OutputFormat::from_str("text", true).unwrap(),
            OutputFormat::Text
        );
        assert_eq!(
            OutputFormat::from_str("json", true).unwrap(),
            OutputFormat::Json
        );
        assert!(OutputFormat::from_str("yaml", true).is_err());
    }

    #[test]
    fn error_envelope_is_valid_json_with_error_field() {
        // Mirror emit_error's JSON branch without capturing process stdout.
        let err = anyhow::anyhow!("boom");
        let body = serde_json::json!({ "error": format!("{err:#}") });
        let rendered = serde_json::to_string(&body).unwrap();
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(value["error"], "boom");
    }
}
