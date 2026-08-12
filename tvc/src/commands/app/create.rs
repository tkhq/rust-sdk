//! App create command - creates an app from a config file.

use crate::{
    client::build_client,
    config::{
        app::{AppConfig, AppConfigValidationErrors, OperatorSetParams},
        turnkey,
    },
    operator::OperatorCandidate,
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
use tracing::{debug, instrument};
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_APP_CONFIG")]
    pub config_file: PathBuf,

    /// Create a new operator instead of reusing a known one.
    ///
    /// By default `app create` reuses a known operator — a registered hosted
    /// operator, or the one from your last `app create` — rather than minting
    /// a new operator ID each time. Pass this to force creating a new
    /// operator.
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

#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args, config: turnkey::Config) -> Result<Outcome> {
    let app_config = if ctx.is_non_interactive() {
        build_app_config_non_interactive(&args).await?
    } else {
        build_app_config_interactive(ctx, &args, &config).await?
    };

    let mut app_config = apply_overrides(app_config, &args.overrides);

    // Reuse a known operator by default so repeated `app create` runs don't
    // mint a fresh operator ID for the same key. The decision itself is pure
    // (`decide_operator_reuse`); this endpoint supplies the candidates and
    // adapts multi-candidate handling to the mode.
    let candidates = config.known_operator_candidates();

    match decide_operator_reuse(
        args.no_operator_reuse,
        app_config.manifest_set_params.as_ref(),
        &candidates,
    ) {
        OperatorReuse::KeepConfig => {}
        OperatorReuse::Reuse(operator) => apply_operator_reuse(ctx, &mut app_config, operator)?,
        OperatorReuse::MultipleCandidates(operators) => {
            // Picking a candidate needs a prompt, which is impossible under
            // --non-interactive, JSON mode, or a non-TTY stdin.
            let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

            if !can_prompt {
                bail!(
                    "multiple operator IDs are known for the active org; \
                     set manifestSetParams.existingOperatorIds in your config to reuse one, \
                     or pass --no-operator-reuse to create a new operator"
                );
            }

            let operator = prompts::select("Select operator to reuse", operators)?;
            apply_operator_reuse(ctx, &mut app_config, operator)?;
        }
    }

    run_with_app_config(ctx, args, config, app_config).await
}

/// What to do with the manifest set's operators at create time.
#[derive(Debug, PartialEq, Eq)]
enum OperatorReuse {
    /// Leave the config as-is: create new operators, or honor an explicit config.
    KeepConfig,
    /// Reuse exactly this known operator.
    Reuse(OperatorCandidate),
    /// Several known operators are candidates; the caller picks one.
    MultipleCandidates(Vec<OperatorCandidate>),
}

/// Decide whether to reuse a known operator for the manifest set.
///
/// Pure and mode-agnostic: it performs no I/O and knows nothing about
/// interactivity. The caller resolves [`OperatorReuse::MultipleCandidates`] by
/// prompting (interactive) or erroring (non-interactive).
fn decide_operator_reuse(
    no_reuse: bool,
    manifest_set_params: Option<&OperatorSetParams>,
    candidates: &[OperatorCandidate],
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
    match candidates {
        [] => OperatorReuse::KeepConfig,
        [sole] => OperatorReuse::Reuse(sole.clone()),
        _ => OperatorReuse::MultipleCandidates(candidates.to_vec()),
    }
}

/// Swap the manifest set from creating new operators to reusing `operator`.
fn apply_operator_reuse<Out: io::Write, Err: io::Write>(
    ctx: &mut Ctx<Out, Err>,
    config: &mut AppConfig,
    operator: OperatorCandidate,
) -> anyhow::Result<()> {
    debug!(operator_id = %operator.id, "reusing existing operator");

    shell_println!(
        ctx,
        "Reusing operator {operator} (pass --no-operator-reuse to create a new one)"
    )?;

    if let Some(params) = config.manifest_set_params.as_mut() {
        params.new_operators.clear();
        params.existing_operator_ids = vec![operator.id];
    }

    Ok(())
}

async fn build_app_config_interactive(
    ctx: &mut StdCtx,
    args: &Args,
    config: &turnkey::Config,
) -> Result<AppConfig> {
    let mut app_config = match read_app_config_file_bytes(&args.config_file).await {
        Ok(bytes) => parse_app_config(&bytes, &args.config_file)?,
        Err(_) => AppConfig::template(None),
    };

    let mut changed = false;
    loop {
        match app_config.validate() {
            Ok(()) => break,
            Err(errors) if errors.has_non_placeholder_error() => {
                return Err(invalid_app_config_error(&args.config_file, errors));
            }
            _ => {
                changed = true;
                let saved_operator_public_key = config.default_operator_public_key().await;
                app_config.fill_interactively(saved_operator_public_key.as_deref())?;
            }
        }
    }

    if changed {
        offer_to_save_app_config(ctx, &args.config_file, &app_config)?;
    }

    Ok(app_config)
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

async fn run_with_app_config(
    ctx: &mut StdCtx,
    args: Args,
    mut config: turnkey::Config,
    app_config: AppConfig,
) -> Result<Outcome> {
    shell_println!(ctx, "Creating app '{}'...", app_config.name)?;

    let auth = build_client(&config).await?;
    // PURE-DEPS-REVIEW T12 (low): inline clock read below; use the shared
    // timestamp_ms() helper (operator.rs).

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

    fn candidate(id: &str) -> OperatorCandidate {
        OperatorCandidate {
            id: id.to_string(),
            name: None,
        }
    }

    /// The opt-out flag always wins: never reuse, even with a single candidate.
    #[test]
    fn decide_reuse_keeps_config_when_flag_set() {
        let params = manifest_params_with_new_operator();
        assert_eq!(
            decide_operator_reuse(true, Some(&params), &[candidate("op-1")]),
            OperatorReuse::KeepConfig
        );
    }

    /// First run (nothing created or registered yet) has nothing to reuse -> create new.
    #[test]
    fn decide_reuse_keeps_config_without_candidates() {
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
        assert_eq!(
            decide_operator_reuse(
                false,
                Some(&params),
                &[candidate("op-1"), candidate("op-2")]
            ),
            OperatorReuse::KeepConfig
        );
    }

    /// No manifest_set_params (e.g. reusing a whole set via manifestSetId) -> nothing to do.
    #[test]
    fn decide_reuse_keeps_config_without_manifest_params() {
        assert_eq!(
            decide_operator_reuse(false, None, &[candidate("op-1")]),
            OperatorReuse::KeepConfig
        );
    }

    /// The common case: exactly one known operator -> reuse it.
    #[test]
    fn decide_reuse_reuses_the_sole_candidate() {
        let params = manifest_params_with_new_operator();
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &[candidate("op-1")]),
            OperatorReuse::Reuse(candidate("op-1"))
        );
    }

    /// Multiple known operators are surfaced for the endpoint to prompt/bail on.
    #[test]
    fn decide_reuse_returns_candidates_for_multiple_known_operators() {
        let params = manifest_params_with_new_operator();
        let candidates = [candidate("op-1"), candidate("op-2")];
        assert_eq!(
            decide_operator_reuse(false, Some(&params), &candidates),
            OperatorReuse::MultipleCandidates(candidates.to_vec())
        );
    }

    /// Applying reuse clears the would-be-created operators and pins the reused id.
    #[test]
    fn apply_operator_reuse_swaps_new_operators_for_existing_id() {
        let mut ctx = Ctx::new(EmptyShell::default(), true);
        let mut config = valid_config();
        apply_operator_reuse(&mut ctx, &mut config, candidate("op-1")).unwrap();

        let params = config.manifest_set_params.unwrap();
        assert!(params.new_operators.is_empty());
        assert_eq!(params.existing_operator_ids, vec!["op-1".to_string()]);
    }
}
