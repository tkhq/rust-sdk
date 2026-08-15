//! Error taxonomy and classification.
//!
//! This module is the single home for TVC's machine-readable error taxonomy:
//! the [`ErrorCode`] enum and its stable snake_case wire names, the typed
//! [`MissingResource`] error, and the [`classify`] logic that walks an
//! [`anyhow::Error`] chain and maps recognized causes to a [`Classification`]
//! `(code, http_status)`. It also owns [`render_error_chain`], the shared
//! human/JSON renderer that preserves the full source chain and caps messages
//! before they cross the CLI output boundary.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::path::Path;
use turnkey_client::TurnkeyClientError;

/// Cap on rendered error messages. Large enough for any real API error body
/// (typical Turnkey error JSON is < 1 KB); small enough that a runaway body
/// (HTML error page, proxy dump) cannot bloat a single NDJSON line.
const MAX_ERROR_MESSAGE_CHARS: usize = 8 * 1024;

/// A resource lookup that returned successfully but found nothing — e.g. an API
/// `Ok` response whose optional payload is `None`. Downcast in [`classify`] to
/// classify these as [`ErrorCode::NotFound`], alongside HTTP 404s.
#[derive(Debug, thiserror::Error)]
#[error("{resource} not found: {id}")]
pub struct MissingResource {
    resource: &'static str,
    id: String,
}

impl MissingResource {
    pub fn new(resource: &'static str, id: impl Into<String>) -> Self {
        Self {
            resource,
            id: id.into(),
        }
    }
}

/// The stable, machine-readable classification of a runtime error, carried in
/// the `code` field of a `command_error` (or `missing_required_input`) message.
///
/// `code` is the taxonomy axis; the message `reason` stays stable
/// (`command_error` for all runtime errors) so the outcome registry is
/// unaffected. Serde derives each variant's stable snake_case wire name
/// directly from the enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// A required value was absent in non-interactive mode.
    MissingRequiredInput,
    /// Bad flags/args — a clap parse failure
    UsageError,
    #[allow(dead_code)]
    /// Semantic validation failure in command code.
    InvalidInput,
    /// HTTP 401/403.
    Unauthorized,
    /// HTTP 404, or an OK response with an empty resource.
    NotFound,
    /// Any other non-success HTTP status, or a failed/unexpected activity.
    ApiError,
    /// The backend rejected this tvc release as older than the minimum
    /// version it supports. Remediation: upgrade tvc.
    ClientVersionTooOld,
    /// An activity needs more approvals.
    ApprovalRequired,
    /// A connect/timeout/DNS failure — the request never reached the server.
    NetworkError,
    /// Fallback for everything else.
    CommandError,
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
///
/// Classification does not alter the original error or its rendered cause
/// chain. Unrecognized errors use [`ErrorCode::CommandError`] with no HTTP
/// status; callers still render the complete original chain.
pub fn classify(error: &anyhow::Error) -> Classification {
    for cause in error.chain() {
        if cause.downcast_ref::<MissingResource>().is_some() {
            return Classification::new(ErrorCode::NotFound, None);
        }
        if let Some(client_error) = cause.downcast_ref::<TurnkeyClientError>() {
            return classify_turnkey_client_error(client_error);
        }
    }
    Classification::new(ErrorCode::CommandError, None)
}

/// A remediation hint for recognized error causes, rendered by the human
/// output layer beneath the error line. Today only the backend's
/// client-version rejection produces one: the hint is the server's own
/// message (it names the running and minimum versions), so the reader does
/// not have to dig it out of the raw response body in the chain.
pub fn hint(error: &anyhow::Error) -> Option<String> {
    error.chain().find_map(|cause| {
        let _envelope = client_version_rejection(cause.downcast_ref::<TurnkeyClientError>()?)?;
        let name = binary_name();

        format!(
            "`{name}` is older than the minimum version the backend supports; \
             upgrade `{name}` to the latest release"
        )
        .into()
    })
}

/// The invoked binary's name for user-facing version messaging: argv[0]'s file
/// stem (so `/usr/local/bin/tvc` and `./tvc` both read as `tvc`), falling back
/// to `tvc` when argv is empty.
pub(crate) fn binary_name() -> String {
    let arg0 = std::env::args().next();

    arg0.as_deref()
        .map(Path::new)
        .and_then(Path::file_stem)
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tvc".to_string())
}

/// Render an error's complete cause chain like anyhow's alternate
/// `{error:#}` display (links joined with `": "`), then truncate the result to
/// [`MAX_ERROR_MESSAGE_CHARS`].
///
/// Every source link is preserved, even when a parent error's `Display`
/// already embeds the source text.
pub(crate) fn render_error_chain(error: &anyhow::Error) -> String {
    let mut rendered = String::new();
    for cause in error.chain() {
        let text = cause.to_string();
        if !rendered.is_empty() {
            rendered.push_str(": ");
        }
        rendered.push_str(&text);
    }
    truncate_message(rendered)
}

fn truncate_message(message: String) -> String {
    if message.len() <= MAX_ERROR_MESSAGE_CHARS {
        return message;
    }

    let total = message.len();
    // Truncate on a char boundary so we never split a UTF-8 sequence.
    let mut cut = MAX_ERROR_MESSAGE_CHARS;
    while !message.is_char_boundary(cut) {
        cut -= 1;
    }
    format!(
        "{}… [error message truncated; {total} bytes total]",
        &message[..cut]
    )
}

/// The `turnkeyErrorCode` the backend attaches when it rejects a tvc release
/// older than the minimum version it supports (the client-version gate).
/// Must match the `TurnkeyErrorCode` enum value name in Turnkey's public
/// error proto.
// TODO(TVC-269): generate `TurnkeyErrorCode` from the synced errors.proto and
// match on the enum variant instead of this string.
const TVC_CLIENT_VERSION_TOO_OLD: &str = "TVC_CLIENT_VERSION_TOO_OLD";

/// The subset of the Turnkey public API error envelope (grpc-gateway's JSON
/// rendering of a gRPC status, plus Turnkey's machine-readable code) that
/// classification reads.
#[derive(Deserialize)]
struct ApiErrorEnvelope {
    // The server's human-readable message. Not consumed today — `hint` templates
    // its own text off the binary name — but kept parsed so we can surface the
    // backend's wording (e.g. the concrete running/minimum versions) once we
    // decide to.
    #[serde(default)]
    _message: String,
    #[serde(default, rename = "turnkeyErrorCode")]
    turnkey_error_code: String,
}

/// Parse the backend's client-version rejection out of a client error, if
/// that is what it is: an HTTP failure whose body is a Turnkey error envelope
/// carrying [`TVC_CLIENT_VERSION_TOO_OLD`]. Shared by [`classify`] and
/// [`hint`] so they cannot disagree on what counts as one.
fn client_version_rejection(error: &TurnkeyClientError) -> Option<ApiErrorEnvelope> {
    let TurnkeyClientError::UnexpectedHttpStatus(_, body) = error else {
        return None;
    };

    serde_json::from_str::<ApiErrorEnvelope>(body)
        .ok()
        .filter(|envelope| envelope.turnkey_error_code == TVC_CLIENT_VERSION_TOO_OLD)
}

/// Map a [`TurnkeyClientError`] to its taxonomy code and optional HTTP status.
///
/// NOTE: this may fail to compile when new errors are introduced in upstream Turnkey code, which is good.
/// We should explicitly decide what kind of error it is here.
fn classify_turnkey_client_error(error: &TurnkeyClientError) -> Classification {
    match error {
        TurnkeyClientError::UnexpectedHttpStatus(status, _) => {
            let code = if client_version_rejection(error).is_some() {
                ErrorCode::ClientVersionTooOld
            } else {
                match StatusCode::from_u16(*status) {
                    Ok(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => ErrorCode::Unauthorized,
                    Ok(StatusCode::NOT_FOUND) => ErrorCode::NotFound,
                    _ => ErrorCode::ApiError,
                }
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
        // The server answered with a redirect the client would not follow, so
        // the request never reached a usable endpoint. Report it as an API
        // error carrying the redirect status.
        TurnkeyClientError::RefusedRedirect(status, _) => {
            Classification::new(ErrorCode::ApiError, Some(*status))
        }
        // The server responded, but its response violated the expected API
        // protocol or the activity did not complete successfully.
        TurnkeyClientError::MissingContentTypeHeader
        | TurnkeyClientError::HeaderToStrError(_)
        | TurnkeyClientError::HeaderFromStrError(_)
        | TurnkeyClientError::UnexpectedMimeType(_)
        | TurnkeyClientError::Decode(_, _)
        | TurnkeyClientError::ActivityFailed(_)
        | TurnkeyClientError::UnexpectedActivityStatus(_)
        | TurnkeyClientError::UnexpectedInnerActivityResult(_)
        | TurnkeyClientError::MissingActivity
        | TurnkeyClientError::MissingResult
        | TurnkeyClientError::MissingInnerResult
        | TurnkeyClientError::ExceededRetries(_) => Classification::new(ErrorCode::ApiError, None),
        // These failures happen while configuring the client or constructing
        // and signing the request, before a usable API response is available.
        // A reqwest error that is not request/connect/timeout-related also
        // falls back to a generic command failure.
        TurnkeyClientError::BuilderMissingApiKey
        | TurnkeyClientError::ReqwestBuilder(_)
        | TurnkeyClientError::Http(_)
        | TurnkeyClientError::SerdeJsonFailure(_)
        | TurnkeyClientError::StamperError(_) => Classification::new(ErrorCode::CommandError, None),
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

    #[derive(Debug, thiserror::Error)]
    #[error("connection refused by peer")]
    struct Leaf;

    /// Mirrors `TurnkeyClientError::Http`: the source is both embedded in
    /// `Display` and linked through `Error::source`.
    #[derive(Debug, thiserror::Error)]
    #[error("HTTP request failed: {0}")]
    struct EmbedsSource(#[from] Leaf);

    fn client_error(error: TurnkeyClientError) -> anyhow::Error {
        anyhow::Error::new(error)
    }

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
        let error = anyhow::Error::new(MissingResource::new("deployment", "abc-123"))
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

    const TOO_OLD_BODY: &str = r#"{"code":3,"message":"tvc 0.12.0 is older than the minimum version this backend supports (0.12.2); upgrade tvc to the latest release","details":[],"turnkeyErrorCode":"TVC_CLIENT_VERSION_TOO_OLD"}"#;

    #[test]
    fn client_version_rejection_maps_to_client_version_too_old() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            400,
            TOO_OLD_BODY.to_string(),
        ));

        assert_eq!(
            classify(&error),
            Classification::new(ErrorCode::ClientVersionTooOld, Some(400))
        );
    }

    #[test]
    fn client_version_rejection_hint_is_the_server_message() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            400,
            TOO_OLD_BODY.to_string(),
        ))
        .context("failed to create TVC deployment");

        assert_eq!(
            hint(&error).as_deref().map(|message| message
                .contains("is older than the minimum version the backend supports")),
            Some(true)
        );
    }

    #[test]
    fn client_version_rejection_without_server_message_still_hints() {
        let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
            400,
            r#"{"turnkeyErrorCode":"TVC_CLIENT_VERSION_TOO_OLD"}"#.to_string(),
        ));

        assert_eq!(
            hint(&error)
                .as_deref()
                .map(|msg| msg.contains("is older than the minimum version the backend supports")),
            Some(true)
        );
    }

    #[test]
    fn other_api_errors_do_not_hint_or_reclassify() {
        let bodies = [
            r#"{"code":3,"message":"bad request","details":[],"turnkeyErrorCode":""}"#,
            r#"{"code":3,"message":"bad request","details":[],"turnkeyErrorCode":"REQUEST_INVALID"}"#,
            "not json at all",
        ];

        for body in bodies {
            let error = client_error(TurnkeyClientError::UnexpectedHttpStatus(
                400,
                body.to_string(),
            ));

            assert_eq!(
                classify(&error),
                Classification::new(ErrorCode::ApiError, Some(400)),
                "body {body:?}"
            );
            assert_eq!(hint(&error), None, "body {body:?}");
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
    fn response_protocol_and_activity_failures_map_to_api_error() {
        let decode_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let errors = [
            TurnkeyClientError::MissingContentTypeHeader,
            TurnkeyClientError::HeaderToStrError("invalid header".to_string()),
            TurnkeyClientError::HeaderFromStrError("invalid MIME type".to_string()),
            TurnkeyClientError::UnexpectedMimeType("text/plain".to_string()),
            TurnkeyClientError::Decode("invalid JSON".to_string(), decode_error),
            TurnkeyClientError::MissingActivity,
            TurnkeyClientError::MissingResult,
            TurnkeyClientError::MissingInnerResult,
            TurnkeyClientError::UnexpectedActivityStatus("REJECTED".to_string()),
            TurnkeyClientError::UnexpectedInnerActivityResult("unexpected result".to_string()),
            TurnkeyClientError::ActivityFailed(None),
            TurnkeyClientError::ExceededRetries(3),
        ];

        for error in errors {
            let error = client_error(error);
            assert_eq!(
                classify(&error),
                Classification::new(ErrorCode::ApiError, None)
            );
        }
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
    fn local_client_failures_map_to_command_error() {
        let serialization_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let errors = [
            TurnkeyClientError::BuilderMissingApiKey,
            TurnkeyClientError::SerdeJsonFailure(serialization_error),
            TurnkeyClientError::StamperError(turnkey_api_key_stamper::StamperError::HexDecode(
                "invalid hex".to_string(),
            )),
        ];

        for error in errors {
            let error = client_error(error);
            assert_eq!(
                classify(&error),
                Classification::new(ErrorCode::CommandError, None)
            );
        }
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

    #[test]
    fn render_error_chain_preserves_embedded_source_links() {
        let error = anyhow::Error::new(EmbedsSource(Leaf)).context("failed to list TVC apps");

        assert_eq!(
            render_error_chain(&error),
            "failed to list TVC apps: HTTP request failed: connection refused by peer: connection refused by peer"
        );
    }

    #[test]
    fn render_error_chain_matches_alternate_display_for_short_chain() {
        let error = anyhow!("root cause").context("mid").context("top");
        let alternate_display = format!("{:#}", error);

        assert_eq!(render_error_chain(&error), alternate_display);
    }

    #[test]
    fn truncated_message_is_capped_and_labeled() {
        let error = anyhow!("failed: {}", "x".repeat(20_000));
        let rendered = render_error_chain(&error);

        assert!(rendered.len() <= MAX_ERROR_MESSAGE_CHARS + 64);
        assert!(rendered.ends_with("… [error message truncated; 20008 bytes total]"));
    }

    #[test]
    fn truncated_message_preserves_utf8_char_boundaries() {
        let message = format!("a{}", "é".repeat(10_000));
        let error = anyhow!("{message}");
        let rendered = render_error_chain(&error);
        let expected = format!(
            "{}… [error message truncated; 20001 bytes total]",
            &message[..MAX_ERROR_MESSAGE_CHARS - 1]
        );

        assert_eq!(rendered, expected);
    }
}
