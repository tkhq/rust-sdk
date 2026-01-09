//! Client utilities for authenticated API calls.

use crate::config::turnkey::{ApiKey, Config};
use anyhow::{Context, Result};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::TurnkeyClient;

/// An authenticated Turnkey client with organization context.
pub struct AuthenticatedClient {
    /// The Turnkey API client.
    pub client: TurnkeyClient<TurnkeyP256ApiKey>,
    /// The organization ID for API calls.
    pub org_id: String,
}

/// Build an authenticated Turnkey client from the local config.
///
/// Loads the active organization and API key from `~/.config/turnkey/`.
/// Returns an error if not logged in.
pub async fn build_client(api_base_url: &str) -> Result<AuthenticatedClient> {
    let config = Config::load().await?;

    let (alias, org_config) = config
        .active_org_config()
        .ok_or_else(|| anyhow::anyhow!("No active organization. Run `tvc login` first."))?;

    let api_key = ApiKey::load(org_config).await?.ok_or_else(|| {
        anyhow::anyhow!("No API key found for org '{alias}'. Run `tvc login` first.")
    })?;

    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    let client = TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

    Ok(AuthenticatedClient {
        client,
        org_id: org_config.id.clone(),
    })
}
