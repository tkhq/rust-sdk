//! Output utilities for human-readable and machine-readable CLI output.
//!
//! All diagnostic/progress messages go to stderr.
//! All data/results go to stdout.

use crate::cli::GlobalOpts;

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
}
