//! Login command for authenticating with Turnkey.

use crate::config::turnkey::{
    API_BASE_URL_PROD, Config, KeyCurve, OrgConfig, StoredApiKey, StoredQosOperatorKey,
};
use crate::prompts;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use qos_p256::P256Pair;
use std::io::{BufRead, Write};
use tracing::debug;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::GetWhoamiRequest;

/// Authenticate with Turnkey and set up local credentials.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID to log in with.
    /// If not provided, will prompt interactively.
    #[arg(long, env = "TVC_ORG")]
    pub org: Option<String>,
    /// Turnkey API base URL. Defaults to production for newly configured orgs.
    #[arg(long, env = "TVC_API_BASE_URL", value_name = "URL")]
    pub api_base_url: Option<String>,
}

/// Run the login command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    debug!(
        org_arg_present = args.org.is_some(),
        api_base_url_override_present = args.api_base_url.is_some(),
        "running login command"
    );

    // Load existing config
    let mut config = Config::load().await?;

    // Select or create org
    let (alias, org_config) = select_or_create_org(
        &mut config,
        args.org.as_deref(),
        args.api_base_url.as_deref(),
    )
    .await?;

    println!("Selected org: {} ({})", alias, org_config.id);

    // Save config with the new/updated org
    config.set_active_org(&alias)?;
    config.save().await?;

    // Get or generate API key
    let api_key = get_or_generate_api_key(&org_config).await?;

    // Verify credentials with whoami
    println!();
    println!("Verifying credentials...");

    let whoami = verify_credentials(&api_key, &org_config.id, &org_config.api_base_url).await?;

    // Get or generate operator key
    let operator_key = get_or_generate_operator_key(&org_config).await?;

    println!();
    println!("Successfully logged in!");
    println!();
    println!(
        "Organization: {} ({})",
        whoami.organization_name, whoami.organization_id
    );
    println!("User: {} ({})", whoami.username, whoami.user_id);
    println!("Active Org: {alias}");
    println!("API Key: {}", api_key.public_key);
    println!("Operator Key: {}", operator_key.public_key);
    println!();
    println!(
        "Config: {}",
        crate::config::turnkey::config_file_path()?.display()
    );
    println!("API Key: {}", org_config.api_key_path.display());
    println!("Operator Key: {}", org_config.operator_key_path.display());

    Ok(())
}

/// Select an existing org or create a new one.
/// Returns the alias and a clone of the org config.
async fn select_or_create_org(
    config: &mut Config,
    org_arg: Option<&str>,
    api_base_url_override: Option<&str>,
) -> Result<(String, OrgConfig)> {
    debug!(
        org_arg_present = org_arg.is_some(),
        api_base_url_override_present = api_base_url_override.is_some(),
        configured_org_count = config.orgs.len(),
        active_org = ?config.active_org,
        "selecting organization"
    );

    // If --org provided, try to find it by alias or ID
    if let Some(org) = org_arg {
        if let Some((alias, _)) = find_org(config, org) {
            let alias = alias.clone();
            debug!(org_alias = %alias, "selected existing organization from argument");
            update_api_base_url_from_override(config, &alias, api_base_url_override);
            let org_config = config.orgs.get(&alias).unwrap().clone();
            return Ok((alias, org_config));
        }
        debug!("organization argument did not match configured organizations");
        bail!(
            "Organization '{org}' not found. Run `tvc login` without --org to set up a new organization."
        );
    }

    // No --org provided — we'd need to prompt. Honor explicit opt-out.
    prompts::bail_if_non_interactive("--org")?;

    // No --org provided, check existing orgs
    let org_count = config.orgs.len();

    if org_count == 0 {
        // No orgs configured - prompt for new org
        debug!("no organizations configured; prompting for new organization");
        println!("No organization configured.");
        return prompt_for_new_org(config, api_base_url_override).await;
    }

    // Show existing orgs in a `Select` list
    let mut options: Vec<OrgChoice> = config
        .orgs
        .iter()
        .map(|(alias, org)| {
            let suffix = if config.active_org.as_ref() == Some(alias) {
                " (active)"
            } else {
                ""
            };
            OrgChoice::Existing {
                display: format!("{alias} ({}){suffix}", org.id),
                alias: alias.clone(),
            }
        })
        .collect();
    options.push(OrgChoice::New);

    match prompts::select("Select organization", options)? {
        OrgChoice::Existing { alias, .. } => {
            update_api_base_url_from_override(config, &alias, api_base_url_override);
            let org_config = config.orgs.get(&alias).unwrap().clone();
            Ok((alias, org_config))
        }
        OrgChoice::New => prompt_for_new_org(config, api_base_url_override).await,
    }
}

/// Choices rendered by the org-selection `Select` widget.
enum OrgChoice {
    Existing { display: String, alias: String },
    New,
}

impl std::fmt::Display for OrgChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgChoice::Existing { display, .. } => write!(f, "{display}"),
            OrgChoice::New => write!(f, "[new] Add a new organization"),
        }
    }
}

/// Prompt the user to enter a new organization ID and alias.
async fn prompt_for_new_org(
    config: &mut Config,
    api_base_url_override: Option<&str>,
) -> Result<(String, OrgConfig)> {
    debug!("prompting for new organization");

    println!("You can find your Organization ID at: https://app.turnkey.com/dashboard/welcome");
    println!();

    let org_id = prompts::text("Organization ID", None)?;
    if org_id.is_empty() {
        bail!("Organization ID is required");
    }

    let alias = prompts::text("Organization alias", Some("default"))?;
    debug!(org_alias = %alias, "user selected organization");

    let api_base_url = new_org_api_base_url(api_base_url_override);
    debug!(org_alias = %alias, %api_base_url, "adding prompted organization");

    config.add_org(&alias, org_id, api_base_url)?;
    let org_config = config.orgs.get(&alias).unwrap().clone();
    Ok((alias, org_config))
}

fn new_org_api_base_url(api_base_url_override: Option<&str>) -> String {
    api_base_url_override
        .unwrap_or(API_BASE_URL_PROD)
        .to_string()
}

fn update_api_base_url_from_override(
    config: &mut Config,
    alias: &str,
    api_base_url_override: Option<&str>,
) {
    if let Some(api_base_url) = api_base_url_override {
        debug!(org_alias = alias, %api_base_url, "updating organization API base URL from override");
        if let Some(org_config) = config.orgs.get_mut(alias) {
            org_config.api_base_url = api_base_url.to_string();
        }
    }
}

/// Get an existing API key or generate a new one.
/// If an API key exists, it's returned directly.
/// If not, a new key is generated, saved, and the user is prompted to add it to the dashboard.
async fn get_or_generate_api_key(org_config: &OrgConfig) -> Result<StoredApiKey> {
    debug!(api_key_path = %org_config.api_key_path.display(), "resolving API key");

    // Check if API key already exists
    if let Some(api_key) = StoredApiKey::load(org_config).await? {
        debug!("using existing API key");
        println!("Using existing API key.");
        return Ok(api_key);
    }

    // Generate new API key
    debug!("generating new API key");
    println!();
    println!("Generating API key...");

    let stamper = TurnkeyP256ApiKey::generate();
    let public_key = hex::encode(stamper.compressed_public_key());
    let private_key = hex::encode(stamper.private_key());

    let api_key = StoredApiKey {
        public_key: public_key.clone(),
        private_key,
        curve: KeyCurve::P256,
    };

    // Save the key
    api_key.save(org_config).await?;

    // Display instructions
    println!();
    println!("API Key Generated!");
    println!();
    println!("Public Key: {public_key}");
    println!();
    println!("Add this API key to your Turnkey dashboard:");
    println!("  1. Go to https://app.turnkey.com/dashboard/users");
    println!("  2. Click your user > Create API Key > Generate API Keys via CLI > Continue");
    println!("  3. Paste the public key > Name it \"TVC CLI\" > Continue > Approve");
    println!();

    wait_for_enter("Press Enter when done...")?;

    Ok(api_key)
}

/// Get an existing operator key or generate a new one.
async fn get_or_generate_operator_key(org_config: &OrgConfig) -> Result<StoredQosOperatorKey> {
    debug!(operator_key_path = %org_config.operator_key_path.display(), "resolving operator key");

    // Check if operator key already exists
    if let Some(operator_key) = StoredQosOperatorKey::load(org_config).await? {
        debug!("using existing operator key");
        println!("Using existing operator key.");
        return Ok(operator_key);
    }

    // Generate new operator key
    debug!("generating new operator key");
    println!();
    println!("Generating operator key...");

    let pair =
        P256Pair::generate().map_err(|e| anyhow!("failed to generate operator key: {e:?}"))?;
    let public_key = hex::encode(pair.public_key().to_bytes());
    let private_key = hex::encode(pair.to_master_seed());

    let operator_key = StoredQosOperatorKey {
        public_key: public_key.clone(),
        private_key,
    };

    // Save the key
    operator_key.save(org_config).await?;

    println!();
    println!("Operator Key Generated!");
    println!();
    println!("Public Key: {public_key}");
    println!();
    println!("This key will be used for approving deployment manifests.");
    println!("Make sure to register this as an operator in your organization.");

    Ok(operator_key)
}

/// Find an org by alias or by org ID.
fn find_org<'a>(config: &'a Config, org: &str) -> Option<(&'a String, &'a OrgConfig)> {
    // First try by alias
    if let Some((alias, org_config)) = config.orgs.get_key_value(org) {
        return Some((alias, org_config));
    }

    // Then try by org ID
    for (alias, org_config) in &config.orgs {
        if org_config.id == org {
            return Some((alias, org_config));
        }
    }

    None
}

/// Wait for the user to press Enter. Skips the blocking read when
/// `TVC_NON_INTERACTIVE` is set so non-interactive callers don't stall after
/// the dashboard-registration instructions.
fn wait_for_enter(message: &str) -> Result<()> {
    print!("{message}");
    std::io::stdout().flush()?;

    if prompts::non_interactive_forced() {
        println!();
        return Ok(());
    }

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(())
}

/// Result of a successful whoami verification.
pub struct WhoamiResult {
    pub organization_name: String,
    pub organization_id: String,
    pub username: String,
    pub user_id: String,
}

/// Verify credentials by calling the whoami endpoint.
/// Returns Ok(WhoamiResult) if credentials are valid, Err otherwise.
async fn verify_credentials(
    api_key: &StoredApiKey,
    org_id: &str,
    api_base_url: &str,
) -> Result<WhoamiResult> {
    debug!(%api_base_url, "verifying credentials with whoami");

    // Build the API key stamper from stored keys
    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    // Build the client
    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

    // Call whoami
    let request = GetWhoamiRequest {
        organization_id: org_id.to_string(),
    };

    let response = client
        .get_whoami(request)
        .await
        .context("whoami request failed")?;

    debug!("whoami verification succeeded");

    Ok(WhoamiResult {
        organization_name: response.organization_name,
        organization_id: response.organization_id,
        username: response.username,
        user_id: response.user_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    const OVERRIDE_URL: &str = "http://127.0.0.1:8081";

    #[test]
    fn new_org_api_base_url_defaults_to_prod() {
        assert_eq!(new_org_api_base_url(None), API_BASE_URL_PROD);
    }

    #[test]
    fn new_org_api_base_url_uses_override() {
        assert_eq!(new_org_api_base_url(Some(OVERRIDE_URL)), OVERRIDE_URL);
    }

    #[test]
    fn absent_override_preserves_existing_org_api_base_url() {
        let mut config = config_with_org("http://existing.example");

        update_api_base_url_from_override(&mut config, "default", None);

        assert_eq!(
            config.orgs["default"].api_base_url,
            "http://existing.example"
        );
    }

    #[test]
    fn explicit_override_updates_existing_org_api_base_url() {
        let mut config = config_with_org(API_BASE_URL_PROD);

        update_api_base_url_from_override(&mut config, "default", Some(OVERRIDE_URL));

        assert_eq!(config.orgs["default"].api_base_url, OVERRIDE_URL);
    }

    fn config_with_org(api_base_url: &str) -> Config {
        Config {
            active_org: Some("default".to_string()),
            orgs: HashMap::from([(
                "default".to_string(),
                OrgConfig {
                    id: "org-test".to_string(),
                    api_key_path: PathBuf::from("api_key.json"),
                    operator_key_path: PathBuf::from("operator.json"),
                    api_base_url: api_base_url.to_string(),
                },
            )]),
            last_created_app_id: HashMap::new(),
            last_operator_ids: HashMap::new(),
        }
    }
}
