//! Deploy create command - creates a deployment from a config file or CLI flags.

use crate::client::build_client;
use crate::config::deploy::DeployConfig;
use crate::config::turnkey::Config;
use crate::prompts;
use crate::prompts::is_interactive;
use crate::pull_secret::encrypt_pivot_pull_secret;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcDeploymentIntent, ValidateTvcImageRequest};

pub(crate) const LONG_ABOUT: &str = "\
Create a new TVC deployment.

Use --config-file, flags, env vars, or a mix of them. Command-line flags
override env vars; env vars override config file values. If --config-file is
omitted, all required deployment fields must be provided by flags or env vars.

Required deployment fields:
  --app-id / TVC_APP_ID
  --qos-version / TVC_QOS_VERSION
  --pivot-image-url / TVC_PIVOT_IMAGE_URL
  --pivot-path / TVC_PIVOT_PATH
  --expected-pivot-digest / TVC_EXPECTED_PIVOT_DIGEST

Special rules:
  --pivot-args replaces the config file's list entirely (does not append).
  --dangerous-deploy-debug-mode is opt-in only: the flag (or its env var) can
  turn debug mode ON, but omitting it does not turn OFF debug mode enabled in
  the config file. It also requires an app created with
  `--dangerous-enable-debug-mode-deployments`; the server rejects debug-mode
  deployments for apps without it.
  --pivot-pull-secret reads an unencrypted pull secret file, encrypts it for the
  active org's API environment, and overrides the encrypted secret in the config.

Interactive vs non-interactive:
  By default, missing required values are filled by interactive prompts when
  stdin is a TTY. Set TVC_NON_INTERACTIVE=1 to disable all prompts, like in CI; the command then
  bails with a precise list of missing fields instead of waiting on input.

Examples:
  tvc deploy create --config-file deploy.json

  # OR

  TVC_ORG_ID=... \\
  TVC_API_KEY_PUBLIC=... \\
  TVC_API_KEY_PRIVATE=... \\
  TVC_APP_ID=... \\
  TVC_QOS_VERSION=... \\
  TVC_PIVOT_PATH=... \\
  TVC_PIVOT_IMAGE_URL=... \\
  TVC_EXPECTED_PIVOT_DIGEST=... \\
    tvc deploy create";

/// Create a new TVC deployment from a config file or CLI flags.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the deployment configuration file (JSON).
    /// Optional when all required fields are provided via flags or env.
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_DEPLOY_CONFIG")]
    pub config_file: Option<PathBuf>,

    /// Override the appId field.
    #[arg(long, env = "TVC_APP_ID")]
    pub app_id: Option<String>,

    /// Override the qosVersion field.
    #[arg(long, env = "TVC_QOS_VERSION")]
    pub qos_version: Option<String>,

    /// Override the pivotContainerImageUrl field.
    #[arg(long, env = "TVC_PIVOT_IMAGE_URL")]
    pub pivot_image_url: Option<String>,

    /// Override the expectedPivotDigest field.
    #[arg(long, env = "TVC_EXPECTED_PIVOT_DIGEST")]
    pub expected_pivot_digest: Option<String>,

    /// Override the pivotPath field.
    #[arg(long, env = "TVC_PIVOT_PATH")]
    pub pivot_path: Option<String>,

    /// Override pivotArgs (replaces the file's list entirely; not appended).
    #[arg(
        long,
        value_name = "ARG",
        value_delimiter = ',',
        env = "TVC_PIVOT_ARGS"
    )]
    pub pivot_args: Vec<String>,

    /// Override the healthCheckPort field.
    #[arg(long, env = "TVC_HEALTH_CHECK_PORT")]
    pub health_check_port: Option<u16>,

    /// Override the publicIngressPort field.
    #[arg(long, env = "TVC_PUBLIC_INGRESS_PORT")]
    pub public_ingress_port: Option<u16>,

    /// Path to an unencrypted pivot container pull secret file.
    ///
    /// The content will be encrypted based on the active org's API environment and
    /// override `pivotContainerEncryptedPullSecret` from the config file.
    #[arg(long, value_name = "PATH", env = "TVC_PIVOT_PULL_SECRET")]
    pub pivot_pull_secret: Option<PathBuf>,

    /// Deploy in debug mode, which forwards secure enclave logs to the host and
    /// zeroes attestation PCRs. This defeats the purpose of a secure enclave,
    /// so it should only be used to debug non-prod applications and view application
    /// logs.
    ///
    /// # WARNING
    ///
    /// Only valid for apps created with `--dangerous-enable-debug-mode-deployments`.
    /// Debug-mode deployments permanently mark the app's quorum key as insecure;
    /// to return to a secure posture, create a new app with a fresh quorum key.
    #[arg(long, env = "TVC_DANGEROUS_DEPLOY_DEBUG_MODE")]
    pub dangerous_deploy_debug_mode: bool,
}

fn build_validate_image_request(
    organization_id: &str,
    image_url: &str,
    pivot_container_encrypted_pull_secret: Option<String>,
) -> ValidateTvcImageRequest {
    ValidateTvcImageRequest {
        organization_id: organization_id.to_string(),
        pivot_container_image_url: image_url.to_string(),
        pivot_container_encrypted_pull_secret,
    }
}

fn build_create_intent(
    deploy_config: &DeployConfig,
    pivot_container_image_url: String,
    pivot_container_encrypted_pull_secret: Option<String>,
) -> CreateTvcDeploymentIntent {
    CreateTvcDeploymentIntent {
        app_id: deploy_config.app_id.clone(),
        qos_version: deploy_config.qos_version.clone(),
        pivot_container_image_url,
        pivot_path: deploy_config.pivot_path.clone(),
        pivot_args: deploy_config.pivot_args.clone(),
        expected_pivot_digest: deploy_config.expected_pivot_digest.clone(),
        pivot_container_encrypted_pull_secret,
        debug_mode: deploy_config.dangerous_deploy_debug_mode.into(),
        nonce: None,
        health_check_type: deploy_config.health_check_type,
        health_check_port: deploy_config.health_check_port as u32,
        public_ingress_port: deploy_config.public_ingress_port as u32,
    }
}

fn pin_image_url(image_url: &str, resolved_digest: &str) -> String {
    if image_url.contains("@") {
        image_url.to_string()
    } else {
        format!("{image_url}@{resolved_digest}") // works for docker pull even if image_url included a :tag
    }
}

fn apply_overrides(config: &mut DeployConfig, args: &Args) {
    if let Some(v) = &args.app_id {
        config.app_id = v.clone();
    }
    if let Some(v) = &args.qos_version {
        config.qos_version = v.clone();
    }
    if let Some(v) = &args.pivot_image_url {
        config.pivot_container_image_url = v.clone();
    }
    if let Some(v) = &args.expected_pivot_digest {
        config.expected_pivot_digest = v.clone();
    }
    if let Some(v) = &args.pivot_path {
        config.pivot_path = v.clone();
    }
    if !args.pivot_args.is_empty() {
        config.pivot_args = args.pivot_args.clone();
    }
    if args.dangerous_deploy_debug_mode {
        config.dangerous_deploy_debug_mode = args.dangerous_deploy_debug_mode;
    }
    if let Some(v) = args.health_check_port {
        config.health_check_port = v;
    }
    if let Some(v) = args.public_ingress_port {
        config.public_ingress_port = v;
    }
}

/// Read a config from the given path, returning the parsed [`DeployConfig`] and
/// a flag indicating whether the file existed (interactive flow may treat
/// missing files as "start from template" when --config-file was provided).
fn read_config_file(path: &Path) -> Result<Option<DeployConfig>> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let config: DeployConfig = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse config file: {}", path.display()))?;
            Ok(Some(config))
        }
        Err(_) if prompts::is_interactive() => Ok(None),
        Err(e) => Err(anyhow!(e))
            .with_context(|| format!("failed to read config file: {}", path.display())),
    }
}

async fn resolve_deploy_config(args: &Args) -> Result<DeployConfig> {
    let (mut config, file_loaded) = match &args.config_file {
        Some(path) => match read_config_file(path)? {
            Some(c) => (c, true),
            None => (DeployConfig::template(None), false),
        },
        None => {
            let mut t = DeployConfig::template(None);
            // Strip the template's "<REMOVE_ME...>" hint so flag-only mode
            // doesn't ship it to the API for public images.
            t.pivot_container_encrypted_pull_secret = None;
            (t, false)
        }
    };

    apply_overrides(&mut config, args);

    let config_updated = resolve_placeholders(&mut config, args).await?;
    if config_updated {
        if let Some(path) = &args.config_file {
            offer_to_save_config(path, &config, file_loaded)?;
        }
    }
    Ok(config)
}

/// Address any remaining placeholders in `config`. Returns `true` iff the
/// resolution required walking interactive prompts
async fn resolve_placeholders(config: &mut DeployConfig, args: &Args) -> Result<bool> {
    if !config.has_placeholders() {
        return Ok(false);
    }
    if !is_interactive() {
        let missing = config.missing_required_fields();
        if !missing.is_empty() {
            let suggestion = if args.config_file.is_some() {
                "Edit the config file or override via flag/TVC_* env."
            } else {
                "Provide via flag, TVC_* env, or --config-file."
            };
            bail!(
                "missing required values: {}. {suggestion}",
                missing.join(", ")
            );
        }

        if config.pull_secret_is_placeholder() {
            bail!(
                "pivotContainerEncryptedPullSecret is placeholder. Set the field to null in the config file (public image), or pass --pivot-pull-secret <PATH> (private image)."
            )
        }
        return Ok(false);
    }

    let saved_app_id = Config::load().await.ok().and_then(|c| c.get_last_app_id());
    config.fill_interactively(saved_app_id.as_deref())?;

    Ok(true)
}

/// Ask the user whether to write the updated config back to disk.
/// `file_loaded` distinguishes "saving over an existing file" from
/// "creating a new file at this path" in the prompt wording.
fn offer_to_save_config(path: &Path, config: &DeployConfig, file_loaded: bool) -> Result<()> {
    if !is_interactive() {
        return Ok(());
    }
    let prompt = if file_loaded {
        format!("Save filled config to {}?", path.display())
    } else {
        format!("Write a new config file at {}?", path.display())
    };
    if prompts::confirm(&prompt, true)? {
        let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        println!("Wrote {}", path.display());
    }
    Ok(())
}

/// Run the deploy create command.
pub async fn run(args: Args) -> Result<()> {
    let deploy_config = resolve_deploy_config(&args).await?;

    println!("Creating deployment for app '{}'...", deploy_config.app_id);

    // Build authenticated client
    let auth = build_client().await?;

    let pivot_container_encrypted_pull_secret = match args.pivot_pull_secret.as_ref() {
        Some(path) => {
            let pull_secret = std::fs::read_to_string(path).with_context(|| {
                format!("failed to read pivot pull secret file: {}", path.display())
            })?;

            if pull_secret.trim().is_empty() {
                bail!(
                    "pivot pull secret file is empty after trimming whitespace: {}",
                    path.display()
                );
            }

            Some(encrypt_pivot_pull_secret(&pull_secret, &auth.api_base_url)?)
        }
        None => deploy_config.pivot_container_encrypted_pull_secret.clone(),
    };

    let validate_image_request = build_validate_image_request(
        &auth.org_id,
        &deploy_config.pivot_container_image_url,
        pivot_container_encrypted_pull_secret.clone(),
    );

    let validate_image_response = auth
        .client
        .validate_tvc_image(validate_image_request)
        .await
        .context("failed to validate TVC image")?;

    let pinned_image_url = pin_image_url(
        &deploy_config.pivot_container_image_url,
        &validate_image_response.resolved_image_digest,
    );

    if pinned_image_url != deploy_config.pivot_container_image_url {
        println!("Using pinned image reference for deployment request: {pinned_image_url}");
    }

    let intent = build_create_intent(
        &deploy_config,
        pinned_image_url,
        pivot_container_encrypted_pull_secret,
    );

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Create the deployment
    let result = auth
        .client
        .create_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC deployment")?;

    println!();
    println!("Deployment created successfully!");
    println!();
    println!("Deployment ID: {}", result.result.deployment_id);
    println!("App ID: {}", deploy_config.app_id);
    if let Some(path) = &args.config_file {
        println!("Config: {}", path.display());
    }
    println!();
    println!("Next steps:");
    println!(
        "  - Run `WIP: tvc deploy status {}` to check deployment status",
        result.result.deployment_id
    );
    println!(
        "  - Run `tvc deploy approve --deploy-id {}` to approve the manifest",
        result.result.deployment_id
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::NamedTempFile;

    #[test]
    fn pin_image_url_appends_digest_to_tagged_reference() {
        let pinned = pin_image_url("ghcr.io/team/app:latest", "sha256:abc123");
        assert_eq!(pinned, "ghcr.io/team/app:latest@sha256:abc123");
    }

    #[test]
    fn pin_image_url_appends_digest_to_untagged_reference() {
        let pinned = pin_image_url("ghcr.io/team/app", "sha256:abc123");
        assert_eq!(pinned, "ghcr.io/team/app@sha256:abc123");
    }

    fn empty_args() -> Args {
        Args {
            config_file: None,
            app_id: None,
            qos_version: None,
            pivot_image_url: None,
            expected_pivot_digest: None,
            pivot_path: None,
            pivot_args: vec![],
            health_check_port: None,
            public_ingress_port: None,
            pivot_pull_secret: None,
            dangerous_deploy_debug_mode: false,
        }
    }

    fn all_required_flags() -> Args {
        Args {
            app_id: Some("flag-app-id".into()),
            qos_version: Some("flag-qos".into()),
            pivot_image_url: Some("flag-image".into()),
            expected_pivot_digest: Some("flag-digest".into()),
            pivot_path: Some("flag-path".into()),
            ..empty_args()
        }
    }

    fn file_config() -> DeployConfig {
        let mut c = DeployConfig::template(None);
        c.app_id = "file-app-id".into();
        c.qos_version = "file-qos".into();
        c.pivot_container_image_url = "file-image".into();
        c.pivot_path = "file-path".into();
        c.pivot_args = vec!["a".into(), "b".into()];
        c.expected_pivot_digest = "file-digest".into();
        c.dangerous_deploy_debug_mode = false;
        c.pivot_container_encrypted_pull_secret = None;
        c.health_check_port = 4000;
        c.public_ingress_port = 5000;
        c
    }

    fn write_config(config: &DeployConfig) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(serde_json::to_string(config).unwrap().as_bytes())
            .unwrap();
        f
    }

    // Resolution tests run inside the tokio runtime since resolve_deploy_config
    // now reaches into Config::load(). They're wired to call .block_on() via
    // a small helper.
    fn run_resolve(args: &Args) -> Result<DeployConfig> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(resolve_deploy_config(args))
    }

    #[test]
    fn flag_overrides_file_value() {
        let file = write_config(&file_config());
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            app_id: Some("flag-app-id".into()),
            ..empty_args()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.app_id, "flag-app-id");
        // Untouched fields keep their file values.
        assert_eq!(resolved.qos_version, "file-qos");
        assert_eq!(resolved.health_check_port, 4000);
    }

    #[test]
    fn file_value_used_when_flag_absent() {
        let file = write_config(&file_config());
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            ..empty_args()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.app_id, "file-app-id");
        assert_eq!(resolved.qos_version, "file-qos");
        assert_eq!(resolved.health_check_port, 4000);
    }

    #[test]
    fn no_file_uses_flag_only_with_template_defaults() {
        let resolved = run_resolve(&all_required_flags()).unwrap();
        // Required fields come from flags.
        assert_eq!(resolved.app_id, "flag-app-id");
        assert_eq!(resolved.qos_version, "flag-qos");
        assert_eq!(resolved.pivot_container_image_url, "flag-image");
        assert_eq!(resolved.pivot_path, "flag-path");
        assert_eq!(resolved.expected_pivot_digest, "flag-digest");
        // Optional fields fall back to template defaults.
        assert_eq!(resolved.health_check_port, 3000);
        assert_eq!(resolved.public_ingress_port, 3000);
        assert!(!resolved.dangerous_deploy_debug_mode);
        assert!(resolved.pivot_args.is_empty());
        // Pull-secret placeholder cleared in flag-only mode.
        assert_eq!(resolved.pivot_container_encrypted_pull_secret, None);
    }

    #[test]
    fn pivot_args_flag_replaces_file_list() {
        let file = write_config(&file_config()); // file has ["a", "b"]
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            pivot_args: vec!["c".into()],
            ..empty_args()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.pivot_args, vec!["c"]);
    }

    /// `--dangerous-deploy-debug-mode` flips the resolved
    /// `dangerous_deploy_debug_mode` from the file's `false` to `true`.
    #[test]
    fn dangerous_debug_mode_flag_enables_debug_mode() {
        let file = write_config(&file_config()); // file has dangerous_deploy_debug_mode = false
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            dangerous_deploy_debug_mode: true,
            ..empty_args()
        };
        let resolved = run_resolve(&args).unwrap();
        assert!(resolved.dangerous_deploy_debug_mode);
    }

    /// Omitting `--dangerous-deploy-debug-mode` must NOT override a config file
    /// that enables debug mode: the flag is opt-in only and can never turn it
    /// off, so a `dangerous_deploy_debug_mode = true` config survives an absent flag.
    #[test]
    fn absent_debug_mode_flag_preserves_config_debug_mode() {
        let mut cfg = file_config();
        cfg.dangerous_deploy_debug_mode = true;
        let file = write_config(&cfg);
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            dangerous_deploy_debug_mode: false,
            ..empty_args()
        };
        let resolved = run_resolve(&args).unwrap();
        assert!(resolved.dangerous_deploy_debug_mode);
    }

    /// The intent builder wraps the resolved config's dangerous_deploy_debug_mode
    /// in `Some(...)` when constructing the outgoing `CreateTvcDeploymentIntent`.
    #[test]
    fn build_intent_forwards_debug_mode() {
        let mut cfg = file_config();
        cfg.dangerous_deploy_debug_mode = true;
        let intent = build_create_intent(&cfg, "image-url".to_string(), None);
        assert_eq!(intent.debug_mode, Some(true));
    }
}
