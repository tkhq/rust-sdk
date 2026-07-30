//! Login command for authenticating with Turnkey.

use crate::client::build_turnkey_client;
use crate::config::turnkey::{
    API_BASE_URL_PROD, Config, KeyCurve, OperatorRecordKind, OrgConfig, StoredApiKey,
    StoredQosOperatorKey, dashboard_base_url, default_api_key_path, default_operator_key_path,
    default_org_dir,
};
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::prompts::{self, error_required_in_non_interactive};
use crate::{shell_eprintln, shell_print, shell_println};
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use qos_p256::P256Pair;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::io::BufRead;
use tracing::{debug, instrument};
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

/// Mark a profile as the default alias for its organization while duplicate
/// profiles exist.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct SetDefaultAliasArgs {
    /// Profile alias to mark as its organization's default.
    #[arg(short, long, value_name = "ALIAS")]
    pub org: String,
}

enum OrgPlan {
    /// A resolved alias of a configured profile. The plan builders validate
    /// the user's query (alias or organization ID) before constructing this,
    /// so it is never a raw query.
    Existing(String),
    New {
        id: String,
        alias: String,
    },
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

#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    debug!(
        non_interactive = ctx.is_non_interactive(),
        org_arg_present = args.org.is_some(),
        api_base_url_override_present = args.api_base_url.is_some(),
        "running login command"
    );

    let mut config = Config::load().await?;

    let plan = if ctx.is_non_interactive() {
        build_login_plan_non_interactive(ctx, args, &config)?
    } else {
        // Consolidate before resolving the org query, so the builders never
        // resolve against profiles that are about to be deleted.
        let duplicates = config.duplicated_org_ids();
        consolidate_duplicate_profiles(ctx, &mut config, duplicates).await?;
        build_login_plan_interactive(ctx, args, &config)?
    };

    execute_login(ctx, config, plan).await
}

/// Fold duplicate profiles down to one per organization before login
/// proceeds: prompt for the profile to keep per duplicated organization,
/// confirm once, then delete the others with full profile-delete cleanup.
/// Nothing is mutated before the confirmation, so declining leaves the config
/// and disk untouched.
async fn consolidate_duplicate_profiles(
    ctx: &mut StdCtx,
    config: &mut Config,
    duplicates: Vec<(String, Vec<String>)>,
) -> Result<()> {
    if duplicates.is_empty() {
        return Ok(());
    }

    shell_eprintln!(
        ctx,
        "Multiple profiles are configured for the same organization. tvc keeps \
         one profile per organization, so the extra profiles must be deleted \
         before login can continue."
    )?;
    shell_eprintln!(ctx, "")?;

    let mut losers: Vec<String> = Vec::new();
    let mut active_repair: Option<String> = None;

    for (org_id, aliases) in &duplicates {
        let keeper = prompts::select(
            &format!("Select the profile to keep for organization '{org_id}'"),
            duplicate_alias_choices(config, aliases),
        )?
        .alias
        .to_string();

        if let Some(active) = &config.active_org
            && aliases.contains(active)
            && *active != keeper
        {
            active_repair = Some(keeper.clone());
        }

        losers.extend(aliases.iter().filter(|alias| **alias != keeper).cloned());
    }

    let listed = losers
        .iter()
        .map(|alias| format!("'{alias}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let noun = if losers.len() == 1 {
        "profile"
    } else {
        "profiles"
    };

    shell_eprintln!(ctx, "")?;
    shell_eprintln!(
        ctx,
        "This deletes the local config entries and key files for {listed}. It \
         does NOT touch the Turnkey dashboard; revocation instructions follow \
         each deletion."
    )?;
    prompts::confirm_or_bail(
        &format!("Permanently delete {noun} {listed} and the key files on disk?"),
        "profile consolidation",
    )?;

    for alias in &losers {
        let deleted = delete_profile(ctx, config, alias).await?;
        shell_println!(ctx, "{deleted}")?;
        shell_println!(ctx)?;
    }

    // `remove_org` cleared `active_org` if the active profile was among the
    // losers; its group's keeper inherits it.
    if let Some(keeper) = active_repair {
        config.set_active_org(&keeper)?;
    }

    config.save().await?;

    Ok(())
}

/// Permanently delete a saved login profile: its config entry and its API and
/// operator key files on disk.
pub async fn run_delete(ctx: &mut StdCtx, args: DeleteArgs) -> Result<Outcome> {
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
    let alias = resolve_profile_alias(&config, args.org, is_non_interactive)?;
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

    let deleted = delete_profile(ctx, &mut config, &alias).await?;

    // Save last: persist the config removal only after the on-disk cleanup
    // succeeds, so a failure leaves the profile listed and the delete retryable.
    config.save().await?;

    Ok(Outcome::ProfileDeleted(deleted))
}

/// Remove profile `alias` from the config registry and delete its key files
/// when they use the default per-org directory layout; custom key paths are
/// left on disk with a warning. Does not save the config: callers batch one
/// or more removals and persist afterwards, preserving the save-last
/// invariant that a failed file cleanup leaves the profile listed and the
/// deletion retryable.
async fn delete_profile(
    ctx: &mut StdCtx,
    config: &mut Config,
    alias: &str,
) -> Result<ProfileDeleted> {
    let removed = config
        .remove_org(alias)
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
    let default_api_key = default_api_key_path(alias)?;
    let default_operator_key = default_operator_key_path(alias)?;
    let local_key_paths: Vec<_> = removed
        .operators
        .iter()
        .filter_map(|operator| match &operator.kind {
            OperatorRecordKind::Local(local) => Some(&local.key_path),
            _ => None,
        })
        .collect();
    let uses_default_layout = removed.api_key_path == default_api_key
        && local_key_paths
            .iter()
            .all(|path| *path == &default_operator_key);

    let removed_dir = if uses_default_layout {
        let dir = default_org_dir(alias)?;
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
        let custom_paths = local_key_paths
            .into_iter()
            .chain(Some(&removed.api_key_path))
            .collect::<BTreeSet<_>>();
        for path in custom_paths {
            shell_eprintln!(ctx, "  {}", path.display())?;
        }
        None
    };

    // A local delete does not touch the dashboard-registered API key, and we
    // can't tell whether it is still there, so hedge with "may" and give steps.
    let dashboard_url = dashboard_base_url(&removed.api_base_url);

    Ok(ProfileDeleted {
        alias: alias.to_string(),
        organization_id: removed.id,
        removed_key_directory: removed_dir.map(|dir| dir.display().to_string()),
        dashboard_url: dashboard_url.to_string(),
        api_public_key,
    })
}

/// Mark a profile as the default alias for its duplicated organization ID and
/// clear the marker from the organization's other profiles. The marker only
/// means something while duplicates exist, so a non-duplicated organization is
/// refused.
pub async fn run_set_default_alias(
    _ctx: &mut StdCtx,
    args: SetDefaultAliasArgs,
) -> Result<Outcome> {
    let mut config = Config::load().await?;
    let alias = args.org;

    let Some(org_id) = config.orgs.get(&alias).map(|org| org.id.clone()) else {
        let id_matches = find_org_aliases(&config, &alias);

        if !id_matches.is_empty() {
            bail!(
                "'{alias}' is an organization ID; pass the profile alias to mark as its default \
                 (one of: {}).",
                id_matches.join(", ")
            );
        }

        bail!(
            "Login profile '{alias}' not found. \
             Run `tvc login` to see configured profiles."
        );
    };

    let mut duplicates: Vec<String> = config
        .orgs
        .iter()
        .filter(|(other, org)| org.id == org_id && *other != &alias)
        .map(|(other, _)| other.clone())
        .collect();
    duplicates.sort_unstable();

    if duplicates.is_empty() {
        bail!(
            "Organization '{org_id}' only has the profile '{alias}'; \
             a default alias is only needed while duplicate profiles exist."
        );
    }

    for (other, org) in config.orgs.iter_mut() {
        if org.id == org_id {
            org.default_alias = *other == alias;
        }
    }

    config.save().await?;

    Ok(Outcome::ProfileDefaultAliasSet(DefaultAliasSet {
        alias,
        organization_id: org_id,
        duplicates,
    }))
}

/// Resolve the alias of a configured profile to delete. Prompts interactively
/// with a picker when no query is given, or when an org-ID query matches
/// several profiles; non-interactive runs must name a single profile.
fn resolve_profile_alias(
    config: &Config,
    org: Option<String>,
    is_non_interactive: bool,
) -> Result<String> {
    match org {
        Some(query) => {
            let aliases = find_org_aliases(config, &query);

            match aliases.as_slice() {
                [] => bail!(
                    "Login profile '{query}' not found. \
                     Run `tvc login` to see configured profiles."
                ),
                [alias] => Ok(alias.clone()),
                _ if is_non_interactive => bail!(
                    "Organization '{query}' is configured under multiple profiles: {}. \
                     Re-run with --org <alias> to select which profile to delete.",
                    aliases.join(", ")
                ),
                _ => Ok(prompts::select(
                    "Select profile to delete",
                    duplicate_alias_choices(config, &aliases),
                )?
                .alias
                .to_string()),
            }
        }
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
                    is_default: org.default_alias,
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
    is_default: bool,
    is_active: bool,
}

impl Display for ProfileChoice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.alias, self.org_id)?;

        if self.is_default {
            write!(f, " (default)")?;
        }

        if self.is_active {
            write!(f, " (active)")?;
        }

        Ok(())
    }
}

/// Selection choices for aliases that share one organization ID, marking the
/// default and active profiles the way the no-query delete picker does.
fn duplicate_alias_choices<'a>(
    config: &'a Config,
    aliases: &'a [String],
) -> Vec<ProfileChoice<'a>> {
    aliases
        .iter()
        .map(|alias| {
            let org = config
                .orgs
                .get(alias)
                .expect("alias was resolved from this config");

            ProfileChoice {
                alias: alias.as_str(),
                org_id: org.id.as_str(),
                is_default: org.default_alias,
                is_active: config.active_org.as_deref() == Some(alias.as_str()),
            }
        })
        .collect()
}

fn build_login_plan_interactive(
    ctx: &mut StdCtx,
    args: Args,
    config: &Config,
) -> Result<LoginPlan> {
    let org = match args.org {
        Some(query) => OrgPlan::Existing(resolve_org_query(ctx, config, &query)?),
        None => prompt_for_org_plan(ctx, config, args.api_base_url.as_deref())?,
    };
    Ok(LoginPlan {
        org,
        api_base_url_override: args.api_base_url,
        api_key_policy: ApiKeyPolicy::AllowGenerate,
    })
}

fn build_login_plan_non_interactive(
    ctx: &mut StdCtx,
    args: Args,
    config: &Config,
) -> Result<LoginPlan> {
    let Some(org_query) = args.org else {
        return Err(error_required_in_non_interactive("--org"));
    };

    let alias = resolve_org_query(ctx, config, &org_query)?;

    Ok(LoginPlan {
        org: OrgPlan::Existing(alias),
        api_base_url_override: args.api_base_url,
        api_key_policy: ApiKeyPolicy::RequireExisting,
    })
}

/// Resolve an org query (profile alias or organization ID) to a single
/// configured profile alias.
///
/// An explicitly named alias is an unambiguous choice and wins outright,
/// except that non-interactive runs refuse a non-default duplicate. An org-ID
/// query matching several profiles prompts for one interactively;
/// non-interactively it follows the `default_alias` marker or fails with
/// instructions. Read-only consumers (login, key backup) share these rules;
/// destructive commands resolve via `resolve_profile_alias` instead, which
/// never guesses among duplicates.
pub(crate) fn resolve_org_query(ctx: &mut StdCtx, config: &Config, query: &str) -> Result<String> {
    let aliases = find_org_aliases(config, query);

    match aliases.as_slice() {
        [] => bail!(
            "Organization '{query}' not found. \
             Run `tvc login` without --org to set up a new organization."
        ),
        [alias] => {
            let org_id = config
                .orgs
                .get(alias.as_str())
                .expect("alias was resolved from this config")
                .id
                .as_str();
            let mut group: Vec<&str> = config
                .orgs
                .iter()
                .filter(|(_, org)| org.id == org_id)
                .map(|(group_alias, _)| group_alias.as_str())
                .collect();
            group.sort_unstable();

            if group.len() > 1 && ctx.is_non_interactive() {
                let defaults: Vec<&str> = group
                    .iter()
                    .copied()
                    .filter(|group_alias| config.orgs[*group_alias].default_alias)
                    .collect();

                match defaults.as_slice() {
                    [default] if *default == alias.as_str() => {
                        warn_duplicate_profiles(ctx, org_id, &group.join(", "), default)?;
                    }
                    [default] => bail!(
                        "Profile '{alias}' is a duplicate: the default profile for \
                         organization '{org_id}' is '{default}'. \
                         Log in with `tvc login --org {default}`, \
                         delete this profile with `tvc profile delete --org {alias}`, \
                         or make it the default with `tvc profile set-default-alias --org {alias}`."
                    ),
                    [] => return Err(no_default_alias_error(org_id, &group.join(", "))),
                    marked => {
                        return Err(multiple_default_aliases_error(org_id, &marked.join(", ")));
                    }
                }
            }

            Ok(alias.clone())
        }
        _ if ctx.is_non_interactive() => {
            let defaults: Vec<&str> = aliases
                .iter()
                .map(String::as_str)
                .filter(|alias| config.orgs[*alias].default_alias)
                .collect();

            match defaults.as_slice() {
                [default] => {
                    warn_duplicate_profiles(ctx, query, &aliases.join(", "), default)?;
                    Ok(default.to_string())
                }
                [] => Err(no_default_alias_error(query, &aliases.join(", "))),
                marked => Err(multiple_default_aliases_error(query, &marked.join(", "))),
            }
        }
        _ => Ok(prompts::select(
            &format!("Select profile for organization '{query}'"),
            duplicate_alias_choices(config, &aliases),
        )?
        .alias
        .to_string()),
    }
}

/// A duplicated organization with no profile marked `default_alias` cannot be
/// resolved non-interactively; name every exit.
fn no_default_alias_error(org_id: &str, alias_list: &str) -> anyhow::Error {
    anyhow!(
        "Multiple profiles are configured for organization '{org_id}': {alias_list}. \
         Mark the one to keep with `tvc profile set-default-alias --org <alias>`, \
         delete an extra with `tvc profile delete --org <alias> --yes`, \
         or run `tvc login` interactively to consolidate."
    )
}

fn multiple_default_aliases_error(org_id: &str, marked_list: &str) -> anyhow::Error {
    anyhow!(
        "Multiple profiles are marked default_alias for organization '{org_id}': {marked_list}. \
         Re-run `tvc profile set-default-alias --org <alias>` to leave exactly one."
    )
}

fn warn_duplicate_profiles(
    ctx: &mut StdCtx,
    org_id: &str,
    alias_list: &str,
    default: &str,
) -> Result<()> {
    shell_eprintln!(
        ctx,
        "WARNING: organization '{org_id}' has duplicate profiles: {alias_list}. \
         Using default profile '{default}'."
    )?;
    shell_eprintln!(
        ctx,
        "Run `tvc login` interactively to consolidate, \
         or delete extras with `tvc profile delete --org <alias> --yes`."
    )?;
    Ok(())
}

async fn execute_login(ctx: &mut StdCtx, mut config: Config, plan: LoginPlan) -> Result<Outcome> {
    let (alias, org_config) = match plan.org {
        OrgPlan::Existing(alias) => {
            update_api_base_url_from_override(
                &mut config,
                &alias,
                plan.api_base_url_override.as_deref(),
            );

            let org_config = config
                .orgs
                .get(&alias)
                .cloned()
                .expect("plan alias was resolved against this config");
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
    let operator_key = find_or_generate_operator_key(ctx, &alias, &org_config).await?;

    // Operator key path now lives in the org's local operator record (the
    // versioned registry), not on `OrgConfig` directly. Resolve it before the
    // struct literal so `alias` is still available (the struct moves it).
    let operator_key_path = org_config
        .select_local_record(&alias)?
        .key_path
        .display()
        .to_string();

    Ok(Outcome::LoggedIn(LoggedIn {
        organization_name: whoami.organization_name,
        organization_id: whoami.organization_id,
        username: whoami.username,
        user_id: whoami.user_id,
        alias,
        api_public_key: api_key.public_key.clone(),
        operator_public_key: operator_key.public_key.clone(),
        config_file_path: crate::config::turnkey::config_file_path()?
            .display()
            .to_string(),
        api_key_path: org_config.api_key_path.display().to_string(),
        operator_key_path,
    }))
}

fn prompt_for_org_plan(
    ctx: &mut StdCtx,
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
        return prompt_for_new_org_inputs(ctx, config, api_base_url_override);
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
        OrgChoice::New => prompt_for_new_org_inputs(ctx, config, api_base_url_override),
    }
}

fn prompt_for_new_org_inputs(
    ctx: &mut StdCtx,
    config: &Config,
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

    // One profile per organization: a second alias for the same ID would make
    // resolution ambiguous again (TVC-159).
    let already_configured = config
        .orgs
        .iter()
        .filter(|(_, org)| org.id == id)
        .map(|(alias, _)| alias)
        .min();

    if let Some(alias) = already_configured {
        bail!(
            "Organization '{id}' is already configured as profile '{alias}'. \
             Run `tvc login --org {alias}` to use it, \
             or `tvc profile delete --org {alias}` to remove it first."
        );
    }

    let alias = prompts::text("Organization alias", Some("default"))?;
    debug!(org_alias = %alias, "user entered new organization inputs");

    if let Some(existing) = config.orgs.get(&alias) {
        bail!(
            "Profile alias '{alias}' is already in use for organization '{}'. \
             Choose a different alias, \
             or run `tvc profile delete --org {alias}` to remove it first.",
            existing.id
        );
    }

    Ok(OrgPlan::New { id, alias })
}

enum OrgChoice {
    Existing { display: String, alias: String },
    New,
}

impl Display for OrgChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OrgChoice::Existing { display, .. } => write!(f, "{display}"),
            OrgChoice::New => write!(f, "[new] Add a new organization"),
        }
    }
}

/// Every configured alias matching an org query, for callers to dispose of
/// per their mode. An exact alias match is unambiguous and wins outright;
/// otherwise all aliases registered for the query as an organization ID are
/// returned, sorted for stable output.
fn find_org_aliases(config: &Config, query: &str) -> Vec<String> {
    if config.orgs.contains_key(query) {
        return vec![query.to_string()];
    }

    let mut aliases: Vec<String> = config
        .orgs
        .iter()
        .filter(|(_, org)| org.id == query)
        .map(|(alias, _)| alias.clone())
        .collect();
    aliases.sort();
    aliases
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

async fn generate_api_key(ctx: &mut StdCtx, org_config: &OrgConfig) -> Result<StoredApiKey> {
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

fn wait_for_dashboard_registration(ctx: &mut StdCtx) -> Result<()> {
    shell_print!(ctx, "Press Enter when done...")?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(())
}

async fn find_or_generate_operator_key(
    ctx: &mut StdCtx,
    org_alias: &str,
    org_config: &OrgConfig,
) -> Result<StoredQosOperatorKey> {
    let local = org_config.select_local_record(org_alias)?;
    debug!(operator_key_path = %local.key_path.display(), "resolving operator key");

    if let Some(operator_key) = StoredQosOperatorKey::load(&local.key_path).await? {
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

    operator_key.save(&local.key_path).await?;

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

    let client = build_turnkey_client(stamper, api_base_url)?;

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

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggedIn {
    organization_name: String,
    organization_id: String,
    username: String,
    user_id: String,
    alias: String,
    api_public_key: String,
    operator_public_key: String,
    config_file_path: String,
    api_key_path: String,
    operator_key_path: String,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDeleted {
    alias: String,
    organization_id: String,
    removed_key_directory: Option<String>,
    dashboard_url: String,
    api_public_key: Option<String>,
}

impl Display for ProfileDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut lines = vec![format!(
            "Deleted login profile '{}' ({}).",
            self.alias, self.organization_id
        )];

        if let Some(directory) = &self.removed_key_directory {
            lines.push(format!("Removed key directory: {directory}"));
        }

        lines.extend([
            String::new(),
            "IMPORTANT: The API key may still be registered on the Turnkey dashboard.".to_string(),
            "It will remain valid until it is manually removed. To remove it:".to_string(),
            format!(
                "  1. Go to {}/dashboard/v2/users and click your user",
                self.dashboard_url
            ),
        ]);

        match &self.api_public_key {
            Some(public_key) => {
                lines.push("  2. Delete the API key with public key:".to_string());
                lines.push(format!("       {public_key}"));
            }
            None => lines.push("  2. Delete the API key associated with this profile".to_string()),
        }

        f.write_str(&lines.join("\n"))
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultAliasSet {
    alias: String,
    organization_id: String,
    duplicates: Vec<String>,
}

impl Display for DefaultAliasSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Marked '{}' as the default alias for organization '{}' (duplicates: {}).",
            self.alias,
            self.organization_id,
            self.duplicates.join(", ")
        )
    }
}

impl Display for LoggedIn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
Successfully logged in!

Organization: {} ({})
User: {} ({})
Active Org: {}

Credentials
  API public key:        {}
  Operator public key:   {}

Saved to
  Config file:    {}
  API key:        {}
  Operator key:   {}"#,
            self.organization_name,
            self.organization_id,
            self.username,
            self.user_id,
            self.alias,
            self.api_public_key,
            self.operator_public_key,
            self.config_file_path,
            self.api_key_path,
            self.operator_key_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        API_BASE_URL_DEV, API_BASE_URL_PREPROD, DASHBOARD_URL_DEV, DASHBOARD_URL_PREPROD,
        DASHBOARD_URL_PROD, OperatorKind, OperatorRecord,
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

    #[test]
    fn find_org_aliases_prefers_exact_alias_match() {
        // The alias "org-x" collides with another profile's organization ID;
        // the exact alias match must win outright.
        let config = config_with_orgs(&[("org-x", "org-y"), ("a", "org-x")]);

        assert_eq!(find_org_aliases(&config, "org-x"), ["org-x"]);
    }

    #[test]
    fn find_org_aliases_returns_all_id_matches_sorted() {
        let config = config_with_orgs(&[("c", "org-1"), ("a", "org-1"), ("b", "org-1")]);

        assert_eq!(find_org_aliases(&config, "org-1"), ["a", "b", "c"]);
    }

    #[test]
    fn find_org_aliases_returns_empty_for_unknown_query() {
        let config = config_with_orgs(&[("a", "org-1")]);

        assert!(find_org_aliases(&config, "org-unknown").is_empty());
    }

    fn config_with_org(api_base_url: &str) -> Config {
        Config {
            active_org: Some("default".to_string()),
            orgs: HashMap::from([(
                "default".to_string(),
                OrgConfig {
                    id: "org-test".to_string(),
                    api_key_path: PathBuf::from("api_key.json"),
                    api_base_url: api_base_url.to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators: vec![OperatorRecord::local(PathBuf::from("operator.json"))],
                    default_alias: false,
                    extra: toml::Table::new(),
                },
            )]),
            last_created_app_id: HashMap::new(),
            last_operator_ids: HashMap::new(),
            extra: toml::Table::new(),
        }
    }

    fn config_with_orgs(entries: &[(&str, &str)]) -> Config {
        Config {
            active_org: None,
            orgs: entries
                .iter()
                .map(|(alias, org_id)| {
                    (
                        alias.to_string(),
                        OrgConfig {
                            id: org_id.to_string(),
                            api_key_path: PathBuf::from("api_key.json"),
                            api_base_url: API_BASE_URL_PROD.to_string(),
                            default_operator_kind: OperatorKind::Local,
                            operators: vec![OperatorRecord::local(PathBuf::from("operator.json"))],
                            default_alias: false,
                            extra: toml::Table::new(),
                        },
                    )
                })
                .collect(),
            last_created_app_id: HashMap::new(),
            last_operator_ids: HashMap::new(),
            extra: toml::Table::new(),
        }
    }
}
