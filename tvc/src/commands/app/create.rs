//! App create command - creates an app from a config file.

use crate::{
    client::build_client,
    config::{
        app::{AppConfig, AppConfigValidationErrors, OperatorSetParams},
        turnkey::{self, StoredQosOperatorKey},
    },
    outcome::Outcome,
    output::{Ctx, StdCtx},
    prompts, shell_println,
};
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    io,
    path::{Path, PathBuf},
};
use tracing::debug;
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_APP_CONFIG")]
    pub config_file: PathBuf,

    /// Create a new operator instead of reusing the most recently created one.
    ///
    /// By default `app create` reuses the operator from your last `app create`
    /// (the same local operator key) rather than minting a new operator ID each
    /// time. Pass this to force creating a new operator.
    #[arg(long, env = "TVC_NO_OPERATOR_REUSE")]
    pub no_operator_reuse: bool,

    #[command(flatten)]
    overrides: Overrides,
}

#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
struct Overrides {
    /// Permit debug-mode deployments for this app. Debug-mode deployments expose
    /// secure-enclave logs and emit zero'd attestation PCRs, so remote
    /// attestation cannot succeed. Cannot be changed after app creation; setting
    /// this true means the app's quorum key is considered permanently insecure.
    #[arg(long, env = "TVC_DANGEROUS_ENABLE_DEBUG_MODE_DEPLOYMENTS")]
    pub dangerous_enable_debug_mode_deployments: bool,
}

pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let config = if ctx.is_non_interactive() {
        build_app_config_non_interactive(&args).await?
    } else {
        build_app_config_interactive(ctx, &args).await?
    };

    let mut app_config = apply_overrides(config, &args.overrides);

    // Reuse the previously-created operator by default so repeated `app create`
    // runs don't mint a fresh operator ID for the same local key. The decision
    // itself is pure (`decide_operator_reuse`); this endpoint does the I/O
    // (loading saved IDs) and adapts multi-candidate handling to the mode.
    let saved_ids = load_saved_operator_ids().await;
    match decide_operator_reuse(
        args.no_operator_reuse,
        app_config.manifest_set_params.as_ref(),
        &saved_ids,
    ) {
        OperatorReuse::KeepConfig => {}
        OperatorReuse::Reuse(id) => apply_operator_reuse(ctx, &mut app_config, id)?,
        OperatorReuse::MultipleCandidates(ids) => {
            if ctx.is_non_interactive() {
                bail!(
                    "multiple saved operator IDs for the active org; \
                     set manifestSetParams.existingOperatorIds in your config to reuse one, \
                     or pass --no-operator-reuse to create a new operator"
                );
            }
            let id = prompts::select("Select operator to reuse", ids)?;
            apply_operator_reuse(ctx, &mut app_config, id)?;
        }
    }

    // Key-collision guards (after reuse resolution, which may have cleared
    // new_operators): duplicated keys mint distinct operator IDs backed by
    // identical keys. Detection is pure; this endpoint resolves per mode.
    let saved_key = load_saved_operator_public_key().await;

    if let Some(SavedKeyCollision {
        operator_names,
        existing_ids,
    }) = find_saved_key_collision(
        app_config.manifest_set_params.as_ref(),
        saved_key.as_deref(),
        &saved_ids,
    ) {
        let names = operator_names.join(", ");

        if ctx.is_non_interactive() {
            let ids = existing_ids.join(", ");
            bail!(
                r#"manifestSetParams.newOperators entries [{names}] use the saved local operator key, which already backs operator ID(s) [{ids}].
Creating the app would mint another operator ID with identical keys.
Set manifestSetParams.existingOperatorIds to reuse an existing operator, use a distinct newOperators publicKey to create a new one, or rerun interactively to choose."#
            );
        }

        shell_println!(
            ctx,
            "Operator(s) [{names}] in manifestSetParams use the saved local operator key, which already backs existing operator(s)."
        )?;

        let mut options: Vec<SavedKeyResolution> = existing_ids
            .into_iter()
            .map(SavedKeyResolution::Reuse)
            .collect();
        options.push(SavedKeyResolution::CreateAnyway);
        options.push(SavedKeyResolution::Cancel);

        match prompts::select(
            "A newOperators key already backs existing operator(s). How do you want to proceed?",
            options,
        )? {
            SavedKeyResolution::Reuse(id) => apply_operator_reuse(ctx, &mut app_config, id)?,
            SavedKeyResolution::CreateAnyway => {}
            SavedKeyResolution::Cancel => bail!("operation cancelled by user: app creation"),
        }
    }

    // Within-set duplicates, checked second (a Reuse choice above clears the
    // manifest set's new_operators). Only sets the user wrote are checked; the
    // built-in default share set is guarded by a unit test instead.
    for (set, params) in [
        ("manifestSetParams", app_config.manifest_set_params.as_ref()),
        ("shareSetParams", app_config.share_set_params.as_ref()),
    ] {
        let Some(params) = params else { continue };

        for group in find_duplicate_new_operator_keys(params) {
            let names = group.join(", ");

            if ctx.is_non_interactive() {
                bail!(
                    r#"{set}.newOperators entries [{names}] share the same public key.
Each entry would become a separate operator ID backed by identical keys.
Give each {set}.newOperators entry a distinct publicKey, or replace duplicates with {set}.existingOperatorIds."#
                );
            }

            prompts::confirm_or_bail(
                &format!(
                    "{set}.newOperators entries [{names}] share the same public key; each will become a separate operator with identical keys. Create anyway?"
                ),
                "app creation",
            )?;
        }
    }

    run_with_config(ctx, args, app_config).await
}

/// What to do with the manifest set's operators at create time.
#[derive(Debug, PartialEq, Eq)]
enum OperatorReuse {
    /// Leave the config as-is: create new operators, or honor an explicit config.
    KeepConfig,
    /// Reuse exactly this saved operator ID.
    Reuse(String),
    /// Several saved operator IDs are candidates; the caller picks one.
    MultipleCandidates(Vec<String>),
}

/// Decide whether to reuse a previously-created operator for the manifest set.
///
/// Pure and mode-agnostic: it performs no I/O and knows nothing about
/// interactivity. The caller resolves [`OperatorReuse::MultipleCandidates`] by
/// prompting (interactive) or erroring (non-interactive).
fn decide_operator_reuse(
    no_reuse: bool,
    manifest_set_params: Option<&OperatorSetParams>,
    saved_ids: &[String],
) -> OperatorReuse {
    // Opt-out: the user explicitly wants a fresh operator.
    if no_reuse {
        return OperatorReuse::KeepConfig;
    }
    // Nothing to override: reusing a whole set by id, or no manifest params.
    let Some(params) = manifest_set_params else {
        return OperatorReuse::KeepConfig;
    };
    // Config already reuses operators explicitly; respect it (also the
    // non-interactive escape hatch for picking a specific operator).
    if !params.existing_operator_ids.is_empty() {
        return OperatorReuse::KeepConfig;
    }
    match saved_ids {
        [] => OperatorReuse::KeepConfig,
        [id] => OperatorReuse::Reuse(id.clone()),
        _ => OperatorReuse::MultipleCandidates(saved_ids.to_vec()),
    }
}

/// Swap the manifest set from creating new operators to reusing `operator_id`.
fn apply_operator_reuse<Out: io::Write, Err: io::Write>(
    ctx: &mut Ctx<Out, Err>,
    config: &mut AppConfig,
    operator_id: String,
) -> anyhow::Result<()> {
    if let Some(params) = config.manifest_set_params.as_mut() {
        params.new_operators.clear();
        params.existing_operator_ids = vec![operator_id.clone()];
    }

    debug!(operator_id = %operator_id, "reusing existing operator");

    shell_println!(
        ctx,
        "Reusing operator {operator_id} (pass --no-operator-reuse to create a new one)"
    )
}

/// Best-effort load of the active org's most recently created operator IDs.
/// Reuse is a convenience, so config-load failures fall back to no reuse; the
/// real error surfaces later when `run_with_config` reloads the config.
async fn load_saved_operator_ids() -> Vec<String> {
    match turnkey::Config::load().await {
        Ok(config) => config.get_last_operator_ids().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Canonical form for comparing operator public keys from configs and local
/// key files: trimmed, ASCII-lowercased hex. A string comparison rather than a
/// parse — app-config keys carry no format guarantee, and a non-key string
/// simply never matches a real key.
fn canonical_operator_key(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

/// Manifest-set newOperators entries whose key already backs known operators.
#[derive(Debug, PartialEq, Eq)]
struct SavedKeyCollision {
    /// Names of the newOperators entries using the saved local operator key.
    operator_names: Vec<String>,
    /// Operator IDs already backed by that key: config existingOperatorIds
    /// first, then last-run IDs, deduplicated preserving order.
    existing_ids: Vec<String>,
}

/// Detect manifest-set newOperators entries that reuse the saved local
/// operator key when operator IDs backed by it are already known.
///
/// Pure and mode-agnostic: the caller prompts to choose (interactive) or
/// errors with remediation (non-interactive). Returns `None` when there is no
/// saved key, no known IDs, or no matching entry — a first run that filled the
/// saved key into a fresh config stays untouched.
fn find_saved_key_collision(
    manifest_set_params: Option<&OperatorSetParams>,
    saved_public_key: Option<&str>,
    saved_operator_ids: &[String],
) -> Option<SavedKeyCollision> {
    let params = manifest_set_params?;
    let saved_key = canonical_operator_key(saved_public_key?);

    let operator_names: Vec<String> = params
        .new_operators
        .iter()
        .filter(|operator| canonical_operator_key(&operator.public_key) == saved_key)
        .map(|operator| operator.name.clone())
        .collect();

    if operator_names.is_empty() {
        return None;
    }

    let mut existing_ids: Vec<String> = Vec::new();

    for id in params
        .existing_operator_ids
        .iter()
        .chain(saved_operator_ids)
    {
        if !existing_ids.contains(id) {
            existing_ids.push(id.clone());
        }
    }

    if existing_ids.is_empty() {
        return None;
    }

    Some(SavedKeyCollision {
        operator_names,
        existing_ids,
    })
}

/// Groups of newOperators names (within one set) sharing an identical public
/// key, in first-occurrence order. Each group would mint multiple operator IDs
/// backed by the same key.
fn find_duplicate_new_operator_keys(params: &OperatorSetParams) -> Vec<Vec<String>> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();

    for operator in &params.new_operators {
        let key = canonical_operator_key(&operator.public_key);

        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, names)) => names.push(operator.name.clone()),
            None => groups.push((key, vec![operator.name.clone()])),
        }
    }

    groups
        .into_iter()
        .filter_map(|(_, names)| (names.len() > 1).then_some(names))
        .collect()
}

/// How the user chose to resolve a saved-key collision.
enum SavedKeyResolution {
    /// Reuse this existing operator ID instead of creating a new operator.
    Reuse(String),
    /// Create a new operator ID backed by the same key anyway.
    CreateAnyway,
    /// Cancel app creation.
    Cancel,
}

impl Display for SavedKeyResolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reuse(id) => write!(f, "Reuse existing operator {id} (do not create a new one)"),
            Self::CreateAnyway => f.write_str("Create a new operator ID backed by the same key"),
            Self::Cancel => f.write_str("Cancel app creation"),
        }
    }
}

async fn build_app_config_interactive(ctx: &mut StdCtx, args: &Args) -> Result<AppConfig> {
    let mut config = match read_app_config_file_bytes(&args.config_file).await {
        Ok(bytes) => parse_app_config(&bytes, &args.config_file)?,
        Err(_) => AppConfig::template(None),
    };

    let mut changed = false;
    loop {
        match config.validate() {
            Ok(()) => break,
            Err(errors) if errors.has_non_placeholder_error() => {
                return Err(invalid_app_config_error(&args.config_file, errors));
            }
            _ => {
                changed = true;
                let saved_operator_public_key = load_saved_operator_public_key().await;
                config.fill_interactively(saved_operator_public_key.as_deref())?;
            }
        }
    }

    if changed {
        offer_to_save_app_config(ctx, &args.config_file, &config)?;
    }

    Ok(config)
}

async fn build_app_config_non_interactive(args: &Args) -> Result<AppConfig> {
    let bytes = read_app_config_file_bytes(&args.config_file).await?;
    let config = parse_app_config(&bytes, &args.config_file)?;

    if let Err(errors) = config.validate() {
        return Err(invalid_app_config_error(&args.config_file, errors));
    }

    Ok(config)
}

async fn read_app_config_file_bytes(path: &Path) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config file: {}", path.display()))
}

fn parse_app_config(content: &str, path: &Path) -> Result<AppConfig> {
    serde_json::from_str(content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))
}

fn invalid_app_config_error(path: &Path, errors: AppConfigValidationErrors) -> anyhow::Error {
    anyhow!("invalid config file: {}: {}", path.display(), errors)
}

fn offer_to_save_app_config(ctx: &mut StdCtx, path: &Path, config: &AppConfig) -> Result<()> {
    let save = prompts::confirm(&format!("Save filled config to {}?", path.display()), true)?;
    if save {
        let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        shell_println!(ctx, "Wrote {}", path.display())?;
    }
    Ok(())
}

/// Best-effort load of the operator public key from the active org's config
/// so we can offer it as the default for new-operator prompts.
async fn load_saved_operator_public_key() -> Option<String> {
    let config = turnkey::Config::load().await.ok()?;
    let (alias, org_config) = config.active_org_config()?;
    let local = org_config.select_local_record(alias).ok()?;
    let operator_key = StoredQosOperatorKey::load(&local.key_path).await.ok()??;
    Some(operator_key.public_key)
}

async fn run_with_config(ctx: &mut StdCtx, args: Args, app_config: AppConfig) -> Result<Outcome> {
    shell_println!(ctx, "Creating app '{}'...", app_config.name)?;

    let auth = build_client().await?;

    let intent = build_create_tvc_app_intent(&app_config);

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .create_tvc_app(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC app")?;

    let app_id = result.result.app_id;
    let operator_ids = result.result.manifest_set_operator_ids;

    let mut config = turnkey::Config::load().await?;
    config.set_last_app_id(&app_id)?;
    config.set_last_operator_ids(&operator_ids)?;
    config.save().await?;

    Ok(Outcome::AppCreated(AppCreated {
        app_id,
        name: app_config.name,
        manifest_set_id: result.result.manifest_set_id,
        manifest_set_operator_ids: operator_ids,
        config_path: args.config_file.display().to_string(),
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCreated {
    app_id: String,
    name: String,
    manifest_set_id: String,
    manifest_set_operator_ids: Vec<String>,
    config_path: String,
}

impl Display for AppCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
App created successfully!

App ID: {}
Name: {}
Manifest Set ID: {}"#,
            self.app_id, self.name, self.manifest_set_id
        )?;

        if !self.manifest_set_operator_ids.is_empty() {
            write!(
                f,
                "\nManifest Set Operator IDs: {}",
                self.manifest_set_operator_ids.join(", ")
            )?;
        }

        write!(
            f,
            r#"
Config: {}

Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`"#,
            self.config_path
        )
    }
}

fn build_create_tvc_app_intent(app_config: &AppConfig) -> CreateTvcAppIntent {
    let share_set_params = app_config.effective_share_set_params();

    CreateTvcAppIntent {
        name: app_config.name.clone(),
        quorum_public_key: app_config.quorum_public_key.clone(),
        manifest_set_id: app_config.manifest_set_id.clone(),
        manifest_set_params: app_config
            .manifest_set_params
            .as_ref()
            .map(to_tvc_operator_set_params),
        share_set_id: app_config.share_set_id.clone(),
        share_set_params: share_set_params.as_ref().map(to_tvc_operator_set_params),
        enable_egress: app_config.enable_egress.into(),
        enable_debug_mode_deployments: app_config.dangerous_enable_debug_mode_deployments.into(),
    }
}

fn apply_overrides(mut config: AppConfig, overrides: &Overrides) -> AppConfig {
    if overrides.dangerous_enable_debug_mode_deployments {
        config.dangerous_enable_debug_mode_deployments =
            overrides.dangerous_enable_debug_mode_deployments;
    }
    config
}

fn to_tvc_operator_set_params(params: &OperatorSetParams) -> TvcOperatorSetParams {
    TvcOperatorSetParams {
        name: params.name.clone(),
        threshold: params.threshold,
        new_operators: params
            .new_operators
            .iter()
            .map(|o| TvcOperatorParams {
                name: o.name.clone(),
                public_key: o.public_key.clone(),
            })
            .collect(),
        existing_operator_ids: params.existing_operator_ids.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::app::{KNOWN_QUORUM_KEY, OperatorParams},
        output::{Ctx, EmptyShell},
    };

    fn valid_config() -> AppConfig {
        AppConfig {
            name: "test-app".to_string(),
            quorum_public_key: KNOWN_QUORUM_KEY.to_string(),
            enable_egress: false,
            manifest_set_id: None,
            manifest_set_params: Some(OperatorSetParams {
                name: "manifest-set".to_string(),
                threshold: 1,
                new_operators: vec![OperatorParams {
                    name: "manifest-operator".to_string(),
                    public_key: "manifest-public-key".to_string(),
                }],
                existing_operator_ids: vec![],
            }),
            share_set_id: None,
            share_set_params: None,
            dangerous_enable_debug_mode_deployments: false,
        }
    }

    #[test]
    fn build_intent_uses_default_share_set_params_when_omitted() {
        let intent = build_create_tvc_app_intent(&valid_config());
        let share_set_params = intent.share_set_params.unwrap();

        assert_eq!(share_set_params.name, "dev-known-share-set");
        assert_eq!(share_set_params.threshold, 2);
        assert_eq!(share_set_params.new_operators.len(), 2);
        assert!(share_set_params.existing_operator_ids.is_empty());
    }

    #[test]
    fn build_intent_sends_enable_egress() {
        let mut config = valid_config();
        config.enable_egress = true;

        let intent = build_create_tvc_app_intent(&config);

        assert_eq!(intent.enable_egress, Some(true));
    }

    #[test]
    fn build_intent_uses_custom_share_set_params_when_configured() {
        let mut config = valid_config();
        config.share_set_params = Some(OperatorSetParams {
            name: "custom-share-set".to_string(),
            threshold: 2,
            new_operators: vec![OperatorParams {
                name: "share-operator".to_string(),
                public_key: "share-public-key".to_string(),
            }],
            existing_operator_ids: vec!["existing-operator-id".to_string()],
        });

        let intent = build_create_tvc_app_intent(&config);
        let share_set_params = intent.share_set_params.unwrap();

        assert_eq!(share_set_params.name, "custom-share-set");
        assert_eq!(share_set_params.threshold, 2);
        assert_eq!(share_set_params.new_operators[0].name, "share-operator");
        assert_eq!(
            share_set_params.existing_operator_ids,
            vec!["existing-operator-id".to_string()]
        );
    }

    /// Default config has debug-mode disabled, and the intent reports `false`
    /// — explicit so the server doesn't have to fall back to a proto default.
    #[test]
    fn build_intent_sends_false_debug_mode_by_default() {
        let intent = build_create_tvc_app_intent(&valid_config());
        assert_eq!(intent.enable_debug_mode_deployments, Some(false));
    }

    /// An explicit `dangerousEnableDebugModeDeployments: true` in the config flows into
    /// the intent so the server records the app's debug-mode capability.
    #[test]
    fn build_intent_forwards_debug_mode_from_config() {
        let mut config = valid_config();
        config.dangerous_enable_debug_mode_deployments = true;

        let intent = build_create_tvc_app_intent(&config);
        assert_eq!(intent.enable_debug_mode_deployments, Some(true));
    }

    /// CLI flag flips a default `false` config to `true` — the user opted in
    /// via the command line rather than the config file.
    #[test]
    fn dangerous_flag_enables_debug_mode_when_config_unset() {
        let config = valid_config();
        let overrides = Overrides {
            dangerous_enable_debug_mode_deployments: true,
        };

        let config = apply_overrides(config, &overrides);
        assert!(config.dangerous_enable_debug_mode_deployments);
    }

    /// Omitting the CLI flag must NOT override a config that enables debug-mode
    /// deployments: the flag is opt-in only and can never turn it off, so a
    /// `true` config survives an absent flag.
    #[test]
    fn absent_dangerous_flag_preserves_config_debug_mode() {
        let mut config = valid_config();
        config.dangerous_enable_debug_mode_deployments = true;
        let overrides = Overrides {
            dangerous_enable_debug_mode_deployments: false,
        };

        let config = apply_overrides(config, &overrides);
        assert!(config.dangerous_enable_debug_mode_deployments);
    }

    /// Exercises every override flag via clap parsing so flag renames or
    /// removals fail this test. The other override tests construct `Args` by
    /// field name and would silently pass.
    #[test]
    fn every_override_flag_changes_config_value() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            args: Args,
        }

        let config_path = "/tmp/test-app.json";
        let args = TestCli::try_parse_from([
            "tvc-app-create",
            "--config-file",
            config_path,
            "--dangerous-enable-debug-mode-deployments",
        ])
        .unwrap()
        .args;

        let baseline = valid_config();
        let resolved = apply_overrides(valid_config(), &args.overrides);

        // Each override moved off its config default ...
        assert_ne!(
            resolved.dangerous_enable_debug_mode_deployments,
            baseline.dangerous_enable_debug_mode_deployments
        );

        // ... to the value passed on the CLI.
        assert!(resolved.dangerous_enable_debug_mode_deployments);

        // config_file isn't an override; verify clap captured the path.
        assert_eq!(args.config_file, PathBuf::from(config_path));
    }

    #[test]
    fn build_intent_uses_share_set_id_when_configured() {
        let mut config = valid_config();
        config.share_set_id = Some("share-set-id".to_string());

        let intent = build_create_tvc_app_intent(&config);

        assert_eq!(intent.share_set_id.as_deref(), Some("share-set-id"));
        assert!(intent.share_set_params.is_none());
    }

    fn manifest_params_with_new_operator() -> OperatorSetParams {
        OperatorSetParams {
            name: "manifest-set".to_string(),
            threshold: 1,
            new_operators: vec![OperatorParams {
                name: "operator-1".to_string(),
                public_key: "operator-public-key".to_string(),
            }],
            existing_operator_ids: vec![],
        }
    }

    /// The opt-out flag always wins: never reuse, even with a single saved id.
    #[test]
    fn decide_reuse_keeps_config_when_flag_set() {
        let params = manifest_params_with_new_operator();
        let saved = vec!["op-1".to_string()];
        assert_eq!(
            decide_operator_reuse(true, Some(&params), &saved),
            OperatorReuse::KeepConfig
        );
    }

    /// First run (nothing created yet) has nothing to reuse -> create new.
    #[test]
    fn decide_reuse_keeps_config_without_saved_ids() {
        let params = manifest_params_with_new_operator();
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &[]),
            OperatorReuse::KeepConfig
        );
    }

    /// A config that already pins existingOperatorIds is respected as-is.
    #[test]
    fn decide_reuse_keeps_config_when_config_pins_existing_ids() {
        let mut params = manifest_params_with_new_operator();
        params.existing_operator_ids = vec!["explicit-op".to_string()];
        let saved = vec!["op-1".to_string(), "op-2".to_string()];
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &saved),
            OperatorReuse::KeepConfig
        );
    }

    /// No manifest_set_params (e.g. reusing a whole set via manifestSetId) -> nothing to do.
    #[test]
    fn decide_reuse_keeps_config_without_manifest_params() {
        let saved = vec!["op-1".to_string()];
        assert_eq!(
            decide_operator_reuse(false, None, &saved),
            OperatorReuse::KeepConfig
        );
    }

    /// The common case: exactly one saved operator -> reuse it.
    #[test]
    fn decide_reuse_reuses_single_saved_id() {
        let params = manifest_params_with_new_operator();
        let saved = vec!["op-1".to_string()];
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &saved),
            OperatorReuse::Reuse("op-1".to_string())
        );
    }

    /// Multiple saved operators are surfaced for the endpoint to prompt/bail on.
    #[test]
    fn decide_reuse_returns_candidates_for_multiple_saved_ids() {
        let params = manifest_params_with_new_operator();
        let saved = vec!["op-1".to_string(), "op-2".to_string()];
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &saved),
            OperatorReuse::MultipleCandidates(saved.clone())
        );
    }

    /// Applying reuse clears the would-be-created operators and pins the reused id.
    #[test]
    fn apply_operator_reuse_swaps_new_operators_for_existing_id() {
        let mut ctx = Ctx::new(EmptyShell::default(), true);
        let mut config = valid_config();
        apply_operator_reuse(&mut ctx, &mut config, "op-1".to_string()).unwrap();

        let params = config.manifest_set_params.unwrap();
        assert!(params.new_operators.is_empty());
        assert_eq!(params.existing_operator_ids, vec!["op-1".to_string()]);
    }

    const SAVED_KEY: &str = "04abcdef";

    fn params_with_operator_keys(operators: &[(&str, &str)]) -> OperatorSetParams {
        OperatorSetParams {
            name: "manifest-set".to_string(),
            threshold: 1,
            new_operators: operators
                .iter()
                .map(|(name, key)| OperatorParams {
                    name: name.to_string(),
                    public_key: key.to_string(),
                })
                .collect(),
            existing_operator_ids: vec![],
        }
    }

    /// A matching key with no known operator IDs is the first-run happy path
    /// (fill_interactively defaulted the saved key) -> no collision.
    #[test]
    fn saved_key_collision_requires_known_operator_ids() {
        let params = params_with_operator_keys(&[("op-a", SAVED_KEY)]);

        assert_eq!(
            find_saved_key_collision(Some(&params), Some(SAVED_KEY), &[]),
            None
        );
    }

    /// IDs saved from the last create run make a matching key a collision.
    #[test]
    fn saved_key_collision_fires_on_saved_last_run_ids() {
        let params = params_with_operator_keys(&[("op-a", SAVED_KEY)]);
        let saved = vec!["op-1".to_string()];

        assert_eq!(
            find_saved_key_collision(Some(&params), Some(SAVED_KEY), &saved),
            Some(SavedKeyCollision {
                operator_names: vec!["op-a".to_string()],
                existing_ids: vec!["op-1".to_string()],
            })
        );
    }

    /// Config pinning existingOperatorIds alongside a colliding newOperators
    /// entry counts as known IDs too.
    #[test]
    fn saved_key_collision_fires_on_config_existing_ids() {
        let mut params = params_with_operator_keys(&[("op-a", SAVED_KEY)]);
        params.existing_operator_ids = vec!["cfg-op".to_string()];

        assert_eq!(
            find_saved_key_collision(Some(&params), Some(SAVED_KEY), &[]),
            Some(SavedKeyCollision {
                operator_names: vec!["op-a".to_string()],
                existing_ids: vec!["cfg-op".to_string()],
            })
        );
    }

    /// Config IDs come first, then saved last-run IDs, deduplicated.
    #[test]
    fn saved_key_collision_merges_and_dedupes_id_sources() {
        let mut params = params_with_operator_keys(&[("op-a", SAVED_KEY)]);
        params.existing_operator_ids = vec!["cfg-op".to_string(), "op-1".to_string()];
        let saved = vec!["op-1".to_string(), "op-2".to_string()];

        let collision = find_saved_key_collision(Some(&params), Some(SAVED_KEY), &saved).unwrap();

        assert_eq!(collision.existing_ids, vec!["cfg-op", "op-1", "op-2"]);
    }

    /// A newOperators key that differs from the saved key is a genuinely new
    /// operator -> no collision.
    #[test]
    fn saved_key_collision_ignores_non_matching_key() {
        let params = params_with_operator_keys(&[("op-a", "04fedcba")]);
        let saved = vec!["op-1".to_string()];

        assert_eq!(
            find_saved_key_collision(Some(&params), Some(SAVED_KEY), &saved),
            None
        );
    }

    /// Without a saved local key there is nothing to compare against.
    #[test]
    fn saved_key_collision_requires_saved_key() {
        let params = params_with_operator_keys(&[("op-a", SAVED_KEY)]);
        let saved = vec!["op-1".to_string()];

        assert_eq!(find_saved_key_collision(Some(&params), None, &saved), None);
    }

    /// Key comparison canonicalizes hex case and surrounding whitespace.
    #[test]
    fn saved_key_collision_canonicalizes_case_and_whitespace() {
        let params = params_with_operator_keys(&[("op-a", "  04ABCDEF  ")]);
        let saved = vec!["op-1".to_string()];

        assert_eq!(
            find_saved_key_collision(Some(&params), Some(SAVED_KEY), &saved),
            Some(SavedKeyCollision {
                operator_names: vec!["op-a".to_string()],
                existing_ids: vec!["op-1".to_string()],
            })
        );
    }

    /// Entries sharing a key are grouped under the key's first occurrence.
    #[test]
    fn duplicate_keys_within_set_are_grouped_by_first_occurrence() {
        let params = params_with_operator_keys(&[("a", "04aa"), ("b", "04bb"), ("c", "04aa")]);

        assert_eq!(
            find_duplicate_new_operator_keys(&params),
            vec![vec!["a".to_string(), "c".to_string()]]
        );
    }

    /// Distinct keys never form a group.
    #[test]
    fn distinct_keys_produce_no_duplicate_groups() {
        let params = params_with_operator_keys(&[("a", "04aa"), ("b", "04bb")]);

        assert_eq!(
            find_duplicate_new_operator_keys(&params),
            Vec::<Vec<String>>::new()
        );
    }

    /// Duplicate detection canonicalizes hex case and whitespace.
    #[test]
    fn duplicate_detection_canonicalizes_case() {
        let params = params_with_operator_keys(&[("a", "04AA"), ("b", " 04aa ")]);

        assert_eq!(
            find_duplicate_new_operator_keys(&params),
            vec![vec!["a".to_string(), "b".to_string()]]
        );
    }

    /// The built-in dev share set must never trip the duplicate guard; this
    /// protects the default happy path if KNOWN_SHARE_SET_KEYS ever change.
    #[test]
    fn default_dev_share_set_has_no_duplicate_keys() {
        assert_eq!(
            find_duplicate_new_operator_keys(&AppConfig::share_set_params()),
            Vec::<Vec<String>>::new()
        );
    }
}
