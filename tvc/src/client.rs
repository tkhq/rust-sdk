//! Client utilities for authenticated API calls.

use crate::config::turnkey::{Config, StoredApiKey};
use anyhow::{Context, Result, anyhow, bail};
use tracing::debug;
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

    build_authed_client(&org_id, &api_base_url, &api_key_public, &api_key_private)
}

pub async fn fetch_tvc_app(auth: &AuthenticatedClient, app_id: &str) -> Result<TvcApp> {
    let response = auth
        .client
        .get_tvc_app(GetTvcAppRequest {
            organization_id: auth.org_id.clone(),
            tvc_app_id: app_id.to_string(),
        })
        .await
        .context("failed to fetch app")?;

    response
        .tvc_app
        .ok_or_else(|| anyhow!("app not found: {app_id}"))
}

pub async fn fetch_tvc_deployment(
    auth: &AuthenticatedClient,
    organization_id: String,
    deployment_id: String,
) -> Result<TvcDeployment> {
    let response = auth
        .client
        .get_tvc_deployment(GetTvcDeploymentRequest {
            organization_id,
            deployment_id,
        })
        .await
        .context("failed to fetch deployment")?;

    response
        .tvc_deployment
        .ok_or_else(|| anyhow!("deployment not found"))
}

async fn load_credentials_from_config() -> Result<(String, String, String, String)> {
    let config = Config::load().await?;

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
    let client = TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

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
