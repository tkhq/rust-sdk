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
//! - [`require_or_prompt`] — single-value variant: take the flag if set,
//!   prompt if interactive, otherwise error.

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

/// Prompt for a selection from a list.
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
    require_or_prompt_impl(value, flag_name, is_interactive(), prompt_fn)
}

fn require_or_prompt_impl<T>(
    value: Option<T>,
    flag_name: &str,
    interactive: bool,
    prompt_fn: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if let Some(v) = value {
        return Ok(v);
    }
    if !interactive {
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
        let result: Result<String> = require_or_prompt_impl(
            Some("hello".to_string()),
            "--foo",
            false,
            || panic!("should not prompt when value is present"),
        );
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn require_or_prompt_errors_when_non_interactive_and_names_flag() {
        let result: Result<String> = require_or_prompt_impl(
            None,
            "--deploy-id",
            false,
            || panic!("should not prompt when non-interactive"),
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("--deploy-id"), "error names the flag: {err}");
        assert!(
            err.contains("non-interactive"),
            "error mentions non-interactive: {err}"
        );
        assert!(
            err.contains(NON_INTERACTIVE_ENV),
            "error names the env var: {err}"
        );
    }

    #[test]
    fn require_or_prompt_runs_prompt_when_interactive() {
        let result: Result<String> =
            require_or_prompt_impl(None, "--foo", true, || Ok("prompted".to_string()));
        assert_eq!(result.unwrap(), "prompted");
    }
}
