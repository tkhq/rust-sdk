//! Client utilities for authenticated API calls.

use crate::config::turnkey::{Config, StoredApiKey};
use crate::errors::MissingResource;
use anyhow::{Context, Result, anyhow, bail};
use opentelemetry::{propagation::TextMapPropagator, trace::TraceContextExt};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderValue};
use thiserror::Error;
use tracing::{debug, instrument};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::{
    TurnkeyClient,
    generated::{
        GetTvcAppRequest, GetTvcDeploymentRequest,
        external::data::v1::{TvcApp, TvcDeployment},
    },
};

/// Number of *required* auth env vars: org_id, api_key_public, api_key_private.
/// `TVC_API_BASE_URL` is optional and defaults to `DEFAULT_API_BASE_URL`.
const NUM_AUTH_ENV_VARS: usize = 3;
const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_BASE_URL: &str = "TVC_API_BASE_URL";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";
const ENV_TRACEPARENT: &str = "TVC_OTEL_TRACEPARENT";
const ENV_TRACESTATE: &str = "TVC_OTEL_TRACESTATE";
const DEFAULT_API_BASE_URL: &str = "https://api.turnkey.com";

/// An authenticated Turnkey client with organization context.
pub struct AuthenticatedClient {
    /// The Turnkey API client.
    pub client: TurnkeyClient<TurnkeyP256ApiKey>,
    /// The organization ID for API calls.
    pub org_id: String,
    /// The API base URL for the active org. Used for environment-specific behavior.
    pub api_base_url: String,
}

/// Build an authenticated Turnkey client.
///
/// Prefers env auth (CI use case): if `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and
/// `TVC_API_KEY_PRIVATE` are all set, builds the client from env vars.
/// `TVC_API_BASE_URL` is optional and defaults to `https://api.turnkey.com`.
///
/// Otherwise, falls back to the active org's stored credentials in `config`
/// (set up by `tvc login`).
///
/// If only some of the three required env vars are set, errors with the list of
/// missing names — no merged resolve between env and disk vars.
#[instrument(skip_all)]
pub async fn build_client(config: &Config) -> Result<AuthenticatedClient> {
    debug!("building authenticated Turnkey client");

    let (org_id, api_base_url, api_key_public, api_key_private) =
        match load_credentials_from_env_vars()? {
            Some(creds) => {
                debug!(auth_source = "env", "using env auth credentials");
                creds
            }
            None => {
                debug!(auth_source = "config", "using local config credentials");
                load_credentials_from_config(config).await?
            }
        };

    build_authed_client(&org_id, &api_base_url, &api_key_public, &api_key_private)
}

#[instrument(skip_all)]
pub async fn fetch_tvc_app(auth: &AuthenticatedClient, app_id: &str) -> Result<TvcApp> {
    let response = auth
        .client
        .get_tvc_app(GetTvcAppRequest {
            organization_id: auth.org_id.clone(),
            tvc_app_id: app_id.to_string(),
        })
        .await
        .with_context(|| format!("failed to fetch app {app_id}"))?;

    response
        .tvc_app
        .ok_or_else(|| MissingResource::new("app", app_id).into())
}

#[instrument(skip_all)]
pub async fn fetch_tvc_deployment(
    auth: &AuthenticatedClient,
    organization_id: String,
    deployment_id: String,
) -> Result<TvcDeployment> {
    let response = auth
        .client
        .get_tvc_deployment(GetTvcDeploymentRequest {
            organization_id,
            deployment_id: deployment_id.clone(),
        })
        .await
        .with_context(|| format!("failed to fetch deployment {deployment_id}"))?;

    response
        .tvc_deployment
        .ok_or_else(|| MissingResource::new("deployment", deployment_id).into())
}

#[instrument(skip_all)]
async fn load_credentials_from_config(config: &Config) -> Result<(String, String, String, String)> {
    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow!("No active organization. Run `tvc login` first."))?;

    debug!(
        org_alias = %alias,
        api_base_url = %org_config.api_base_url,
        api_key_path = %org_config.api_key_path.display(),
        "resolved active organization config"
    );

    let api_key = StoredApiKey::load(org_config)
        .await?
        .ok_or_else(|| anyhow!("No API key found for org '{alias}'. Run `tvc login` first."))?;

    Ok((
        org_config.id.clone(),
        org_config.api_base_url.clone(),
        api_key.public_key.clone(),
        api_key.private_key.clone(),
    ))
}

/// Header carrying the tvc release version on every API request. The backend
/// enforces a minimum-version floor on it (enforce-when-present) to retire
/// known-defective releases with an upgrade prompt.
const TVC_CLIENT_VERSION_HEADER: &str = "X-TVC-CLIENT-VERSION";
const TRACEPARENT_HEADER: HeaderName = HeaderName::from_static("traceparent");
const TRACESTATE_HEADER: HeaderName = HeaderName::from_static("tracestate");
const MAX_TRACE_CONTEXT_HEADER_LEN: usize = 512;

#[derive(Debug, Error)]
enum BoundedHeaderValueError {
    #[error("header value exceeds its maximum length")]
    TooLong,
    #[error("invalid HTTP header value: {0}")]
    Invalid(#[source] InvalidHeaderValue),
}

/// An HTTP header value whose encoded length is bounded at construction.
struct BoundedHeaderValue<const MAX_LEN: usize>(HeaderValue);

impl<const MAX_LEN: usize> TryFrom<&str> for BoundedHeaderValue<MAX_LEN> {
    type Error = BoundedHeaderValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() > MAX_LEN {
            return Err(BoundedHeaderValueError::TooLong);
        }

        HeaderValue::from_str(value)
            .map(Self)
            .map_err(BoundedHeaderValueError::Invalid)
    }
}

impl<const MAX_LEN: usize> From<BoundedHeaderValue<MAX_LEN>> for HeaderValue {
    fn from(value: BoundedHeaderValue<MAX_LEN>) -> Self {
        value.0
    }
}

type TraceContextHeaderValue = BoundedHeaderValue<MAX_TRACE_CONTEXT_HEADER_LEN>;

fn parse_trace_context_header(name: &str, value: &str) -> Option<TraceContextHeaderValue> {
    match TraceContextHeaderValue::try_from(value) {
        Ok(value) => Some(value),
        Err(error) => {
            debug!(name, %error, "ignoring propagated trace header");
            None
        }
    }
}

/// Round-trips trace headers through the official W3C propagator.
fn normalize_trace_context_headers(carrier: HeaderMap) -> Option<HeaderMap> {
    let propagator = TraceContextPropagator::new();
    let context = propagator.extract(&HeaderExtractor(&carrier));
    if !context.span().span_context().is_valid() {
        debug!("ignoring invalid propagated trace context");
        return None;
    }

    let had_tracestate = carrier.contains_key(TRACESTATE_HEADER);
    let mut headers = HeaderMap::new();
    propagator.inject_context(&context, &mut HeaderInjector(&mut headers));

    // The propagator injects an empty `tracestate` when no valid state survived
    // extraction. Remove that placeholder and log when supplied state was rejected.
    if headers
        .get(TRACESTATE_HEADER)
        .is_some_and(|value| value.is_empty())
    {
        headers.remove(TRACESTATE_HEADER);
    }
    if had_tracestate && !headers.contains_key(TRACESTATE_HEADER) {
        debug!("ignoring invalid propagated tracestate");
    }

    Some(headers)
}

/// Builds the default headers, normalizing optional W3C trace context through
/// OpenTelemetry's official propagator. Trace propagation is deliberately
/// best-effort so malformed telemetry can never block an API request.
fn tvc_client_headers(traceparent: Option<&str>, tracestate: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        TVC_CLIENT_VERSION_HEADER,
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );

    let tracestate = tracestate.filter(|value| !value.is_empty());
    let (traceparent, tracestate) = match (traceparent, tracestate) {
        (Some(traceparent), tracestate) => {
            let Some(traceparent) =
                parse_trace_context_header(TRACEPARENT_HEADER.as_str(), traceparent)
            else {
                return headers;
            };

            (
                traceparent,
                tracestate.and_then(|tracestate| {
                    parse_trace_context_header(TRACESTATE_HEADER.as_str(), tracestate)
                }),
            )
        }
        (None, Some(_)) => {
            debug!("ignoring propagated tracestate without traceparent");
            return headers;
        }
        (None, None) => return headers,
    };

    let mut carrier = HeaderMap::new();
    carrier.insert(TRACEPARENT_HEADER, traceparent.into());

    if let Some(tracestate) = tracestate {
        carrier.insert(TRACESTATE_HEADER, tracestate.into());
    }

    if let Some(trace_context) = normalize_trace_context_headers(carrier) {
        headers.extend(trace_context);
    }

    headers
}

/// Acquires the complete default header set for a tvc API client.
pub(crate) fn tvc_client_headers_from_env() -> HeaderMap {
    let traceparent = read_env_var(ENV_TRACEPARENT);
    let tracestate = read_env_var(ENV_TRACESTATE);

    tvc_client_headers(traceparent.as_deref(), tracestate.as_deref())
}

/// Build a Turnkey API client from provided deps
#[instrument(skip_all)]
pub(crate) fn build_turnkey_client(
    stamper: TurnkeyP256ApiKey,
    api_base_url: &str,
    headers: HeaderMap,
) -> Result<TurnkeyClient<TurnkeyP256ApiKey>> {
    TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .with_reqwest_builder(|builder| builder.default_headers(headers))
        .build()
        .context("failed to build Turnkey client")
}

#[instrument(skip_all)]
fn build_authed_client(
    org_id: &str,
    api_base_url: &str,
    api_key_public: &str,
    api_key_private: &str,
) -> Result<AuthenticatedClient> {
    debug!("constructing API key stamper");
    let stamper = TurnkeyP256ApiKey::from_strings(api_key_private, Some(api_key_public))
        .context("failed to load API key")?;

    debug!(%api_base_url, "building Turnkey API client");
    let headers = tvc_client_headers_from_env();
    let client = build_turnkey_client(stamper, api_base_url, headers)?;

    debug!("authenticated Turnkey client ready");

    Ok(AuthenticatedClient {
        client,
        org_id: org_id.to_string(),
        api_base_url: api_base_url.to_string(),
    })
}

/// Read an env var, treating empty strings as unset. CI tools may default missing
/// secrets/vars to `""` which could cause downstream errors.
fn read_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Parse auth env vars for building client.
///
/// - `Ok(None)`: none of the required env vars set; caller should fall back to disk.
/// - `Ok(Some((org_id, api_base_url, api_key_public, api_key_private)))`: all three
///   required vars set; `api_base_url` falls back to the default if unset.
/// - `Err`: only some of the required vars are set; the error names which.
#[instrument(skip_all)]
fn load_credentials_from_env_vars() -> Result<Option<(String, String, String, String)>> {
    let org_id = read_env_var(ENV_ORG_ID);
    let api_key_public = read_env_var(ENV_API_KEY_PUBLIC);
    let api_key_private = read_env_var(ENV_API_KEY_PRIVATE);
    // Optional; defaults to prod if unset.
    let api_base_url =
        read_env_var(ENV_API_BASE_URL).unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string());

    let mut missing: Vec<&str> = Vec::new();
    if org_id.is_none() {
        missing.push(ENV_ORG_ID);
    }
    if api_key_public.is_none() {
        missing.push(ENV_API_KEY_PUBLIC);
    }
    if api_key_private.is_none() {
        missing.push(ENV_API_KEY_PRIVATE);
    }

    debug!(
        tvc_org_id_set = org_id.is_some(),
        tvc_api_key_public_set = api_key_public.is_some(),
        tvc_api_key_private_set = api_key_private.is_some(),
        tvc_api_base_url_set = read_env_var(ENV_API_BASE_URL).is_some(),
        missing = ?missing,
        "read auth env vars"
    );

    // Acceptable to have none set: fall back to disk.
    if missing.len() == NUM_AUTH_ENV_VARS {
        return Ok(None);
    }

    // Partial: bail with the list of missing names.
    if !missing.is_empty() {
        bail!(
            "partial env var auth: missing {}. Set all three ({}, {}, {}) env vars or none.",
            missing.join(", "),
            ENV_ORG_ID,
            ENV_API_KEY_PUBLIC,
            ENV_API_KEY_PRIVATE,
        );
    }

    Ok(Some((
        org_id.unwrap(),
        api_base_url,
        api_key_public.unwrap(),
        api_key_private.unwrap(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    const VALID_TRACESTATE: &str = "vendor=value";

    #[test]
    fn parse_trace_context_header_accepts_only_bounded_http_values() {
        let maximum_length = "a".repeat(MAX_TRACE_CONTEXT_HEADER_LEN);
        let oversized = "a".repeat(MAX_TRACE_CONTEXT_HEADER_LEN + 1);

        let parsed =
            parse_trace_context_header(TRACEPARENT_HEADER.as_str(), maximum_length.as_str())
                .expect("maximum-length HTTP header value should parse");
        let parsed: HeaderValue = parsed.into();

        assert_eq!(parsed, maximum_length);
        assert!(
            parse_trace_context_header(TRACEPARENT_HEADER.as_str(), oversized.as_str()).is_none()
        );
        assert!(
            parse_trace_context_header(TRACEPARENT_HEADER.as_str(), "invalid\r\nheader").is_none()
        );
    }

    #[test]
    fn normalize_trace_context_headers_preserves_only_valid_state() {
        for (tracestate, expected_tracestate) in [
            (Some(VALID_TRACESTATE), Some(VALID_TRACESTATE)),
            (None, None),
            (Some("Vendor=uppercase-keys-are-invalid"), None),
        ] {
            let mut carrier = HeaderMap::new();
            carrier.insert(
                TRACEPARENT_HEADER,
                HeaderValue::from_static(VALID_TRACEPARENT),
            );
            if let Some(tracestate) = tracestate {
                carrier.insert(TRACESTATE_HEADER, HeaderValue::from_static(tracestate));
            }

            let headers = normalize_trace_context_headers(carrier)
                .expect("valid traceparent should produce normalized headers");

            assert_eq!(headers.get(TRACEPARENT_HEADER).unwrap(), VALID_TRACEPARENT);
            assert_eq!(
                headers
                    .get(TRACESTATE_HEADER)
                    .map(|value| value.to_str().unwrap()),
                expected_tracestate
            );
        }
    }

    #[tokio::test]
    async fn built_clients_stamp_the_tvc_release_version_on_every_request() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("X-TVC-CLIENT-VERSION", env!("CARGO_PKG_VERSION")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&server)
            .await;

        let headers = tvc_client_headers(None, None);
        let client = build_turnkey_client(TurnkeyP256ApiKey::generate(), &server.uri(), headers)
            .expect("client builds");
        let response: serde_json::Value = client
            .process_request(&serde_json::json!({}), "/any/path".to_string())
            .await
            .expect("request carrying the version header matches the mock");

        assert_eq!(response, serde_json::json!({}));
        server.verify().await;
    }

    #[tokio::test]
    async fn built_clients_forward_vivosuite_trace_context_on_the_request() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("traceparent", VALID_TRACEPARENT))
            .and(header("tracestate", VALID_TRACESTATE))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&server)
            .await;

        let headers = tvc_client_headers(Some(VALID_TRACEPARENT), Some(VALID_TRACESTATE));
        let client = build_turnkey_client(TurnkeyP256ApiKey::generate(), &server.uri(), headers)
            .expect("client builds");
        let response: serde_json::Value = client
            .process_request(&serde_json::json!({}), "/any/path".to_string())
            .await
            .expect("request carrying trace context matches the mock");

        assert_eq!(response, serde_json::json!({}));
        server.verify().await;
    }

    #[test]
    fn client_headers_forward_vivosuite_trace_context() {
        let headers = tvc_client_headers(Some(VALID_TRACEPARENT), Some(VALID_TRACESTATE));

        assert_eq!(headers.get(TRACEPARENT_HEADER).unwrap(), VALID_TRACEPARENT);
        assert_eq!(headers.get(TRACESTATE_HEADER).unwrap(), VALID_TRACESTATE);
    }

    #[test]
    fn malformed_traceparent_fails_open_and_drops_the_trace_context() {
        let headers = tvc_client_headers(Some("not-a-w3c-traceparent\r\n"), Some(VALID_TRACESTATE));

        assert_eq!(
            headers.get(TVC_CLIENT_VERSION_HEADER).unwrap(),
            env!("CARGO_PKG_VERSION")
        );
        assert!(!headers.contains_key(TRACEPARENT_HEADER));
        assert!(!headers.contains_key(TRACESTATE_HEADER));

        build_turnkey_client(TurnkeyP256ApiKey::generate(), "http://localhost", headers)
            .expect("invalid optional trace context must not prevent client construction");
    }

    #[test]
    fn malformed_tracestate_is_dropped_without_dropping_traceparent() {
        let headers = tvc_client_headers(
            Some(VALID_TRACEPARENT),
            Some("Vendor=uppercase-keys-are-invalid"),
        );

        assert_eq!(headers.get(TRACEPARENT_HEADER).unwrap(), VALID_TRACEPARENT);
        assert!(!headers.contains_key(TRACESTATE_HEADER));
    }

    #[test]
    fn tracestate_without_traceparent_is_not_forwarded() {
        let headers = tvc_client_headers(None, Some(VALID_TRACESTATE));

        assert!(!headers.contains_key(TRACEPARENT_HEADER));
        assert!(!headers.contains_key(TRACESTATE_HEADER));
    }

    #[test]
    fn empty_tracestate_is_not_forwarded() {
        for tracestate in ["", " \t "] {
            let headers = tvc_client_headers(Some(VALID_TRACEPARENT), Some(tracestate));

            assert_eq!(headers.get(TRACEPARENT_HEADER).unwrap(), VALID_TRACEPARENT);
            assert!(!headers.contains_key(TRACESTATE_HEADER));
        }
    }

    #[test]
    fn oversized_tracestate_is_dropped_without_dropping_traceparent() {
        let oversized_tracestate = "a".repeat(MAX_TRACE_CONTEXT_HEADER_LEN + 1);
        let headers = tvc_client_headers(Some(VALID_TRACEPARENT), Some(&oversized_tracestate));

        assert_eq!(headers.get(TRACEPARENT_HEADER).unwrap(), VALID_TRACEPARENT);
        assert!(!headers.contains_key(TRACESTATE_HEADER));
    }

    #[test]
    fn oversized_traceparent_fails_open_and_drops_the_trace_context() {
        let oversized_traceparent = "a".repeat(MAX_TRACE_CONTEXT_HEADER_LEN + 1);
        let headers = tvc_client_headers(Some(&oversized_traceparent), Some(VALID_TRACESTATE));

        assert_eq!(
            headers.get(TVC_CLIENT_VERSION_HEADER).unwrap(),
            env!("CARGO_PKG_VERSION")
        );
        assert!(!headers.contains_key(TRACEPARENT_HEADER));
        assert!(!headers.contains_key(TRACESTATE_HEADER));
    }

    #[test]
    fn invalid_w3c_identifiers_are_not_forwarded() {
        for traceparent in [
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
            "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
        ] {
            let headers = tvc_client_headers(Some(traceparent), None);

            assert!(!headers.contains_key(TRACEPARENT_HEADER));
        }
    }
}
