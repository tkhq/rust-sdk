//! Deploy create command - creates a deployment from a config file or CLI flags.

use super::format_port_summary;
use crate::client::{build_client, fetch_tvc_app};
use crate::config::deploy::{DeployConfig, DeployConfigValidationErrors};
use crate::config::turnkey::Config;
use crate::output::StdCtx;
use crate::prompts;
use crate::pull_secret::encrypt_pivot_pull_secret;
use crate::shell_println;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::try_join;
use turnkey_client::generated::{CreateTvcDeploymentIntent, ValidateTvcImageRequest};

pub(crate) const LONG_ABOUT: &str = r#"
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
  stdin is a TTY. Use --non-interactive or set TVC_NON_INTERACTIVE=true to
  disable all prompts, like in CI; the command then bails with a precise list
  of missing fields instead of waiting on input.

Examples:
  tvc deploy create --config-file deploy.json

  # OR

  TVC_ORG_ID=... \
  TVC_API_KEY_PUBLIC=... \
  TVC_API_KEY_PRIVATE=... \
  TVC_APP_ID=... \
  TVC_QOS_VERSION=... \
  TVC_PIVOT_PATH=... \
  TVC_PIVOT_IMAGE_URL=... \
  TVC_EXPECTED_PIVOT_DIGEST=... \
    tvc deploy create"#;

/// Create a new TVC deployment from a config file or CLI flags.
#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the deployment configuration file (JSON).
    /// Optional when all required fields are provided via flags or env.
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_DEPLOY_CONFIG")]
    pub config_file: Option<PathBuf>,

    /// Path to an unencrypted pivot container pull secret file.
    ///
    /// The content will be encrypted based on the active org's API environment and
    /// override `pivotContainerEncryptedPullSecret` from the config file.
    #[arg(long, value_name = "PATH", env = "TVC_PIVOT_PULL_SECRET")]
    pub pivot_pull_secret: Option<PathBuf>,

    #[command(flatten)]
    overrides: Overrides,
}

#[derive(Debug, ClapArgs)]
#[cfg_attr(test, derive(Default))]
struct Overrides {
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

    /// Container port TVC probes to decide whether the deployment is healthy.
    ///
    /// Use the same value as --public-ingress-port unless your binary exposes
    /// health checks on a separate listener.
    #[arg(long, env = "TVC_HEALTH_CHECK_PORT")]
    pub health_check_port: Option<u16>,

    /// Container port that receives public app traffic.
    ///
    /// This is usually the port your server listens on for external requests.
    #[arg(long, env = "TVC_PUBLIC_INGRESS_PORT")]
    pub public_ingress_port: Option<u16>,
}

struct ResolvedDeployInputs {
    config_path: Option<PathBuf>,
    config: DeployConfig,
    pivot_pull_secret: Option<String>,
}

pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<()> {
    let inputs = if ctx.is_non_interactive() {
        build_inputs_non_interactive(args).await?
    } else {
        build_inputs_interactive(ctx, args).await?
    };

    run_with_resolved_inputs(ctx, inputs).await
}

async fn build_inputs_interactive(ctx: &mut StdCtx, args: Args) -> Result<ResolvedDeployInputs> {
    let Args {
        config_file: config_path,
        pivot_pull_secret,
        overrides,
    } = args;
    let (mut config, file_loaded) = read_config_file_bytes(config_path.as_deref())
        .await?
        .map(|(path, contents)| {
            serde_json::from_str(&contents)
                .with_context(|| format!("failed to parse config file: {}", path.display()))
        })
        .transpose()?
        .map(|config| (config, true))
        .unwrap_or_else(|| (flag_only_template(), false));

    apply_overrides(&mut config, &overrides);

    let mut config_updated = false;
    loop {
        match config.validate() {
            Ok(()) => break,
            Err(errors) if errors.has_non_placeholder_error() => {
                return Err(invalid_deploy_config_error(errors));
            }
            _ => {
                config_updated = true;
                let saved_app_id = Config::load().await.ok().and_then(|c| c.get_last_app_id());
                config.fill_interactively(ctx, saved_app_id.as_deref())?;
            }
        }
    }

    if config_updated && let Some(path) = &config_path {
        offer_to_save_config(ctx, path, &config, file_loaded)?;
    }
    let pivot_pull_secret = read_pivot_pull_secret(pivot_pull_secret.as_deref()).await?;

    Ok(ResolvedDeployInputs {
        config_path,
        config,
        pivot_pull_secret,
    })
}

async fn build_inputs_non_interactive(args: Args) -> Result<ResolvedDeployInputs> {
    let Args {
        config_file: config_path,
        pivot_pull_secret,
        overrides,
    } = args;

    let (file, pivot_pull_secret) = try_join!(
        read_config_file_bytes(config_path.as_deref()),
        read_pivot_pull_secret(pivot_pull_secret.as_deref())
    )?;

    let mut config = match file {
        Some((path, bytes)) => serde_json::from_str(&bytes)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?,
        _ => flag_only_template(),
    };

    apply_overrides(&mut config, &overrides);

    if let Err(errors) = config.validate() {
        return Err(invalid_deploy_config_error(errors));
    }

    Ok(ResolvedDeployInputs {
        config_path,
        config,
        pivot_pull_secret,
    })
}

async fn read_config_file_bytes(path: Option<&Path>) -> Result<Option<(&Path, String)>> {
    let Some(path) = path else {
        return Ok(None);
    };

    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config file: {}", path.display()))
        .map(|contents| (path, contents))
        .map(Into::into)
}

async fn read_pivot_pull_secret(path: Option<&Path>) -> Result<Option<String>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let pull_secret = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read pivot pull secret file: {}", path.display()))?;

    if pull_secret.trim().is_empty() {
        bail!(
            "pivot pull secret file is empty after trimming whitespace: {}",
            path.display()
        );
    }

    Ok(pull_secret.into())
}

fn flag_only_template() -> DeployConfig {
    let mut template = DeployConfig::template(None);
    // Strip the template's "<REMOVE_ME...>" hint so flag-only mode doesn't ship
    // it to the API for public images.
    template.pivot_container_encrypted_pull_secret = None;
    template
}

fn apply_overrides(config: &mut DeployConfig, overrides: &Overrides) {
    if let Some(v) = &overrides.app_id {
        config.app_id = v.clone();
    }
    if let Some(v) = &overrides.qos_version {
        config.qos_version = v.clone();
    }
    if let Some(v) = &overrides.pivot_image_url {
        config.pivot_container_image_url = v.clone();
    }
    if let Some(v) = &overrides.expected_pivot_digest {
        config.expected_pivot_digest = v.clone();
    }
    if let Some(v) = &overrides.pivot_path {
        config.pivot_path = v.clone();
    }
    if !overrides.pivot_args.is_empty() {
        config.pivot_args = overrides.pivot_args.clone();
    }
    // One-way: only ever flips false -> true.
    if overrides.dangerous_deploy_debug_mode {
        config.dangerous_deploy_debug_mode = true;
    }
    if let Some(v) = overrides.health_check_port {
        config.health_check_port = v;
    }
    if let Some(v) = overrides.public_ingress_port {
        config.public_ingress_port = v;
    }
}

fn invalid_deploy_config_error(errors: DeployConfigValidationErrors) -> anyhow::Error {
    anyhow!("invalid deploy config: {}", errors)
}

/// Ask the user whether to write the updated config back to disk.
/// `file_loaded` distinguishes "saving over an existing file" from
/// "creating a new file at this path" in the prompt wording.
fn offer_to_save_config(
    ctx: &mut StdCtx,
    path: &Path,
    config: &DeployConfig,
    file_loaded: bool,
) -> Result<()> {
    let prompt = if file_loaded {
        format!("Save filled config to {}?", path.display())
    } else {
        format!("Write a new config file at {}?", path.display())
    };
    if prompts::confirm(&prompt, true)? {
        let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        shell_println!(ctx, "Wrote {}", path.display())?;
    }
    Ok(())
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

async fn run_with_resolved_inputs(ctx: &mut StdCtx, inputs: ResolvedDeployInputs) -> Result<()> {
    let deploy_config = inputs.config;

    shell_println!(
        ctx,
        "Creating deployment for app '{}'...",
        deploy_config.app_id
    )?;
    shell_println!(ctx, "{}", format_port_summary(&deploy_config))?;
    shell_println!(ctx)?;

    let auth = build_client().await?;

    // validate that the app exists
    fetch_tvc_app(&auth, &deploy_config.app_id).await?;

    let pivot_container_encrypted_pull_secret = match inputs.pivot_pull_secret.as_ref() {
        Some(pull_secret) => Some(encrypt_pivot_pull_secret(pull_secret, &auth.api_base_url)?),
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
        shell_println!(
            ctx,
            "Using pinned image reference for deployment request: {pinned_image_url}"
        )?;
    }

    let intent = build_create_intent(
        &deploy_config,
        pinned_image_url,
        pivot_container_encrypted_pull_secret,
    );

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .create_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC deployment")?;

    shell_println!(ctx)?;
    shell_println!(ctx, "Deployment created successfully!")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Deployment ID: {}", result.result.deployment_id)?;
    shell_println!(ctx, "App ID: {}", deploy_config.app_id)?;
    if let Some(path) = &inputs.config_path {
        shell_println!(ctx, "Config: {}", path.display())?;
    }
    shell_println!(ctx)?;
    shell_println!(ctx, "Next steps:")?;
    shell_println!(
        ctx,
        "  - Run `tvc deploy status --deploy-id {}` to check deployment status",
        result.result.deployment_id
    )?;
    shell_println!(
        ctx,
        "  - Run `tvc deploy approve --deploy-id {} --operator-id <operator-id>` to approve the manifest",
        result.result.deployment_id
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // App IDs are validated as UUIDs, so test fixtures must use well-formed ones.
    const FLAG_APP_ID: &str = "11111111-1111-1111-1111-111111111111";
    const FILE_APP_ID: &str = "22222222-2222-2222-2222-222222222222";

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

    fn all_required_flags() -> Args {
        let overrides = Overrides {
            app_id: Some(FLAG_APP_ID.into()),
            qos_version: Some("flag-qos".into()),
            pivot_image_url: Some("flag-image".into()),
            expected_pivot_digest: Some("flag-digest".into()),
            pivot_path: Some("flag-path".into()),
            ..Default::default()
        };

        Args {
            overrides,
            ..Default::default()
        }
    }

    fn file_config() -> DeployConfig {
        let mut c = DeployConfig::template(None);
        c.app_id = FILE_APP_ID.into();
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

    /// Mirror the non-interactive resolution pipeline (read bytes → parse →
    /// apply overrides → validate) so the existing flag/env/config-file
    /// composition tests still have a single helper to call.
    fn run_resolve(args: &Args) -> Result<DeployConfig> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut config = match read_config_file_bytes(args.config_file.as_deref()).await? {
                    Some((path, bytes)) => serde_json::from_str(&bytes).with_context(|| {
                        format!("failed to parse config file: {}", path.display())
                    })?,
                    None => flag_only_template(),
                };
                apply_overrides(&mut config, &args.overrides);
                if let Err(errors) = config.validate() {
                    return Err(invalid_deploy_config_error(errors));
                }
                anyhow::Ok(config)
            })
    }

    #[test]
    fn flag_overrides_file_value() {
        let file = write_config(&file_config());
        let overrides = Overrides {
            app_id: Some(FLAG_APP_ID.into()),
            ..Default::default()
        };
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            overrides,
            ..Default::default()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.app_id, FLAG_APP_ID);
        // Untouched fields keep their file values.
        assert_eq!(resolved.qos_version, "file-qos");
        assert_eq!(resolved.health_check_port, 4000);
    }

    #[test]
    fn file_value_used_when_flag_absent() {
        let file = write_config(&file_config());
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            ..Default::default()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.app_id, FILE_APP_ID);
        assert_eq!(resolved.qos_version, "file-qos");
        assert_eq!(resolved.health_check_port, 4000);
    }

    #[test]
    fn no_file_uses_flag_only_with_template_defaults() {
        let resolved = run_resolve(&all_required_flags()).unwrap();
        // Required fields come from flags.
        assert_eq!(resolved.app_id, FLAG_APP_ID);
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
    fn no_file_no_required_flags_bails_naming_each_field() {
        let err = run_resolve(&Default::default()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("app_id"), "{msg}");
        assert!(msg.contains("pivot_container_image_url"), "{msg}");
        assert!(msg.contains("pivot_path"), "{msg}");
        assert!(msg.contains("expected_pivot_digest"), "{msg}");
    }

    #[test]
    fn pivot_args_flag_replaces_file_list() {
        let file = write_config(&file_config()); // file has ["a", "b"]
        let overrides = Overrides {
            pivot_args: vec!["c".into()],
            ..Default::default()
        };
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            overrides,
            ..Default::default()
        };
        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.pivot_args, vec!["c"]);
    }

    /// `--dangerous-deploy-debug-mode` flips the resolved
    /// `dangerous_deploy_debug_mode` from the file's `false` to `true`.
    #[test]
    fn dangerous_debug_mode_flag_enables_debug_mode() {
        let file = write_config(&file_config()); // file has dangerous_deploy_debug_mode = false
        let overrides = Overrides {
            dangerous_deploy_debug_mode: true,
            ..Default::default()
        };
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            overrides,
            ..Default::default()
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
        let overrides = Overrides {
            dangerous_deploy_debug_mode: false,
            ..Default::default()
        };
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            overrides,
            ..Default::default()
        };
        let resolved = run_resolve(&args).unwrap();
        assert!(resolved.dangerous_deploy_debug_mode);
    }

    /// All required fields are filled but the pull-secret sentinel is still
    /// present. In non-interactive mode we can't prompt the user about it, so
    /// the resolve bails rather than silently mutating the config or shipping
    /// the sentinel to the API.
    #[test]
    fn pull_secret_placeholder_bails_when_non_interactive() {
        let mut cfg = file_config();
        cfg.pivot_container_encrypted_pull_secret =
            Some("<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>".to_string());
        let file = write_config(&cfg);
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            ..Default::default()
        };

        let err = run_resolve(&args).unwrap_err().to_string();
        assert!(
            err.contains("pivotContainerEncryptedPullSecret"),
            "error should name the offending field: {err}"
        );
        assert!(
            err.contains("--pivot-pull-secret"),
            "error should point the user at the resolution flag: {err}"
        );
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

    /// Exercises every override flag via clap parsing so flag renames or
    /// removals fail this test. The other override tests construct `Args` by
    /// field name and would silently pass.
    #[test]
    fn every_override_flag_changes_template_value() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            args: Args,
        }

        let pull_secret_path = "/tmp/test-pull-secret";
        let args = TestCli::try_parse_from([
            "tvc-deploy-create",
            "--app-id",
            "test-app",
            "--qos-version",
            "test-qos",
            "--pivot-image-url",
            "test-image",
            "--expected-pivot-digest",
            "sha256:test",
            "--pivot-path",
            "/usr/bin/pivot",
            "--pivot-args",
            "arg1,arg2",
            "--health-check-port",
            "8080",
            "--public-ingress-port",
            "9090",
            "--pivot-pull-secret",
            pull_secret_path,
            "--dangerous-deploy-debug-mode",
        ])
        .unwrap()
        .args;

        let template = flag_only_template();
        let mut resolved = flag_only_template();
        apply_overrides(&mut resolved, &args.overrides);

        // Each override moved off its template default ...
        assert_ne!(resolved.app_id, template.app_id);
        assert_ne!(resolved.qos_version, template.qos_version);
        assert_ne!(
            resolved.pivot_container_image_url,
            template.pivot_container_image_url
        );
        assert_ne!(
            resolved.expected_pivot_digest,
            template.expected_pivot_digest
        );
        assert_ne!(resolved.pivot_path, template.pivot_path);
        assert_ne!(resolved.pivot_args, template.pivot_args);
        assert_ne!(resolved.health_check_port, template.health_check_port);
        assert_ne!(resolved.public_ingress_port, template.public_ingress_port);
        assert_ne!(
            resolved.dangerous_deploy_debug_mode,
            template.dangerous_deploy_debug_mode
        );

        // ... to the value passed on the CLI.
        assert_eq!(resolved.app_id, "test-app");
        assert_eq!(resolved.qos_version, "test-qos");
        assert_eq!(resolved.pivot_container_image_url, "test-image");
        assert_eq!(resolved.expected_pivot_digest, "sha256:test");
        assert_eq!(resolved.pivot_path, "/usr/bin/pivot");
        assert_eq!(resolved.pivot_args, vec!["arg1", "arg2"]);
        assert_eq!(resolved.health_check_port, 8080);
        assert_eq!(resolved.public_ingress_port, 9090);
        assert!(resolved.dangerous_deploy_debug_mode);

        // pivot_pull_secret isn't part of apply_overrides; verify clap captured the path.
        assert_eq!(
            args.pivot_pull_secret.as_deref(),
            Some(Path::new(pull_secret_path))
        );
    }

    #[test]
    fn non_interactive_overrides_can_complete_placeholder_file_without_prompting_to_save() {
        let mut cfg = DeployConfig::template(None);
        cfg.pivot_container_encrypted_pull_secret = None;
        let file = write_config(&cfg);
        let args = Args {
            config_file: Some(file.path().to_path_buf()),
            ..all_required_flags()
        };

        let resolved = run_resolve(&args).unwrap();
        assert_eq!(resolved.app_id, FLAG_APP_ID);
        assert_eq!(resolved.qos_version, "flag-qos");
        assert_eq!(resolved.pivot_container_image_url, "flag-image");
        assert_eq!(resolved.pivot_path, "flag-path");
        assert_eq!(resolved.expected_pivot_digest, "flag-digest");

        let persisted: DeployConfig =
            serde_json::from_str(&std::fs::read_to_string(file.path()).unwrap()).unwrap();
        assert!(
            persisted.has_placeholders(),
            "non-interactive resolution must not offer to save or mutate config files"
        );
    }
}
