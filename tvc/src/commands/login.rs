//! Login command for authenticating with Turnkey.

use crate::config::turnkey::{
    API_BASE_URL_PROD, Config, KeyCurve, OrgConfig, StoredApiKey, StoredQosOperatorKey,
    dashboard_base_url,
};
use crate::prompts::{self, error_required_in_non_interactive};
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

enum OrgPlan {
    Existing(String),
    New { id: String, alias: String },
}

enum ApiKeyPolicy {
    AllowGenerate,
    RequireExisting,
}

struct LoginPlan {
    org: OrgPlan,
    api_base_url_override: Option<String>,
    api_key_policy: ApiKeyPolicy,
}

pub async fn run(args: Args, is_non_interactive: bool) -> Result<()> {
    debug!(
        non_interactive = is_non_interactive,
        org_arg_present = args.org.is_some(),
        api_base_url_override_present = args.api_base_url.is_some(),
        "running login command"
    );

    let config = Config::load().await?;

    let plan = if is_non_interactive {
        build_login_plan_non_interactive(args)?
    } else {
        build_login_plan_interactive(args, &config)?
    };

    execute_login(config, plan).await
}

fn build_login_plan_interactive(args: Args, config: &Config) -> Result<LoginPlan> {
    let org = match args.org {
        Some(query) => OrgPlan::Existing(query),
        None => prompt_for_org_plan(config, args.api_base_url.as_deref())?,
    };
    Ok(LoginPlan {
        org,
        api_base_url_override: args.api_base_url,
        api_key_policy: ApiKeyPolicy::AllowGenerate,
    })
}

fn build_login_plan_non_interactive(args: Args) -> Result<LoginPlan> {
    let Some(org_query) = args.org else {
        return Err(error_required_in_non_interactive("--org"));
    };

    Ok(LoginPlan {
        org: OrgPlan::Existing(org_query),
        api_base_url_override: args.api_base_url,
        api_key_policy: ApiKeyPolicy::RequireExisting,
    })
}

async fn execute_login(mut config: Config, plan: LoginPlan) -> Result<()> {
    let (alias, org_config) = match plan.org {
        OrgPlan::Existing(query) => {
            let alias = match find_org(&config, &query) {
                Some((alias, _)) => alias.clone(),
                None => bail!(
                    "Organization '{query}' not found. \
                     Run `tvc login` without --org to set up a new organization."
                ),
            };
            update_api_base_url_from_override(
                &mut config,
                &alias,
                plan.api_base_url_override.as_deref(),
            );
            let org_config = config.orgs.get(&alias).unwrap().clone();
            (alias, org_config)
        }
        OrgPlan::New { id, alias } => {
            let api_base_url = new_org_api_base_url(plan.api_base_url_override.as_deref());
            generate_org(&mut config, id, alias, api_base_url)?
        }
    };

    println!("Selected org: {} ({})", alias, org_config.id);

    config.set_active_org(&alias)?;
    config.save().await?;

    let api_key = match StoredApiKey::load(&org_config).await? {
        Some(api_key) => {
            debug!("using existing API key");
            println!("Using existing API key.");
            api_key
        }
        None => match plan.api_key_policy {
            ApiKeyPolicy::AllowGenerate => {
                let api_key = generate_api_key(&org_config).await?;
                wait_for_dashboard_registration()?;
                api_key
            }
            ApiKeyPolicy::RequireExisting => bail!(
                "API key is required in non-interactive mode for org '{}'. \
                 Run `tvc login` interactively to generate and register one first.",
                org_config.id
            ),
        },
    };

    println!();
    println!("Verifying credentials...");

    let whoami = verify_credentials(&api_key, &org_config.id, &org_config.api_base_url).await?;
    let operator_key = find_or_generate_operator_key(&org_config).await?;

    print_success(&alias, &org_config, &api_key, &operator_key, &whoami)
}

fn prompt_for_org_plan(config: &Config, api_base_url_override: Option<&str>) -> Result<OrgPlan> {
    debug!(
        configured_org_count = config.orgs.len(),
        active_org = ?config.active_org,
        "prompting for organization plan"
    );

    if config.orgs.is_empty() {
        debug!("no organizations configured; prompting for new organization");
        println!("No organization configured.");
        return prompt_for_new_org_inputs(api_base_url_override);
    }

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
        OrgChoice::Existing { alias, .. } => Ok(OrgPlan::Existing(alias)),
        OrgChoice::New => prompt_for_new_org_inputs(api_base_url_override),
    }
}

fn prompt_for_new_org_inputs(api_base_url_override: Option<&str>) -> Result<OrgPlan> {
    let dashboard_url = dashboard_base_url(api_base_url_override.unwrap_or(API_BASE_URL_PROD));
    println!("You can find your Organization ID at: {dashboard_url}/dashboard/welcome");
    println!();

    let id = prompts::text("Organization ID", None)?;
    if id.is_empty() {
        bail!("Organization ID is required");
    }

    let alias = prompts::text("Organization alias", Some("default"))?;
    debug!(org_alias = %alias, "user entered new organization inputs");

    Ok(OrgPlan::New { id, alias })
}

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

fn generate_org(
    config: &mut Config,
    id: String,
    alias: String,
    api_base_url: String,
) -> Result<(String, OrgConfig)> {
    debug!(org_alias = %alias, %api_base_url, "adding organization");
    config.add_org(&alias, id, api_base_url)?;
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

async fn generate_api_key(org_config: &OrgConfig) -> Result<StoredApiKey> {
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

    api_key.save(org_config).await?;

    println!();
    println!("API Key Generated!");
    println!();
    println!("Public Key: {public_key}");
    println!();
    let dashboard_url = dashboard_base_url(&org_config.api_base_url);
    println!("Add this API key to your Turnkey dashboard:");
    println!("  1. Go to {dashboard_url}/dashboard/v2/users and click your user");
    println!(
        "  2. Click \"New API Key\", expand \"Advanced Settings\", then check \"Generate API key via CLI\""
    );
    println!("  3. Name it \"TVC CLI\", paste the public key above, then Continue > Approve");
    println!();

    Ok(api_key)
}

fn wait_for_dashboard_registration() -> Result<()> {
    print!("Press Enter when done...");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(())
}

async fn find_or_generate_operator_key(org_config: &OrgConfig) -> Result<StoredQosOperatorKey> {
    debug!(operator_key_path = %org_config.operator_key_path.display(), "resolving operator key");

    if let Some(operator_key) = StoredQosOperatorKey::load(org_config).await? {
        debug!("using existing operator key");
        println!("Using existing operator key.");
        return Ok(operator_key);
    }

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
    debug!(%api_base_url, "verifying credentials with whoami");

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

    debug!("whoami verification succeeded");

    Ok(WhoamiResult {
        organization_name: response.organization_name,
        organization_id: response.organization_id,
        username: response.username,
        user_id: response.user_id,
    })
}

fn print_success(
    alias: &str,
    org_config: &OrgConfig,
    api_key: &StoredApiKey,
    operator_key: &StoredQosOperatorKey,
    whoami: &WhoamiResult,
) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        API_BASE_URL_DEV, API_BASE_URL_PREPROD, DASHBOARD_URL_DEV, DASHBOARD_URL_PREPROD,
        DASHBOARD_URL_PROD,
    };
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
    fn dashboard_url_matches_selected_environment() {
        assert_eq!(dashboard_base_url(API_BASE_URL_PROD), DASHBOARD_URL_PROD);
        assert_eq!(
            dashboard_base_url(API_BASE_URL_PREPROD),
            DASHBOARD_URL_PREPROD
        );
        assert_eq!(dashboard_base_url(API_BASE_URL_DEV), DASHBOARD_URL_DEV);
    }

    #[test]
    fn dashboard_url_falls_back_to_prod_for_unknown_hosts() {
        // Local and other unrecognized hosts fall back to the prod dashboard.
        assert_eq!(dashboard_base_url(OVERRIDE_URL), DASHBOARD_URL_PROD);
        assert_eq!(
            dashboard_base_url("https://api.staging.turnkey.engineering"),
            DASHBOARD_URL_PROD
        );
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
