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

    /// Print result data: JSON when `--json` is set, otherwise call the human-readable formatter.
    /// In `--quiet` mode, all output is suppressed.
    pub fn result<T, F>(&self, data: &T, human_fn: F) -> anyhow::Result<()>
    where
        T: Serialize,
        F: FnOnce(),
    {
        if self.global.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        } else if !self.global.quiet {
            human_fn();
        }
        Ok(())
    }
}
