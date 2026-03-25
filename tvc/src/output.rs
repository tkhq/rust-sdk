//! Output utilities for human-readable and machine-readable CLI output.
//!
//! All diagnostic/progress messages go to stderr.
//! All data/results go to stdout.

use crate::cli::GlobalOpts;
use serde::Serialize;

/// Output helper that respects global flags for JSON, quiet, and formatting.
pub struct Output<'a> {
    global: &'a GlobalOpts,
}

impl<'a> Output<'a> {
    /// Create a new Output helper from global options.
    pub fn new(global: &'a GlobalOpts) -> Self {
        Self { global }
    }

    /// Print a status/progress message to stderr.
    /// Suppressed when `--quiet` or `--json` is set.
    pub fn status(&self, msg: &str) {
        if !self.global.quiet && !self.global.json {
            eprintln!("{msg}");
        }
    }

    /// Print an informational message to stderr.
    /// Suppressed when `--quiet` is set.
    pub fn info(&self, msg: &str) {
        if !self.global.quiet {
            eprintln!("{msg}");
        }
    }

    /// Print structured data as JSON to stdout.
    /// Only prints when `--json` is set.
    pub fn print_json<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        if self.global.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        Ok(())
    }

    /// Print human-readable text to stdout.
    /// Suppressed when `--json` or `--quiet` is set.
    pub fn print_text(&self, msg: &str) {
        if !self.global.json && !self.global.quiet {
            println!("{msg}");
        }
    }

    /// Print result data: JSON when `--json` is set, otherwise call the human-readable formatter.
    /// This is the primary method commands should use for their result output.
    pub fn result<T, F>(&self, data: &T, human_fn: F) -> anyhow::Result<()>
    where
        T: Serialize,
        F: FnOnce(),
    {
        if self.global.json {
            self.print_json(data)?;
        } else if !self.global.quiet {
            human_fn();
        }
        Ok(())
    }

    /// Returns true if the CLI is in JSON output mode.
    pub fn is_json(&self) -> bool {
        self.global.json
    }

    /// Returns true if the CLI is in quiet mode.
    pub fn is_quiet(&self) -> bool {
        self.global.quiet
    }

    /// Returns true if interactive prompts are disabled.
    pub fn is_no_input(&self) -> bool {
        self.global.no_input
    }
}
