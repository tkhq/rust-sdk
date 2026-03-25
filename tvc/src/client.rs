//! Client utilities for authenticated API calls.

use crate::cli::GlobalOpts;
use crate::config::turnkey::{Config, StoredApiKey};
use anyhow::{Context, Result};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::TurnkeyClient;

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
/// When global override flags (--api-key-file, --api-url, --org-id) are all provided,
/// the client is built directly from those values without loading config from disk.
/// Otherwise, loads the active organization and API key from `~/.config/turnkey/`.
pub async fn build_client_with_overrides(global: &GlobalOpts) -> Result<AuthenticatedClient> {
    // If all override flags are set, build client directly (no login required)
    if let (Some(api_key_file), Some(api_url), Some(org_id)) =
        (&global.api_key_file, &global.api_url, &global.org_id)
    {
        let content = std::fs::read_to_string(api_key_file)
            .with_context(|| format!("failed to read API key file: {}", api_key_file.display()))?;
        let api_key: StoredApiKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse API key file: {}", api_key_file.display()))?;

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

    // Fall back to config-based client, applying any partial overrides
    let config = Config::load().await?;

    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow::anyhow!("No active organization. Run `tvc login` first."))?;

    // Use override API key file or default from org config
    let api_key = match &global.api_key_file {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read API key file: {}", path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("failed to parse API key file: {}", path.display()))?
        }
        None => StoredApiKey::load(org_config).await?.ok_or_else(|| {
            anyhow::anyhow!("No API key found for org '{alias}'. Run `tvc login` first.")
        })?,
    };

    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    let api_base_url = global
        .api_url
        .as_deref()
        .unwrap_or(&org_config.api_base_url);

    let org_id = global.org_id.as_deref().unwrap_or(&org_config.id);

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

/// Build an authenticated Turnkey client from the local config.
///
/// Loads the active organization and API key from `~/.config/turnkey/`.
/// Returns an error if not logged in.
pub async fn build_client() -> Result<AuthenticatedClient> {
    let config = Config::load().await?;

    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow::anyhow!("No active organization. Run `tvc login` first."))?;

    let api_key = StoredApiKey::load(org_config).await?.ok_or_else(|| {
        anyhow::anyhow!("No API key found for org '{alias}'. Run `tvc login` first.")
    })?;

    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    let client = TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(&org_config.api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

    Ok(AuthenticatedClient {
        client,
        org_id: org_config.id.clone(),
        api_base_url: org_config.api_base_url.clone(),
    })
}
