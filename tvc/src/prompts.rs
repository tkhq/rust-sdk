//! Interactive prompts for the TVC CLI.
//!
//! Thin wrappers over [`inquire`]. Every primitive requires a real TTY; callers
//! that expect to drive prompts from CI or scripts must use --non-interactive,
//! set TVC_NON_INTERACTIVE=true, or both.

use anyhow::{Result, bail};
use inquire::{Confirm, Password, Select, Text};
use std::fmt::Display;
use std::io::IsTerminal;

/// Env var that disables interactive prompts when parsed as true.
pub const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

pub fn stdin_can_prompt() -> bool {
    std::io::stdin().is_terminal()
}

pub fn error_required_in_non_interactive(flag_hint: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "{flag_hint} is required in non-interactive mode \
     (set {flag_hint} or run in a TTY without --non-interactive \
     / {NON_INTERACTIVE_ENV}=true)"
    )
}

pub fn bail_required_in_non_interactive(flag_hint: &str) -> Result<()> {
    bail!(error_required_in_non_interactive(flag_hint));
}

pub fn bail_interactive_conflicts_with_non_interactive() -> Result<()> {
    bail!("--interactive conflicts with --non-interactive or {NON_INTERACTIVE_ENV}=true");
}

pub fn ensure_stdin_is_tty() -> Result<()> {
    if !stdin_can_prompt() {
        bail!("--interactive requires a TTY");
    }
    Ok(())
}

/// Prompt for a non-empty line of text. Bails with `{message} cannot be empty`
/// if the user submits an empty (or whitespace-only) value.
pub fn required_text(message: &str, default: Option<&str>) -> Result<String> {
    let value = text(message, default)?;
    if value.trim().is_empty() {
        bail!("{message} cannot be empty");
    }
    Ok(value)
}

/// Prompt for a line of text, optionally with a default value.
pub fn text(message: &str, default: Option<&str>) -> Result<String> {
    let mut prompt = Text::new(message);
    if let Some(d) = default {
        prompt = prompt.with_default(d);
    }
    Ok(prompt.prompt()?)
}

/// Prompt for a yes/no confirmation with a default.
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new(message).with_default(default).prompt()?)
}

/// Prompt for yes/no confirmation; bail with `"operation cancelled by user:
/// {operation}"` on No. Default is No (safer for destructive prompts).
///
/// `operation` names the action being confirmed (e.g. `"approval"`,
/// `"deletion"`) so the error string identifies which step the user backed out
/// of when multiple confirmation prompts run in sequence.
pub fn confirm_or_bail(message: &str, operation: &str) -> Result<()> {
    if !confirm(message, false)? {
        bail!("operation cancelled by user: {operation}");
    }
    Ok(())
}

/// Prompt for a selection from a list.
pub fn select<T: Display>(message: &str, options: Vec<T>) -> Result<T> {
    Ok(Select::new(message, options).prompt()?)
}

/// Prompt for a secret value with masked input.
pub fn password(message: &str) -> Result<String> {
    Ok(Password::new(message).without_confirmation().prompt()?)
}
