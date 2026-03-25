//! Login command for authenticating with Turnkey.

use crate::cli::GlobalOpts;
use crate::config::turnkey::{
    Config, KeyCurve, OrgConfig, StoredApiKey, StoredQosOperatorKey, API_BASE_URL_DEV,
    API_BASE_URL_LOCAL, API_BASE_URL_PREPROD, API_BASE_URL_PROD,
};
use crate::output::Output;
use anyhow::{anyhow, bail, Context, Result};
use clap::Args as ClapArgs;
use qos_p256::P256Pair;
use serde::Serialize;
use std::io::{BufRead, Write};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::GetWhoamiRequest;

/// Authenticate with Turnkey and set up local credentials.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID to log in with (select existing org).
    /// If not provided, will prompt interactively.
    #[arg(long)]
    pub org: Option<String>,

    /// Alias for the organization config (used with --org-id).
    #[arg(long, env = "TVC_ORG_ALIAS", default_value = "default")]
    pub alias: String,

    /// API environment to use (used with --org-id).
    #[arg(long, env = "TVC_API_ENV", value_parser = ["prod", "preprod", "dev", "local"])]
    pub api_env: Option<String>,

    /// Skip the interactive prompt after API key generation.
    #[arg(long)]
    pub skip_api_key_wait: bool,
}

#[derive(Serialize)]
struct LoginOutput {
    organization_name: String,
    organization_id: String,
    username: String,
    user_id: String,
    active_org: String,
    api_key_public: String,
    operator_key_public: String,
    config_path: String,
    api_key_path: String,
    operator_key_path: String,
}

/// Run the login command.
pub async fn run(args: Args, global: &GlobalOpts) -> anyhow::Result<()> {
    let output = Output::new(global);

    // Load existing config
    let mut config = Config::load().await?;

    // Select or create org
    let (alias, org_config) = select_or_create_org(&mut config, &args, global).await?;

    output.status(&format!("Selected org: {} ({})", alias, org_config.id));

    // Save config with the new/updated org
    config.set_active_org(&alias)?;
    config.save().await?;

    // Get or generate API key
    let api_key = get_or_generate_api_key(&org_config, &args, global).await?;

    // Verify credentials with whoami
    output.status("");
    output.status("Verifying credentials...");

    let whoami = verify_credentials(&api_key, &org_config.id, &org_config.api_base_url).await?;

    // Get or generate operator key
    let operator_key = get_or_generate_operator_key(&org_config, &output).await?;

    let config_path = crate::config::turnkey::config_file_path()?
        .display()
        .to_string();
    let api_key_path = org_config.api_key_path.display().to_string();
    let operator_key_path = org_config.operator_key_path.display().to_string();

    let result = LoginOutput {
        organization_name: whoami.organization_name.clone(),
        organization_id: whoami.organization_id.clone(),
        username: whoami.username.clone(),
        user_id: whoami.user_id.clone(),
        active_org: alias.clone(),
        api_key_public: api_key.public_key.clone(),
        operator_key_public: operator_key.public_key.clone(),
        config_path: config_path.clone(),
        api_key_path: api_key_path.clone(),
        operator_key_path: operator_key_path.clone(),
    };

    output.result(&result, || {
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
        println!("Config: {config_path}");
        println!("API Key: {api_key_path}");
        println!("Operator Key: {operator_key_path}");
    })?;

    Ok(())
}

/// Select an existing org or create a new one.
/// Returns the alias and a clone of the org config.
async fn select_or_create_org(
    config: &mut Config,
    args: &Args,
    global: &GlobalOpts,
) -> Result<(String, OrgConfig)> {
    // If --org-id provided (via global flag), create/update org non-interactively
    if let Some(ref org_id) = global.org_id {
        let api_base_url = resolve_api_env(args.api_env.as_deref())?;
        config.add_org(&args.alias, org_id.clone(), api_base_url)?;
        let org_config = config.orgs.get(&args.alias).unwrap().clone();
        return Ok((args.alias.clone(), org_config));
    }

    // If --org provided, try to find it by alias or ID
    if let Some(ref org) = args.org {
        if let Some((alias, org_config)) = find_org(config, org) {
            return Ok((alias.clone(), org_config.clone()));
        }
        bail!("Organization '{org}' not found. Run `tvc login` without --org to set up a new organization.");
    }

    // Non-interactive mode requires --org or --org-id
    if global.no_input {
        bail!(
            "No organization specified in non-interactive mode. \
             Use --org <ALIAS> to select an existing org, or \
             --org-id <ID> to create a new org config."
        );
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

/// Resolve an API environment name to a URL.
fn resolve_api_env(api_env: Option<&str>) -> Result<String> {
    let url = match api_env.unwrap_or("prod") {
        "prod" => API_BASE_URL_PROD,
        "preprod" => API_BASE_URL_PREPROD,
        "dev" => API_BASE_URL_DEV,
        "local" => API_BASE_URL_LOCAL,
        other => bail!("Invalid API environment: {other}"),
    };
    Ok(url.to_string())
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
    println!("  3. dev            - {API_BASE_URL_DEV}");
    println!("  4. local          - {API_BASE_URL_LOCAL}");
    println!();

    let selection = prompt_with_default("API URL [1/2/3/4]", "1")?;

    let url = match selection.as_str() {
        "1" | "prod" | "" => API_BASE_URL_PROD,
        "2" | "preprod" => API_BASE_URL_PREPROD,
        "3" | "dev" => API_BASE_URL_DEV,
        "4" | "local" => API_BASE_URL_LOCAL,
        _ => bail!("Invalid selection: {selection}"),
    };

    Ok(url.to_string())
}

/// Get an existing API key or generate a new one.
/// If an API key exists, it's returned directly.
/// If not, a new key is generated, saved, and the user is prompted to add it to the dashboard.
async fn get_or_generate_api_key(
    org_config: &OrgConfig,
    args: &Args,
    global: &GlobalOpts,
) -> Result<StoredApiKey> {
    // Check if API key already exists
    if let Some(api_key) = StoredApiKey::load(org_config).await? {
        eprintln!("Using existing API key.");
        return Ok(api_key);
    }

    // Generate new API key
    eprintln!();
    eprintln!("Generating API key...");

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
    eprintln!();
    eprintln!("API Key Generated!");
    eprintln!();
    eprintln!("Public Key: {public_key}");
    eprintln!();
    eprintln!("Add this API key to your Turnkey dashboard:");
    eprintln!("  1. Go to https://app.turnkey.com/dashboard/users");
    eprintln!("  2. Click your user > Create API Key > Generate API Keys via CLI > Continue");
    eprintln!("  3. Paste the public key > Name it \"TVC CLI\" > Continue > Approve");
    eprintln!();

    // Skip wait in non-interactive mode or with --skip-api-key-wait
    if !global.no_input && !args.skip_api_key_wait {
        wait_for_enter("Press Enter when done...")?;
    }

    Ok(api_key)
}

/// Get an existing operator key or generate a new one.
async fn get_or_generate_operator_key(
    org_config: &OrgConfig,
    output: &Output<'_>,
) -> Result<StoredQosOperatorKey> {
    // Check if operator key already exists
    if let Some(operator_key) = StoredQosOperatorKey::load(org_config).await? {
        output.status("Using existing operator key.");
        return Ok(operator_key);
    }

    // Generate new operator key
    output.status("");
    output.status("Generating operator key...");

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

    output.status("");
    output.status("Operator Key Generated!");
    output.status("");
    output.status(&format!("Public Key: {public_key}"));
    output.status("");
    output.status("This key will be used for approving deployment manifests.");
    output.status("Make sure to register this as an operator in your organization.");

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
    eprint!("{message}: ");
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt the user for input with a default value.
/// If the user enters nothing, returns the default.
fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    eprint!("{message} [{default}]: ");
    std::io::stderr().flush()?;

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
    eprint!("{message}");
    std::io::stderr().flush()?;

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
