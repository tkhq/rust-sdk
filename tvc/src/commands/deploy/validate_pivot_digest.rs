//! Validate pivot digest command - compute or validate a pivot digest locally.

use crate::pivot_digest::{
    compute_pivot_digest, validate_expected_digest, PivotDigestResult, PivotDigestSource,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Compute or validate the digest of the file at `pivot_path` inside a container image.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Container image reference to inspect locally.
    #[arg(long, value_name = "REF")]
    pub image_url: String,

    /// Path to the pivot file inside the container image.
    #[arg(long, value_name = "PATH")]
    pub pivot_path: String,

    /// Expected pivot digest to compare against.
    #[arg(long, value_name = "HEX")]
    pub expected_digest: Option<String>,

    /// Path to an unencrypted Docker-style pull secret JSON file.
    #[arg(long, value_name = "PATH")]
    pub pull_secret: Option<PathBuf>,
}

/// Run the validate pivot digest command.
pub async fn run(args: Args) -> Result<()> {
    let result = compute_pivot_digest(
        &PivotDigestSource {
            image_url: args.image_url,
            pivot_path: args.pivot_path,
        },
        args.pull_secret.as_deref(),
    )
    .await?;

    print_result(&result);

    if let Some(expected_digest) = args.expected_digest.as_deref() {
        validate_expected_digest(&result.digest, expected_digest)?;
        println!("Pivot digest validated successfully.");
    }

    Ok(())
}

pub fn print_result(result: &PivotDigestResult) {
    println!("Image: {}", result.image_url);
    println!("Pivot Path: {}", result.pivot_path);
    println!("Pivot Digest: {}", result.digest);
}
