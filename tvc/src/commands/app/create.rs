//! App create command - creates an app from a config file.

use crate::{
    client::build_client,
    config::{
        app::{AppConfig, AppConfigValidationErrors, OperatorSetParams},
        turnkey::{self, StoredQosOperatorKey},
    },
    output::Ctx,
    prompts, shell_line,
};
use anyhow::{Context, Result, anyhow};
use clap::Args as ClapArgs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_APP_CONFIG")]
    pub config_file: PathBuf,

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

pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> Result<()> {
    let config = if ctx.is_non_interactive() {
        build_app_config_non_interactive(&args).await?
    } else {
        build_app_config_interactive(ctx, &args).await?
    };

    let app_config = apply_overrides(config, &args.overrides);
    run_with_config(ctx, args, app_config).await
}

async fn build_app_config_interactive<W: Write>(
    ctx: &mut Ctx<W>,
    args: &Args,
) -> Result<AppConfig> {
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

fn offer_to_save_app_config<W: Write>(
    ctx: &mut Ctx<W>,
    path: &Path,
    config: &AppConfig,
) -> Result<()> {
    let save = prompts::confirm(&format!("Save filled config to {}?", path.display()), true)?;
    if save {
        let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        shell_line!(ctx, "Wrote {}", path.display())?;
    }
    Ok(())
}

/// Best-effort load of the operator public key from the active org's config
/// so we can offer it as the default for new-operator prompts.
async fn load_saved_operator_public_key() -> Option<String> {
    let config = turnkey::Config::load().await.ok()?;
    let (_, org_config) = config.active_org_config()?;
    let operator_key = StoredQosOperatorKey::load(org_config).await.ok()??;
    Some(operator_key.public_key)
}

async fn run_with_config<W: Write>(
    ctx: &mut Ctx<W>,
    args: Args,
    app_config: AppConfig,
) -> Result<()> {
    shell_line!(ctx, "Creating app '{}'...", app_config.name)?;

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

    shell_line!(ctx)?;
    shell_line!(ctx, "App created successfully!")?;
    shell_line!(ctx)?;
    shell_line!(ctx, "App ID: {app_id}")?;
    shell_line!(ctx, "Name: {}", app_config.name)?;
    shell_line!(ctx, "Manifest Set ID: {}", result.result.manifest_set_id)?;
    if !operator_ids.is_empty() {
        shell_line!(
            ctx,
            "Manifest Set Operator IDs: {}",
            operator_ids.join(", ")
        )?;
    }
    shell_line!(ctx, "Config: {}", args.config_file.display())?;
    shell_line!(ctx)?;
    shell_line!(
        ctx,
        "Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`"
    )?;

    Ok(())
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
    use crate::config::app::{KNOWN_QUORUM_KEY, OperatorParams};

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
}
