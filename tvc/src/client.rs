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
use uuid::Uuid;

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
    pub org_id: Uuid,
    /// The API base URL for the active org. Used for environment-specific behavior.
    pub api_base_url: String,
}

/// Build an authenticated Turnkey client.
///
/// Prefers env auth (CI use case): if `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and
/// `TVC_API_KEY_PRIVATE` are all set, builds the client from env vars.
/// `TVC_API_BASE_URL` is optional and defaults to `https://api.turnkey.com`.
///
/// Otherwise, falls back to loading from `~/.config/turnkey/` (after `tvc login`).
///
/// If only some of the three required env vars are set, errors with the list of
/// missing names — no merged resolve between env and disk vars.
#[instrument(skip_all)]
pub async fn build_client() -> Result<AuthenticatedClient> {
    debug!("building authenticated Turnkey client");

    let (org_id, api_base_url, api_key_public, api_key_private) =
        match load_credentials_from_env_vars()? {
            Some(creds) => {
                debug!(auth_source = "env", "using env auth credentials");
                creds
            }
            None => {
                debug!(auth_source = "config", "using local config credentials");
                load_credentials_from_config().await?
            }
        };

    build_authed_client(org_id, &api_base_url, &api_key_public, &api_key_private)
}

#[instrument(skip_all)]
pub async fn fetch_tvc_app(auth: &AuthenticatedClient, app_id: &str) -> Result<TvcApp> {
    let response = auth
        .client
        .get_tvc_app(GetTvcAppRequest {
            organization_id: auth.org_id.to_string(),
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
async fn load_credentials_from_config() -> Result<(Uuid, String, String, String)> {
    let config = Config::load().await?;

    let (org_id, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow!("No active organization. Run `tvc login` first."))?;

    debug!(
        %org_id,
        api_base_url = %org_config.api_base_url,
        api_key_path = %org_config.api_key_path.display(),
        "resolved active organization config"
    );

    let api_key = StoredApiKey::load(org_config).await?.ok_or_else(|| {
        anyhow!(
            "No API key found for org '{}'. Run `tvc login` first.",
            config.display_name(org_id)
        )
    })?;

    Ok((
        org_id,
        org_config.api_base_url.clone(),
        api_key.public_key.clone(),
        api_key.private_key.clone(),
    ))
}

/// Header carrying the tvc release version on every API request. The backend
/// enforces a minimum-version floor on it (enforce-when-present) to retire
/// known-defective releases with an upgrade prompt.
const TVC_CLIENT_VERSION_HEADER: &str = "X-TVC-CLIENT-VERSION";

/// Build the Turnkey API client used for all tvc requests.
///
/// Every tvc client must be constructed here: this is the single place that
/// stamps [`TVC_CLIENT_VERSION_HEADER`] with this crate's release version, and
/// the backend's version gate relies on it riding every request.
#[instrument(skip_all)]
pub(crate) fn build_turnkey_client(
    stamper: TurnkeyP256ApiKey,
    api_base_url: &str,
) -> Result<TurnkeyClient<TurnkeyP256ApiKey>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        TVC_CLIENT_VERSION_HEADER,
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );

    TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .with_reqwest_builder(|builder| builder.default_headers(headers))
        .build()
        .context("failed to build Turnkey client")
}

#[instrument(skip_all)]
fn build_authed_client(
    org_id: Uuid,
    api_base_url: &str,
    api_key_public: &str,
    api_key_private: &str,
) -> Result<AuthenticatedClient> {
    debug!("constructing API key stamper");
    let stamper = TurnkeyP256ApiKey::from_strings(api_key_private, Some(api_key_public))
        .context("failed to load API key")?;

    debug!(%api_base_url, "building Turnkey API client");
    let client = build_turnkey_client(stamper, api_base_url)?;

    debug!("authenticated Turnkey client ready");

    Ok(AuthenticatedClient {
        client,
        org_id,
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
fn load_credentials_from_env_vars() -> Result<Option<(Uuid, String, String, String)>> {
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

    let org_id = org_id
        .unwrap()
        .parse()
        .with_context(|| format!("{ENV_ORG_ID} must be a UUID"))?;

    Ok(Some((
        org_id,
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

    #[tokio::test]
    async fn built_clients_stamp_the_tvc_release_version_on_every_request() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("X-TVC-CLIENT-VERSION", env!("CARGO_PKG_VERSION")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&server)
            .await;

        let client = build_turnkey_client(TurnkeyP256ApiKey::generate(), &server.uri())
            .expect("client builds");
        let response: serde_json::Value = client
            .process_request(&serde_json::json!({}), "/any/path".to_string())
            .await
            .expect("request carrying the version header matches the mock");

        assert_eq!(response, serde_json::json!({}));
        server.verify().await;
    }
}
