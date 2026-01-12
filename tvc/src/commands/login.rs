//! Login command for authenticating with Turnkey.

use crate::config::turnkey::{
    ApiKey, Config, OperatorKey, OrgConfig, API_BASE_URL_LOCAL, API_BASE_URL_PREPROD,
    API_BASE_URL_PROD,
};
use anyhow::{anyhow, bail, Context, Result};
use clap::Args as ClapArgs;
use qos_p256::P256Pair;
use std::io::{BufRead, Write};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::GetWhoamiRequest;

/// Authenticate with Turnkey and set up local credentials.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID to log in with.
    /// If not provided, will prompt interactively.
    #[arg(long)]
    pub org: Option<String>,
}

/// Run the login command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    // Load existing config
    let mut config = Config::load().await?;

    // Select or create org
    let (alias, org_config) = select_or_create_org(&mut config, args.org.as_deref()).await?;

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
) -> Result<(String, OrgConfig)> {
    // If --org provided, try to find it by alias or ID
    if let Some(org) = org_arg {
        if let Some((alias, org_config)) = find_org(config, org) {
            return Ok((alias.clone(), org_config.clone()));
        }
        bail!("Organization '{org}' not found. Run `tvc login` without --org to set up a new organization.");
    }

    // No --org provided, check existing orgs
    let org_count = config.orgs.len();

    if org_count == 0 {
        // No orgs configured - prompt for new org
        println!("No organization configured.");
        return prompt_for_new_org(config).await;
    }

    // Show existing orgs and let user select or add new
    println!("Organization choices:");
    for (alias, org) in &config.orgs {
        let active = if config.active_org.as_ref() == Some(alias) {
            " (active)"
        } else {
            ""
        };
        println!("  - {} ({}){}", alias, org.id, active);
    }
    println!("  - [new] Add a new organization");
    println!();

    let selection = prompt("Enter organization alias or 'new'")?;
    println!();

    if selection == "new" {
        return prompt_for_new_org(config).await;
    }

    if let Some(org_config) = config.orgs.get(&selection) {
        return Ok((selection, org_config.clone()));
    }

    bail!("Organization '{}' not found", selection)
}

/// Prompt the user to enter a new organization ID and alias.
async fn prompt_for_new_org(config: &mut Config) -> Result<(String, OrgConfig)> {
    println!("You can find your Organization ID at: https://app.turnkey.com/dashboard/welcome");
    println!();

    let org_id = prompt("Organization ID")?;
    if org_id.is_empty() {
        bail!("Organization ID is required");
    }

    let alias = prompt_with_default("Organization alias", "default")?;

    // Prompt for API base URL
    let api_base_url = prompt_for_api_url()?;

    config.add_org(&alias, org_id, api_base_url)?;
    let org_config = config.orgs.get(&alias).unwrap().clone();
    Ok((alias, org_config))
}

/// Prompt the user to select a Turnkey API URL.
fn prompt_for_api_url() -> Result<String> {
    println!();
    println!("Select Turnkey API URL:");
    println!("  1. prod (default) - {API_BASE_URL_PROD}");
    println!("  2. preprod        - {API_BASE_URL_PREPROD}");
    println!("  3. local          - {API_BASE_URL_LOCAL}");
    println!();

    let selection = prompt_with_default("API URL [1/2/3]", "1")?;

    let url = match selection.as_str() {
        "1" | "prod" | "" => API_BASE_URL_PROD,
        "2" | "preprod" => API_BASE_URL_PREPROD,
        "3" | "local" => API_BASE_URL_LOCAL,
        _ => bail!("Invalid selection: {selection}"),
    };

    Ok(url.to_string())
}

/// Get an existing API key or generate a new one.
/// If an API key exists, it's returned directly.
/// If not, a new key is generated, saved, and the user is prompted to add it to the dashboard.
async fn get_or_generate_api_key(org_config: &OrgConfig) -> Result<ApiKey> {
    // Check if API key already exists
    if let Some(api_key) = ApiKey::load(org_config).await? {
        println!("Using existing API key.");
        return Ok(api_key);
    }

    // Generate new API key
    println!();
    println!("Generating API key...");

    let stamper = TurnkeyP256ApiKey::generate();
    let public_key = hex::encode(stamper.compressed_public_key());
    let private_key = hex::encode(stamper.private_key());

    let api_key = ApiKey {
        public_key: public_key.clone(),
        private_key,
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
async fn get_or_generate_operator_key(org_config: &OrgConfig) -> Result<OperatorKey> {
    // Check if operator key already exists
    if let Some(operator_key) = OperatorKey::load(org_config).await? {
        println!("Using existing operator key.");
        return Ok(operator_key);
    }

    // Generate new operator key
    println!();
    println!("Generating operator key...");

    let pair =
        P256Pair::generate().map_err(|e| anyhow!("failed to generate operator key: {e:?}"))?;
    let public_key = hex::encode(pair.public_key().to_bytes());
    let private_key = hex::encode(pair.to_master_seed());

    let operator_key = OperatorKey {
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

/// Prompt the user for input and return the trimmed response.
fn prompt(message: &str) -> Result<String> {
    print!("{message}: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt the user for input with a default value.
/// If the user enters nothing, returns the default.
fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    print!("{message} [{default}]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

/// Wait for the user to press Enter.
fn wait_for_enter(message: &str) -> Result<()> {
    print!("{message}");
    std::io::stdout().flush()?;

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
    api_key: &ApiKey,
    org_id: &str,
    api_base_url: &str,
) -> Result<WhoamiResult> {
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

    Ok(WhoamiResult {
        organization_name: response.organization_name,
        organization_id: response.organization_id,
        username: response.username,
        user_id: response.user_id,
    })
}
