//! Approve deploy command - cryptographically approve a QOS manifest.

use crate::config::app::KNOWN_SHARE_SET_KEYS;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::pair::LocalPair;
use crate::util::{read_file_to_string, write_file};
use anyhow::{anyhow, bail, Context};
use clap::{ArgGroup, Args as ClapArgs};
use qos_core::protocol::services::boot::Approval;
use qos_core::protocol::services::boot::{
    Manifest, ManifestSet, Namespace, NitroConfig, PivotConfig, QuorumMember, ShareSet,
};
use qos_core::protocol::QosHash;
use std::io::{BufRead, Write};
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
        help_heading = "Manifest source (pick one)",
        value_name = "PATH"
    )]
    pub manifest: Option<PathBuf>,

    /// ID of the deployment the manifest belongs to.
    #[arg(
        short,
        long,
        help_heading = "Manifest source (pick one)",
        env = "TVC_DEPLOY_ID"
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
    #[arg(long, help_heading = "Operator signing key", value_name = "PATH")]
    pub operator_seed: Option<PathBuf>,

    /// Walk through manifest approval prompts but do not generate an approval.
    #[arg(long)]
    pub dry_run: bool,

    /// DANGEROUS: skip interactive prompts for approving each aspect of manifest.
    #[arg(long)]
    pub dangerous_skip_interactive: bool,

    /// Write approval to file instead of stdout.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Don't post approval to the API.
    #[arg(long)]
    pub skip_post: bool,
}

/// Run the approve deploy command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    // Fetch manifest - track manifest_id if fetched from API
    let (manifest, fetched_manifest_id) = match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => (read_manifest_from_path(path).await?, None),
        (_, Some(deploy_id)) => {
            let (manifest, manifest_id) = fetch_manifest_from_deploy(deploy_id).await?;
            (manifest, Some(manifest_id))
        }
        (None, None) => bail!("a manifest source is required"),
    };

    if !args.dangerous_skip_interactive {
        interactive_approve(&manifest)?;
    }

    if args.dry_run {
        println!("Dry run complete. No approval generated.");
        return Ok(());
    }

    // Get operator key - from --operator-seed or from logged-in config
    let pair: Box<dyn crate::pair::Pair> = match &args.operator_seed {
        Some(path) => Box::new(LocalPair::from_master_seed(path).await?),
        None => {
            // Default to operator key from logged-in org config
            let tvc_config = Config::load().await?;
            let (alias, org_config) = tvc_config.active_org_config().ok_or_else(|| {
                anyhow!("No active organization. Run `tvc login` first or provide --operator-seed.")
            })?;

            let operator_key = StoredQosOperatorKey::load(org_config)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No operator key found for org '{alias}'. \
                     Run `tvc login` first or provide --operator-seed."
                    )
                })?;

            Box::new(LocalPair::from_hex_seed(&operator_key.private_key)?)
        }
    };

    let approval = generate_approval(pair, &manifest).await?;
    let json = serde_json::to_string_pretty(&approval)?;

    // Write to file or stdout
    if let Some(ref path) = args.output {
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
            // Try to load from config
            let config = Config::load().await?;
            let saved_ids = config.get_last_operator_ids().ok_or_else(|| {
                anyhow!(
                    "--operator-id is required to post approval to API. \
                     No saved operator IDs found. \
                     Use --skip-post to only generate the approval locally."
                )
            })?;

            // Use the first operator Id from the list
            saved_ids
                .first()
                .ok_or_else(|| anyhow!("No operator IDs available"))?
                .clone()
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

fn confirm(prompt: &str) -> anyhow::Result<()> {
    print!("{prompt} [y/N]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    let approved = input == "yes" || input == "y";

    if !approved {
        bail!("approval cancelled by user");
    }
    Ok(())
}

fn review_namespace(namespace: &Namespace) -> anyhow::Result<()> {
    println!("NAMESPACE");
    println!("─────────────────────────────────────");
    println!("  Name:       {}", namespace.name);
    println!("  Nonce:      {}", namespace.nonce);
    println!("  Quorum Key: {}", hex::encode(&namespace.quorum_key));
    println!();

    confirm("Approve namespace?")
}

fn review_enclave(enclave: &NitroConfig) -> anyhow::Result<()> {
    println!("ENCLAVE (AWS Nitro)");
    println!("─────────────────────────────────────");
    println!("  PCR0 (image):     {}", hex::encode(&enclave.pcr0));
    println!("  PCR1 (kernel):    {}", hex::encode(&enclave.pcr1));
    println!("  PCR2 (app):       {}", hex::encode(&enclave.pcr2));
    println!("  PCR3 (IAM role):  {}", hex::encode(&enclave.pcr3));
    // Skip the QOS commit since its not cryptographically linked
    println!();

    confirm("Approve enclave configuration?")
}

fn review_pivot(pivot: &PivotConfig) -> anyhow::Result<()> {
    println!("PIVOT BINARY");
    println!("─────────────────────────────────────");
    println!("  Pivot Binary Hash: {}", hex::encode(pivot.hash));
    if pivot.args.is_empty() {
        println!("  CLI Args: (none)");
    } else {
        println!("  CLI Args:\n   {}", pivot.args.join("\n   "));
    }
    println!();

    confirm("Approve pivot binary?")
}

fn print_quorum_members(members: &[QuorumMember]) {
    for member in members.iter() {
        println!("    {} ({})", member.alias, hex::encode(&member.pub_key));
    }
}

fn review_manifest_set(set: &ManifestSet) -> anyhow::Result<()> {
    println!("MANIFEST SET");
    println!("─────────────────────────────────────");
    println!("  Threshold: {} of {}", set.threshold, set.members.len());
    println!("  Members:");
    print_quorum_members(&set.members);
    println!();

    confirm("Approve manifest set?")
}

fn review_share_set(set: &ShareSet) -> anyhow::Result<()> {
    // Verify the share set matches the known keys (no interactive prompt)
    let expected_keys: std::collections::HashSet<Vec<u8>> = KNOWN_SHARE_SET_KEYS
        .iter()
        .map(|(_, key)| hex::decode(key).expect("known key should be valid hex"))
        .collect();

    let actual_keys: std::collections::HashSet<Vec<u8>> =
        set.members.iter().map(|m| m.pub_key.clone()).collect();

    if expected_keys != actual_keys {
        bail!(
            "Share set public keys do not match known keys.\n\
             Expected keys defined in KNOWN_SHARE_SET_KEYS (config/app.rs).\n\
             Found: {:?}",
            set.members
                .iter()
                .map(|m| hex::encode(&m.pub_key))
                .collect::<Vec<_>>()
        );
    }

    if set.threshold != 2 {
        bail!("Share set threshold must be 2, found: {}", set.threshold);
    }

    println!("SHARE SET");
    println!("─────────────────────────────────────");
    println!("  ✓ Keys and threshold match dev known share set operators");
    println!();

    Ok(())
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

    // Deserialize manifest from bytes
    let manifest: Manifest = serde_json::from_slice(&tvc_manifest.manifest)
        .context("failed to parse manifest from deployment")?;

    println!("✓ Manifest loaded (manifest_id: {})", tvc_manifest.id);

    Ok((manifest, tvc_manifest.id))
}
