//! Error taxonomy and classification.
//!
//! This module is the single home for TVC's machine-readable error taxonomy:
//! the [`ErrorCode`] enum and its stable snake_case wire names, the typed
//! [`NotFound`] error, and the [`classify`] logic that walks an
//! [`anyhow::Error`] chain and maps recognized causes to a [`Classification`]
//! `(code, http_status)`.
//!
//! Consumers stay thin: `output.rs` builds the JSON/human error envelope by
//! calling [`classify`], `client.rs` returns [`NotFound`] from its empty-response
//! sites, and `cli.rs` renders clap usage errors via [`strip_ansi`]. None of
//! them redefine any of the taxonomy.
//!
//! The message `reason` (owned by `output.rs`) stays stable — `command-error`
//! for all runtime errors — so this taxonomy lives entirely in the `code` axis
//! and leaves the outcome `reason` registry untouched.

use reqwest::StatusCode;
use serde::{Serialize, Serializer};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use turnkey_client::TurnkeyClientError;

/// A resource lookup that returned successfully but found nothing — e.g. an API
/// `Ok` response whose optional payload is `None`. Downcast in [`classify`] to
/// classify these as [`ErrorCode::NotFound`], alongside HTTP 404s.
#[derive(Debug)]
pub struct NotFound {
    resource: &'static str,
    id: String,
}

impl NotFound {
    pub fn new(resource: &'static str, id: impl Into<String>) -> Self {
        Self {
            resource,
            id: id.into(),
        }
    }
}

impl Display for NotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { resource, id } = self;
        write!(f, "{resource} not found: {id}")
    }
}

impl Error for NotFound {}

/// The stable, machine-readable classification of a runtime error, carried in
/// the `code` field of a `command-error` (or `missing-required-input`) message.
///
/// `code` is the taxonomy axis; the message `reason` stays stable
/// (`command-error` for all runtime errors) so the outcome registry is
/// unaffected. Every variant serializes to its snake_case [`ErrorCode::as_str`]
/// name, and that mapping is the single source of truth for the taxonomy —
/// exercised by the uniqueness/snake_case registry test below.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// A required value was absent in non-interactive mode.
    MissingRequiredInput,
    /// Bad flags/args — a clap parse failure (emitted from `cli.rs`).
    UsageError,
    /// Semantic validation failure in command code.
    InvalidInput,
    /// HTTP 401/403.
    Unauthorized,
    /// HTTP 404, or an OK response with an empty resource.
    NotFound,
    /// Any other non-success HTTP status, or a failed/unexpected activity.
    ApiError,
    /// An activity needs more approvals.
    ApprovalRequired,
    /// A connect/timeout/DNS failure — the request never reached the server.
    NetworkError,
    /// Fallback for everything else.
    CommandError,
}

impl ErrorCode {
    /// The snake_case wire name. Single source of truth for the taxonomy.
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorCode::MissingRequiredInput => "missing_required_input",
            ErrorCode::UsageError => "usage_error",
            ErrorCode::InvalidInput => "invalid_input",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::NotFound => "not_found",
            ErrorCode::ApiError => "api_error",
            ErrorCode::ApprovalRequired => "approval_required",
            ErrorCode::NetworkError => "network_error",
            ErrorCode::CommandError => "command_error",
        }
    }

    /// Every taxonomy variant, in declaration order. Single source of truth for
    /// the registry uniqueness/snake_case test below.
    #[cfg(test)]
    const ALL: [ErrorCode; 9] = [
        ErrorCode::MissingRequiredInput,
        ErrorCode::UsageError,
        ErrorCode::InvalidInput,
        ErrorCode::Unauthorized,
        ErrorCode::NotFound,
        ErrorCode::ApiError,
        ErrorCode::ApprovalRequired,
        ErrorCode::NetworkError,
        ErrorCode::CommandError,
    ];
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// The result of classifying an error: its taxonomy [`ErrorCode`] and, when the
/// cause is an HTTP failure, the numeric status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Classification {
    pub code: ErrorCode,
    pub http_status: Option<u16>,
}

impl Classification {
    fn new(code: ErrorCode, http_status: Option<u16>) -> Self {
        Self { code, http_status }
    }
}

/// Walk the cause chain and classify the first typed error we recognize.
/// Falls back to [`ErrorCode::CommandError`] with no status.
pub fn classify(error: &anyhow::Error) -> Classification {
    for cause in error.chain() {
        if cause.downcast_ref::<NotFound>().is_some() {
            return Classification::new(ErrorCode::NotFound, None);
        }
        if let Some(client_error) = cause.downcast_ref::<TurnkeyClientError>() {
            return classify_client_error(client_error);
        }
    }
    Classification::new(ErrorCode::CommandError, None)
}

/// Map a [`TurnkeyClientError`] to its taxonomy code and optional HTTP status.
fn classify_client_error(error: &TurnkeyClientError) -> Classification {
    match error {
        TurnkeyClientError::UnexpectedHttpStatus(status, _) => {
            let code = match StatusCode::from_u16(*status) {
                Ok(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => ErrorCode::Unauthorized,
                Ok(StatusCode::NOT_FOUND) => ErrorCode::NotFound,
                _ => ErrorCode::ApiError,
            };
            Classification::new(code, Some(*status))
        }
        // A connect/timeout/DNS failure means the request never reached the
        // server — classify as a network error rather than an API error.
        TurnkeyClientError::Http(reqwest_error)
            if reqwest_error.is_connect()
                || reqwest_error.is_timeout()
                || reqwest_error.is_request() =>
        {
            Classification::new(ErrorCode::NetworkError, None)
        }
        TurnkeyClientError::ActivityRequiresApproval(_) => {
            Classification::new(ErrorCode::ApprovalRequired, None)
        }
        TurnkeyClientError::ActivityFailed(_)
        | TurnkeyClientError::UnexpectedActivityStatus(_)
        | TurnkeyClientError::UnexpectedInnerActivityResult(_)
        | TurnkeyClientError::MissingActivity
        | TurnkeyClientError::MissingResult
        | TurnkeyClientError::MissingInnerResult => Classification::new(ErrorCode::ApiError, None),
        _ => Classification::new(ErrorCode::CommandError, None),
    }
}

/// Remove ANSI escape sequences (CSI `\x1b[ ... m` etc.) from `input`.
///
/// clap's error rendering embeds styling escapes; strip them so a JSON error
/// `message` built from clap's text is plain regardless of terminal detection.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // CSI sequences are ESC '[' <params/intermediates> <final>, where
            // the final byte is in the range @-~ (0x40-0x7e). Skip the leading
            // '[' (also in that range) before scanning for the final byte.
            if chars.peek() == Some(&'[') {
                chars.next();
            }
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    fn client_error(error: TurnkeyClientError) -> anyhow::Error {
        anyhow::Error::new(error)
    }

    // --- Taxonomy registry (Part B / F8) ---
    //
    // Mirrors outcome.rs's `all_reasons_are_unique`: the `code` taxonomy is an
    // external contract, so every code must be unique and snake_case.

    #[test]
    fn all_codes_are_unique() {
        use std::collections::HashSet;

        let codes: Vec<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        let unique: HashSet<&str> = codes.iter().copied().collect();
        assert_eq!(
            unique.len(),
            codes.len(),
            "duplicate error code strings in: {codes:?}"
        );
    }

    #[test]
    fn all_codes_are_snake_case() {
        for code in ErrorCode::ALL {
            let s = code.as_str();
            assert!(
                s.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "code `{s}` is not snake_case"
            );
            assert!(
                !s.starts_with('_') && !s.ends_with('_'),
                "code `{s}` edge _"
            );
            assert!(!s.contains("__"), "code `{s}` has a double underscore");
        }
    }

    // --- Classification (Part A / F1) ---

    #[test]
    fn unexpected_http_404_is_not_found_with_status() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            404,
            r#"{"message":"missing deployment"}"#.to_string(),
        ))
        .context("failed to fetch deployment abc-123");

        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::NotFound, Some(404))
        );
    }

    #[test]
    fn empty_response_not_found_maps_to_not_found_without_status() {
        let error = anyhow::Error::new(NotFound::new("deployment", "abc-123"))
            .context("failed to fetch deployment abc-123");

        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::NotFound, None)
        );
    }

    #[test]
    fn http_401_and_403_map_to_unauthorized() {
        for status in [401u16, 403] {
            let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
                status,
                "denied".to_string(),
            ));
            assert_eq!(
                classify(&error),
                Classification::new(ErrorCode::Unauthorized, Some(status)),
                "status {status}"
            );
        }
    }

    #[test]
    fn other_http_status_maps_to_api_error() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            500,
            "boom".to_string(),
        ));
        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::ApiError, Some(500))
        );
    }

    #[test]
    fn activity_failed_maps_to_api_error_without_status() {
        let error = client_error(TurnkeyClientError::ActivityFailed(None));
        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::ApiError, None)
        );
    }

    #[test]
    fn activity_requires_approval_maps_to_approval_required() {
        let error = client_error(TurnkeyClientError::ActivityRequiresApproval(
            "act-1".to_string(),
        ));
        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::ApprovalRequired, None)
        );
    }

    #[test]
    fn unrecognized_error_falls_back_to_command_error() {
        let error = anyhow!("some other failure").context("while doing a thing");
        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::CommandError, None)
        );
    }

    #[test]
    fn strip_ansi_removes_escape_sequences() {
        let styled = "\x1b[1mUsage:\x1b[0m tvc \x1b[32m<COMMAND>\x1b[0m";
        assert_eq!(strip_ansi(styled), "Usage: tvc <COMMAND>");
    }
}
