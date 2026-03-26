//! Client utilities for authenticated API calls.

use crate::config::turnkey::{Config, StoredApiKey};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::TurnkeyClient;

#[derive(Debug, Clone, Default)]
pub struct ClientOverrides {
    pub api_key_file: Option<PathBuf>,
    pub api_url: Option<String>,
    pub org_id: Option<String>,
}

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
/// When overrides for API key file, API URL, and org ID are all provided,
/// the client is built directly from those values without loading config from disk.
/// Otherwise, loads the active organization and API key from `~/.config/turnkey/`.
pub async fn build_client(overrides: &ClientOverrides) -> Result<AuthenticatedClient> {
    if let (Some(api_key_file), Some(api_url), Some(org_id)) = (
        &overrides.api_key_file,
        &overrides.api_url,
        &overrides.org_id,
    ) {
        let api_key = load_api_key_from_path(api_key_file)?;
        let stamper =
            TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
                .context("failed to load API key")?;

        let client = TurnkeyClient::builder()
            .api_key(stamper)
            .base_url(api_url)
            .build()
            .context("failed to build Turnkey client")?;

        return Ok(AuthenticatedClient {
            client,
            org_id: org_id.clone(),
            api_base_url: api_url.clone(),
        });
    }

    let config = Config::load().await?;

    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow::anyhow!("No active organization. Run `tvc login` first."))?;

    let api_key = match &overrides.api_key_file {
        Some(path) => load_api_key_from_path(path)?,
        None => StoredApiKey::load(org_config).await?.ok_or_else(|| {
            anyhow::anyhow!("No API key found for org '{alias}'. Run `tvc login` first.")
        })?,
    };

    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    let api_base_url = overrides
        .api_url
        .as_deref()
        .unwrap_or(&org_config.api_base_url);
    let org_id = overrides.org_id.as_deref().unwrap_or(&org_config.id);

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

fn load_api_key_from_path(path: &Path) -> Result<StoredApiKey> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read API key file: {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse API key file: {}", path.display()))
}
