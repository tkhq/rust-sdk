//! Interactive prompts for the TVC CLI.
//!
//! Thin wrappers over [`inquire`]. Every primitive requires a real TTY; callers
//! that expect to drive prompts from CI or scripts must use --non-interactive,
//! set TVC_NON_INTERACTIVE=true, or both.

use anyhow::{Result, bail};
use inquire::{Confirm, Password, Select, Text};
use std::fmt::Display;
use std::io::IsTerminal;

use crate::output::MissingRequiredInput;
use crate::terminal::TerminalText;

/// Env var that disables interactive prompts when parsed as true.
pub const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

pub fn stdin_can_prompt() -> bool {
    std::io::stdin().is_terminal()
}

pub fn error_required_in_non_interactive(flag_hint: &str) -> anyhow::Error {
    MissingRequiredInput::new(flag_hint).into()
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
    let message = TerminalText(message).single_line();
    let placeholder = default.map(|value| TerminalText(value).single_line());
    let formatter = |value: &str| {
        let value = if value.is_empty() {
            default.unwrap_or(value)
        } else {
            value
        };
        TerminalText(value).single_line()
    };
    let mut prompt = Text::new(&message).with_formatter(&formatter);
    if let Some(value) = &placeholder {
        // Keep the raw default out of inquire's renderer and return it unchanged.
        prompt = prompt
            .with_placeholder(value)
            .with_help_message("Press Enter to keep the saved value.");
    }
    let value = prompt.prompt()?;
    Ok(if value.is_empty() {
        default.map(String::from).unwrap_or(value)
    } else {
        value
    })
}

/// Prompt for a yes/no confirmation with a default.
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    let message = TerminalText(message).single_line();
    Ok(Confirm::new(&message).with_default(default).prompt()?)
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
    let message = TerminalText(message).single_line();
    let labels = options
        .iter()
        .map(|value| TerminalText(value).single_line())
        .collect();
    let selected = Select::new(&message, labels).raw_prompt()?;
    Ok(options
        .into_iter()
        .nth(selected.index)
        .expect("inquire selected an existing option"))
}

/// Prompt for a secret value with masked input.
pub fn password(message: &str) -> Result<String> {
    let message = TerminalText(message).single_line();
    Ok(Password::new(&message).without_confirmation().prompt()?)
}
