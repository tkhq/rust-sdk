//! Output utilities for human-readable and machine-readable CLI output.
//!
//! All diagnostic/progress messages go to stderr.
//! All data/results go to stdout.

use crate::cli::GlobalOpts;
use serde::Serialize;

/// Output helper that respects global flags for JSON and quiet mode.
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

    /// Print an important notice to stderr even when quiet/json is enabled.
    pub fn notice(&self, msg: &str) {
        eprintln!("{msg}");
    }

    /// Print result data: JSON when `--json` is set, otherwise call the human-readable formatter.
    pub fn result<T, F>(&self, data: &T, human_fn: F) -> anyhow::Result<()>
    where
        T: Serialize,
        F: FnOnce(),
    {
        if self.global.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        } else {
            human_fn();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Output;
    use crate::cli::GlobalOpts;
    use std::cell::Cell;

    #[test]
    fn quiet_still_runs_human_result_formatter() {
        let global = GlobalOpts {
            json: false,
            no_input: false,
            quiet: true,
            api_key_file: None,
            api_url: None,
            org_id: None,
        };
        let output = Output::new(&global);
        let called = Cell::new(false);

        output
            .result(&serde_json::json!({"ok": true}), || called.set(true))
            .unwrap();

        assert!(called.get());
    }
}
