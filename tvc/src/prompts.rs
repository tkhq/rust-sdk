//! Interactive prompts for the TVC CLI.
//!
//! Each primitive has two code paths:
//!
//! - **Real TTY** — delegates to `inquire` for a polished rendering.
//! - **Piped stdin** — falls back to a plain `read_line` so tests with
//!   `assert_cmd::Command::write_stdin` and other piped flows keep working.
//!
//! Non-interactive awareness:
//!
//! - [`is_interactive`] — true only when stdin is a TTY **and**
//!   `TVC_NON_INTERACTIVE` is unset. Used by [`require_or_prompt`] to decide
//!   whether to prompt for a missing flag value or fail fast.
//! - [`bail_if_non_interactive`] — errors only when the user has explicitly
//!   set `TVC_NON_INTERACTIVE`. Used by callers that want piped-stdin tests to
//!   keep working but still respect an explicit opt-out.

use anyhow::{bail, Result};
use inquire::{Confirm, Password, Select, Text};
use std::fmt::Display;
use std::io::{BufRead, IsTerminal, Write};

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

/// Error with a clear message when `TVC_NON_INTERACTIVE` is set.
///
/// `flag_hint` is the flag the user should set instead (e.g. `"--org"`).
pub fn bail_if_non_interactive(flag_hint: &str) -> Result<()> {
    if non_interactive_forced() {
        bail!(
            "{flag_hint} is required in non-interactive mode \
             (set {flag_hint} or unset {NON_INTERACTIVE_ENV})"
        );
    }
    Ok(())
}

/// Prompt for a line of text, optionally with a default value.
pub fn text(message: &str, default: Option<&str>) -> Result<String> {
    if std::io::stdin().is_terminal() {
        let mut prompt = Text::new(message);
        if let Some(d) = default {
            prompt = prompt.with_default(d);
        }
        return Ok(prompt.prompt()?);
    }

    // Piped-stdin fallback.
    match default {
        Some(d) => print!("{message} [{d}]: "),
        None => print!("{message}: "),
    }
    std::io::stdout().flush()?;
    let input = read_trimmed_line()?;
    Ok(match (input.is_empty(), default) {
        (true, Some(d)) => d.to_string(),
        (_, _) => input,
    })
}

/// Prompt for a yes/no confirmation with a default.
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    if std::io::stdin().is_terminal() {
        return Ok(Confirm::new(message).with_default(default).prompt()?);
    }

    // TODO(daniil): remove piped-stdin fallback after team consensus
    // Piped-stdin fallback. Matches the legacy `[y/N]` prompt style so
    // existing tests that pipe "yes\n" / "y\n" / "no\n" keep working.
    let label = if default { "[Y/n]" } else { "[y/N]" };
    print!("{message} {label}: ");
    std::io::stdout().flush()?;
    let input = read_trimmed_line()?.to_lowercase();
    Ok(match input.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    })
}

/// Prompt for a selection from a list. Requires a real TTY — piped stdin
/// will fail because `Select` drives the terminal in raw mode.
pub fn select<T: Display>(message: &str, options: Vec<T>) -> Result<T> {
    Ok(Select::new(message, options).prompt()?)
}

/// Prompt for a secret value with masked input.
pub fn password(message: &str) -> Result<String> {
    if std::io::stdin().is_terminal() {
        return Ok(Password::new(message).without_confirmation().prompt()?);
    }

    // Piped-stdin fallback (no masking — stdin is already a pipe).
    print!("{message}: ");
    std::io::stdout().flush()?;
    read_trimmed_line()
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

fn read_trimmed_line() -> Result<String> {
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
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
