//! CLI exit-code helpers.

use thiserror::Error;

pub const CONSENSUS_NEEDED_EXIT_CODE: i32 = 2;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ExitError {
    code: i32,
    message: String,
}

impl ExitError {
    pub fn consensus_needed() -> Self {
        Self {
            code: CONSENSUS_NEEDED_EXIT_CODE,
            message: "activity is pending consensus".to_string(),
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}
