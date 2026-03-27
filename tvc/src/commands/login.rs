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
    pending_key_registration: bool,
}

/// Run the login command.
pub async fn run(args: Args, global: &GlobalOpts) -> anyhow::Result<()> {
    let output = Output::new(global);

    let mut config = Config::load().await?;
    let (alias, org_config) = select_or_create_org(&mut config, &args, global).await?;

    output.status(&format!("Selected org: {} ({})", alias, org_config.id));

    config.set_active_org(&alias)?;
    config.save().await?;

    let (api_key, freshly_generated) =
        get_or_generate_api_key(&org_config, global, &output).await?;

    output.status("");
    output.status("Verifying credentials...");

    let verification = verify_credentials(&api_key, &org_config.id, &org_config.api_base_url).await;

    // If the key was just generated and verification fails, the user still needs to
    // register it in the dashboard. Save config and exit cleanly instead of erroring.
    if freshly_generated && verification.is_err() {
        let operator_key = get_or_generate_operator_key(&org_config, &output).await?;

        let config_path = crate::config::turnkey::config_file_path()?
            .display()
            .to_string();
        let api_key_path = org_config.api_key_path.display().to_string();
        let operator_key_path = org_config.operator_key_path.display().to_string();

        let result = LoginOutput {
            organization_name: String::new(),
            organization_id: org_config.id.clone(),
            username: String::new(),
            user_id: String::new(),
            active_org: alias.clone(),
            api_key_public: api_key.public_key.clone(),
            operator_key_public: operator_key.public_key.clone(),
            config_path: config_path.clone(),
            api_key_path: api_key_path.clone(),
            operator_key_path: operator_key_path.clone(),
            pending_key_registration: true,
        };

        output.result(&result, || {
            println!();
            println!("Key saved. After adding the public key to your dashboard, run `tvc login --org {alias}` to verify.");
            println!();
            println!("Organization ID: {}", org_config.id);
            println!("Active Org: {alias}");
            println!("API Key: {}", api_key.public_key);
            println!("Operator Key: {}", operator_key.public_key);
            println!();
            println!("Config: {config_path}");
            println!("API Key: {api_key_path}");
            println!("Operator Key: {operator_key_path}");
        })?;

        return Ok(());
    }

    let whoami = verification?;
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
        pending_key_registration: false,
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

async fn select_or_create_org(
    config: &mut Config,
    args: &Args,
    global: &GlobalOpts,
) -> Result<(String, OrgConfig)> {
    if let Some(ref org_id) = global.org_id {
        let api_base_url = match args.api_env.as_deref().unwrap_or("prod") {
            "prod" => API_BASE_URL_PROD,
            "preprod" => API_BASE_URL_PREPROD,
            "dev" => API_BASE_URL_DEV,
            "local" => API_BASE_URL_LOCAL,
            _ => unreachable!("clap validates api_env"),
        };
        config.add_org(&args.alias, org_id.clone(), api_base_url.to_string())?;
        let org_config = config.orgs.get(&args.alias).unwrap().clone();
        return Ok((args.alias.clone(), org_config));
    }

    if let Some(ref org) = args.org {
        if let Some((alias, org_config)) = find_org(config, org) {
            return Ok((alias.clone(), org_config.clone()));
        }
        bail!("Organization '{org}' not found. Run `tvc login` without --org to set up a new organization.");
    }

    if global.no_input {
        bail!(
            "No organization specified in non-interactive mode. \
             Use --org <ALIAS> to select an existing org, or \
             --org-id <ID> to create a new org config."
        );
    }

    if config.orgs.is_empty() {
        println!("No organization configured.");
        return prompt_for_new_org(config).await;
    }

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

async fn prompt_for_new_org(config: &mut Config) -> Result<(String, OrgConfig)> {
    println!("You can find your Organization ID at: https://app.turnkey.com/dashboard/welcome");
    println!();

    let org_id = prompt("Organization ID")?;
    if org_id.is_empty() {
        bail!("Organization ID is required");
    }

    let alias = prompt_with_default("Organization alias", "default")?;
    let api_base_url = prompt_for_api_url()?;

    config.add_org(&alias, org_id, api_base_url)?;
    let org_config = config.orgs.get(&alias).unwrap().clone();
    Ok((alias, org_config))
}

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

/// A single entry from the Turnkey dashboard's exported credentials JSON array.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DashboardExportedKey {
    #[allow(dead_code)]
    api_key_name: Option<String>,
    public_key: String,
    curve_type: Option<String>,
    private_key: String,
}

/// Parse an API key file that may be in either the dashboard export format
/// (JSON array with camelCase fields) or the CLI internal format (single JSON
/// object with snake_case fields).
fn parse_api_key_file(content: &str) -> Result<StoredApiKey> {
    // Try CLI internal format first (single object)
    if let Ok(key) = serde_json::from_str::<StoredApiKey>(content) {
        return Ok(key);
    }

    // Try dashboard export format (array of objects)
    let entries: Vec<DashboardExportedKey> = serde_json::from_str(content)
        .context("failed to parse API key file as either CLI format or dashboard export format")?;

    let entry = entries
        .into_iter()
        .next()
        .context("API key file contains an empty array")?;

    let curve = match entry.curve_type.as_deref() {
        Some("API_KEY_CURVE_P256") | None => KeyCurve::P256,
        Some("API_KEY_CURVE_SECP256K1") => KeyCurve::Secp256k1,
        Some(other) => bail!("unsupported curve type: {other}"),
    };

    Ok(StoredApiKey {
        public_key: entry.public_key,
        private_key: entry.private_key,
        curve,
    })
}

/// Returns `(api_key, freshly_generated)` where `freshly_generated` is true
/// when a brand-new keypair was created (the user still needs to register it).
async fn get_or_generate_api_key(
    org_config: &OrgConfig,
    global: &GlobalOpts,
    output: &Output<'_>,
) -> Result<(StoredApiKey, bool)> {
    // If --api-key-file is provided, import it and save to the org's key path
    if let Some(ref api_key_file) = global.api_key_file {
        output.status(&format!(
            "Importing API key from {}...",
            api_key_file.display()
        ));

        let content = tokio::fs::read_to_string(api_key_file)
            .await
            .with_context(|| format!("failed to read API key file: {}", api_key_file.display()))?;

        let api_key = parse_api_key_file(&content)?;
        api_key.save(org_config).await?;
        output.status("API key imported successfully.");
        return Ok((api_key, false));
    }

    if let Some(api_key) = StoredApiKey::load(org_config).await? {
        output.status("Using existing API key.");
        return Ok((api_key, false));
    }

    output.status("");
    output.status("Generating API key...");

    let stamper = TurnkeyP256ApiKey::generate();
    let public_key = hex::encode(stamper.compressed_public_key());
    let private_key = hex::encode(stamper.private_key());

    let api_key = StoredApiKey {
        public_key: public_key.clone(),
        private_key,
        curve: KeyCurve::P256,
    };

    api_key.save(org_config).await?;
    print_api_key_setup_instructions(output, &public_key);

    if !global.no_input {
        wait_for_enter("Press Enter when done...")?;
    }

    Ok((api_key, true))
}

async fn get_or_generate_operator_key(
    org_config: &OrgConfig,
    output: &Output<'_>,
) -> Result<StoredQosOperatorKey> {
    if let Some(operator_key) = StoredQosOperatorKey::load(org_config).await? {
        output.status("Using existing operator key.");
        return Ok(operator_key);
    }

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

fn print_api_key_setup_instructions(output: &Output<'_>, public_key: &str) {
    output.notice("");
    output.notice("API Key Generated!");
    output.notice("");
    output.notice(&format!("Public Key: {public_key}"));
    output.notice("");
    output.notice("Add this API key to your Turnkey dashboard:");
    output.notice("  1. Go to https://app.turnkey.com/dashboard/users");
    output.notice("  2. Click your user > Create API Key > Generate API Keys via CLI > Continue");
    output.notice("  3. Paste the public key > Name it \"TVC CLI\" > Continue > Approve");
    output.notice("");
}

fn find_org<'a>(config: &'a Config, org: &str) -> Option<(&'a String, &'a OrgConfig)> {
    if let Some((alias, org_config)) = config.orgs.get_key_value(org) {
        return Some((alias, org_config));
    }

    for (alias, org_config) in &config.orgs {
        if org_config.id == org {
            return Some((alias, org_config));
        }
    }

    None
}

fn prompt(message: &str) -> Result<String> {
    eprint!("{message}: ");
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

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

fn wait_for_enter(message: &str) -> Result<()> {
    eprint!("{message}");
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
pub struct WhoamiResult {
    pub organization_name: String,
    pub organization_id: String,
    pub username: String,
    pub user_id: String,
}

async fn verify_credentials(
    api_key: &StoredApiKey,
    org_id: &str,
    api_base_url: &str,
) -> Result<WhoamiResult> {
    let stamper = TurnkeyP256ApiKey::from_strings(&api_key.private_key, Some(&api_key.public_key))
        .context("failed to load API key")?;

    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(stamper)
        .base_url(api_base_url)
        .build()
        .context("failed to build Turnkey client")?;

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
