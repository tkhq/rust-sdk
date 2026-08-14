//! Client utilities for authenticated API calls.

use crate::config::turnkey::{Config, StoredApiKey};
use crate::errors::MissingResource;
use anyhow::{Context, Result, anyhow, bail};
use reqwest::header::{HeaderMap, HeaderValue};
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

/// Auth material for one organization, resolved from env vars or the local
/// config at the wiring layer before any client is constructed.
///
/// `Debug` stays test-only: the struct carries the API private key.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Credentials {
    org_id: String,
    api_base_url: String,
    api_key_public: String,
    api_key_private: String,
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

    let credentials = match load_credentials_from_env_vars()? {
        Some(credentials) => {
            debug!(auth_source = "env", "using env auth credentials");
            credentials
        }
        None => {
            debug!(auth_source = "config", "using local config credentials");
            load_credentials_from_config(config).await?
        }
    };

    build_authed_client(credentials)
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
async fn load_credentials_from_config(config: &Config) -> Result<Credentials> {
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

    let StoredApiKey {
        public_key,
        private_key,
        curve: _,
    } = api_key;

    Ok(Credentials {
        org_id: org_config.id.clone(),
        api_base_url: org_config.api_base_url.clone(),
        api_key_public: public_key,
        api_key_private: private_key,
    })
}

/// Header carrying the tvc release version on every API request. The backend
/// enforces a minimum-version floor on it (enforce-when-present) to retire
/// known-defective releases with an upgrade prompt.
const TVC_CLIENT_VERSION_HEADER: &str = "X-TVC-CLIENT-VERSION";

/// Default headers every tvc request carries: [`TVC_CLIENT_VERSION_HEADER`]
/// stamped with this crate's release version. The backend's version gate
/// relies on it riding every request, so callers pass this map to
/// [`build_turnkey_client`] as-is or extended — never replaced.
pub(crate) fn tvc_client_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        TVC_CLIENT_VERSION_HEADER,
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );
    headers
}

/// Build the Turnkey API client used for all tvc requests from fully
/// constructed inputs.
///
/// `headers` is assembled by the caller, starting from [`tvc_client_headers`].
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
fn build_authed_client(credentials: Credentials) -> Result<AuthenticatedClient> {
    let Credentials {
        org_id,
        api_base_url,
        api_key_public,
        api_key_private,
    } = credentials;

    debug!("constructing API key stamper");
    let stamper = TurnkeyP256ApiKey::from_strings(&api_key_private, Some(&api_key_public))
        .context("failed to load API key")?;

    debug!(%api_base_url, "building Turnkey API client");
    let client = build_turnkey_client(stamper, &api_base_url, tvc_client_headers())?;

    debug!("authenticated Turnkey client ready");

    Ok(AuthenticatedClient {
        client,
        org_id,
        api_base_url,
    })
}

/// Read an env var, treating empty strings as unset. CI tools may default missing
/// secrets/vars to `""` which could cause downstream errors.
fn read_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Read auth env vars and assemble [`Credentials`] from them.
///
/// - `Ok(None)`: none of the required env vars set; caller should fall back to disk.
/// - `Ok(Some(credentials))`: all three required vars set; `api_base_url`
///   falls back to the default if unset.
/// - `Err`: only some of the required vars are set; the error names which.
#[instrument(skip_all)]
fn load_credentials_from_env_vars() -> Result<Option<Credentials>> {
    let org_id = read_env_var(ENV_ORG_ID);
    let api_key_public = read_env_var(ENV_API_KEY_PUBLIC);
    let api_key_private = read_env_var(ENV_API_KEY_PRIVATE);
    // Optional; defaults to prod if unset.
    let api_base_url = read_env_var(ENV_API_BASE_URL);

    debug!(
        tvc_org_id_set = org_id.is_some(),
        tvc_api_key_public_set = api_key_public.is_some(),
        tvc_api_key_private_set = api_key_private.is_some(),
        tvc_api_base_url_set = api_base_url.is_some(),
        "read auth env vars"
    );

    credentials_from_env_values(org_id, api_key_public, api_key_private, api_base_url)
}

/// Assemble [`Credentials`] from already-read env values.
///
/// Pure over its inputs so the all/partial/none contract is unit-tested
/// without touching process env.
fn credentials_from_env_values(
    org_id: Option<String>,
    api_key_public: Option<String>,
    api_key_private: Option<String>,
    api_base_url: Option<String>,
) -> Result<Option<Credentials>> {
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

    Ok(Some(Credentials {
        org_id: org_id.unwrap(),
        api_base_url: api_base_url.unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string()),
        api_key_public: api_key_public.unwrap(),
        api_key_private: api_key_private.unwrap(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set(value: &str) -> Option<String> {
        Some(value.to_string())
    }

    #[test]
    fn env_credentials_absent_when_no_auth_vars_set() {
        let credentials =
            credentials_from_env_values(None, None, None, None).expect("no vars set is valid");
        assert!(credentials.is_none());
    }

    #[test]
    fn env_credentials_error_names_every_missing_var() {
        let error = credentials_from_env_values(set("org"), None, None, None)
            .expect_err("partial env auth must error");
        assert_eq!(
            error.to_string(),
            "partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. \
             Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none."
        );
    }

    #[test]
    fn env_credentials_default_base_url_when_unset() {
        let credentials = credentials_from_env_values(set("org"), set("pub"), set("priv"), None)
            .expect("complete env auth")
            .expect("credentials present");
        assert_eq!(credentials.org_id, "org");
        assert_eq!(credentials.api_base_url, DEFAULT_API_BASE_URL);
        assert_eq!(credentials.api_key_public, "pub");
        assert_eq!(credentials.api_key_private, "priv");
    }

    #[test]
    fn env_credentials_use_explicit_base_url() {
        let credentials = credentials_from_env_values(
            set("org"),
            set("pub"),
            set("priv"),
            set("http://localhost:8081"),
        )
        .expect("complete env auth")
        .expect("credentials present");
        assert_eq!(credentials.api_base_url, "http://localhost:8081");
    }

    #[test]
    fn client_headers_carry_exactly_the_release_version() {
        let headers = tvc_client_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers
                .get(TVC_CLIENT_VERSION_HEADER)
                .map(|value| value.to_str().expect("version header is ascii")),
            Some(env!("CARGO_PKG_VERSION"))
        );
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

        let client = build_turnkey_client(
            TurnkeyP256ApiKey::generate(),
            &server.uri(),
            tvc_client_headers(),
        )
        .expect("client builds");
        let response: serde_json::Value = client
            .process_request(&serde_json::json!({}), "/any/path".to_string())
            .await
            .expect("request carrying the version header matches the mock");

        assert_eq!(response, serde_json::json!({}));
        server.verify().await;
    }
}
