//! Approve manifest command.

use crate::pair::LocalPair;
use crate::util::{read_file_to_string, write_file};
use anyhow::anyhow;
use anyhow::{bail, Context};
use clap::{ArgGroup, Args as ClapArgs};
use qos_core::protocol::services::boot::Approval;
use qos_core::protocol::services::boot::{
    Manifest, ManifestSet, Namespace, NitroConfig, PivotConfig, QuorumMember, ShareSet,
};
use qos_core::protocol::QosHash;
use std::io::{BufRead, Write};
use std::path::Path;
use std::path::PathBuf;

/// Approve a QOS manifest.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
#[command(group(ArgGroup::new("operator").args(["operator_seed", "operator_id"])))]
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

    /// Path to the file containing the master seed for the operator key.
    #[arg(
        long,
        help_heading = "Operator to approve with (pick one)",
        value_name = "PATH"
    )]
    pub operator_seed: Option<PathBuf>,

    /// Operator ID to use.
    #[arg(
        long,
        help_heading = "Operator to approve with (pick one)",
        env = "TVC_OPERATOR_ID"
    )]
    pub operator_id: Option<String>,

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

/// Run the approve manifest command.
pub async fn run(args: Args, _config: &crate::cli::GlobalConfig) -> anyhow::Result<()> {
    let manifest = match (&args.manifest, &args.deploy_id) {
        (Some(path), _) => read_manifest_from_path(path).await?,
        (_, Some(deploy_id)) => fetch_manifest_from_deploy(deploy_id).await?,
        (None, None) => bail!("a manifest source is required"),
    };

    if !args.dangerous_skip_interactive {
        interactive_approve(&manifest)?;
    }

    if !args.dry_run {
        let pair: Box<dyn crate::pair::Pair> = match (&args.operator_seed, &args.operator_id) {
            (Some(path), _) => Box::new(LocalPair::from_master_seed(path).await?),
            (_, Some(_id)) => todo!("implement signer from operator id"),
            (None, None) => bail!("an operator is required"),
        };

        let approval = generate_approval(pair, &manifest).await?;
        let json = serde_json::to_string_pretty(&approval)?;

        if let Some(ref path) = args.output {
            write_file(path, &json).await?
        } else {
            println!("{json}")
        }
        if !args.skip_post {
            println!("posting to api not yet implemented")
        }
    };

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
    println!("SHARE SET");
    println!("─────────────────────────────────────");
    println!("  Threshold: {} of {}", set.threshold, set.members.len());
    println!("  Members:");
    print_quorum_members(&set.members);
    println!();

    confirm("Approve share set?")
}

async fn read_manifest_from_path(path: &Path) -> anyhow::Result<Manifest> {
    let content = read_file_to_string(path).await?;
    let manifest: Manifest = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse manifest JSON from: {}", path.display()))?;
    Ok(manifest)
}

async fn fetch_manifest_from_deploy(_deploy_id: &str) -> anyhow::Result<Manifest> {
    todo!("fetching manifest with deployment id is not yet implemented")
}
