//! Approve deploy command - cryptographically approve a QOS manifest.

use crate::{
    approvals::{ApprovalVerdict, validate_deployment_approvals},
    client::build_client,
    config::turnkey::Config,
    local_operator_key::LocalOperatorSeedSource,
    operator::{OperatorCtx, ResolvedOperator, resolve_operator},
    outcome::Outcome,
    output::StdCtx,
    pair::HexSeed,
    prompts::{self, bail_required_in_non_interactive, stdin_can_prompt},
    shell_print, shell_println,
    util::{read_file_to_string, write_file},
};
use anyhow::{Context, anyhow, bail};
use clap::{ArgGroup, Args as ClapArgs};
use qos_core::protocol::services::boot::{
    Approval, ManifestSet, Namespace, NitroConfig, QuorumMember, ShareSet, VersionedManifest,
};
use serde::{Serialize, Serializer};
use std::fmt::{self, Display, Formatter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use turnkey_client::generated::external::data::v1::{TvcDeployment, TvcOperatorApproval};
use turnkey_client::generated::{
    CreateTvcManifestApprovalsIntent, GetTvcDeploymentRequest, TvcManifestApproval,
};
use uuid::Uuid;

const QUORUM_REACHED_MESSAGE: &str =
    "Manifest approval quorum reached. Your deployment will be available soon.";

/// Cryptographically approve a QOS manifest for a deployment with your operator's manifest set key.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
#[command(group(ArgGroup::new("manifest-source").args(["manifest", "deploy_id"])))]
pub struct Args {
    /// Path to QOS manifest file.
    #[arg(
        short,
        long,
        value_name = "PATH",
        env = "TVC_MANIFEST",
        help_heading = "Manifest source (pick one)"
    )]
    pub manifest: Option<PathBuf>,

    /// ID of the deployment the manifest belongs to.
    #[arg(
        short,
        long,
        env = "TVC_DEPLOY_ID",
        help_heading = "Manifest source (pick one)"
    )]
    pub deploy_id: Option<String>,

    /// Turnkey manifest ID (UUID) for the manifest being approved.
    /// Required when posting approval to the API.
    #[arg(long, env = "TVC_MANIFEST_ID")]
    pub manifest_id: Option<String>,

    /// Turnkey operator UUID for the approving operator. When posting without
    /// this flag, TVC selects from the last manifest's saved operator IDs. A
    /// configured hosted operator is selected by its UUID.
    #[arg(long, env = "TVC_OPERATOR_ID")]
    pub operator_id: Option<Uuid>,

    /// Hex-encoded 32-byte master seed for the operator key.
    /// If no seed flag is provided, uses the operator key from the logged-in org config.
    #[arg(
        long,
        value_name = "HEX_SEED",
        env = "TVC_OPERATOR_SEED",
        help_heading = "Operator seed (pick one)"
    )]
    pub operator_seed: Option<HexSeed>,

    /// Path to a file containing the hex-encoded master seed for the operator key.
    #[arg(
        long,
        value_name = "PATH",
        env = "TVC_OPERATOR_SEED_PATH",
        help_heading = "Operator seed (pick one)"
    )]
    pub operator_seed_path: Option<PathBuf>,

    /// Walk through manifest approval prompts but do not generate an approval.
    #[arg(long, env = "TVC_DRY_RUN")]
    pub dry_run: bool,

    /// DANGEROUS: skip interactive prompts for approving each aspect of manifest.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_INTERACTIVE")]
    pub dangerous_skip_interactive: bool,

    /// Write approval to file instead of stdout.
    #[arg(short = 'o', long, value_name = "PATH", env = "TVC_APPROVAL_OUT")]
    pub approval_out: Option<PathBuf>,

    /// Generate a local-operator approval without posting it to the API. This
    /// supports offline signing and cannot be used with a hosted operator.
    #[arg(long, env = "TVC_SKIP_POST")]
    pub skip_post: bool,
}

struct PostApprovalPlan<'a> {
    manifest_id: &'a str,
    operator_id: &'a Uuid,
    deploy_id: Option<&'a str>,
}

/// What `post_approval_to_api` learned from the API: the created approval IDs
/// and whether the manifest approval quorum is now reached (`None` when the
/// post-check deployment fetch failed or was not attempted).
struct PostedApproval {
    approval_ids: Vec<String>,
    quorum_reached: Option<bool>,
}

/// Terminal shapes for `deploy approve` (reasons share the
/// `manifest-approval-*` prefix).
pub enum ApproveOutcome {
    /// Approval generated and posted to the API.
    Posted(ApprovalPosted),
    /// Approval generated but not posted (`--skip-post`).
    NotPosted(ApprovalGenerated),
    /// The operator already has an approval on this manifest; nothing posted.
    AlreadyPosted(ApprovalAlreadyPosted),
    /// `--dry-run`: manifest review completed, no approval generated.
    DryRun(ApprovalDryRun),
}

impl Serialize for ApproveOutcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ApproveOutcome::Posted(msg) => msg.serialize(serializer),
            ApproveOutcome::NotPosted(msg) => msg.serialize(serializer),
            ApproveOutcome::AlreadyPosted(msg) => msg.serialize(serializer),
            ApproveOutcome::DryRun(msg) => msg.serialize(serializer),
        }
    }
}

impl Display for ApproveOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ApproveOutcome::Posted(msg) => msg.fmt(f),
            ApproveOutcome::NotPosted(msg) => msg.fmt(f),
            ApproveOutcome::AlreadyPosted(msg) => msg.fmt(f),
            ApproveOutcome::DryRun(msg) => msg.fmt(f),
        }
    }
}

/// Render the approval payload part of a human message: the pretty-printed
/// approval JSON, or the path it was written to (`--approval-out`).
fn approval_payload_human_message(
    approval: &Option<Approval>,
    written_to: &Option<String>,
) -> String {
    match (approval, written_to) {
        (Some(approval), _) => serde_json::to_string_pretty(approval)
            .expect("serializing manifest approval should not fail"),
        (None, Some(path)) => format!("Approval written to: {path}"),
        (None, None) => String::new(),
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalPosted {
    /// Present when the approval was printed inline (no `--approval-out`).
    #[serde(skip_serializing_if = "Option::is_none")]
    approval: Option<Approval>,
    /// Present when the approval was written to a file via `--approval-out`.
    #[serde(skip_serializing_if = "Option::is_none")]
    written_to: Option<String>,
    manifest_id: String,
    operator_id: String,
    approval_ids: Vec<String>,
    /// `None` when the post-check deployment fetch failed or was not
    /// attempted (no `--deploy-id`), so quorum state is unknown.
    quorum_reached: Option<bool>,
}

impl Display for ApprovalPosted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&approval_payload_human_message(
            &self.approval,
            &self.written_to,
        ))?;
        write!(
            f,
            r#"
Approval posted successfully!

Approval IDs: {:?}
Manifest ID: {}
Operator ID: {}"#,
            self.approval_ids, self.manifest_id, self.operator_id
        )?;

        if let Some(reached) = self.quorum_reached {
            let quorum_line = if reached {
                QUORUM_REACHED_MESSAGE
            } else {
                "Your approval has been posted. Deployment requires additional manifest approvals before it can be deployed on TVC."
            };
            write!(f, "\n\n{quorum_line}")?;
        }

        Ok(())
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalGenerated {
    /// Present when the approval was printed inline (no `--approval-out`).
    #[serde(skip_serializing_if = "Option::is_none")]
    approval: Option<Approval>,
    /// Present when the approval was written to a file via `--approval-out`.
    #[serde(skip_serializing_if = "Option::is_none")]
    written_to: Option<String>,
}

impl Display for ApprovalGenerated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&approval_payload_human_message(
            &self.approval,
            &self.written_to,
        ))
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAlreadyPosted {
    operator_id: String,
    approval_id: String,
}

impl Display for ApprovalAlreadyPosted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Operator {} has already approved this manifest (approval ID: {}). Nothing to post.",
            self.operator_id, self.approval_id
        )
    }
}

#[derive(Default, Serialize)]
pub struct ApprovalDryRun {}

impl Display for ApprovalDryRun {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Dry run complete. No approval generated.")
    }
}

struct PostTarget {
    manifest_id: String,
    deploy_id: Option<String>,
}

struct ResolvedApproveInputs {
    manifest: VersionedManifest,
    operator_seed_source: Option<LocalOperatorSeedSource>,
    operator_id: Option<Uuid>,
    approval_out: Option<PathBuf>,
    dry_run: bool,
    skip_post: bool,
    post_target: Option<PostTarget>,
    /// Present only when the manifest came from a deployment fetch; carries
    /// the already-posted approvals for validation and duplicate checks.
    deployment: Option<TvcDeployment>,
}

pub async fn run(ctx: &mut StdCtx, mut args: Args) -> anyhow::Result<Outcome> {
    let operator_seed_source = LocalOperatorSeedSource::from_args(
        args.operator_seed.take(),
        args.operator_seed_path.take(),
    )?;

    let inputs = if ctx.is_non_interactive() {
        build_inputs_non_interactive(ctx, args, operator_seed_source).await?
    } else {
        build_inputs_interactive(ctx, args, operator_seed_source).await?
    };

    let outcome = run_with_resolved_inputs(ctx, inputs).await?;
    Ok(Outcome::DeployApprove(outcome))
}

async fn build_inputs_interactive(
    ctx: &mut StdCtx,
    args: Args,
    operator_seed_source: Option<LocalOperatorSeedSource>,
) -> anyhow::Result<ResolvedApproveInputs> {
    let do_prompt_user = !args.dangerous_skip_interactive;

    // Guard: bail fast before fetching the manifest if review prompts are
    // required but the caller has no TTY to answer them.
    if do_prompt_user && !stdin_can_prompt() {
        bail_required_in_non_interactive("--dangerous-skip-interactive")?;
    }

    let (manifest, fetched_manifest_id, deployment) = load_manifest(ctx, &args).await?;

    if do_prompt_user {
        interactive_approve(ctx, &manifest)?;
    }

    let mut operator_id = args.operator_id;
    let post_target = if !args.dry_run && !args.skip_post {
        let manifest_id = resolve_manifest_id(&args, fetched_manifest_id.as_deref())?;
        if operator_id.is_none() {
            operator_id = Some({
                let saved_ids = load_saved_operator_ids().await?;
                match &*saved_ids {
                    [] => bail!(
                        "--operator-id is required to post approval to API. \
                         No saved operator IDs found. \
                         Use --skip-post to only generate the approval locally."
                    ),
                    [id] => *id,
                    _ if stdin_can_prompt() => {
                        prompts::select("Select approving operator", saved_ids)?
                    }
                    _ => bail!(
                        "--operator-id is required to post approval to API when multiple saved operator IDs are available"
                    ),
                }
            });
        }
        Some(PostTarget {
            manifest_id,
            deploy_id: args.deploy_id.clone(),
        })
    } else {
        None
    };

    Ok(ResolvedApproveInputs {
        manifest,
        operator_seed_source,
        operator_id,
        approval_out: args.approval_out,
        dry_run: args.dry_run,
        skip_post: args.skip_post,
        post_target,
        deployment,
    })
}

async fn build_inputs_non_interactive(
    ctx: &mut StdCtx,
    args: Args,
    operator_seed_source: Option<LocalOperatorSeedSource>,
) -> anyhow::Result<ResolvedApproveInputs> {
    if !args.dangerous_skip_interactive {
        bail_required_in_non_interactive("--dangerous-skip-interactive")?;
    }

    let (manifest, fetched_manifest_id, deployment) = load_manifest(ctx, &args).await?;

    let mut operator_id = args.operator_id;
    let post_target = if !args.dry_run && !args.skip_post {
        let manifest_id = resolve_manifest_id(&args, fetched_manifest_id.as_deref())?;
        if operator_id.is_none() {
            operator_id = Some({
                let saved_ids = load_saved_operator_ids().await?;
                match &*saved_ids {
                    [] => bail!(
                        "--operator-id is required to post approval to API. \
                         No saved operator IDs found. \
                         Use --skip-post to only generate the approval locally."
                    ),
                    [id] => *id,
                    _ => bail!(
                        "--operator-id is required to post approval to API when multiple saved operator IDs are available"
                    ),
                }
            });
        }
        Some(PostTarget {
            manifest_id,
            deploy_id: args.deploy_id.clone(),
        })
    } else {
        None
    };

    Ok(ResolvedApproveInputs {
        manifest,
        operator_seed_source,
        operator_id,
        approval_out: args.approval_out,
        dry_run: args.dry_run,
        skip_post: args.skip_post,
        post_target,
        deployment,
    })
}

async fn run_with_resolved_inputs(
    ctx: &mut StdCtx,
    inputs: ResolvedApproveInputs,
) -> anyhow::Result<ApproveOutcome> {
    if inputs.dry_run {
        return Ok(ApproveOutcome::DryRun(ApprovalDryRun {}));
    }

    if let Some(deployment) = &inputs.deployment {
        warn_invalid_approvals(ctx, &inputs.manifest, &deployment.manifest_approvals)?;
    }

    let operator = resolve_operator(inputs.operator_seed_source, inputs.operator_id).await?;
    if inputs.skip_post && operator.is_hosted() {
        bail!("--skip-post is only supported for local operators");
    }
    let auth = if operator.is_hosted() {
        Some(build_client().await?)
    } else {
        None
    };
    let approval = sign_and_write_approval(
        &operator,
        &OperatorCtx {
            auth: auth.as_ref(),
        },
        inputs.approval_out.as_deref(),
        &inputs.manifest,
    )
    .await?;

    let written_to = inputs
        .approval_out
        .as_ref()
        .map(|path| path.display().to_string());
    // The approval payload rides in the outcome only when it was not written
    // to a file; otherwise the outcome carries the file path.
    let inline_approval = if written_to.is_none() {
        Some(approval.clone())
    } else {
        None
    };

    match inputs.post_target {
        Some(target) => {
            let operator_id = operator
                .id()
                .context("resolved operator ID required to post approval")?;

            let existing = inputs.deployment.as_ref().and_then(|deployment| {
                find_own_approval(
                    &deployment.manifest_approvals,
                    &operator_id,
                    &approval.member.pub_key,
                )
            });
            if let Some(existing) = existing {
                return Ok(ApproveOutcome::AlreadyPosted(ApprovalAlreadyPosted {
                    operator_id: operator_id.to_string(),
                    approval_id: existing.id.clone(),
                }));
            }

            let plan = PostApprovalPlan {
                manifest_id: target.manifest_id.as_str(),
                operator_id: &operator_id,
                deploy_id: target.deploy_id.as_deref(),
            };
            let posted = post_approval_to_api(ctx, plan, &approval, &inputs.manifest).await?;

            Ok(ApproveOutcome::Posted(ApprovalPosted {
                approval: inline_approval,
                written_to,
                manifest_id: target.manifest_id,
                operator_id: operator_id.to_string(),
                approval_ids: posted.approval_ids,
                quorum_reached: posted.quorum_reached,
            }))
        }
        None => Ok(ApproveOutcome::NotPosted(ApprovalGenerated {
            approval: inline_approval,
            written_to,
        })),
    }
}

/// Warn about already-posted approvals that QOS would reject at enclave boot.
fn warn_invalid_approvals(
    ctx: &mut StdCtx,
    manifest: &VersionedManifest,
    approvals: &[TvcOperatorApproval],
) -> anyhow::Result<()> {
    let validation = validate_deployment_approvals(manifest, approvals);
    for approval in &validation.approvals {
        if approval.verdict == ApprovalVerdict::Valid {
            continue;
        }

        ctx.shell().human().warn(format!(
            "existing approval from {} is {}; QOS will reject it at enclave boot",
            approval.operator_label(),
            approval.verdict
        ))?;
    }

    Ok(())
}

/// Find an approval already posted by this operator, matched by operator ID or
/// by the manifest set member public key.
fn find_own_approval<'a>(
    approvals: &'a [TvcOperatorApproval],
    operator_id: &Uuid,
    member_pub_key: &[u8],
) -> Option<&'a TvcOperatorApproval> {
    approvals.iter().find(|approval| {
        approval.operator.as_ref().is_some_and(|operator| {
            operator.id.parse::<Uuid>().ok().as_ref() == Some(operator_id)
                || hex::decode(&operator.public_key).is_ok_and(|key| key == member_pub_key)
        })
    })
}

async fn load_manifest(
    ctx: &mut StdCtx,
    args: &Args,
) -> anyhow::Result<(VersionedManifest, Option<String>, Option<TvcDeployment>)> {
    match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => Ok((read_manifest_from_path(path).await?, None, None)),
        (_, Some(deploy_id)) => {
            let (manifest, manifest_id, deployment) =
                fetch_manifest_from_deploy(ctx, deploy_id).await?;
            Ok((manifest, Some(manifest_id), Some(deployment)))
        }
        (None, None) => bail!("a manifest source is required"),
    }
}

/// Sign the manifest and, when `--approval-out` is set, write the approval to
/// that file. Reporting the approval (inline payload or file path) is the
/// terminal outcome's job.
async fn sign_and_write_approval(
    operator: &ResolvedOperator,
    operator_ctx: &OperatorCtx<'_>,
    approval_out: Option<&Path>,
    manifest: &VersionedManifest,
) -> anyhow::Result<Approval> {
    let approval = operator.approve_manifest(operator_ctx, manifest).await?;

    if let Some(path) = approval_out {
        let json = serde_json::to_string_pretty(&approval)?;
        write_file(path, &json).await?;
    }

    Ok(approval)
}

fn resolve_manifest_id(args: &Args, fetched_manifest_id: Option<&str>) -> anyhow::Result<String> {
    fetched_manifest_id
        .map(|s| s.to_string())
        .or_else(|| args.manifest_id.clone())
        .ok_or_else(|| {
            anyhow!(
                "--manifest-id is required to post approval to API (or use --deploy-id). \
                 Use --skip-post to only generate the approval locally."
            )
        })
}

async fn load_saved_operator_ids() -> anyhow::Result<Vec<Uuid>> {
    let config = Config::load().await?;
    config
        .get_last_operator_ids()
        .unwrap_or_default()
        .into_iter()
        .map(|id| {
            Uuid::parse_str(&id).with_context(|| format!("saved operator ID '{id}' is not a UUID"))
        })
        .collect()
}

async fn post_approval_to_api(
    ctx: &mut StdCtx,
    plan: PostApprovalPlan<'_>,
    approval: &Approval,
    manifest: &VersionedManifest,
) -> anyhow::Result<PostedApproval> {
    shell_println!(ctx)?;
    shell_println!(ctx, "Posting approval to Turnkey...")?;

    let auth = crate::client::build_client().await?;

    let tvc_approval = TvcManifestApproval {
        operator_id: plan.operator_id.to_string(),
        signature: hex::encode(&approval.signature),
    };

    let intent = CreateTvcManifestApprovalsIntent {
        manifest_id: plan.manifest_id.into(),
        approvals: vec![tvc_approval],
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .create_tvc_manifest_approvals(auth.org_id.clone(), timestamp_ms, intent)
        .await
        .context("failed to post manifest approval")?;

    let quorum_reached = match plan.deploy_id {
        Some(deploy_id) => {
            let request = GetTvcDeploymentRequest {
                organization_id: auth.org_id.clone(),
                deployment_id: deploy_id.to_string(),
            };

            match auth.client.get_tvc_deployment(request).await {
                Ok(response) => Some(response.tvc_deployment.is_some_and(|deployment| {
                    validate_deployment_approvals(manifest, &deployment.manifest_approvals)
                        .quorum_reached()
                })),
                Err(error) => {
                    debug!(
                        deploy_id,
                        %error,
                        "failed to fetch deployment after posting manifest approval"
                    );
                    None
                }
            }
        }
        None => None,
    };

    Ok(PostedApproval {
        approval_ids: result.result.approval_ids,
        quorum_reached,
    })
}

/// Walk the user through each section of the manifest for approval.
fn interactive_approve(ctx: &mut StdCtx, manifest: &VersionedManifest) -> anyhow::Result<()> {
    shell_println!(ctx, "\n========================================")?;
    shell_println!(ctx, "         MANIFEST APPROVAL")?;
    shell_println!(ctx, "========================================\n")?;

    review_namespace(ctx, manifest.namespace())?;
    review_enclave(ctx, manifest.enclave())?;
    review_pivot(ctx, manifest)?;
    review_manifest_set(ctx, manifest.manifest_set())?;
    review_share_set(ctx, manifest.share_set())?;

    shell_println!(ctx, "\n========================================")?;
    shell_println!(ctx, "    ALL SECTIONS APPROVED")?;
    shell_println!(ctx, "========================================\n")?;

    Ok(())
}

fn render_namespace(namespace: &Namespace) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "NAMESPACE");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(s, "  Name:       {}", namespace.name);
    let _ = writeln!(s, "  Nonce:      {}", namespace.nonce);
    let _ = writeln!(s, "  Quorum Key: {}", hex::encode(&namespace.quorum_key));
    let _ = writeln!(s);
    s
}

fn review_namespace(ctx: &mut StdCtx, namespace: &Namespace) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_namespace(namespace))?;
    prompts::confirm_or_bail("Approve namespace?", "approval")
}

fn render_enclave(enclave: &NitroConfig) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "ENCLAVE (AWS Nitro)");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(s, "  PCR0 (image):     {}", hex::encode(&enclave.pcr0));
    let _ = writeln!(s, "  PCR1 (kernel):    {}", hex::encode(&enclave.pcr1));
    let _ = writeln!(s, "  PCR2 (app):       {}", hex::encode(&enclave.pcr2));
    let _ = writeln!(s, "  PCR3 (IAM role):  {}", hex::encode(&enclave.pcr3));
    // Skip the QOS commit since it's not cryptographically linked
    let _ = writeln!(s);
    s
}

fn review_enclave(ctx: &mut StdCtx, enclave: &NitroConfig) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_enclave(enclave))?;
    prompts::confirm_or_bail("Approve enclave configuration?", "approval")
}

fn render_pivot(manifest: &VersionedManifest) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "PIVOT BINARY");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(
        s,
        "  Pivot Binary Hash: {}",
        hex::encode(manifest.pivot_hash())
    );
    if manifest.args().is_empty() {
        let _ = writeln!(s, "  CLI Args: (none)");
    } else {
        let _ = writeln!(s, "  CLI Args:\n   {}", manifest.args().join("\n   "));
    }
    let _ = writeln!(s);
    s
}

fn review_pivot(ctx: &mut StdCtx, manifest: &VersionedManifest) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_pivot(manifest))?;
    prompts::confirm_or_bail("Approve pivot binary?", "approval")
}

fn render_quorum_members(members: &[QuorumMember]) -> String {
    let mut s = String::new();
    for member in members.iter() {
        let _ = writeln!(s, "    {} ({})", member.alias, hex::encode(&member.pub_key));
    }
    s
}

fn render_manifest_set(set: &ManifestSet) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "MANIFEST SET");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(s, "  Threshold: {} of {}", set.threshold, set.members.len());
    let _ = writeln!(s, "  Members:");
    s.push_str(&render_quorum_members(&set.members));
    let _ = writeln!(s);
    s
}

fn review_manifest_set(ctx: &mut StdCtx, set: &ManifestSet) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_manifest_set(set))?;
    prompts::confirm_or_bail("Approve manifest set?", "approval")
}

fn render_share_set(set: &ShareSet) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "SHARE SET");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(s, "  Threshold: {} of {}", set.threshold, set.members.len());
    let _ = writeln!(s, "  Members:");
    s.push_str(&render_quorum_members(&set.members));
    let _ = writeln!(s);
    s
}

fn review_share_set(ctx: &mut StdCtx, set: &ShareSet) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_share_set(set))?;
    prompts::confirm_or_bail("Approve share set?", "approval")
}

async fn read_manifest_from_path(path: &Path) -> anyhow::Result<VersionedManifest> {
    let content = read_file_to_string(path).await?;
    let manifest = VersionedManifest::try_from_slice_compat(content.as_bytes())
        .with_context(|| format!("failed to parse manifest JSON from: {}", path.display()))?;
    Ok(manifest)
}

/// Fetch manifest from Turnkey using GetTvcDeployment API.
/// Returns the manifest, its Turnkey manifest_id, and the deployment itself.
async fn fetch_manifest_from_deploy(
    ctx: &mut StdCtx,
    deploy_id: &str,
) -> anyhow::Result<(VersionedManifest, String, TvcDeployment)> {
    shell_println!(ctx, "Fetching deployment {deploy_id}...")?;

    let auth = crate::client::build_client().await?;

    let request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: deploy_id.to_string(),
    };

    let response = auth
        .client
        .get_tvc_deployment(request)
        .await
        .context("failed to fetch deployment")?;

    let deployment = response
        .tvc_deployment
        .ok_or_else(|| anyhow!("deployment not found: {deploy_id}"))?;

    let tvc_manifest = deployment
        .manifest
        .as_ref()
        .ok_or_else(|| anyhow!("manifest not found in deployment"))?;

    let manifest = VersionedManifest::try_from_slice_compat(&tvc_manifest.manifest)
        .context("failed to parse manifest from deployment")?;
    let manifest_id = tvc_manifest.id.clone();

    shell_println!(ctx, "✓ Manifest loaded (manifest_id: {manifest_id})")?;

    Ok((manifest, manifest_id, deployment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Message;
    use turnkey_client::generated::external::data::v1::TvcOperator;

    fn fixture_manifest() -> VersionedManifest {
        VersionedManifest::try_from_slice_compat(include_bytes!("../../../fixtures/manifest.json"))
            .expect("fixture manifest should parse")
    }

    fn test_operator(id: &str) -> TvcOperator {
        TvcOperator {
            id: id.to_string(),
            name: format!("operator-{id}"),
            public_key: format!("public-key-{id}"),
            created_at: None,
            updated_at: None,
        }
    }

    fn test_approval(index: usize, operator_id: Option<&str>) -> TvcOperatorApproval {
        TvcOperatorApproval {
            id: format!("approval-{index}"),
            manifest_id: "manifest-123".to_string(),
            operator: operator_id.map(test_operator),
            approval: vec![],
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn render_namespace_includes_name_nonce_and_quorum_key() {
        let manifest = fixture_manifest();
        let rendered = render_namespace(manifest.namespace());
        assert!(rendered.contains("NAMESPACE"));
        assert!(rendered.contains("turnkey-prod"));
        assert!(rendered.contains("Nonce:"));
        assert!(rendered.contains("Quorum Key:"));
    }

    #[test]
    fn render_enclave_includes_all_four_pcrs() {
        let manifest = fixture_manifest();
        let rendered = render_enclave(manifest.enclave());
        assert!(rendered.contains("ENCLAVE (AWS Nitro)"));
        assert!(rendered.contains("PCR0"));
        assert!(rendered.contains("PCR1"));
        assert!(rendered.contains("PCR2"));
        assert!(rendered.contains("PCR3"));
    }

    #[test]
    fn render_pivot_includes_header_and_args() {
        let manifest = fixture_manifest();
        let rendered = render_pivot(&manifest);
        assert!(rendered.contains("PIVOT BINARY"));
        assert!(rendered.contains("Pivot Binary Hash:"));
        assert!(rendered.contains("--flag"));
        assert!(rendered.contains("positional"));
    }

    #[test]
    fn render_manifest_set_includes_threshold_and_each_member() {
        let manifest = fixture_manifest();
        let rendered = render_manifest_set(manifest.manifest_set());
        assert!(rendered.contains("MANIFEST SET"));
        assert!(rendered.contains("Threshold: 2 of 3"));
        assert!(rendered.contains("operator-alice"));
        assert!(rendered.contains("operator-bob"));
        assert!(rendered.contains("operator-charlie"));
    }

    #[test]
    fn find_own_approval_matches_by_operator_id() {
        let operator_id = Uuid::from_u128(1);
        let approvals = [test_approval(0, Some(&operator_id.to_string()))];

        let found = find_own_approval(&approvals, &operator_id, &[0xab]).unwrap();
        assert_eq!(found.id, "approval-0");
    }

    #[test]
    fn find_own_approval_matches_by_member_public_key() {
        let mut approval = test_approval(0, Some(&Uuid::from_u128(1).to_string()));
        approval.operator.as_mut().unwrap().public_key = hex::encode([0xab, 0xcd]);

        assert!(find_own_approval(&[approval], &Uuid::from_u128(2), &[0xab, 0xcd]).is_some());
    }

    #[test]
    fn find_own_approval_ignores_other_operators() {
        let approvals = [
            test_approval(0, Some(&Uuid::from_u128(1).to_string())),
            test_approval(1, None),
        ];

        assert!(find_own_approval(&approvals, &Uuid::from_u128(2), &[0xab]).is_none());
    }

    fn posted_to_file() -> ApprovalPosted {
        ApprovalPosted {
            approval: None,
            written_to: Some("approval.json".to_string()),
            manifest_id: "manifest-123".to_string(),
            operator_id: "operator-456".to_string(),
            approval_ids: vec!["approval-1".to_string()],
            quorum_reached: Some(true),
        }
    }

    #[test]
    fn approval_posted_serializes_expected_json() {
        let value: serde_json::Value = serde_json::from_str(
            &Outcome::DeployApprove(ApproveOutcome::Posted(posted_to_file())).to_json_string(),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest-approval-posted",
                "writtenTo": "approval.json",
                "manifestId": "manifest-123",
                "operatorId": "operator-456",
                "approvalIds": ["approval-1"],
                "quorumReached": true,
            })
        );
    }

    #[test]
    fn approval_generated_serializes_expected_json() {
        let generated = ApprovalGenerated {
            approval: Some(Approval {
                signature: vec![0xde, 0xad],
                member: QuorumMember {
                    alias: "operator-alice".to_string(),
                    pub_key: vec![0xaa],
                },
            }),
            written_to: None,
        };

        let value: serde_json::Value = serde_json::from_str(
            &Outcome::DeployApprove(ApproveOutcome::NotPosted(generated)).to_json_string(),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest-approval-generated",
                "approval": {
                    "signature": "dead",
                    "member": {
                        "alias": "operator-alice",
                        "pubKey": "aa",
                    },
                },
            })
        );
    }

    #[test]
    fn approval_already_posted_serializes_expected_json() {
        let already_posted = ApprovalAlreadyPosted {
            operator_id: "operator-456".to_string(),
            approval_id: "approval-1".to_string(),
        };

        let value: serde_json::Value = serde_json::from_str(
            &Outcome::DeployApprove(ApproveOutcome::AlreadyPosted(already_posted)).to_json_string(),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest-approval-already-posted",
                "operatorId": "operator-456",
                "approvalId": "approval-1",
            })
        );
    }

    #[test]
    fn approval_dry_run_serializes_reason_only() {
        let value: serde_json::Value = serde_json::from_str(
            &Outcome::DeployApprove(ApproveOutcome::DryRun(ApprovalDryRun {})).to_json_string(),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({ "reason": "manifest-approval-dry-run" })
        );
    }

    /// The quorum line is branch-dependent: present with the reached/pending
    /// sentence when the post-check fetch succeeded, absent when quorum state
    /// is unknown (matching the pre-outcome behavior of a silent failed
    /// fetch).
    #[test]
    fn approval_posted_human_message_includes_quorum_line_only_when_known() {
        let mut posted = posted_to_file();

        posted.quorum_reached = Some(true);
        assert!(
            posted
                .to_string()
                .contains("Manifest approval quorum reached.")
        );

        posted.quorum_reached = Some(false);
        assert!(
            posted
                .to_string()
                .contains("requires additional manifest approvals")
        );

        posted.quorum_reached = None;
        let human = posted.to_string();
        assert!(!human.contains("quorum"));
        assert!(human.ends_with("Operator ID: operator-456"));
    }
}
