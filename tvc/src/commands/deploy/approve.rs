//! Approve deploy command - cryptographically approve a QOS manifest.

use crate::config::turnkey::Config;
use crate::operator_key::load_operator_pair;
use crate::prompts;
use crate::prompts::{bail_required_in_non_interactive, stdin_can_prompt};
use crate::util::{read_file_to_string, write_file};
use anyhow::{Context, anyhow, bail};
use clap::{ArgGroup, Args as ClapArgs};
use qos_core::protocol::services::boot::Approval;
use qos_core::protocol::services::boot::{
    ManifestSet, Namespace, NitroConfig, QuorumMember, ShareSet, VersionedManifest,
};
use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use turnkey_client::generated::external::data::v1::TvcDeployment;
use turnkey_client::generated::{
    CreateTvcManifestApprovalsIntent, GetTvcDeploymentRequest, TvcManifestApproval,
};

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

    /// Turnkey operator ID (UUID) for the approving operator.
    /// Required when posting approval to the API.
    #[arg(long, env = "TVC_OPERATOR_ID")]
    pub operator_id: Option<String>,

    /// Path to the file containing the master seed for the operator key.
    /// If not provided, uses the operator key from the logged-in org config.
    #[arg(long, value_name = "PATH", env = "TVC_OPERATOR_SEED")]
    pub operator_seed: Option<PathBuf>,

    /// Walk through manifest approval prompts but do not generate an approval.
    #[arg(long, env = "TVC_DRY_RUN")]
    pub dry_run: bool,

    /// DANGEROUS: skip interactive prompts for approving each aspect of manifest.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_INTERACTIVE")]
    pub dangerous_skip_interactive: bool,

    /// Write approval to file instead of stdout.
    #[arg(short = 'o', long, value_name = "PATH", env = "TVC_APPROVAL_OUT")]
    pub approval_out: Option<PathBuf>,

    /// Don't post approval to the API.
    #[arg(long, env = "TVC_SKIP_POST")]
    pub skip_post: bool,
}

struct PostApprovalPlan<'a> {
    manifest_id: &'a str,
    operator_id: &'a str,
    deploy_id: Option<&'a str>,
}

struct PostTarget {
    manifest_id: String,
    operator_id: String,
    deploy_id: Option<String>,
}

struct ResolvedApproveInputs {
    manifest: VersionedManifest,
    operator_seed: Option<PathBuf>,
    approval_out: Option<PathBuf>,
    dry_run: bool,
    post_target: Option<PostTarget>,
}

pub async fn run(args: Args, is_non_interactive: bool) -> anyhow::Result<()> {
    let inputs = if is_non_interactive {
        build_inputs_non_interactive(args).await?
    } else {
        build_inputs_interactive(args).await?
    };

    run_with_resolved_inputs(inputs).await
}

async fn build_inputs_interactive(args: Args) -> anyhow::Result<ResolvedApproveInputs> {
    let do_prompt_user = !args.dangerous_skip_interactive;

    // Guard: bail fast before fetching the manifest if review prompts are
    // required but the caller has no TTY to answer them.
    if do_prompt_user && !stdin_can_prompt() {
        bail_required_in_non_interactive("--dangerous-skip-interactive")?;
    }

    let (manifest, fetched_manifest_id) = load_manifest(&args).await?;

    if do_prompt_user {
        interactive_approve(&manifest)?;
    }

    let post_target = if !args.dry_run && !args.skip_post {
        let manifest_id = resolve_manifest_id(&args, fetched_manifest_id.as_deref())?;
        let operator_id = match &args.operator_id {
            Some(id) => id.clone(),
            None => {
                let saved_ids = load_saved_operator_ids().await?;
                match &*saved_ids {
                    [] => bail!(
                        "--operator-id is required to post approval to API. \
                         No saved operator IDs found. \
                         Use --skip-post to only generate the approval locally."
                    ),
                    [id] => id.clone(),
                    _ if stdin_can_prompt() => prompts::select(
                        "Select approving operator",
                        saved_ids.iter().map(String::as_str).collect::<Vec<_>>(),
                    )?
                    .to_string(),
                    _ => bail!(
                        "--operator-id is required to post approval to API when multiple saved operator IDs are available"
                    ),
                }
            }
        };
        Some(PostTarget {
            manifest_id,
            operator_id,
            deploy_id: args.deploy_id.clone(),
        })
    } else {
        None
    };

    Ok(ResolvedApproveInputs {
        manifest,
        operator_seed: args.operator_seed,
        approval_out: args.approval_out,
        dry_run: args.dry_run,
        post_target,
    })
}

async fn build_inputs_non_interactive(args: Args) -> anyhow::Result<ResolvedApproveInputs> {
    if !args.dangerous_skip_interactive {
        bail_required_in_non_interactive("--dangerous-skip-interactive")?;
    }

    let (manifest, fetched_manifest_id) = load_manifest(&args).await?;

    let post_target = if !args.dry_run && !args.skip_post {
        let manifest_id = resolve_manifest_id(&args, fetched_manifest_id.as_deref())?;
        let operator_id = match &args.operator_id {
            Some(id) => id.clone(),
            None => {
                let saved_ids = load_saved_operator_ids().await?;
                match &*saved_ids {
                    [] => bail!(
                        "--operator-id is required to post approval to API. \
                         No saved operator IDs found. \
                         Use --skip-post to only generate the approval locally."
                    ),
                    [id] => id.clone(),
                    _ => bail!(
                        "--operator-id is required to post approval to API when multiple saved operator IDs are available"
                    ),
                }
            }
        };
        Some(PostTarget {
            manifest_id,
            operator_id,
            deploy_id: args.deploy_id.clone(),
        })
    } else {
        None
    };

    Ok(ResolvedApproveInputs {
        manifest,
        operator_seed: args.operator_seed,
        approval_out: args.approval_out,
        dry_run: args.dry_run,
        post_target,
    })
}

async fn run_with_resolved_inputs(inputs: ResolvedApproveInputs) -> anyhow::Result<()> {
    if inputs.dry_run {
        println!("Dry run complete. No approval generated.");
        return Ok(());
    }

    let approval = sign_and_output(
        inputs.operator_seed.as_deref(),
        inputs.approval_out.as_deref(),
        &inputs.manifest,
    )
    .await?;

    if let Some(target) = inputs.post_target {
        let plan = PostApprovalPlan {
            manifest_id: target.manifest_id.as_str(),
            operator_id: target.operator_id.as_str(),
            deploy_id: target.deploy_id.as_deref(),
        };
        post_approval_to_api(plan, &approval).await?;
    }

    Ok(())
}

async fn load_manifest(args: &Args) -> anyhow::Result<(VersionedManifest, Option<String>)> {
    match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => Ok((read_manifest_from_path(path).await?, None)),
        (_, Some(deploy_id)) => {
            let (manifest, manifest_id) = fetch_manifest_from_deploy(deploy_id).await?;
            Ok((manifest, Some(manifest_id)))
        }
        (None, None) => bail!("a manifest source is required"),
    }
}

async fn sign_and_output(
    operator_seed: Option<&Path>,
    approval_out: Option<&Path>,
    manifest: &VersionedManifest,
) -> anyhow::Result<Approval> {
    let pair: Box<dyn crate::pair::Pair> = Box::new(load_operator_pair(operator_seed).await?);

    let approval = generate_approval(pair, manifest).await?;
    let json = serde_json::to_string_pretty(&approval)?;

    if let Some(path) = approval_out {
        write_file(path, &json).await?;
        println!("Approval written to: {}", path.display());
    } else {
        println!("{json}");
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

async fn load_saved_operator_ids() -> anyhow::Result<Vec<String>> {
    let config = Config::load().await?;
    Ok(config.get_last_operator_ids().unwrap_or_default())
}

async fn post_approval_to_api(
    plan: PostApprovalPlan<'_>,
    approval: &Approval,
) -> anyhow::Result<()> {
    println!();
    println!("Posting approval to Turnkey...");

    let auth = crate::client::build_client().await?;

    let tvc_approval = TvcManifestApproval {
        operator_id: plan.operator_id.into(),
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

    let result = match auth
        .client
        .create_tvc_manifest_approvals(auth.org_id.clone(), timestamp_ms, intent)
        .await
    {
        Ok(result) => result,
        Err(error) => {
            if let Some(activity) = crate::commands::consensus::pending_consensus_from_error(&error)
            {
                return crate::commands::consensus::pending_consensus_result(&activity);
            }
            return Err(error).context("failed to post manifest approval");
        }
    };

    println!();
    println!("Approval posted successfully!");
    println!();
    println!("Approval IDs: {:?}", result.result.approval_ids);
    println!("Manifest ID: {}", plan.manifest_id);
    println!("Operator ID: {}", plan.operator_id);

    if let Some(deploy_id) = plan.deploy_id {
        let request = GetTvcDeploymentRequest {
            organization_id: auth.org_id.clone(),
            deployment_id: deploy_id.to_string(),
        };

        match auth.client.get_tvc_deployment(request).await {
            Ok(response) => {
                println!();

                if response
                    .tvc_deployment
                    .as_ref()
                    .is_some_and(manifest_approval_quorum_reached)
                {
                    println!("{QUORUM_REACHED_MESSAGE}");
                } else {
                    println!(
                        "Your approval has been posted. Deployment requires additional manifest approvals before it can be deployed on TVC."
                    );
                };
            }
            Err(error) => {
                debug!(
                    deploy_id,
                    %error,
                    "failed to fetch deployment after posting manifest approval"
                );
            }
        }
    }

    Ok(())
}

fn manifest_approval_quorum_reached(deployment: &TvcDeployment) -> bool {
    let Some(manifest_set) = &deployment.manifest_set else {
        return false;
    };

    if manifest_set.threshold == 0 {
        return false;
    }

    let mut operator_ids = HashSet::new();
    for approval in &deployment.manifest_approvals {
        let Some(operator) = &approval.operator else {
            return false;
        };

        if operator.id.is_empty() {
            return false;
        }

        operator_ids.insert(operator.id.as_str());
    }

    operator_ids.len() >= manifest_set.threshold as usize
}

async fn generate_approval(
    pair: Box<dyn crate::pair::Pair>,
    manifest: &VersionedManifest,
) -> anyhow::Result<Approval> {
    let public_key = pair.public_key();
    let member = manifest
        .manifest_set()
        .members
        .iter()
        .find(|m| m.pub_key == public_key)
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "operator ({}) not part of manifest set",
                hex::encode(&public_key)
            )
        })?;

    let signature = pair.sign(manifest.manifest_hash().to_vec()).await?;

    Ok(Approval { signature, member })
}

/// Walk the user through each section of the manifest for approval.
fn interactive_approve(manifest: &VersionedManifest) -> anyhow::Result<()> {
    println!("\n========================================");
    println!("         MANIFEST APPROVAL");
    println!("========================================\n");

    review_namespace(manifest.namespace())?;
    review_enclave(manifest.enclave())?;
    review_pivot(manifest)?;
    review_manifest_set(manifest.manifest_set())?;
    review_share_set(manifest.share_set())?;

    println!("\n========================================");
    println!("    ALL SECTIONS APPROVED");
    println!("========================================\n");

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

fn review_namespace(namespace: &Namespace) -> anyhow::Result<()> {
    print!("{}", render_namespace(namespace));
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

fn review_enclave(enclave: &NitroConfig) -> anyhow::Result<()> {
    print!("{}", render_enclave(enclave));
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

fn review_pivot(manifest: &VersionedManifest) -> anyhow::Result<()> {
    print!("{}", render_pivot(manifest));
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

fn review_manifest_set(set: &ManifestSet) -> anyhow::Result<()> {
    print!("{}", render_manifest_set(set));
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

fn review_share_set(set: &ShareSet) -> anyhow::Result<()> {
    print!("{}", render_share_set(set));
    prompts::confirm_or_bail("Approve share set?", "approval")
}

async fn read_manifest_from_path(path: &Path) -> anyhow::Result<VersionedManifest> {
    let content = read_file_to_string(path).await?;
    let manifest = VersionedManifest::try_from_slice_compat(content.as_bytes())
        .with_context(|| format!("failed to parse manifest JSON from: {}", path.display()))?;
    Ok(manifest)
}

/// Fetch manifest from Turnkey using GetTvcDeployment API.
/// Returns the manifest and its Turnkey manifest_id.
async fn fetch_manifest_from_deploy(
    deploy_id: &str,
) -> anyhow::Result<(VersionedManifest, String)> {
    println!("Fetching deployment {deploy_id}...");

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
        .ok_or_else(|| anyhow!("manifest not found in deployment"))?;

    let manifest = VersionedManifest::try_from_slice_compat(&tvc_manifest.manifest)
        .context("failed to parse manifest from deployment")?;

    println!("✓ Manifest loaded (manifest_id: {})", tvc_manifest.id);

    Ok((manifest, tvc_manifest.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use turnkey_client::generated::external::data::v1::{
        TvcOperator, TvcOperatorApproval, TvcOperatorSet,
    };

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

    fn test_deployment(
        manifest_threshold: Option<u32>,
        approval_operator_ids: &[Option<&str>],
    ) -> TvcDeployment {
        let operators = approval_operator_ids
            .iter()
            .filter_map(|operator_id| operator_id.map(test_operator))
            .collect();

        TvcDeployment {
            id: "deployment-123".to_string(),
            organization_id: "org-123".to_string(),
            app_id: "app-123".to_string(),
            manifest_set: manifest_threshold.map(|threshold| TvcOperatorSet {
                id: "manifest-set-123".to_string(),
                name: "manifest-set".to_string(),
                organization_id: "org-123".to_string(),
                operators,
                threshold,
                created_at: None,
                updated_at: None,
            }),
            share_set: None,
            manifest: None,
            manifest_approvals: approval_operator_ids
                .iter()
                .enumerate()
                .map(|(index, operator_id)| test_approval(index, *operator_id))
                .collect(),
            qos_version: "qos-v1".to_string(),
            pivot_container: None,
            debug_mode: false,
            created_at: None,
            updated_at: None,
            delete: false,
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
    fn manifest_approval_quorum_reached_is_false_below_threshold() {
        let deployment = test_deployment(Some(2), &[Some("operator-1")]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_reached_is_true_at_threshold() {
        let deployment = test_deployment(Some(2), &[Some("operator-1"), Some("operator-2")]);

        assert!(manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_counts_distinct_operators() {
        let deployment = test_deployment(Some(2), &[Some("operator-1"), Some("operator-1")]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_is_false_without_manifest_set() {
        let deployment = test_deployment(None, &[Some("operator-1"), Some("operator-2")]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_is_false_with_zero_threshold() {
        let deployment = test_deployment(Some(0), &[Some("operator-1")]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_is_false_when_approval_lacks_operator() {
        let deployment = test_deployment(Some(2), &[Some("operator-1"), None]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }

    #[test]
    fn manifest_approval_quorum_is_false_when_operator_id_is_empty() {
        let deployment = test_deployment(Some(1), &[Some("")]);

        assert!(!manifest_approval_quorum_reached(&deployment));
    }
}
