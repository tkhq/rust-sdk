//! Login command for authenticating with Turnkey.

use crate::{
    client::build_turnkey_client,
    commands::keys::backup_operator_key::{
        OperatorKeyBackedUp, back_up, prompt_for_backup_destination,
    },
    config::turnkey::{
        API_BASE_URL_PROD, Config, KeyCurve, LocalOperatorRecord, NewOrgOperator, OperatorKind,
        OperatorRecord, OperatorRecordKind, OrgConfig, OrgQuery, QosOperatorPublicKey, Resolved,
        SelectYubiKeyOperatorError, StoredApiKey, StoredQosOperatorKey, YubiKeyOperatorRecord,
        YubiKeySerial, dashboard_base_url, default_api_key_path, default_operator_key_path,
        default_org_dir, legacy_org_dir,
    },
    outcome::Outcome,
    output::{MissingRequiredInput, StdCtx},
    prompts::{self, error_required_in_non_interactive},
    shell_eprintln, shell_print, shell_println,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::Args as ClapArgs;
use qos_p256::P256Pair;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    io::BufRead,
    str::FromStr,
};
use thiserror::Error;
use tracing::{debug, instrument};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::GetWhoamiRequest;
use uuid::Uuid;

/// Authenticate with Turnkey and set up local credentials.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID to log in with.
    /// If not provided, will prompt interactively.
    #[arg(long, env = "TVC_ORG", value_parser = OrgQuery::from_str)]
    pub org: Option<OrgQuery>,
    /// Turnkey API base URL. Defaults to production for newly configured orgs.
    #[arg(long, env = "TVC_API_BASE_URL", value_name = "URL")]
    pub api_base_url: Option<String>,
    /// Serial (hex) of the YubiKey operator to log in with, when the
    /// organization registers several. Unused for other operator kinds.
    #[arg(long, value_name = "SERIAL")]
    pub serial: Option<YubiKeySerial>,
}

/// Permanently delete a saved login profile, including its API and any local
/// operator key files on disk.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct DeleteArgs {
    /// Organization alias or ID of the profile to delete.
    /// If not provided, will prompt interactively.
    #[arg(short, long, value_name = "ORG", value_parser = OrgQuery::from_str)]
    pub org: Option<OrgQuery>,
    /// Skip the confirmation prompt (required to delete in non-interactive mode).
    #[arg(short, long)]
    pub yes: bool,
}

enum OrgPlan {
    /// A configured organization resolved from the user's query or picked
    /// interactively, with the name to echo in output (aliases only appear
    /// in output when the user used one).
    Existing { id: Uuid, name: Option<String> },
    /// A confirmed new name for an organization that is already configured;
    /// login binds it and proceeds against the existing entry.
    BindAlias { id: Uuid, alias: String },
    New {
        id: Uuid,
        alias: String,
        operator: NewOrgOperatorPlan,
    },
}

/// The operator backend chosen for a new organization at plan time.
enum NewOrgOperatorPlan {
    Local,
    RegisteredYubikey(YubiKeySerial),
}

/// The yubikey leg of a newly created organization: what the coherent save
/// must be able to recover, and what the post-save guidance surfaces.
struct NewOrgYubiKey {
    public_key: QosOperatorPublicKey,
}

enum ApiKeyPolicy {
    AllowGenerate,
    RequireExisting,
}

struct LoginPlan {
    org: OrgPlan,
    api_base_url_override: Option<String>,
    api_key_policy: ApiKeyPolicy,
    /// The YubiKey operator selected at the endpoint. `None` remains valid
    /// for non-interactive runs with a sole record.
    yubikey_serial: Option<YubiKeySerial>,
}

#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args, mut config: Config) -> Result<Outcome> {
    debug!(
        non_interactive = ctx.is_non_interactive(),
        org_arg_present = args.org.is_some(),
        api_base_url_override_present = args.api_base_url.is_some(),
        "running login command"
    );

    let plan = if ctx.is_non_interactive() {
        build_login_plan_non_interactive(args, &config)?
    } else {
        // Move legacy alias-keyed key directories to the id-keyed layout
        // before anything resolves paths against the config.
        let legacy_profiles = config.legacy_layout_profiles()?;

        if !legacy_profiles.is_empty() {
            migrate_legacy_key_directories(ctx, &mut config, legacy_profiles).await?;
        }

        build_login_plan_interactive(ctx, args, &config)?
    };

    execute_login(ctx, config, plan).await
}

/// Migrate legacy alias-keyed key directories to the id-keyed layout: copy
/// the files across (verifying every copy), rewrite the organization's
/// paths, save, and only then delete the old directory. Every crash window
/// therefore leaves the config pointing at files that exist — the legacy
/// originals before the save, the verified copies after it. A failure warns
/// and skips the organization: its legacy paths remain valid, and login must
/// not die over a tidiness move.
async fn migrate_legacy_key_directories(
    ctx: &mut StdCtx,
    config: &mut Config,
    profiles: Vec<(String, Uuid)>,
) -> Result<()> {
    for (alias, org_id) in profiles {
        let source = legacy_org_dir(&alias)?;
        let target = default_org_dir(org_id)?;

        let copied = async {
            // A missing source with a populated target is an interrupted
            // run's post-copy window (or an old rename-based move); there is
            // nothing left to copy.
            if !source.is_dir() {
                ensure!(
                    target.is_dir(),
                    "the legacy directory is missing and nothing was copied to {}",
                    target.display()
                );
                return Ok(());
            }

            // Only resume our own interrupted copy: every file already at
            // the target must be an identical copy of a legacy file —
            // anything else means the directory belongs to someone else.
            if target.is_dir() {
                let mut entries = tokio::fs::read_dir(&target).await?;

                while let Some(entry) = entries.next_entry().await? {
                    let name = entry.file_name();
                    let source_file = source.join(&name);
                    let matches = entry.file_type().await?.is_file()
                        && tokio::fs::try_exists(&source_file).await?
                        && tokio::fs::read(entry.path()).await?
                            == tokio::fs::read(&source_file).await?;

                    ensure!(
                        matches,
                        "{} already contains {}",
                        target.display(),
                        name.display()
                    );
                }
            }

            let mut names = Vec::new();
            let mut entries = tokio::fs::read_dir(&source).await?;

            while let Some(entry) = entries.next_entry().await? {
                ensure!(
                    entry.file_type().await?.is_file(),
                    "{} is not a regular file",
                    entry.path().display()
                );
                names.push(entry.file_name());
            }

            tokio::fs::create_dir_all(&target).await?;

            for name in names {
                let bytes = tokio::fs::read(source.join(&name)).await?;
                let to = target.join(&name);

                // Present files passed the identical-copy scan above.
                if tokio::fs::try_exists(&to).await? {
                    continue;
                }

                tokio::fs::write(&to, &bytes).await?;

                // Verify the copy before the config ever points at it.
                ensure!(
                    tokio::fs::read(&to).await? == bytes,
                    "verification failed for {}",
                    to.display()
                );
            }

            Ok(())
        }
        .await;

        if let Err(error) = copied {
            shell_eprintln!(
                ctx,
                "WARNING: could not move key directory {} -> {}: {error:#}. \
                 The organization keeps its current paths.",
                source.display(),
                target.display()
            )?;
            continue;
        }

        if let Some(org) = config.orgs.get_mut(&org_id) {
            let operator_key_path = default_operator_key_path(org_id)?;
            org.api_key_path = default_api_key_path(org_id)?;
            org.operators
                .iter_mut()
                .filter_map(|operator| match &mut operator.kind {
                    OperatorRecordKind::Local(local) => Some(local),
                    _ => None,
                })
                .for_each(|local| local.key_path = operator_key_path.clone());
        }

        config.save().await?;
        shell_println!(
            ctx,
            "Moved key directory: {} -> {}",
            source.display(),
            target.display()
        )?;

        // The copies are saved and verified; the originals are redundant now.
        match tokio::fs::remove_dir_all(&source).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                shell_eprintln!(
                    ctx,
                    "WARNING: could not remove the old key directory {}: {error}. \
                     Remove it manually; the config no longer references it.",
                    source.display()
                )?;
            }
        }
    }

    Ok(())
}

/// Permanently delete a saved organization login: its config entry, every
/// alias bound to it, and its API and any local operator key files on disk.
/// YubiKey operator references vanish with the login, but the devices and
/// their shared registry entries are never touched.
pub async fn run_delete(ctx: &mut StdCtx, args: DeleteArgs, mut config: Config) -> Result<Outcome> {
    let is_non_interactive = ctx.is_non_interactive();
    debug!(
        non_interactive = is_non_interactive,
        org_arg_present = args.org.is_some(),
        skip_confirm = args.yes,
        "running login delete command"
    );

    // Validate inputs before any business logic: non-interactive mode cannot
    // prompt, so it requires --org (which organization) and --yes (confirmation).
    if is_non_interactive {
        if args.org.is_none() {
            return Err(error_required_in_non_interactive("--org"));
        }
        if !args.yes {
            return Err(error_required_in_non_interactive("--yes"));
        }
    }

    let (org_id, org_display, dashboard_url) = match args.org {
        Some(query) => {
            let (resolved, org) = resolve_org_query(&config, &query)?;
            (
                resolved.id(),
                resolved.to_string(),
                dashboard_base_url(&org.api_base_url),
            )
        }
        None => {
            // Reached only in interactive mode; a non-interactive run without
            // --org is rejected up front before we get here.
            if config.orgs.is_empty() {
                bail!("No organization logins to delete.");
            }

            let row = prompts::select("Select organization to delete", org_rows(&config))?;
            let dashboard_url = config
                .orgs
                .get(&row.id)
                .map(|org| dashboard_base_url(&org.api_base_url))
                .unwrap_or(dashboard_base_url(API_BASE_URL_PROD));
            (
                row.id,
                row.name.unwrap_or_else(|| row.id.to_string()),
                dashboard_url,
            )
        }
    };

    // Interactive confirmation. A non-interactive run without --yes was rejected
    // up front, so reaching here with !args.yes means we can prompt.
    if !args.yes {
        shell_eprintln!(ctx, "")?;
        shell_eprintln!(
            ctx,
            "WARNING: This permanently deletes the login for organization '{org_display}' ({org_id})."
        )?;
        shell_eprintln!(
            ctx,
            "  - Removes the config entry, its aliases, and deletes the API and any local"
        )?;
        shell_eprintln!(
            ctx,
            "    operator key files from disk. This cannot be undone."
        )?;
        shell_eprintln!(
            ctx,
            "  - It does NOT touch the Turnkey dashboard ({dashboard_url}). If this API"
        )?;
        shell_eprintln!(
            ctx,
            "    key is registered there, it stays valid until you remove it"
        )?;
        shell_eprintln!(ctx, "    (instructions are printed after deletion).")?;

        let references_yubikeys = config
            .orgs
            .get(&org_id)
            .is_some_and(|org| org.yubikey_operators().next().is_some());

        if references_yubikeys {
            shell_eprintln!(
                ctx,
                "  - YubiKey operator references are removed from this login only: the"
            )?;
            shell_eprintln!(
                ctx,
                "    devices and their [[yubikeys]] registry entries are NOT touched"
            )?;
            shell_eprintln!(
                ctx,
                "    (other organizations may share them). After removing every reference,"
            )?;
            shell_eprintln!(
                ctx,
                "    forget a device locally with `tvc yubikey unregister`; this does not"
            )?;
            shell_eprintln!(
                ctx,
                "    modify the device or revoke it from an organization."
            )?;
        }

        shell_eprintln!(ctx, "")?;
        prompts::confirm_or_bail(
            &format!("Permanently delete '{org_display}' ({org_id}) and its key files?"),
            "deletion",
        )?;
    }

    let deleted = delete_org_login(ctx, &mut config, org_id).await?;

    Ok(Outcome::ProfileDeleted(deleted))
}

/// Permanently delete the login for `org_id`: remove it from the config
/// (with every alias bound to it), delete its key files when they use a
/// default per-org directory layout (custom key paths are left on disk with
/// a warning), then save the config. The save comes last so that dying
/// between the file cleanup and the save leaves the login listed and the
/// deletion retryable; the file deletion tolerates an already-missing
/// directory for exactly that retry.
async fn delete_org_login(
    ctx: &mut StdCtx,
    config: &mut Config,
    org_id: Uuid,
) -> Result<ProfileDeleted> {
    // Legacy directory candidates are named by the aliases, so collect them
    // before the removal unbinds the names.
    let legacy_dirs = config
        .aliases
        .names_of(org_id)
        .map(legacy_org_dir)
        .collect::<Result<Vec<_>>>()?;

    let Some((removed, unbound_aliases)) = config.remove_org(org_id) else {
        bail!("Organization '{org_id}' is not configured.");
    };

    // Read the API key's public key before deleting its file, so the
    // dashboard-revocation reminder below can name exactly which key to remove.
    // Best-effort: a missing or unreadable key file just omits the value.
    let api_public_key = StoredApiKey::load(&removed)
        .await
        .ok()
        .flatten()
        .map(|key| key.public_key);

    // The default layout stores both key files in one per-org directory —
    // id-keyed today, alias-keyed before TVC-55 — so a default-layout login
    // is removed by deleting that directory. Custom (hand-edited) key paths
    // are left untouched with a warning, since the user placed them
    // deliberately and they may live outside our config tree.
    let owned_dir = std::iter::once(default_org_dir(org_id)?)
        .chain(legacy_dirs)
        .find(|dir| removed.has_default_layout_at(dir));

    let removed_dir = match owned_dir {
        Some(dir) => {
            // A hand-edited config can point several organizations into one
            // directory; deleting it would take the survivors' keys with it.
            let still_used = config.orgs.values().any(|org| {
                org.api_key_path.starts_with(&dir)
                    || org.operators.iter().any(|operator| {
                        matches!(&operator.kind, OperatorRecordKind::Local(local)
                            if local.key_path.starts_with(&dir))
                    })
            });

            if still_used {
                shell_eprintln!(
                    ctx,
                    "WARNING: key directory {} is still used by another organization and was NOT deleted.",
                    dir.display()
                )?;
                None
            } else {
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
                        return Err(e).with_context(|| {
                            format!("failed to delete key directory: {}", dir.display())
                        });
                    }
                }
            }
        }
        None => {
            shell_eprintln!(
                ctx,
                "WARNING: custom key paths are configured and were NOT deleted."
            )?;
            shell_eprintln!(ctx, "Remove them manually if no longer needed:")?;

            let custom_paths = removed
                .operators
                .iter()
                .filter_map(|operator| match &operator.kind {
                    OperatorRecordKind::Local(local) => Some(&local.key_path),
                    _ => None,
                })
                .chain(Some(&removed.api_key_path))
                .collect::<BTreeSet<_>>();

            for path in custom_paths {
                shell_eprintln!(ctx, "  {}", path.display())?;
            }

            None
        }
    };

    config.save().await?;

    // A local delete does not touch the dashboard-registered API key, and we
    // can't tell whether it is still there, so hedge with "may" and give steps.
    let retained_yubikey_serials = removed
        .yubikey_operators()
        .map(|(_, yubikey)| yubikey.serial)
        .collect::<Vec<_>>();

    Ok(ProfileDeleted {
        aliases: unbound_aliases,
        organization_id: org_id,
        removed_key_directory: removed_dir.map(|dir| dir.display().to_string()),
        retained_yubikey_serials,
        dashboard_url: dashboard_base_url(&removed.api_base_url).to_string(),
        api_public_key,
    })
}

/// One picker row for an organization: its aliases (or bare ID) plus the
/// active marker. `name` carries the first alias for output echoing.
struct OrgRow {
    id: Uuid,
    name: Option<String>,
    display: String,
}

impl Display for OrgRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.display.fmt(f)
    }
}

fn org_rows(config: &Config) -> Vec<OrgRow> {
    config
        .orgs
        .keys()
        .map(|id| {
            let names = config.aliases.names_of(*id).collect::<Vec<_>>();
            let active = if config.active_org == Some(*id) {
                " (active)"
            } else {
                ""
            };
            let display = if names.is_empty() {
                format!("{id}{active}")
            } else {
                format!("{} ({id}){active}", names.join(", "))
            };

            OrgRow {
                id: *id,
                name: names.first().map(|name| name.to_string()),
                display,
            }
        })
        .collect()
}

fn build_login_plan_interactive(
    ctx: &mut StdCtx,
    args: Args,
    config: &Config,
) -> Result<LoginPlan> {
    let Args {
        org,
        api_base_url,
        serial,
    } = args;
    let org = match org {
        Some(query) => {
            let (resolved, _) = resolve_org_query(config, &query)?;
            OrgPlan::Existing {
                id: resolved.id(),
                name: resolved.name().map(str::to_string),
            }
        }
        None => prompt_for_org_plan(ctx, config, api_base_url.as_deref(), serial)?,
    };

    let yubikey_serial = match &org {
        OrgPlan::Existing { id, .. } | OrgPlan::BindAlias { id, .. } => config
            .orgs
            .get(id)
            .filter(|org| org.default_operator_kind == OperatorKind::Yubikey)
            .map(|org| -> Result<YubiKeySerial> {
                if let Some(serial) = serial {
                    return Ok(org.select_yubikey_operator(Some(serial))?.1.serial);
                }

                match org.select_yubikey_operator(None) {
                    Ok((_, yubikey)) => Ok(yubikey.serial),
                    Err(SelectYubiKeyOperatorError::MultipleYubiKeyOperators { .. }) => {
                        struct Choice<'a>(&'a OperatorRecord, &'a YubiKeyOperatorRecord);

                        impl Display for Choice<'_> {
                            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                                write!(f, "{} (serial {})", self.0.name, self.1.serial)
                            }
                        }

                        let choices = org
                            .yubikey_operators()
                            .map(|(operator, yubikey)| Choice(operator, yubikey))
                            .collect();
                        let Choice(_, yubikey) =
                            prompts::select("Select YubiKey operator", choices)?;

                        Ok(yubikey.serial)
                    }
                    Err(error) => Err(error.into()),
                }
            })
            .transpose()?,
        OrgPlan::New { .. } => None,
    };

    Ok(LoginPlan {
        org,
        api_base_url_override: api_base_url,
        api_key_policy: ApiKeyPolicy::AllowGenerate,
        yubikey_serial,
    })
}

fn build_login_plan_non_interactive(args: Args, config: &Config) -> Result<LoginPlan> {
    let Some(query) = args.org else {
        return Err(error_required_in_non_interactive("--org"));
    };

    let (resolved, _) = resolve_org_query(config, &query)?;

    Ok(LoginPlan {
        org: OrgPlan::Existing {
            id: resolved.id(),
            name: resolved.name().map(str::to_string),
        },
        api_base_url_override: args.api_base_url,
        api_key_policy: ApiKeyPolicy::RequireExisting,
        yubikey_serial: args.serial,
    })
}

/// Refusals from resolving an org query against the configured profiles.
/// Typed so the remediation text lives in one place and the error keeps its
/// shape through the `anyhow` chain.
#[derive(Debug, Error)]
enum ResolveError {
    #[error(
        "Organization '{query}' not found. \
         Run `tvc login` without --org to set up a new organization."
    )]
    NotFound { query: OrgQuery },
}

/// Resolve an org query (alias or organization ID) to a configured
/// organization. A query names at most one organization by construction, so
/// resolution never prompts and never guesses; every org-selecting command
/// shares it.
pub(crate) fn resolve_org_query<'c>(
    config: &'c Config,
    query: &OrgQuery,
) -> Result<(Resolved<'c>, &'c OrgConfig)> {
    config.resolve(query).ok_or_else(|| {
        ResolveError::NotFound {
            query: query.clone(),
        }
        .into()
    })
}

async fn execute_login(ctx: &mut StdCtx, mut config: Config, plan: LoginPlan) -> Result<Outcome> {
    let (org_id, org_name, yubikey) = match plan.org {
        OrgPlan::Existing { id, name } => {
            update_api_base_url_from_override(
                &mut config,
                id,
                plan.api_base_url_override.as_deref(),
            );
            (id, name, None)
        }
        OrgPlan::BindAlias { id, alias } => {
            // The endpoint confirmed the binding (and any re-point); the save
            // below persists it.
            config.aliases.bind(alias.clone(), id);
            update_api_base_url_from_override(
                &mut config,
                id,
                plan.api_base_url_override.as_deref(),
            );
            (id, Some(alias), None)
        }
        OrgPlan::New {
            id,
            alias,
            operator,
        } => {
            // The registry entry already exists; the save below persists only
            // the new organization and its serial reference.
            let (operator, yubikey) = match operator {
                NewOrgOperatorPlan::Local => (NewOrgOperator::LocalKeyFile, None),
                NewOrgOperatorPlan::RegisteredYubikey(serial) => {
                    let entry = config.yubikeys.get(serial).ok_or_else(|| {
                        anyhow!(
                            "YubiKey {serial} is not in the device registry; install its \
                             certificates and run `tvc keys refresh-yubikey --serial {serial}` \
                             first"
                        )
                    })?;

                    (
                        NewOrgOperator::Yubikey(serial),
                        Some(NewOrgYubiKey {
                            public_key: entry.public_key,
                        }),
                    )
                }
            };

            debug!(org_id = %id, org_alias = %alias, "adding organization");
            config.add_org(
                id,
                new_org_api_base_url(plan.api_base_url_override.as_deref()),
                operator,
            )?;
            config.aliases.bind(alias.clone(), id);
            (id, Some(alias), yubikey)
        }
    };

    config.set_active_org(org_id)?;

    config.save().await?;

    // All mutation is done, so the rest of the flow can borrow the entry;
    // the plan was resolved against (or inserted into) this same config.
    let Some(org_config) = config.orgs.get(&org_id) else {
        bail!("organization '{org_id}' disappeared from the config");
    };

    // Echo the reference the user gave us: their alias when they used one,
    // the bare ID otherwise.
    match &org_name {
        Some(name) => shell_println!(ctx, "Selected org: {name} ({org_id})")?,
        None => shell_println!(ctx, "Selected org: {org_id}")?,
    }

    let org_display = org_name.clone().unwrap_or_else(|| org_id.to_string());

    if let Some(yubikey) = &yubikey {
        shell_println!(ctx)?;
        shell_println!(ctx, "Operator public key: {}", yubikey.public_key)?;
        shell_println!(ctx)?;
        shell_println!(
            ctx,
            "This key will be used for approving deployment manifests."
        )?;
        shell_println!(
            ctx,
            "Make sure to register this as an operator in your organization."
        )?;
    }

    let api_key = match StoredApiKey::load(org_config).await? {
        Some(api_key) => {
            debug!("using existing API key");
            shell_println!(ctx, "Using existing API key.")?;
            api_key
        }
        None => match plan.api_key_policy {
            ApiKeyPolicy::AllowGenerate => {
                let api_key = generate_api_key(ctx, org_config).await?;
                wait_for_dashboard_registration(ctx)?;
                api_key
            }
            ApiKeyPolicy::RequireExisting => bail!(
                "API key is required in non-interactive mode for org '{org_display}'. \
                 Run `tvc login` interactively to generate and register one first."
            ),
        },
    };

    shell_println!(ctx)?;
    shell_println!(ctx, "Verifying credentials...")?;

    let whoami = verify_credentials(&api_key, org_id, &org_config.api_base_url).await?;

    // Login ensures the org's default backend is usable, and never crosses
    // over: a local default finds or generates the registered key file; a
    // hosted default requires the registered hosted operator and has no key
    // material to generate; a yubikey default requires the registered device
    // and reads its cached key, without needing the device connected.
    let operator = match org_config.default_operator_kind {
        OperatorKind::Local => {
            let (_, local) = org_config
                .select_local_operator()
                .with_context(|| format!("org '{org_display}'"))?;
            let operator_key = find_or_generate_operator_key(ctx, &org_display, local).await?;

            LoggedInOperator::Local {
                operator_public_key: operator_key.public_key,
                operator_key_path: local.key_path.display().to_string(),
            }
        }
        OperatorKind::Hosted => {
            let (name, hosted) = org_config
                .select_hosted_operator()
                .with_context(|| format!("org '{org_display}'"))?;
            shell_println!(
                ctx,
                "Using hosted operator '{}' ({}).",
                name,
                hosted.operator_id
            )?;

            LoggedInOperator::Hosted {
                operator_name: name.to_owned(),
                operator_id: hosted.operator_id,
                operator_public_key: format!(
                    "{}{}",
                    hosted.encrypt_public_key, hosted.sign_public_key
                ),
            }
        }
        OperatorKind::Yubikey => {
            let selected = org_config.select_yubikey_operator(plan.yubikey_serial);
            let (operator, yubikey) = match selected {
                Err(error @ SelectYubiKeyOperatorError::MultipleYubiKeyOperators { .. })
                    if plan.yubikey_serial.is_none() =>
                {
                    return Err(anyhow::Error::new(error)
                        .context(MissingRequiredInput::new("--serial"))
                        .context(format!("org '{org_display}'")));
                }
                selected => selected.with_context(|| format!("org '{org_display}'"))?,
            };
            let entry = config.yubikeys.get(yubikey.serial).ok_or_else(|| {
                anyhow!(
                    "YubiKey {serial} of operator '{name}' is not in the device registry; \
                     install its certificates and run \
                     `tvc keys refresh-yubikey --serial {serial}` first",
                    serial = yubikey.serial,
                    name = operator.name,
                )
            })?;
            shell_println!(
                ctx,
                "Using YubiKey operator '{}' (serial {}).",
                operator.name,
                yubikey.serial
            )?;

            LoggedInOperator::Yubikey {
                operator_name: operator.name.clone(),
                serial: yubikey.serial,
                operator_public_key: entry.public_key,
            }
        }
    };

    Ok(Outcome::LoggedIn(LoggedIn {
        organization_name: whoami.organization_name,
        organization_id: whoami.organization_id,
        username: whoami.username,
        user_id: whoami.user_id,
        alias: org_name,
        api_public_key: api_key.public_key.clone(),
        config_file_path: crate::config::turnkey::config_file_path()?
            .display()
            .to_string(),
        api_key_path: org_config.api_key_path.display().to_string(),
        operator,
    }))
}

fn prompt_for_org_plan(
    ctx: &mut StdCtx,
    config: &Config,
    api_base_url_override: Option<&str>,
    serial: Option<YubiKeySerial>,
) -> Result<OrgPlan> {
    debug!(
        configured_org_count = config.orgs.len(),
        active_org = ?config.active_org,
        "prompting for organization plan"
    );

    if config.orgs.is_empty() {
        debug!("no organizations configured; prompting for new organization");
        shell_println!(ctx, "No organization configured.")?;
        return prompt_for_new_org_inputs(ctx, config, api_base_url_override, serial);
    }

    let mut options = org_rows(config)
        .into_iter()
        .map(OrgChoice::Existing)
        .collect::<Vec<_>>();
    options.push(OrgChoice::New);

    match prompts::select("Select organization", options)? {
        OrgChoice::Existing(row) => Ok(OrgPlan::Existing {
            id: row.id,
            name: row.name,
        }),
        OrgChoice::New => prompt_for_new_org_inputs(ctx, config, api_base_url_override, serial),
    }
}

fn prompt_for_new_org_inputs(
    ctx: &mut StdCtx,
    config: &Config,
    api_base_url_override: Option<&str>,
    serial: Option<YubiKeySerial>,
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

    let id: Uuid = id
        .trim()
        .parse()
        .context("Organization ID must be a UUID")?;

    // The registry is keyed by ID, so a second login for the same
    // organization binds another name to the existing entry — after an
    // explicit confirmation — instead of reconfiguring it.
    if config.orgs.contains_key(&id) {
        let known = config.display_name(id);
        shell_eprintln!(
            ctx,
            "Organization '{id}' is already configured as '{known}'."
        )?;
        prompts::confirm_or_bail(
            &format!("Bind another alias to '{known}' and log in to it?"),
            "alias binding",
        )?;

        let alias = prompt_for_alias_binding(ctx, config, id)?;

        return Ok(OrgPlan::BindAlias { id, alias });
    }

    let alias = prompt_for_alias_binding(ctx, config, id)?;
    debug!(org_alias = %alias, "user entered new organization inputs");

    let operator = {
        enum OperatorKeyChoice {
            Local,
            Yubikey,
        }

        impl Display for OperatorKeyChoice {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Local => f.write_str("Local key file"),
                    Self::Yubikey => f.write_str("YubiKey"),
                }
            }
        }

        match prompts::select(
            "Operator key type",
            vec![OperatorKeyChoice::Local, OperatorKeyChoice::Yubikey],
        )? {
            OperatorKeyChoice::Local => NewOrgOperatorPlan::Local,
            OperatorKeyChoice::Yubikey => {
                let serial = match serial {
                    Some(serial) => {
                        ensure!(
                            config.yubikeys.contains(serial),
                            "YubiKey {serial} is not in the device registry; install its \
                             certificates and run `tvc keys refresh-yubikey --serial {serial}` \
                             first"
                        );
                        serial
                    }
                    None => {
                        let registered = config.yubikeys.serials().collect::<Vec<_>>();
                        match registered.as_slice() {
                            [] => bail!(
                                "no YubiKeys are registered; complete the external setup and run \
                                 `tvc keys refresh-yubikey` first"
                            ),
                            [sole] => *sole,
                            _ => prompts::select("YubiKey to use as the operator", registered)?,
                        }
                    }
                };

                NewOrgOperatorPlan::RegisteredYubikey(serial)
            }
        }
    };

    Ok(OrgPlan::New {
        id,
        alias,
        operator,
    })
}

/// Prompt for an alias to bind to `org_id`, enforcing the binding policy:
/// UUID-shaped names are refused outright — org queries parse UUID-shaped
/// input as an organization ID, so such a name could never be looked up —
/// and re-pointing a name away from another organization requires an
/// explicit confirmation.
fn prompt_for_alias_binding(ctx: &mut StdCtx, config: &Config, org_id: Uuid) -> Result<String> {
    let alias = prompts::text("Organization alias", Some("default"))?;

    if Uuid::parse_str(&alias).is_ok() {
        bail!(
            "Alias '{alias}' is UUID-shaped, so it would always resolve as an \
             organization ID instead of a name. Choose a non-UUID alias."
        );
    }

    if let Some(existing) = config.aliases.resolve(&alias) {
        // Already naming this organization is a no-op, not a re-point.
        if existing.id() != org_id {
            shell_eprintln!(
                ctx,
                "Alias '{alias}' currently names organization '{}'.",
                existing.id()
            )?;
            prompts::confirm_or_bail(
                &format!("Re-point alias '{alias}' to organization '{org_id}'?"),
                "alias re-pointing",
            )?;
        }
    }

    Ok(alias)
}

enum OrgChoice {
    Existing(OrgRow),
    New,
}

impl Display for OrgChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OrgChoice::Existing(row) => row.fmt(f),
            OrgChoice::New => write!(f, "[new] Add a new organization"),
        }
    }
}

fn new_org_api_base_url(api_base_url_override: Option<&str>) -> String {
    api_base_url_override
        .unwrap_or(API_BASE_URL_PROD)
        .to_string()
}

fn update_api_base_url_from_override(
    config: &mut Config,
    org_id: Uuid,
    api_base_url_override: Option<&str>,
) {
    if let Some(api_base_url) = api_base_url_override {
        debug!(%org_id, %api_base_url, "updating organization API base URL from override");
        if let Some(org_config) = config.orgs.get_mut(&org_id) {
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
    local: &LocalOperatorRecord,
) -> Result<StoredQosOperatorKey> {
    debug!(operator_key_path = %local.key_path.display(), "resolving operator key");

    if let Some(operator_key) = StoredQosOperatorKey::load(&local.key_path).await? {
        debug!("using existing operator key");
        shell_println!(ctx, "Using existing operator key.")?;
        shell_println!(ctx, "Tip: back it up with `tvc keys backup-operator-key`.")?;
        return Ok(operator_key);
    }

    debug!("generating new operator key");
    shell_println!(ctx)?;
    shell_println!(ctx, "Generating operator key...")?;

    let pair =
        P256Pair::generate().map_err(|e| anyhow!("failed to generate operator key: {e:?}"))?;
    let public_key = QosOperatorPublicKey::try_from(pair.public_key().to_bytes().as_slice())
        .context("generated operator public key")?;

    let operator_key = StoredQosOperatorKey {
        public_key,
        private_key: hex::encode(pair.to_master_seed()),
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

    // Onboarding nudge for the freshly generated key. JSON mode already
    // forces non-interactive; the TTY check keeps piped runs from hanging on
    // the prompt.
    if !ctx.is_non_interactive() && prompts::stdin_can_prompt() {
        shell_println!(ctx)?;
        shell_println!(
            ctx,
            "WARNING: This key exists only on this machine; if it's lost you \
             cannot approve deployments with it."
        )?;

        // Everything below is advisory: a prompt the user escapes out of and a
        // backup that fails both degrade to a warning, because the config and
        // both key files are already saved by this point and the login outcome
        // must still land.
        let attempt: Result<Option<OperatorKeyBackedUp>> = async {
            if !prompts::confirm("Back up your operator key now?", true)? {
                return Ok(None);
            }

            let Some(destination) = prompt_for_backup_destination(org_alias)? else {
                return Ok(None);
            };

            back_up(org_alias.to_string(), local.key_path.clone(), destination)
                .await
                .map(Some)
        }
        .await;

        let backed_up = match attempt {
            Ok(report) => report,
            Err(error) => {
                shell_eprintln!(ctx, "WARNING: backup skipped: {error:#}")?;
                None
            }
        };

        if let Some(report) = backed_up {
            shell_println!(ctx)?;
            shell_println!(ctx, "{report}")?;
        } else {
            shell_println!(
                ctx,
                "You can back up any time with `tvc keys backup-operator-key`."
            )?;
        }
    }

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
    org_id: Uuid,
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
    /// The alias the user logged in with, when they used one.
    alias: Option<String>,
    api_public_key: String,
    config_file_path: String,
    api_key_path: String,
    #[serde(flatten)]
    operator: LoggedInOperator,
}

/// The operator backend the login landed on: a local login reports its key
/// file, a hosted login reports the registered identity, a yubikey login
/// reports the device serial. All carry the qos composite public key.
#[derive(Serialize)]
#[serde(
    tag = "operatorKind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum LoggedInOperator {
    Local {
        operator_public_key: QosOperatorPublicKey,
        operator_key_path: String,
    },
    Hosted {
        operator_name: String,
        operator_id: Uuid,
        operator_public_key: String,
    },
    Yubikey {
        operator_name: String,
        serial: YubiKeySerial,
        operator_public_key: QosOperatorPublicKey,
    },
}

/// Local, like every org `login` itself creates; exists for [`LoggedIn`]'s
/// `Default`, which the outcome payload-enumeration tests construct.
impl Default for LoggedInOperator {
    fn default() -> Self {
        Self::Local {
            operator_public_key: QosOperatorPublicKey::default(),
            operator_key_path: String::new(),
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDeleted {
    aliases: Vec<String>,
    organization_id: Uuid,
    removed_key_directory: Option<String>,
    /// Serials of the YubiKey operators the profile referenced. The devices
    /// and their registry entries are kept; other profiles may share them.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    retained_yubikey_serials: Vec<YubiKeySerial>,
    dashboard_url: String,
    api_public_key: Option<String>,
}

impl Display for ProfileDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let names = if self.aliases.is_empty() {
            String::new()
        } else {
            format!(" ('{}')", self.aliases.join("', '"))
        };
        let mut lines = vec![format!(
            "Deleted the login for organization {}{names}.",
            self.organization_id
        )];

        if let Some(directory) = &self.removed_key_directory {
            lines.push(format!("Removed key directory: {directory}"));
        }

        if !self.retained_yubikey_serials.is_empty() {
            let serials: Vec<String> = self
                .retained_yubikey_serials
                .iter()
                .map(ToString::to_string)
                .collect();

            lines.push(format!(
                "Kept the YubiKey registry entries (serials {}); after removing every \
                 profile reference, forget them locally with `tvc yubikey unregister`.",
                serials.join(", ")
            ));
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
            None => lines.push("  2. Delete the API key associated with this login".to_string()),
        }

        f.write_str(&lines.join("\n"))
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
  API public key:        {}"#,
            self.organization_name,
            self.organization_id,
            self.username,
            self.user_id,
            self.alias.as_deref().unwrap_or(&self.organization_id),
            self.api_public_key,
        )?;

        match &self.operator {
            LoggedInOperator::Local {
                operator_public_key,
                operator_key_path,
            } => write!(
                f,
                r#"
  Operator public key:   {operator_public_key}

Saved to
  Config file:    {}
  API key:        {}
  Operator key:   {operator_key_path}"#,
                self.config_file_path, self.api_key_path,
            ),
            LoggedInOperator::Hosted {
                operator_name,
                operator_id,
                operator_public_key,
            } => write!(
                f,
                r#"
  Operator public key:   {operator_public_key}
  Hosted operator:       {operator_name} ({operator_id})

Saved to
  Config file:    {}
  API key:        {}"#,
                self.config_file_path, self.api_key_path,
            ),
            LoggedInOperator::Yubikey {
                operator_name,
                serial,
                operator_public_key,
            } => write!(
                f,
                r#"
  Operator public key:   {operator_public_key}
  YubiKey operator:      {operator_name} (serial {serial})

Saved to
  Config file:    {}
  API key:        {}"#,
                self.config_file_path, self.api_key_path,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        API_BASE_URL_DEV, API_BASE_URL_PREPROD, DASHBOARD_URL_DEV, DASHBOARD_URL_PREPROD,
        DASHBOARD_URL_PROD, OperatorKind, OperatorRecord,
    };
    use std::path::PathBuf;

    const OVERRIDE_URL: &str = "http://127.0.0.1:8081";

    fn logged_in_with(operator: LoggedInOperator) -> LoggedIn {
        LoggedIn {
            organization_name: "Org".to_string(),
            organization_id: "org-1".to_string(),
            username: "user".to_string(),
            user_id: "user-1".to_string(),
            alias: Some("prod".to_string()),
            api_public_key: "api-key".to_string(),
            config_file_path: "/config/tvc.config.toml".to_string(),
            api_key_path: "/keys/api_key.json".to_string(),
            operator,
        }
    }

    /// The local outcome keeps its pre-hosted JSON fields and gains only the
    /// additive `operatorKind` tag — this is a compatibility contract.
    #[test]
    fn logged_in_local_json_reports_the_key_file() {
        let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
        let logged_in = logged_in_with(LoggedInOperator::Local {
            operator_public_key: composite.parse().unwrap(),
            operator_key_path: "/keys/operator.json".to_string(),
        });

        assert_eq!(
            serde_json::to_value(&logged_in).unwrap(),
            serde_json::json!({
                "organizationName": "Org",
                "organizationId": "org-1",
                "username": "user",
                "userId": "user-1",
                "alias": "prod",
                "apiPublicKey": "api-key",
                "configFilePath": "/config/tvc.config.toml",
                "apiKeyPath": "/keys/api_key.json",
                "operatorKind": "local",
                "operatorPublicKey": composite,
                "operatorKeyPath": "/keys/operator.json",
            })
        );
    }

    /// The hosted outcome reports the registered identity and no key path.
    #[test]
    fn logged_in_hosted_json_reports_the_registered_identity() {
        let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
        let logged_in = logged_in_with(LoggedInOperator::Hosted {
            operator_name: "hosted-op".to_string(),
            operator_id: Uuid::from_u128(0x11),
            operator_public_key: composite.clone(),
        });

        assert_eq!(
            serde_json::to_value(&logged_in).unwrap(),
            serde_json::json!({
                "organizationName": "Org",
                "organizationId": "org-1",
                "username": "user",
                "userId": "user-1",
                "alias": "prod",
                "apiPublicKey": "api-key",
                "configFilePath": "/config/tvc.config.toml",
                "apiKeyPath": "/keys/api_key.json",
                "operatorKind": "hosted",
                "operatorName": "hosted-op",
                "operatorId": Uuid::from_u128(0x11).to_string(),
                "operatorPublicKey": composite,
            })
        );
    }

    /// The yubikey outcome reports the device serial and no key path.
    #[test]
    fn logged_in_yubikey_json_reports_the_device_serial() {
        let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
        let logged_in = logged_in_with(LoggedInOperator::Yubikey {
            operator_name: "yubikey-op".to_string(),
            serial: YubiKeySerial::from(0x01c9_5c1f),
            operator_public_key: composite.parse().unwrap(),
        });

        assert_eq!(
            serde_json::to_value(&logged_in).unwrap(),
            serde_json::json!({
                "organizationName": "Org",
                "organizationId": "org-1",
                "username": "user",
                "userId": "user-1",
                "alias": "prod",
                "apiPublicKey": "api-key",
                "configFilePath": "/config/tvc.config.toml",
                "apiKeyPath": "/keys/api_key.json",
                "operatorKind": "yubikey",
                "operatorName": "yubikey-op",
                "serial": "01c95c1f",
                "operatorPublicKey": composite,
            })
        );
    }

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

        update_api_base_url_from_override(&mut config, Uuid::from_u128(1), None);

        assert_eq!(
            config.orgs[&Uuid::from_u128(1)].api_base_url,
            "http://existing.example"
        );
    }

    #[test]
    fn explicit_override_updates_existing_org_api_base_url() {
        let mut config = config_with_org(API_BASE_URL_PROD);

        update_api_base_url_from_override(&mut config, Uuid::from_u128(1), Some(OVERRIDE_URL));

        assert_eq!(config.orgs[&Uuid::from_u128(1)].api_base_url, OVERRIDE_URL);
    }

    fn config_with_org(api_base_url: &str) -> Config {
        let mut config = Config::default();
        config.orgs.insert(
            Uuid::from_u128(1),
            OrgConfig {
                api_key_path: PathBuf::from("api_key.json"),
                api_base_url: api_base_url.to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: vec![OperatorRecord::local(PathBuf::from("operator.json"))],
                extra: toml::Table::new(),
            },
        );
        config
            .aliases
            .bind("default".to_string(), Uuid::from_u128(1));
        config.set_active_org(Uuid::from_u128(1)).unwrap();
        config
    }
}
