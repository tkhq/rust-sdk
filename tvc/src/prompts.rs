//! Interactive prompts for the TVC CLI.
//!
//! Thin wrappers over `inquire` with TTY-gating and a `TVC_NON_INTERACTIVE`
//! env override. Every prompt must have a corresponding CLI flag; callers
//! should route missing args through [`require_or_prompt`] so non-interactive
//! callers get a clear error instead of hanging.

use anyhow::{bail, Result};
use inquire::{Confirm, Password, Select, Text};
use std::fmt::Display;
use std::io::IsTerminal;

/// Env var that forces non-interactive mode even when stdin is a TTY.
pub const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

/// True when prompting is allowed: stdin is a TTY and `TVC_NON_INTERACTIVE`
/// is unset.
pub fn is_interactive() -> bool {
    if std::env::var_os(NON_INTERACTIVE_ENV).is_some() {
        return false;
    }
    std::io::stdin().is_terminal()
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

/// Prompt for a selection from a list. Requires a real TTY — piped stdin
/// will fail because `Select` drives the terminal in raw mode.
pub fn select<T: Display>(message: &str, options: Vec<T>) -> Result<T> {
    Ok(Select::new(message, options).prompt()?)
}

/// Prompt for a secret value with masked input.
pub fn password(message: &str) -> Result<String> {
    Ok(Password::new(message).without_confirmation().prompt()?)
}

/// Returns the flag value if set; otherwise prompts the user when interactive;
/// otherwise errors with a message naming the flag to set.
///
/// `flag_name` is the literal CLI flag (e.g. `"--deploy-id"`) shown in the
/// non-interactive error message.
pub fn require_or_prompt<T>(
    value: Option<T>,
    flag_name: &str,
    prompt_fn: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if let Some(v) = value {
        return Ok(v);
    }
    if !is_interactive() {
        bail!(
            "flag {flag_name} is required in non-interactive mode \
             (set {flag_name} or unset {NON_INTERACTIVE_ENV})"
        );
    }
    prompt_fn()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_or_prompt_returns_supplied_value_without_prompting() {
        let result: Result<String> =
            require_or_prompt(Some("hello".to_string()), "--foo", || {
                panic!("should not prompt when value is present")
            });
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn require_or_prompt_errors_in_non_interactive_with_flag_name() {
        // `cargo test` runs with a non-TTY stdin, so `is_interactive()` is false.
        let result: Result<String> = require_or_prompt(None, "--deploy-id", || {
            panic!("should not prompt when non-interactive")
        });
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("--deploy-id"),
            "error should name the flag: {err}"
        );
        assert!(
            err.contains("non-interactive"),
            "error should mention non-interactive mode: {err}"
        );
    }
}
