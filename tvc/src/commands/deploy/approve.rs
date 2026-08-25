//! Approve deploy command - cryptographically approve a QOS manifest.

use crate::{
    approvals::{ApprovalVerdict, OperatorApproval, ValidatedManifest},
    client::build_client,
    commands::Run,
    config::turnkey::{
        Config, OperatorKind, OperatorRecord, SelectYubiKeyOperatorError, YubiKeyOperatorRecord,
        YubiKeySerial,
    },
    errors::MissingResource,
    local_operator_key::LocalOperatorSeedSource,
    operator::{SelectedYubiKey, SignerRequirement},
    outcome::Outcome,
    output::{MissingRequiredInput, StdCtx},
    pair::HexSeed,
    prompts::{self, bail_required_in_non_interactive, stdin_can_prompt},
    shell_print, shell_println,
    util::{read_file_to_string, write_file},
    yubikey::{self, Pin},
};
use anyhow::{Context, bail};
use clap::{ArgGroup, Args as ClapArgs};
use displaydoc::Display;
use qos_core::protocol::services::boot::{
    Approval, BridgeConfig, ManifestSet, Namespace, NitroConfig, QuorumMember, RestartPolicy,
    ShareSet, VersionedManifest,
};
use serde::Serialize;
use std::fmt::{self, Formatter, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use tracing::{debug, instrument};
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
    pub deploy_id: Option<Uuid>,

    /// Turnkey manifest ID (UUID) for the manifest being approved.
    /// Required when posting approval to the API.
    #[arg(long, env = "TVC_MANIFEST_ID")]
    pub manifest_id: Option<Uuid>,

    /// Turnkey operator UUID for the approving operator. When posting without
    /// this flag, TVC selects from the last manifest's saved operator IDs. A
    /// configured hosted operator is selected by its UUID.
    #[arg(long, env = "TVC_OPERATOR_ID")]
    pub operator_id: Option<Uuid>,

    /// Serial (hex) of the YubiKey operator to approve with, when the
    /// organization registers several. Unused for other operator kinds.
    #[arg(long, value_name = "SERIAL")]
    pub serial: Option<YubiKeySerial>,

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

impl Run for Args {
    type Outcome = ApproveOutcome;

    #[instrument(skip_all)]
    async fn run(self, ctx: &mut StdCtx, config: Config) -> anyhow::Result<Self::Outcome> {
        let args = ArgsWithResolvedOperatorSeedSource::try_from(self)?;

        // Manifest review prompts are required unless explicitly skipped;
        // bail before fetching anything if nobody can answer them.
        if !args.dangerous_skip_interactive && (ctx.is_non_interactive() || !stdin_can_prompt()) {
            bail_required_in_non_interactive("--dangerous-skip-interactive")?;
        }

        if !args.dry_run && !args.skip_post && args.manifest.is_some() && args.manifest_id.is_none()
        {
            return Err(ApproveInputError::MissingManifestId.into());
        }

        let can_prompt = !ctx.is_non_interactive() && stdin_can_prompt();
        let operator_id = if args.dry_run || args.skip_post || args.operator_id.is_some() {
            args.operator_id
        } else {
            let candidates = config
                .known_operator_candidates()
                .into_iter()
                .map(|candidate| {
                    let id = Uuid::parse_str(&candidate.id).with_context(|| {
                        format!("saved operator ID '{}' is not a UUID", candidate.id)
                    })?;

                    Ok(ApprovingOperator {
                        id,
                        name: candidate.name,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            match candidates.as_slice() {
                [] => bail!(
                    "--operator-id is required to post approval to API. \
                     No registered or saved operator IDs found. \
                     Use --skip-post to only generate the approval locally."
                ),
                [candidate] => Some(candidate.id),
                _ if !can_prompt => bail!(
                    "--operator-id is required to post approval to API when multiple operator IDs are available"
                ),
                _ => Some(prompts::select("Select approving operator", candidates)?.id),
            }
        };

        let hosted_selected = operator_id
            .map(|id| config.find_hosted_operator(&id))
            .transpose()?
            .flatten()
            .is_some();
        let selected_yubikey = config
            .active_org_config()
            .filter(|(_, org)| {
                !args.dry_run
                    && args.operator_seed_source.is_none()
                    && !hosted_selected
                    && org.default_operator_kind == OperatorKind::Yubikey
            })
            .map(|(alias, org)| {
                let selected = match args.serial {
                    Some(serial) => Ok(org.select_yubikey_operator(Some(serial))?),
                    None => match org.select_yubikey_operator(None) {
                        Ok(selected) => Ok(selected),
                        Err(SelectYubiKeyOperatorError::MultipleYubiKeyOperators { .. })
                            if can_prompt =>
                        {
                            struct Choice<'a>(&'a OperatorRecord, &'a YubiKeyOperatorRecord);

                            impl fmt::Display for Choice<'_> {
                                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                                    write!(f, "{} (serial {})", self.0.name, self.1.serial)
                                }
                            }

                            let choices = org
                                .yubikey_operators()
                                .map(|(operator, yubikey)| Choice(operator, yubikey))
                                .collect();
                            let Choice(operator, yubikey) =
                                prompts::select("Select YubiKey operator", choices)?;

                            Ok((operator, yubikey))
                        }
                        Err(
                            error @ SelectYubiKeyOperatorError::MultipleYubiKeyOperators { .. },
                        ) => Err(anyhow::Error::new(error)
                            .context(MissingRequiredInput::new("--serial"))),
                        Err(error) => Err(error.into()),
                    },
                }
                .with_context(|| format!("org '{alias}'"))?;

                if !can_prompt {
                    bail!(
                        "a YubiKey operator needs its PIN typed at an interactive prompt; \
                         the PIN is never read from config or the environment"
                    );
                }

                let pin = Pin::from(prompts::password(
                    "YubiKey PIV PIN (touch the device each time it blinks)",
                )?);

                Ok(SelectedYubiKey::new(selected.1.serial, pin))
            })
            .transpose()?;

        let (manifest, fetched) = load_manifest(ctx, &args, &config).await?;
        let manifest = ValidatedManifest::try_from(&manifest)?;

        if !args.dangerous_skip_interactive {
            interactive_approve(ctx, &manifest)?;
        }

        let post_target = if args.dry_run || args.skip_post {
            None
        } else {
            let manifest_id = fetched
                .as_ref()
                .map(|(id, _)| *id)
                .or(args.manifest_id)
                .ok_or(ApproveInputError::MissingManifestId)?;

            Some(PostTarget {
                manifest_id,
                deploy_id: args.deploy_id,
            })
        };

        let inputs = ResolvedApproveInputs {
            manifest,
            operator_seed_source: args.operator_seed_source,
            operator_id,
            approval_out: args.approval_out,
            dry_run: args.dry_run,
            skip_post: args.skip_post,
            selected_yubikey,
            post_target,
            posted_approvals: fetched.map(|(_, approvals)| approvals).unwrap_or_default(),
        };

        run_with_resolved_inputs(ctx, inputs, &config).await
    }
}

// TODO: this file is a bit of a mess
// and is almost 1k lines before the tests mod
// it should probably be split up into submodules

struct PostApprovalPlan<'a> {
    manifest_id: Uuid,
    operator_id: &'a Uuid,
    deploy_id: Option<&'a Uuid>,
}

/// What `post_approval_to_api` learned from the API: the created approval IDs
/// and whether the manifest approval quorum is now reached (`None` when the
/// post-check deployment fetch failed or was not attempted).
struct PostedApproval {
    approval_ids: Vec<String>,
    quorum_reached: Option<bool>,
}

/// Terminal shapes for `deploy approve` (reasons share the `manifest_approval_*` prefix).
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

impl From<ApproveOutcome> for Outcome {
    fn from(outcome: ApproveOutcome) -> Self {
        match outcome {
            ApproveOutcome::Posted(msg) => Outcome::ManifestApprovalPosted(msg),
            ApproveOutcome::NotPosted(msg) => Outcome::ManifestApprovalGenerated(msg),
            ApproveOutcome::AlreadyPosted(msg) => Outcome::ManifestApprovalAlreadyPosted(msg),
            ApproveOutcome::DryRun(msg) => Outcome::ManifestApprovalDryRun(msg),
        }
    }
}

/// The approval payload as an outcome reports it: inline when it went to
/// stdout, the file path when `--approval-out` wrote it.
#[derive(Display, Serialize)]
#[serde(rename_all = "camelCase")]
enum ApprovalOutput {
    /// Approval written to: {0}
    WrittenTo(PathBuf),
    /// {0}
    Approval(JsonPretty<Approval>),
}

#[derive(Serialize)]
struct JsonPretty<T>(T);

impl<T: Serialize> std::fmt::Display for JsonPretty<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string_pretty(self).map_err(|_| fmt::Error)?;
        f.write_str(&json)
    }
}

impl<T> Deref for JsonPretty<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for JsonPretty<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl ApprovalOutput {
    fn new(approval: Approval, approval_out: Option<PathBuf>) -> Self {
        match approval_out {
            Some(path) => Self::WrittenTo(path),
            None => Self::Approval(approval.into()),
        }
    }
}

// `#[default]` only applies to unit variants, so the registry-fixture Default
// is a manual, test-only impl.
#[cfg(test)]
impl Default for ApprovalOutput {
    fn default() -> Self {
        Self::WrittenTo(Default::default())
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct ApprovalPosted {
    #[serde(flatten)]
    approval_or_path: ApprovalOutput,
    manifest_id: String,
    operator_id: String,
    approval_ids: Vec<String>,
    /// `None` when the post-check deployment fetch failed or was not
    /// attempted (no `--deploy-id`), so quorum state is unknown.
    quorum_reached: Option<bool>,
}

impl fmt::Display for ApprovalPosted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"{}
Approval posted successfully!

Approval IDs: {:?}
Manifest ID: {}
Operator ID: {}"#,
            self.approval_or_path, self.approval_ids, self.manifest_id, self.operator_id
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

#[derive(Display, Serialize)]
#[cfg_attr(test, derive(Default))]
/// {0}
pub struct ApprovalGenerated(ApprovalOutput);

#[derive(Display, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Operator {operator_id} has already approved this manifest (approval ID: {approval_id}). Nothing to post.
pub struct ApprovalAlreadyPosted {
    operator_id: String,
    approval_id: String,
}

#[derive(Default, Display, Serialize)]
/// Dry run complete. No approval generated.
pub struct ApprovalDryRun;

struct PostTarget {
    manifest_id: Uuid,
    deploy_id: Option<Uuid>,
}

/// Approve inputs that cannot be resolved into a postable approval; carries
/// the user-facing remediation and converts into `anyhow` at the `?` site.
#[derive(Debug, thiserror::Error)]
enum ApproveInputError {
    #[error(
        "--manifest-id is required to post approval to API (or use --deploy-id). \
         Use --skip-post to only generate the approval locally."
    )]
    MissingManifestId,
}

struct ResolvedApproveInputs<'a> {
    manifest: ValidatedManifest<'a>,
    operator_seed_source: Option<LocalOperatorSeedSource>,
    operator_id: Option<Uuid>,
    approval_out: Option<PathBuf>,
    dry_run: bool,
    skip_post: bool,
    selected_yubikey: Option<SelectedYubiKey>,
    post_target: Option<PostTarget>,
    /// Present only when the manifest came from a deployment fetch: the
    /// already-posted approvals, parsed at that boundary, for validation and
    /// duplicate checks.
    posted_approvals: Vec<OperatorApproval>,
}

/// [`Args`] with the mutually-exclusive seed flags already parsed into a
/// seed source, so input resolution can no longer observe the raw flags.
struct ArgsWithResolvedOperatorSeedSource {
    manifest: Option<PathBuf>,
    deploy_id: Option<Uuid>,
    manifest_id: Option<Uuid>,
    operator_id: Option<Uuid>,
    serial: Option<YubiKeySerial>,
    operator_seed_source: Option<LocalOperatorSeedSource>,
    dry_run: bool,
    dangerous_skip_interactive: bool,
    approval_out: Option<PathBuf>,
    skip_post: bool,
}

impl TryFrom<Args> for ArgsWithResolvedOperatorSeedSource {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> anyhow::Result<Self> {
        let Args {
            operator_seed,
            operator_seed_path,
            manifest,
            deploy_id,
            manifest_id,
            operator_id,
            serial,
            dry_run,
            dangerous_skip_interactive,
            approval_out,
            skip_post,
        } = args;

        let operator_seed_source =
            LocalOperatorSeedSource::from_args(operator_seed, operator_seed_path)?;

        Ok(Self {
            manifest,
            deploy_id,
            manifest_id,
            operator_id,
            serial,
            operator_seed_source,
            dry_run,
            dangerous_skip_interactive,
            approval_out,
            skip_post,
        })
    }
}

/// A candidate for posting without `--operator-id`: the parsed ID that
/// resolution needs, displayed with its registry name when it has one.
struct ApprovingOperator {
    id: Uuid,
    name: Option<String>,
}

impl fmt::Display for ApprovingOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{name} ({})", self.id),
            None => fmt::Display::fmt(&self.id, f),
        }
    }
}

async fn run_with_resolved_inputs(
    ctx: &mut StdCtx,
    inputs: ResolvedApproveInputs<'_>,
    config: &Config,
) -> anyhow::Result<ApproveOutcome> {
    if inputs.dry_run {
        return Ok(ApproveOutcome::DryRun(ApprovalDryRun));
    }

    let validation = inputs.manifest.validate(inputs.posted_approvals);

    validation
        .approvals
        .iter()
        .filter(|validated| validated.verdict != ApprovalVerdict::Valid)
        .for_each(|validated| {
            let message = format!(
                "existing approval from {} is {}; enclave will reject this approval and fail to start",
                validated.approval, validated.verdict
            );

            if let Err(error) = ctx.shell().human().warn(message) {
                debug!(%error, "failed to write approval warning");
            }
        });

    let requirement = if inputs.skip_post {
        SignerRequirement::OfflineApproval
    } else {
        SignerRequirement::Any
    };

    let operator = config
        .resolve_operator(
            yubikey::open,
            inputs.operator_seed_source,
            inputs.operator_id,
            requirement,
            inputs.selected_yubikey,
        )
        .await?;

    let approval = operator.approve_manifest(&inputs.manifest).await?;

    // Reporting the approval (inline payload or file path) is the terminal
    // outcome's job; `--approval-out` additionally persists it here.
    if let Some(path) = inputs.approval_out.as_deref() {
        let json = serde_json::to_string_pretty(&approval)?;
        write_file(path, &json).await?;
    }

    match inputs.post_target {
        Some(target) => {
            let operator_id = operator
                .id()
                .context("resolved operator ID required to post approval")?;

            // An approval already posted by this operator, matched by
            // operator ID or by the manifest set member public key.
            let existing = validation.approvals.iter().find(|validated| {
                validated.approval.operator_id == operator_id
                    || validated.approval.public_key.to_bytes() == approval.member.pub_key
            });

            if let Some(existing) = existing {
                return Ok(ApproveOutcome::AlreadyPosted(ApprovalAlreadyPosted {
                    operator_id: operator_id.to_string(),
                    approval_id: existing.approval.id.to_string(),
                }));
            }

            let plan = PostApprovalPlan {
                manifest_id: target.manifest_id,
                operator_id: &operator_id,
                deploy_id: target.deploy_id.as_ref(),
            };
            let posted =
                post_approval_to_api(ctx, plan, &approval, &inputs.manifest, config).await?;

            Ok(ApproveOutcome::Posted(ApprovalPosted {
                approval_or_path: ApprovalOutput::new(approval, inputs.approval_out),
                manifest_id: target.manifest_id.to_string(),
                operator_id: operator_id.to_string(),
                approval_ids: posted.approval_ids,
                quorum_reached: posted.quorum_reached,
            }))
        }
        None => Ok(ApproveOutcome::NotPosted(ApprovalGenerated(
            ApprovalOutput::new(approval, inputs.approval_out),
        ))),
    }
}

async fn load_manifest(
    ctx: &mut StdCtx,
    args: &ArgsWithResolvedOperatorSeedSource,
    config: &Config,
) -> anyhow::Result<(VersionedManifest, Option<(Uuid, Vec<OperatorApproval>)>)> {
    match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => Ok((read_manifest_from_path(path).await?, None)),
        (_, Some(deploy_id)) => {
            let (manifest, manifest_id, approvals) =
                fetch_manifest_from_deploy(ctx, &deploy_id.to_string(), config).await?;
            Ok((manifest, Some((manifest_id, approvals))))
        }
        (None, None) => bail!("a manifest source is required"),
    }
}

#[instrument(skip_all, fields(manifest_id = %plan.manifest_id, operator_id = %plan.operator_id, deploy_id = ?plan.deploy_id))]
async fn post_approval_to_api(
    ctx: &mut StdCtx,
    plan: PostApprovalPlan<'_>,
    approval: &Approval,
    manifest: &ValidatedManifest<'_>,
    config: &Config,
) -> anyhow::Result<PostedApproval> {
    shell_println!(ctx)?;
    shell_println!(ctx, "Posting approval to Turnkey...")?;

    let auth = crate::client::build_client(config).await?;

    let tvc_approval = TvcManifestApproval {
        operator_id: plan.operator_id.to_string(),
        signature: hex::encode(&approval.signature),
    };

    let intent = CreateTvcManifestApprovalsIntent {
        manifest_id: plan.manifest_id.to_string(),
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
                Ok(response) => match response.tvc_deployment {
                    Some(deployment) => {
                        let approvals = deployment
                            .manifest_approvals
                            .into_iter()
                            .map(OperatorApproval::try_from)
                            .collect::<Result<Vec<_>, _>>()?;

                        Some(manifest.validate(approvals).quorum_reached())
                    }
                    None => Some(false),
                },
                Err(error) => {
                    debug!(
                        %deploy_id,
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

    review_schema(ctx, manifest)?;
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

fn render_schema(manifest: &VersionedManifest) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "MANIFEST SCHEMA");
    let _ = writeln!(s, "─────────────────────────────────────");
    // The schema branch shown here is the one the enclave will execute, which
    // is the decoded variant rather than any version string in the JSON.
    let (version, dns) = match manifest {
        VersionedManifest::V2(_) => (
            "v2",
            match manifest.dns_config() {
                Some(dns) if dns.resolvers.is_empty() => {
                    "(empty; /etc/resolv.conf cleared)".to_owned()
                }
                Some(dns) => dns
                    .resolvers
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                None => "(unset; /etc/resolv.conf unchanged)".to_owned(),
            },
        ),
        VersionedManifest::V1(_) => (
            "v1 (legacy)",
            "(not in v1 schema; effective: unchanged)".to_owned(),
        ),
        VersionedManifest::V0(_) => (
            "v0 (legacy)",
            "(not in v0 schema; effective: unchanged)".to_owned(),
        ),
        _ => ("unrecognized", "(unknown schema)".to_owned()),
    };
    let _ = writeln!(s, "  Version:       {version}");
    let _ = writeln!(s, "  DNS Resolvers: {dns}");
    let _ = writeln!(s);
    s
}

fn review_schema(ctx: &mut StdCtx, manifest: &VersionedManifest) -> anyhow::Result<()> {
    shell_print!(ctx, "{}", render_schema(manifest))?;
    prompts::confirm_or_bail("Approve manifest schema and DNS?", "approval")
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
    let _ = writeln!(
        s,
        "  Restart Policy: {}",
        match manifest.restart() {
            RestartPolicy::Never => "Never",
            RestartPolicy::Always => "Always",
        }
    );
    if let VersionedManifest::V0(_) = manifest {
        let _ = writeln!(s, "  Bridge Config: (not in v0 schema; effective: none)");
        let _ = writeln!(s, "  Debug Mode: (not in v0 schema; effective: disabled)");
    } else {
        if manifest.bridge_config().is_empty() {
            let _ = writeln!(s, "  Bridge Config: (none)");
        } else {
            let _ = writeln!(s, "  Bridge Config:");
            for bridge in manifest.bridge_config() {
                let _ = writeln!(s, "   {}", render_bridge(bridge));
            }
        }
        let _ = writeln!(
            s,
            "  Debug Mode: {}",
            if manifest.debug_mode() {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
    let _ = writeln!(s);
    s
}

fn render_bridge(bridge: &BridgeConfig) -> String {
    match bridge {
        BridgeConfig::Server { port, host } => {
            format!("Server bridge: host bind={host}:{port}; enclave pivot target=127.0.0.1:{port}")
        }
        BridgeConfig::Client { port, host } => {
            let host = host.as_deref().unwrap_or("(none)");
            format!(
                "Client bridge: requests transparent egress; configured host={host}, port={port} ignored"
            )
        }
    }
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
#[instrument(skip(ctx, config))]
async fn fetch_manifest_from_deploy(
    ctx: &mut StdCtx,
    deploy_id: &str,
    config: &Config,
) -> anyhow::Result<(VersionedManifest, Uuid, Vec<OperatorApproval>)> {
    shell_println!(ctx, "Fetching deployment {deploy_id}...")?;

    let auth = build_client(config).await?;

    let request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: deploy_id.to_string(),
    };

    let response = auth
        .client
        .get_tvc_deployment(request)
        .await
        .with_context(|| format!("failed to fetch deployment {deploy_id}"))?;

    let deployment = response
        .tvc_deployment
        .ok_or_else(|| MissingResource::new("deployment", deploy_id.to_string()))?;

    let tvc_manifest = deployment
        .manifest
        .as_ref()
        .ok_or_else(|| MissingResource::new("manifest", format!("deployment {deploy_id}")))?;

    let manifest = VersionedManifest::try_from_slice_compat(&tvc_manifest.manifest)
        .context("failed to parse manifest from deployment")?;

    let manifest_id = tvc_manifest
        .id
        .parse::<Uuid>()
        .with_context(|| format!("manifest ID '{}' is not a UUID", tvc_manifest.id))?;

    let approvals = deployment
        .manifest_approvals
        .into_iter()
        .map(OperatorApproval::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    shell_println!(ctx, "✓ Manifest loaded (manifest_id: {manifest_id})")?;

    Ok((manifest, manifest_id, approvals))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_manifest() -> VersionedManifest {
        VersionedManifest::try_from_slice_compat(include_bytes!("../../../fixtures/manifest.json"))
            .expect("fixture manifest should parse")
    }

    fn fixture_json() -> serde_json::Value {
        serde_json::from_slice(include_bytes!("../../../fixtures/manifest.json"))
            .expect("fixture manifest should be valid JSON")
    }

    fn manifest_from_value(value: serde_json::Value) -> VersionedManifest {
        VersionedManifest::try_from_slice_compat(&serde_json::to_vec(&value).unwrap())
            .expect("mutated fixture manifest should parse")
    }

    fn fixture_manifest_v0() -> VersionedManifest {
        let mut value = fixture_json();
        let pivot = value["pivot"].as_object_mut().unwrap();
        pivot.remove("bridgeConfig");
        pivot.remove("debugMode");
        let manifest = manifest_from_value(value);
        assert!(matches!(manifest, VersionedManifest::V0(_)));
        manifest
    }

    fn fixture_manifest_v2(dns_resolvers: Option<Vec<&str>>) -> VersionedManifest {
        let mut value = fixture_json();
        value.as_object_mut().unwrap().remove("patchSet");
        value["version"] = "v2".into();
        if let Some(resolvers) = dns_resolvers {
            value["dns"] = serde_json::json!({ "resolvers": resolvers });
        }
        let manifest = manifest_from_value(value);
        assert!(matches!(manifest, VersionedManifest::V2(_)));
        manifest
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
    fn render_pivot_includes_restart_policy_bridge_config_and_debug_mode() {
        let rendered = render_pivot(&fixture_manifest());
        assert!(rendered.contains("Restart Policy: Never"));
        assert!(rendered.contains("Bridge Config:"));
        assert!(rendered.contains(
            "Server bridge: host bind=0.0.0.0:3000; enclave pivot target=127.0.0.1:3000"
        ));
        assert!(rendered.contains("Debug Mode: disabled"));
    }

    #[test]
    fn render_pivot_renders_client_bridges_as_egress_and_flags_enabled_debug_mode() {
        let mut value = fixture_json();
        value["pivot"]["bridgeConfig"] = serde_json::json!([
            { "type": "client", "port": 4000, "host": "api.example.com" },
            { "type": "client", "port": 4001 },
        ]);
        value["pivot"]["debugMode"] = true.into();
        let rendered = render_pivot(&manifest_from_value(value));
        assert!(rendered.contains(
            "Client bridge: requests transparent egress; configured host=api.example.com, port=4000 ignored"
        ));
        assert!(rendered.contains(
            "Client bridge: requests transparent egress; configured host=(none), port=4001 ignored"
        ));
        assert!(rendered.contains("Debug Mode: enabled"));
    }

    #[test]
    fn render_pivot_empty_bridge_config_is_explicit() {
        let mut value = fixture_json();
        value["pivot"]["bridgeConfig"] = serde_json::json!([]);
        let rendered = render_pivot(&manifest_from_value(value));
        assert!(rendered.contains("Bridge Config: (none)"));
    }

    #[test]
    fn render_pivot_v0_marks_bridge_config_and_debug_mode_absent() {
        let rendered = render_pivot(&fixture_manifest_v0());
        assert!(rendered.contains("Restart Policy: Never"));
        assert!(rendered.contains("Bridge Config: (not in v0 schema; effective: none)"));
        assert!(rendered.contains("Debug Mode: (not in v0 schema; effective: disabled)"));
    }

    #[test]
    fn render_schema_v2_lists_dns_resolvers() {
        let rendered = render_schema(&fixture_manifest_v2(Some(vec!["1.1.1.1", "8.8.8.8"])));
        assert!(rendered.contains("MANIFEST SCHEMA"));
        assert!(rendered.contains("Version:       v2"));
        assert!(rendered.contains("DNS Resolvers: 1.1.1.1, 8.8.8.8"));
    }

    #[test]
    fn render_schema_v2_without_dns_is_explicit() {
        let rendered = render_schema(&fixture_manifest_v2(None));
        assert!(rendered.contains("Version:       v2"));
        assert!(rendered.contains("DNS Resolvers: (unset; /etc/resolv.conf unchanged)"));
    }

    #[test]
    fn render_schema_v2_empty_dns_list_reports_resolv_conf_is_emptied() {
        let rendered = render_schema(&fixture_manifest_v2(Some(vec![])));
        assert!(rendered.contains("DNS Resolvers: (empty; /etc/resolv.conf cleared)"));
    }

    #[test]
    fn render_schema_v1_and_v0_mark_dns_unsupported() {
        let rendered = render_schema(&fixture_manifest());
        assert!(rendered.contains("Version:       v1 (legacy)"));
        assert!(rendered.contains("DNS Resolvers: (not in v1 schema; effective: unchanged)"));

        let rendered = render_schema(&fixture_manifest_v0());
        assert!(rendered.contains("Version:       v0 (legacy)"));
        assert!(rendered.contains("DNS Resolvers: (not in v0 schema; effective: unchanged)"));
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

    fn posted_to_file() -> ApprovalPosted {
        ApprovalPosted {
            approval_or_path: ApprovalOutput::WrittenTo("approval.json".into()),
            manifest_id: "manifest-123".to_string(),
            operator_id: "operator-456".to_string(),
            approval_ids: vec!["approval-1".to_string()],
            quorum_reached: Some(true),
        }
    }

    #[test]
    fn approval_posted_serializes_expected_json() {
        let value =
            serde_json::to_value(Outcome::from(ApproveOutcome::Posted(posted_to_file()))).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest_approval_posted",
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
        let generated = ApprovalGenerated(ApprovalOutput::Approval(
            Approval {
                signature: vec![0xde, 0xad],
                member: QuorumMember {
                    alias: "operator-alice".to_string(),
                    pub_key: vec![0xaa],
                },
            }
            .into(),
        ));

        let value =
            serde_json::to_value(Outcome::from(ApproveOutcome::NotPosted(generated))).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest_approval_generated",
                "approval": {
                    "signature": "dead",
                    "member": {
                        "alias": "operator-alice",
                        "pubKey": "aa",
                    },
                },
                "reason": "manifest_approval_generated"
            })
        );
    }

    #[test]
    fn approval_already_posted_serializes_expected_json() {
        let already_posted = ApprovalAlreadyPosted {
            operator_id: "operator-456".to_string(),
            approval_id: "approval-1".to_string(),
        };

        let value =
            serde_json::to_value(Outcome::from(ApproveOutcome::AlreadyPosted(already_posted)))
                .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "manifest_approval_already_posted",
                "operatorId": "operator-456",
                "approvalId": "approval-1",
            })
        );
    }

    #[test]
    fn approval_dry_run_serializes_reason_only() {
        let value =
            serde_json::to_value(Outcome::from(ApproveOutcome::DryRun(ApprovalDryRun))).unwrap();

        assert_eq!(
            value,
            serde_json::json!({ "reason": "manifest_approval_dry_run" })
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
