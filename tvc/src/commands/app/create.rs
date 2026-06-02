//! App create command - creates an app from a config file.

use crate::client::build_client;
use crate::config::app::{AppConfig, OperatorSetParams};
use crate::config::turnkey::{self, StoredQosOperatorKey};
use crate::prompts;
use anyhow::{anyhow, Context, Result};
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_APP_CONFIG")]
    pub config_file: PathBuf,

    /// Permit debug-mode deployments for this app. Debug-mode deployments expose
    /// secure-enclave logs and emit zero'd attestation PCRs, so remote
    /// attestation cannot succeed. Cannot be changed after app creation; setting
    /// this true means the app's quorum key is considered permanently insecure.
    #[arg(long, env = "TVC_DANGEROUS_ENABLE_DEBUG_MODE_DEPLOYMENTS")]
    pub dangerous_enable_debug_mode_deployments: bool,
}

/// Run the app create command.
pub async fn run(args: Args) -> Result<()> {
    let app_config = apply_overrides(load_or_fill_app_config(&args.config_file).await?, &args);

    app_config
        .validate()
        .with_context(|| format!("invalid config file: {}", args.config_file.display()))?;

    println!("Creating app '{}'...", app_config.name);

    // Build authenticated client
    let auth = build_client().await?;

    // Convert config to API intent
    let intent = build_create_tvc_app_intent(&app_config);

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Create the app
    let result = auth
        .client
        .create_tvc_app(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC app")?;

    let app_id = result.result.app_id;
    let operator_ids = result.result.manifest_set_operator_ids;

    // save the app ID and operator_ids to config
    let mut config = turnkey::Config::load().await?;
    config.set_last_app_id(&app_id)?;
    config.set_last_operator_ids(&operator_ids)?;
    config.save().await?;

    println!();
    println!("App created successfully!");
    println!();
    println!("App ID: {app_id}");
    println!("Name: {}", app_config.name);
    println!("Manifest Set ID: {}", result.result.manifest_set_id);
    if !operator_ids.is_empty() {
        println!("Manifest Set Operator IDs: {}", operator_ids.join(", "));
    }
    println!("Config: {}", args.config_file.display());
    println!();
    println!(
        "Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`"
    );

    Ok(())
}

/// Load the app config, walking placeholders interactively when allowed.
async fn load_or_fill_app_config(path: &Path) -> Result<AppConfig> {
    let read = std::fs::read_to_string(path);
    let (mut config, file_existed) = match read {
        Ok(content) => {
            let config: AppConfig = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse config file: {}", path.display()))?;
            (config, true)
        }
        Err(_) if prompts::is_interactive() => (AppConfig::template(None), false),
        Err(e) => {
            return Err(anyhow!(e))
                .with_context(|| format!("failed to read config file: {}", path.display()));
        }
    };

    if file_existed && !config.has_placeholders() {
        return Ok(config);
    }

    if !prompts::is_interactive() {
        anyhow::bail!(
            "Config file contains placeholder values (<FILL_IN_...>). \
             Please edit {} and fill in all required values.",
            path.display()
        );
    }

    let saved_operator_public_key = load_saved_operator_public_key().await;
    config.fill_interactively(saved_operator_public_key.as_deref())?;

    let save = prompts::confirm(&format!("Save filled config to {}?", path.display()), true)?;
    if save {
        let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        println!("Wrote {}", path.display());
    }

    Ok(config)
}

/// Best-effort load of the operator public key from the active org's config
/// so we can offer it as the default for new-operator prompts.
async fn load_saved_operator_public_key() -> Option<String> {
    let config = turnkey::Config::load().await.ok()?;
    let (_, org_config) = config.active_org_config()?;
    let operator_key = StoredQosOperatorKey::load(org_config).await.ok()??;
    Some(operator_key.public_key)
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
        enable_egress: app_config.external_connectivity,
        enable_debug_mode_deployments: app_config.dangerous_enable_debug_mode_deployments.into(),
    }
}

fn apply_overrides(mut config: AppConfig, args: &Args) -> AppConfig {
    if args.dangerous_enable_debug_mode_deployments {
        config.dangerous_enable_debug_mode_deployments = args.dangerous_enable_debug_mode_deployments;
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
    use crate::config::app::{OperatorParams, KNOWN_QUORUM_KEY};

    fn valid_config() -> AppConfig {
        AppConfig {
            name: "test-app".to_string(),
            quorum_public_key: KNOWN_QUORUM_KEY.to_string(),
            external_connectivity: Some(false),
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
        let args = Args {
            config_file: PathBuf::new(),
            dangerous_enable_debug_mode_deployments: true,
        };

        let config = apply_overrides(config, &args);
        assert!(config.dangerous_enable_debug_mode_deployments);
    }

    /// Omitting the CLI flag must NOT override a config that enables debug-mode
    /// deployments: the flag is opt-in only and can never turn it off, so a
    /// `true` config survives an absent flag.
    #[test]
    fn absent_dangerous_flag_preserves_config_debug_mode() {
        let mut config = valid_config();
        config.dangerous_enable_debug_mode_deployments = true;
        let args = Args {
            config_file: PathBuf::new(),
            dangerous_enable_debug_mode_deployments: false,
        };

        let config = apply_overrides(config, &args);
        assert!(config.dangerous_enable_debug_mode_deployments);
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
