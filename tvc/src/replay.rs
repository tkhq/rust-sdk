//! Replay-command banner printed at the end of interactive command runs.
//!
//! Shows the user the equivalent non-interactive `tvc ...` invocation so they
//! can rerun the same configuration without going through prompts again.
//!
//! ## Rules
//!
//! - Scalar flags are always printed — even when the user's answer matched
//!   the default — so the rendered command is future-proof against default
//!   changes.
//! - Boolean flags are printed when true and omitted when false, matching
//!   standard Unix CLI conventions.
//! - Secret values are never echoed; use [`ReplayValue::Redacted`] to show a
//!   placeholder like `<PATH>` instead.

use std::fmt::Write;

/// How a flag's value should be rendered in the replay banner.
pub enum ReplayValue {
    /// Literal value — printed as `--flag <value>` with POSIX shell quoting.
    Literal(String),
    /// Presence-only boolean. Rendered as `--flag` with no value.
    Flag,
    /// Redacted secret. Rendered as `--flag <PLACEHOLDER>`; the real value is
    /// never shown.
    Redacted(&'static str),
}

/// Builder for the replay banner. Each interactive command should construct
/// one at the end of `run()` and call [`ReplayHint::print`].
pub struct ReplayHint {
    subcommand: &'static str,
    flags: Vec<(String, ReplayValue)>,
}

impl ReplayHint {
    /// Create a new hint for a subcommand (e.g. `"deploy approve"`).
    pub fn new(subcommand: &'static str) -> Self {
        Self {
            subcommand,
            flags: Vec::new(),
        }
    }

    /// Add a scalar flag with a literal value.
    pub fn literal(mut self, name: &str, value: impl Into<String>) -> Self {
        self.flags
            .push((name.to_string(), ReplayValue::Literal(value.into())));
        self
    }

    /// Add a presence-only boolean flag. Only call this when the flag's value
    /// is `true`; omit the call entirely when false.
    pub fn flag(mut self, name: &str) -> Self {
        self.flags.push((name.to_string(), ReplayValue::Flag));
        self
    }

    /// Add a flag whose value is a secret that must not be shown.
    pub fn redacted(mut self, name: &str, placeholder: &'static str) -> Self {
        self.flags
            .push((name.to_string(), ReplayValue::Redacted(placeholder)));
        self
    }

    // TODO(Daniil): remove ne if arg is changed to remove positional and be a flag
    /// Add a positional argument (e.g. the config-file path on
    /// `tvc deploy create`). Rendered without a leading `--` token.
    pub fn positional(mut self, value: impl Into<String>) -> Self {
        self.flags
            .push((String::new(), ReplayValue::Literal(value.into())));
        self
    }

    /// Render the banner as a string without printing it.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "─────────────────────────────────────");
        let _ = writeln!(out, "  To skip prompts next time, run:");
        let _ = writeln!(out);
        if self.flags.is_empty() {
            let _ = writeln!(out, "  tvc {}", self.subcommand);
        } else {
            let _ = writeln!(out, "  tvc {} \\", self.subcommand);
            let last = self.flags.len() - 1;
            for (i, (name, value)) in self.flags.iter().enumerate() {
                let tail = if i == last { "" } else { " \\" };
                let rendered = match value {
                    ReplayValue::Literal(v) if name.is_empty() => shell_quote(v),
                    ReplayValue::Literal(v) => format!("{name} {}", shell_quote(v)),
                    ReplayValue::Flag => name.clone(),
                    ReplayValue::Redacted(placeholder) => format!("{name} {placeholder}"),
                };
                let _ = writeln!(out, "    {rendered}{tail}");
            }
        }
        let _ = writeln!(out, "─────────────────────────────────────");
        out
    }

    /// Print the banner to stdout, preceded by a blank line.
    pub fn print(&self) {
        println!();
        print!("{}", self.render());
    }
}

/// POSIX single-quote a value for shell consumption. Safe characters pass
/// through unquoted; anything else is wrapped in single quotes with `'`
/// itself escaped as `'\''`.
fn shell_quote(s: &str) -> String {
    if !s.is_empty() && s.chars().all(is_safe_unquoted) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str(r"'\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn is_safe_unquoted(c: char) -> bool {
    matches!(c,
        'a'..='z' | 'A'..='Z' | '0'..='9' |
        '-' | '_' | '.' | '/' | ':' | '@' | '=' | ',' | '+'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_passes_safe_values_unchanged() {
        assert_eq!(shell_quote("deploy-abc123"), "deploy-abc123");
        assert_eq!(shell_quote("my-app.json"), "my-app.json");
        assert_eq!(shell_quote("ghcr.io/team/app:v1"), "ghcr.io/team/app:v1");
        assert_eq!(
            shell_quote("sha256:abc+def=ghi,jkl"),
            "sha256:abc+def=ghi,jkl"
        );
    }

    #[test]
    fn shell_quote_wraps_values_with_spaces() {
        assert_eq!(shell_quote("hello world"), "'hello world'");
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn shell_quote_wraps_empty_string() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn shell_quote_wraps_shell_metacharacters() {
        assert_eq!(shell_quote("$HOME"), "'$HOME'");
        assert_eq!(shell_quote("a;b"), "'a;b'");
        assert_eq!(shell_quote("a|b"), "'a|b'");
    }

    #[test]
    fn render_with_no_flags_prints_bare_command() {
        let hint = ReplayHint::new("login");
        let output = hint.render();
        assert!(output.contains("tvc login"));
        assert!(!output.contains("\\"));
    }

    #[test]
    fn render_literal_flags_use_backslash_continuations() {
        let hint = ReplayHint::new("deploy approve")
            .literal("--deploy-id", "deploy_abc123")
            .literal("--operator-id", "op_111");
        let output = hint.render();

        assert!(output.contains("tvc deploy approve \\"));
        assert!(output.contains("--deploy-id deploy_abc123 \\"));
        assert!(output.contains("--operator-id op_111"));

        // Last flag line has no trailing backslash.
        let last_flag_line = output
            .lines()
            .find(|l| l.contains("--operator-id"))
            .unwrap();
        assert!(!last_flag_line.ends_with("\\"));
    }

    #[test]
    fn render_flag_variant_is_presence_only() {
        let hint = ReplayHint::new("deploy approve").flag("--assume-yes");
        let output = hint.render();
        let line = output
            .lines()
            .find(|l| l.contains("--assume-yes"))
            .unwrap();
        assert_eq!(line.trim(), "--assume-yes");
    }

    #[test]
    fn render_redacted_shows_placeholder_not_value() {
        let hint = ReplayHint::new("deploy create")
            .redacted("--pivot-pull-secret", "<PATH>");
        let output = hint.render();
        assert!(output.contains("--pivot-pull-secret <PATH>"));
    }

    #[test]
    fn render_quotes_values_with_spaces() {
        let hint = ReplayHint::new("login").literal("--org", "my org with spaces");
        let output = hint.render();
        assert!(output.contains("--org 'my org with spaces'"));
    }

    #[test]
    fn render_positional_argument_without_flag_prefix() {
        let hint = ReplayHint::new("deploy create")
            .positional("my-deploy.json")
            .redacted("--pivot-pull-secret", "<PATH>");
        let output = hint.render();
        // Positional appears as bare value, not prefixed with --something.
        let positional_line = output
            .lines()
            .find(|l| l.contains("my-deploy.json"))
            .unwrap();
        assert_eq!(positional_line.trim(), "my-deploy.json \\");
        assert!(output.contains("--pivot-pull-secret <PATH>"));
    }

    #[test]
    fn render_preserves_insertion_order() {
        let hint = ReplayHint::new("x")
            .literal("--first", "1")
            .literal("--second", "2")
            .literal("--third", "3");
        let output = hint.render();
        let first = output.find("--first").unwrap();
        let second = output.find("--second").unwrap();
        let third = output.find("--third").unwrap();
        assert!(first < second && second < third);
    }
}
