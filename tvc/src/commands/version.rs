//! Version command for printing the tvc release version.

use crate::outcome::Outcome;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// The tvc release version.
#[derive(Default, Serialize)]
pub struct CliVersion {
    version: &'static str,
}

impl Display for CliVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.version)
    }
}

pub fn run() -> anyhow::Result<Outcome> {
    Ok(Outcome::Version(CliVersion {
        version: env!("CARGO_PKG_VERSION"),
    }))
}
