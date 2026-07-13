//! Login command for authenticating with Turnkey.

use crate::config::turnkey::{
    API_BASE_URL_PROD, Config, KeyCurve, OrgConfig, StoredApiKey, StoredQosOperatorKey,
    dashboard_base_url, default_api_key_path, default_operator_key_path, default_org_dir,
};
use crate::output::Ctx;
use crate::prompts::{self, error_required_in_non_interactive};
use crate::{shell_eprintln, shell_print, shell_println};
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

/// Permanently delete a saved login profile, including its API and operator
/// key files on disk.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct DeleteArgs {
    /// Organization alias or ID of the profile to delete.
    /// If not provided, will prompt interactively.
    #[arg(short, long, value_name = "ORG")]
    pub org: Option<String>,
    /// Skip the confirmation prompt (required to delete in non-interactive mode).
    #[arg(short, long)]
    pub yes: bool,
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

pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> Result<()> {
    debug!(
        non_interactive = ctx.is_non_interactive(),
        org_arg_present = args.org.is_some(),
        api_base_url_override_present = args.api_base_url.is_some(),
        "running login command"
    );

    let config = Config::load().await?;

    let plan = if ctx.is_non_interactive() {
        build_login_plan_non_interactive(args)?
    } else {
        build_login_plan_interactive(ctx, args, &config)?
    };

    execute_login(ctx, config, plan).await
}

/// Permanently delete a saved login profile: its config entry and its API and
/// operator key files on disk.
pub async fn run_delete<W: Write>(ctx: &mut Ctx<W>, args: DeleteArgs) -> Result<()> {
    let is_non_interactive = ctx.is_non_interactive();
    debug!(
        non_interactive = is_non_interactive,
        org_arg_present = args.org.is_some(),
        skip_confirm = args.yes,
        "running login delete command"
    );

    // Validate inputs before any business logic: non-interactive mode cannot
    // prompt, so it requires --org (which profile) and --yes (confirmation).
    if is_non_interactive {
        if args.org.is_none() {
            return Err(error_required_in_non_interactive("--org"));
        }
        if !args.yes {
            return Err(error_required_in_non_interactive("--yes"));
        }
    }

    let mut config = Config::load().await?;
    let alias = resolve_profile_alias(&config, args.org)?;
    let org_id = config
        .orgs
        .get(&alias)
        .map(|org| org.id.clone())
        .unwrap_or_default();

    // Interactive confirmation. A non-interactive run without --yes was rejected
    // up front, so reaching here with !args.yes means we can prompt.
    if !args.yes {
        let dashboard_url = config
            .orgs
            .get(&alias)
            .map(|org| dashboard_base_url(&org.api_base_url))
            .unwrap_or_default();
        shell_eprintln!(ctx, "")?;
        shell_eprintln!(
            ctx,
            "WARNING: This permanently deletes login profile '{alias}' ({org_id})."
        )?;
        shell_eprintln!(
            ctx,
            "  - Removes the local config entry and deletes the API and operator key"
        )?;
        shell_eprintln!(ctx, "    files from disk. This cannot be undone.")?;
        shell_eprintln!(
            ctx,
            "  - It does NOT touch the Turnkey dashboard ({dashboard_url}). If this API"
        )?;
        shell_eprintln!(
            ctx,
            "    key is registered there, it stays valid until you remove it"
        )?;
        shell_eprintln!(ctx, "    (instructions are printed after deletion).")?;
        shell_eprintln!(ctx, "")?;
        prompts::confirm_or_bail(
            &format!("Permanently delete profile '{alias}' ({org_id}) and its key files?"),
            "deletion",
        )?;
    }

    let removed = config
        .remove_org(&alias)
        .expect("alias was resolved from config");

    // Read the API key's public key before deleting its file, so the
    // dashboard-revocation reminder below can name exactly which key to remove.
    // Best-effort: a missing or unreadable key file just omits the value.
    let api_public_key = StoredApiKey::load(&removed)
        .await
        .ok()
        .flatten()
        .map(|key| key.public_key);

    // The default layout stores both key files in the per-org directory, so a
    // default profile is removed by deleting that whole directory. Custom
    // (hand-edited) key paths are left untouched with a warning, since the user
    // placed them deliberately and they may live outside our config tree.
    let uses_default_layout = removed.api_key_path == default_api_key_path(&alias)?
        && removed.operator_key_path == default_operator_key_path(&alias)?;

    let removed_dir = if uses_default_layout {
        let dir = default_org_dir(&alias)?;
        match tokio::fs::remove_dir_all(&dir).await {
            Ok(()) => Some(dir),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                shell_eprintln!(
                    ctx,
                    "WARNING: key directory was not on disk: {}",
                    dir.display()
                )?;
                None
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("failed to delete key directory: {}", dir.display()));
            }
        }
    } else {
        shell_eprintln!(
            ctx,
            "WARNING: custom key paths are configured and were NOT deleted."
        )?;
        shell_eprintln!(ctx, "Remove them manually if no longer needed:")?;
        shell_eprintln!(ctx, "  {}", removed.api_key_path.display())?;
        shell_eprintln!(ctx, "  {}", removed.operator_key_path.display())?;
        None
    };

    // Save last: persist the config removal only after the on-disk cleanup above
    // succeeds, so a failure leaves the profile listed and the delete retryable.
    config.save().await?;

    shell_println!(ctx, "Deleted login profile '{alias}' ({}).", removed.id)?;
    if let Some(dir) = removed_dir {
        shell_println!(ctx, "Removed key directory: {}", dir.display())?;
    }

    // A local delete does not touch the dashboard-registered API key, and we
    // can't tell whether it is still there, so hedge with "may" and give steps.
    let dashboard_url = dashboard_base_url(&removed.api_base_url);
    shell_println!(ctx)?;
    shell_println!(
        ctx,
        "IMPORTANT: The API key may still be registered on the Turnkey dashboard."
    )?;
    shell_println!(
        ctx,
        "It will remain valid until it is manually removed. To remove it:"
    )?;
    shell_println!(
        ctx,
        "  1. Go to {dashboard_url}/dashboard/v2/users and click your user"
    )?;
    match api_public_key {
        Some(public_key) => {
            shell_println!(ctx, "  2. Delete the API key with public key:")?;
            shell_println!(ctx, "       {public_key}")?;
        }
        None => shell_println!(ctx, "  2. Delete the API key associated with this profile")?,
    }

    Ok(())
}

/// Resolve the alias of a configured profile to delete. Prompts interactively
/// with a picker when no query is given; a query that matches nothing is handled
/// by `find_org` returning `None`.
fn resolve_profile_alias(config: &Config, org: Option<String>) -> Result<String> {
    match org {
        Some(query) => match find_org(config, &query) {
            Some((alias, _)) => Ok(alias.clone()),
            None => bail!(
                "Login profile '{query}' not found. \
                 Run `tvc login` to see configured profiles."
            ),
        },
        None => {
            // Reached only in interactive mode; a non-interactive run without
            // --org is rejected up front in `run_delete` before we get here.
            if config.orgs.is_empty() {
                bail!("No login profiles to delete.");
            }
            let choices: Vec<_> = config
                .orgs
                .iter()
                .map(|(alias, org)| ProfileChoice {
                    alias: alias.as_str(),
                    org_id: org.id.as_str(),
                    is_active: config.active_org.as_deref() == Some(alias.as_str()),
                })
                .collect();
            Ok(prompts::select("Select profile to delete", choices)?
                .alias
                .to_string())
        }
    }
}

struct ProfileChoice<'a> {
    alias: &'a str,
    org_id: &'a str,
    is_active: bool,
}

impl std::fmt::Display for ProfileChoice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suffix = if self.is_active { " (active)" } else { "" };
        write!(f, "{} ({}){suffix}", self.alias, self.org_id)
    }
}

fn build_login_plan_interactive<W: Write>(
    ctx: &mut Ctx<W>,
    args: Args,
    config: &Config,
) -> Result<LoginPlan> {
    let org = match args.org {
        Some(query) => OrgPlan::Existing(query),
        None => prompt_for_org_plan(ctx, config, args.api_base_url.as_deref())?,
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

async fn execute_login<W: Write>(
    ctx: &mut Ctx<W>,
    mut config: Config,
    plan: LoginPlan,
) -> Result<()> {
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

    shell_println!(ctx, "Selected org: {} ({})", alias, org_config.id)?;

    config.set_active_org(&alias)?;
    config.save().await?;

    let api_key = match StoredApiKey::load(&org_config).await? {
        Some(api_key) => {
            debug!("using existing API key");
            shell_println!(ctx, "Using existing API key.")?;
            api_key
        }
        None => match plan.api_key_policy {
            ApiKeyPolicy::AllowGenerate => {
                let api_key = generate_api_key(ctx, &org_config).await?;
                wait_for_dashboard_registration(ctx)?;
                api_key
            }
            ApiKeyPolicy::RequireExisting => bail!(
                "API key is required in non-interactive mode for org '{}'. \
                 Run `tvc login` interactively to generate and register one first.",
                org_config.id
            ),
        },
    };

    shell_println!(ctx)?;
    shell_println!(ctx, "Verifying credentials...")?;

    let whoami = verify_credentials(&api_key, &org_config.id, &org_config.api_base_url).await?;
    let operator_key = find_or_generate_operator_key(ctx, &org_config).await?;

    print_success(ctx, &alias, &org_config, &api_key, &operator_key, &whoami)
}

fn prompt_for_org_plan<W: Write>(
    ctx: &mut Ctx<W>,
    config: &Config,
    api_base_url_override: Option<&str>,
) -> Result<OrgPlan> {
    debug!(
        configured_org_count = config.orgs.len(),
        active_org = ?config.active_org,
        "prompting for organization plan"
    );

    if config.orgs.is_empty() {
        debug!("no organizations configured; prompting for new organization");
        shell_println!(ctx, "No organization configured.")?;
        return prompt_for_new_org_inputs(ctx, api_base_url_override);
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
        OrgChoice::New => prompt_for_new_org_inputs(ctx, api_base_url_override),
    }
}

fn prompt_for_new_org_inputs<W: Write>(
    ctx: &mut Ctx<W>,
    api_base_url_override: Option<&str>,
) -> Result<OrgPlan> {
    let dashboard_url = dashboard_base_url(api_base_url_override.unwrap_or(API_BASE_URL_PROD));
    shell_println!(
        ctx,
        "You can find your Organization ID at: {dashboard_url}/dashboard/welcome"
    )?;
    shell_println!(ctx)?;

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

async fn generate_api_key<W: Write>(
    ctx: &mut Ctx<W>,
    org_config: &OrgConfig,
) -> Result<StoredApiKey> {
    debug!("generating new API key");
    shell_println!(ctx)?;
    shell_println!(ctx, "Generating API key...")?;

    let stamper = TurnkeyP256ApiKey::generate();
    let public_key = hex::encode(stamper.compressed_public_key());
    let private_key = hex::encode(stamper.private_key());

    let api_key = StoredApiKey {
        public_key: public_key.clone(),
        private_key,
        curve: KeyCurve::P256,
    };

    api_key.save(org_config).await?;

    shell_println!(ctx)?;
    shell_println!(ctx, "API Key Generated!")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "API public key: {public_key}")?;
    shell_println!(ctx)?;
    let dashboard_url = dashboard_base_url(&org_config.api_base_url);
    shell_println!(ctx, "Add this API key to your Turnkey dashboard:")?;
    shell_println!(
        ctx,
        "  1. Go to {dashboard_url}/dashboard/v2/users and click your user"
    )?;
    shell_println!(
        ctx,
        "  2. Click \"New API Key\", expand \"Advanced Settings\", then check \"Generate API key via CLI\""
    )?;
    shell_println!(
        ctx,
        "  3. Name it \"TVC CLI\", paste the public key above, then Continue > Approve"
    )?;
    shell_println!(ctx)?;

    Ok(api_key)
}

fn wait_for_dashboard_registration<W: Write>(ctx: &mut Ctx<W>) -> Result<()> {
    shell_print!(ctx, "Press Enter when done...")?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(())
}

async fn find_or_generate_operator_key<W: Write>(
    ctx: &mut Ctx<W>,
    org_config: &OrgConfig,
) -> Result<StoredQosOperatorKey> {
    debug!(operator_key_path = %org_config.operator_key_path.display(), "resolving operator key");

    if let Some(operator_key) = StoredQosOperatorKey::load(org_config).await? {
        debug!("using existing operator key");
        shell_println!(ctx, "Using existing operator key.")?;
        return Ok(operator_key);
    }

    debug!("generating new operator key");
    shell_println!(ctx)?;
    shell_println!(ctx, "Generating operator key...")?;

    let pair =
        P256Pair::generate().map_err(|e| anyhow!("failed to generate operator key: {e:?}"))?;
    let public_key = hex::encode(pair.public_key().to_bytes());
    let private_key = hex::encode(pair.to_master_seed());

    let operator_key = StoredQosOperatorKey {
        public_key: public_key.clone(),
        private_key,
    };

    operator_key.save(org_config).await?;

    shell_println!(ctx)?;
    shell_println!(ctx, "Operator Key Generated!")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Operator public key: {public_key}")?;
    shell_println!(ctx)?;
    shell_println!(
        ctx,
        "This key will be used for approving deployment manifests."
    )?;
    shell_println!(
        ctx,
        "Make sure to register this as an operator in your organization."
    )?;

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

fn print_success<W: Write>(
    ctx: &mut Ctx<W>,
    alias: &str,
    org_config: &OrgConfig,
    api_key: &StoredApiKey,
    operator_key: &StoredQosOperatorKey,
    whoami: &WhoamiResult,
) -> Result<()> {
    shell_println!(ctx)?;
    shell_println!(ctx, "Successfully logged in!")?;
    shell_println!(ctx)?;
    shell_println!(
        ctx,
        "Organization: {} ({})",
        whoami.organization_name,
        whoami.organization_id
    )?;
    shell_println!(ctx, "User: {} ({})", whoami.username, whoami.user_id)?;
    shell_println!(ctx, "Active Org: {alias}")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Credentials")?;
    shell_println!(ctx, "  API public key:        {}", api_key.public_key)?;
    shell_println!(ctx, "  Operator public key:   {}", operator_key.public_key)?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Saved to")?;
    shell_println!(
        ctx,
        "  Config file:    {}",
        crate::config::turnkey::config_file_path()?.display()
    )?;
    shell_println!(
        ctx,
        "  API key:        {}",
        org_config.api_key_path.display()
    )?;
    shell_println!(
        ctx,
        "  Operator key:   {}",
        org_config.operator_key_path.display()
    )?;

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
