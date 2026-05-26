//! Interactive prompts for the TVC CLI.
//!
//! Thin wrappers over [`inquire`]. Every primitive requires a real TTY; callers
//! that expect to drive prompts from CI or scripts must use the corresponding
//! flag, set `TVC_NON_INTERACTIVE=1`, or both.
//!
//! Non-interactive awareness:
//!
//! - [`is_interactive`] — true only when stdin is a TTY **and**
//!   `TVC_NON_INTERACTIVE` is unset.
//! - [`bail_if_non_interactive`] — errors with a clear message naming the
//!   flag the caller should set instead. Used at the top of any function
//!   that's about to prompt.

use anyhow::{bail, Result};
use inquire::{Confirm, Password, Select, Text};
use std::fmt::Display;
use std::io::IsTerminal;

/// Env var that forces non-interactive mode.
pub const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

/// True when we are in a full interactive session: stdin is a TTY and
/// `TVC_NON_INTERACTIVE` is unset.
pub fn is_interactive() -> bool {
    if non_interactive_forced() {
        return false;
    }
    std::io::stdin().is_terminal()
}

/// True when the user has explicitly set `TVC_NON_INTERACTIVE`.
pub fn non_interactive_forced() -> bool {
    std::env::var_os(NON_INTERACTIVE_ENV).is_some()
}

/// Error with a clear message when we cannot prompt — either the env var is
/// set, or stdin is not a TTY.
///
/// `flag_hint` is the flag the user should set instead (e.g. `"--org"`).
pub fn bail_if_non_interactive(flag_hint: &str) -> Result<()> {
    if !is_interactive() {
        bail!(
            "{flag_hint} is required in non-interactive mode \
             (set {flag_hint} or run in a TTY without {NON_INTERACTIVE_ENV})"
        );
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
