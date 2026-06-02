//! Approve deploy command - cryptographically approve a QOS manifest.

use crate::config::turnkey::Config;
use crate::operator_key::load_operator_pair;
use crate::prompts;
use crate::util::{read_file_to_string, write_file};
use anyhow::{Context, anyhow, bail};
use clap::{ArgGroup, Args as ClapArgs};
use qos_core::protocol::QosHash;
use qos_core::protocol::services::boot::Approval;
use qos_core::protocol::services::boot::{
    Manifest, ManifestSet, Namespace, NitroConfig, PivotConfig, QuorumMember, ShareSet,
};
use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{
    CreateTvcManifestApprovalsIntent, GetTvcDeploymentRequest, TvcManifestApproval,
};

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

/// Run the approve deploy command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let do_prompt_user = !args.dangerous_skip_interactive;

    // Guard: Bail fast before fetching the manifest if we cannot prompt the user
    if do_prompt_user {
        prompts::bail_if_non_interactive("--dangerous-skip-interactive")?;
    }

    // Fetch manifest - track manifest_id if fetched from API
    let (manifest, fetched_manifest_id) = match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => (read_manifest_from_path(path).await?, None),
        (_, Some(deploy_id)) => {
            let (manifest, manifest_id) = fetch_manifest_from_deploy(deploy_id).await?;
            (manifest, Some(manifest_id))
        }
        (None, None) => bail!("a manifest source is required"),
    };

    if do_prompt_user {
        interactive_approve(&manifest)?;
    }

    if args.dry_run {
        println!("Dry run complete. No approval generated.");
        return Ok(());
    }

    let pair: Box<dyn crate::pair::Pair> =
        Box::new(load_operator_pair(args.operator_seed.as_deref()).await?);

    let approval = generate_approval(pair, &manifest).await?;
    let json = serde_json::to_string_pretty(&approval)?;

    // Write to file or stdout
    if let Some(ref path) = args.approval_out {
        write_file(path, &json).await?;
        println!("Approval written to: {}", path.display());
    } else {
        println!("{json}");
    }

    // Post to API if not skipped
    if !args.skip_post {
        post_approval_to_api(&args, &approval, fetched_manifest_id.as_deref()).await?;
    }

    Ok(())
}

async fn post_approval_to_api(
    args: &Args,
    approval: &Approval,
    fetched_manifest_id: Option<&str>,
) -> anyhow::Result<()> {
    // Use fetched manifest_id (from --deploy-id) or fall back to --manifest-id arg
    let manifest_id = fetched_manifest_id
        .map(|s| s.to_string())
        .or_else(|| args.manifest_id.clone())
        .ok_or_else(|| {
            anyhow!(
                "--manifest-id is required to post approval to API (or use --deploy-id). \
                 Use --skip-post to only generate the approval locally."
            )
        })?;

    let operator_id = match &args.operator_id {
        Some(id) => id.clone(),
        None => {
            let config = Config::load().await?;
            let saved_ids = config.get_last_operator_ids().ok_or_else(|| {
                anyhow!(
                    "--operator-id is required to post approval to API. \
                     No saved operator IDs found. \
                     Use --skip-post to only generate the approval locally."
                )
            })?;

            match saved_ids.len() {
                0 => bail!("No operator IDs available"),
                1 => saved_ids[0].clone(),
                _ if prompts::is_interactive() => {
                    prompts::select("Select approving operator", saved_ids.clone())?
                }
                // Non-interactive with multiple saved IDs: pick the first entry.
                // Users who want a specific operator should pass --operator-id.
                _ => saved_ids[0].clone(),
            }
        }
    };

    println!();
    println!("Posting approval to Turnkey...");

    // Build authenticated client
    let auth = crate::client::build_client().await?;

    // Convert local approval to API format
    let tvc_approval = TvcManifestApproval {
        operator_id: operator_id.clone(),
        signature: hex::encode(&approval.signature),
    };

    let intent = CreateTvcManifestApprovalsIntent {
        manifest_id: manifest_id.clone(),
        approvals: vec![tvc_approval],
    };

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Post the approval
    let result = auth
        .client
        .create_tvc_manifest_approvals(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to post manifest approval")?;

    println!();
    println!("Approval posted successfully!");
    println!();
    println!("Approval IDs: {:?}", result.result.approval_ids);
    println!("Manifest ID: {manifest_id}");
    println!("Operator ID: {operator_id}");

    Ok(())
}

async fn generate_approval(
    pair: Box<dyn crate::pair::Pair>,
    manifest: &Manifest,
) -> anyhow::Result<Approval> {
    let public_key = pair.public_key();
    let member = manifest
        .manifest_set
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

    let signature = pair.sign(manifest.qos_hash().to_vec()).await?;

    Ok(Approval { signature, member })
}

/// Walk the user through each section of the manifest for approval.
fn interactive_approve(manifest: &Manifest) -> anyhow::Result<()> {
    println!("\n========================================");
    println!("         MANIFEST APPROVAL");
    println!("========================================\n");

    review_namespace(&manifest.namespace)?;
    review_enclave(&manifest.enclave)?;
    review_pivot(&manifest.pivot)?;
    review_manifest_set(&manifest.manifest_set)?;
    review_share_set(&manifest.share_set)?;

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

fn render_pivot(pivot: &PivotConfig) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "PIVOT BINARY");
    let _ = writeln!(s, "─────────────────────────────────────");
    let _ = writeln!(s, "  Pivot Binary Hash: {}", hex::encode(pivot.hash));
    if pivot.args.is_empty() {
        let _ = writeln!(s, "  CLI Args: (none)");
    } else {
        let _ = writeln!(s, "  CLI Args:\n   {}", pivot.args.join("\n   "));
    }
    let _ = writeln!(s);
    s
}

fn review_pivot(pivot: &PivotConfig) -> anyhow::Result<()> {
    print!("{}", render_pivot(pivot));
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

async fn read_manifest_from_path(path: &Path) -> anyhow::Result<Manifest> {
    let content = read_file_to_string(path).await?;
    let manifest: Manifest = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse manifest JSON from: {}", path.display()))?;
    Ok(manifest)
}

/// Fetch manifest from Turnkey using GetTvcDeployment API.
/// Returns the manifest and its Turnkey manifest_id.
async fn fetch_manifest_from_deploy(deploy_id: &str) -> anyhow::Result<(Manifest, String)> {
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

    let manifest: Manifest = serde_json::from_slice(&tvc_manifest.manifest)
        .context("failed to parse manifest from deployment")?;

    println!("✓ Manifest loaded (manifest_id: {})", tvc_manifest.id);

    Ok((manifest, tvc_manifest.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_manifest() -> Manifest {
        serde_json::from_str(include_str!("../../../fixtures/manifest.json"))
            .expect("fixture manifest should parse")
    }

    #[test]
    fn render_namespace_includes_name_nonce_and_quorum_key() {
        let manifest = fixture_manifest();
        let rendered = render_namespace(&manifest.namespace);
        assert!(rendered.contains("NAMESPACE"));
        assert!(rendered.contains("turnkey-prod"));
        assert!(rendered.contains("Nonce:"));
        assert!(rendered.contains("Quorum Key:"));
    }

    #[test]
    fn render_enclave_includes_all_four_pcrs() {
        let manifest = fixture_manifest();
        let rendered = render_enclave(&manifest.enclave);
        assert!(rendered.contains("ENCLAVE (AWS Nitro)"));
        assert!(rendered.contains("PCR0"));
        assert!(rendered.contains("PCR1"));
        assert!(rendered.contains("PCR2"));
        assert!(rendered.contains("PCR3"));
    }

    #[test]
    fn render_pivot_includes_header_and_args() {
        let manifest = fixture_manifest();
        let rendered = render_pivot(&manifest.pivot);
        assert!(rendered.contains("PIVOT BINARY"));
        assert!(rendered.contains("Pivot Binary Hash:"));
        assert!(rendered.contains("--flag"));
        assert!(rendered.contains("positional"));
    }

    #[test]
    fn render_manifest_set_includes_threshold_and_each_member() {
        let manifest = fixture_manifest();
        let rendered = render_manifest_set(&manifest.manifest_set);
        assert!(rendered.contains("MANIFEST SET"));
        assert!(rendered.contains("Threshold: 2 of 3"));
        assert!(rendered.contains("operator-alice"));
        assert!(rendered.contains("operator-bob"));
        assert!(rendered.contains("operator-charlie"));
    }
}
