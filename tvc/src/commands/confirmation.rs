//! Reusable CLI confirmation helpers for interactive prompts.
//!
//! Public functions use standard streams, while internals accept injected IO
//! so tests can exercise prompt behavior without terminal input.

use anyhow::{bail, Result};
use std::io::{BufRead, Write};

const CANCELLED: &str = "operation cancelled by user";

pub fn confirm_yes_no(prompt: &str) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    confirm_yes_no_with_io(prompt, &mut stdin, &mut stdout)
}

pub fn confirm_typed(prompt: &str, expected: &str) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    confirm_typed_with_io(prompt, expected, &mut stdin, &mut stdout)
}

fn confirm_yes_no_with_io<R: BufRead, W: Write>(
    prompt: &str,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    write!(writer, "{prompt} [y/N]: ")?;
    writer.flush()?;

    let mut input = String::new();
    if reader.read_line(&mut input)? == 0 {
        bail!(CANCELLED);
    }

    let input = input.trim();
    if !(input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("yes")) {
        bail!(CANCELLED);
    }

    Ok(())
}

fn confirm_typed_with_io<R: BufRead, W: Write>(
    prompt: &str,
    expected: &str,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    write!(writer, "{prompt}: ")?;
    writer.flush()?;

    let mut input = String::new();
    if reader.read_line(&mut input)? == 0 {
        bail!(CANCELLED);
    }

    if trim_line_ending(&input) != expected {
        bail!(CANCELLED);
    }

    Ok(())
}

fn trim_line_ending(input: &str) -> &str {
    let input = input.strip_suffix('\n').unwrap_or(input);

    input.strip_suffix('\r').unwrap_or(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn assert_cancelled(result: anyhow::Result<()>) {
        let err = result.expect_err("confirmation should be rejected");

        assert_eq!(err.to_string(), CANCELLED);
    }

    #[test]
    fn confirm_yes_no_accepts_y() {
        let mut input = Cursor::new("y\n");
        let mut output = Vec::new();

        confirm_yes_no_with_io("Continue?", &mut input, &mut output).unwrap();
    }

    #[test]
    fn confirm_yes_no_accepts_yes() {
        let mut input = Cursor::new("yes\n");
        let mut output = Vec::new();

        confirm_yes_no_with_io("Continue?", &mut input, &mut output).unwrap();
    }

    #[test]
    fn confirm_yes_no_rejects_n() {
        let mut input = Cursor::new("n\n");
        let mut output = Vec::new();

        assert_cancelled(confirm_yes_no_with_io("Continue?", &mut input, &mut output));
    }

    #[test]
    fn confirm_yes_no_rejects_empty_input() {
        let mut input = Cursor::new("\n");
        let mut output = Vec::new();

        assert_cancelled(confirm_yes_no_with_io("Continue?", &mut input, &mut output));
    }

    #[test]
    fn confirm_yes_no_rejects_eof() {
        let mut input = Cursor::new("");
        let mut output = Vec::new();

        assert_cancelled(confirm_yes_no_with_io("Continue?", &mut input, &mut output));
    }

    #[test]
    fn confirm_typed_accepts_exact_app_id() {
        let mut input = Cursor::new("app_123\n");
        let mut output = Vec::new();

        confirm_typed_with_io("Type app id", "app_123", &mut input, &mut output).unwrap();
    }

    #[test]
    fn confirm_typed_rejects_mismatch() {
        let mut input = Cursor::new("app_456\n");
        let mut output = Vec::new();

        assert_cancelled(confirm_typed_with_io(
            "Type app id",
            "app_123",
            &mut input,
            &mut output,
        ));
    }

    #[test]
    fn confirm_typed_rejects_eof() {
        let mut input = Cursor::new("");
        let mut output = Vec::new();

        assert_cancelled(confirm_typed_with_io(
            "Type app id",
            "app_123",
            &mut input,
            &mut output,
        ));
    }
}
