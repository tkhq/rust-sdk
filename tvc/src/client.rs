//! Client utilities for authenticated API calls.

use crate::config::turnkey::{Config, StoredApiKey};
use anyhow::{anyhow, bail, Context, Result};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::TurnkeyClient;

const NUM_AUTH_ENV_VARS: usize = 4;
const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_BASE_URL: &str = "TVC_API_BASE_URL";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";

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
/// Prefers direct env auth (CI use case): if all required env vars are set, builds the
/// client from env
///
/// Otherwise falls back to loading from `~/.config/turnkey/` (after `tvc login`).
///
/// If only some of the four env vars are set, errors with the list of missing
/// names — no silent fallback to disk.
pub async fn build_client() -> Result<AuthenticatedClient> {
    // Try env first; fall back to disk config.
    let (org_id, api_base_url, api_key_public, api_key_private) = match load_credentials_from_env_vars()? {
        Some(creds) => creds,
        None => load_credentials_from_config().await?,
    };
    build_authed_client(&org_id, &api_base_url, &api_key_public, &api_key_private)
}

async fn load_credentials_from_config() -> Result<(String, String, String, String)> {
    let config = Config::load().await?;

    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow!("No active organization. Run `tvc login` first."))?;

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
    let stamper = TurnkeyP256ApiKey::from_strings(api_key_private, Some(api_key_public))
        .context("failed to load API key")?;

    let client = TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

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
/// - `Ok(None)`: none set; caller should fall back to disk.
/// - `Ok(Some((org_id, api_base_url, api_key_public, api_key_private)))`: all four set.
/// - `Err`: only some are set; the error names which.
fn load_credentials_from_env_vars() -> Result<Option<(String, String, String, String)>> {
    let org_id = read_env_var(ENV_ORG_ID);
    let api_base_url = read_env_var(ENV_API_BASE_URL);
    let api_key_public = read_env_var(ENV_API_KEY_PUBLIC);
    let api_key_private = read_env_var(ENV_API_KEY_PRIVATE);

    let mut missing: Vec<&str> = Vec::new();
    if org_id.is_none() {
        missing.push(ENV_ORG_ID);
    }
    if api_base_url.is_none() {
        missing.push(ENV_API_BASE_URL);
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
            "partial env var auth: missing {}. Set all four ({}, {}, {}, {}) env vars or none.",
            missing.join(", "),
            ENV_ORG_ID,
            ENV_API_BASE_URL,
            ENV_API_KEY_PUBLIC,
            ENV_API_KEY_PRIVATE,
        );
    }

    Ok(Some((
        org_id.unwrap(),
        api_base_url.unwrap(),
        api_key_public.unwrap(),
        api_key_private.unwrap(),
    )))
}
