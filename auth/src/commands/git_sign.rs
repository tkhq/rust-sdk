use clap::Args as ClapArgs;

use anyhow::anyhow;

use crate::config::Config;
use crate::ssh;
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub ssh_keygen_args: Vec<String>,
}

/// Runs the `auth git-sign` subcommand or direct SSH signer invocation.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let invocation = ssh::git::GitSignInvocation::parse(&args.ssh_keygen_args)?;
    let signer = TurnkeySigner::new(Config::resolve().await?)?;
    let payload = tokio::fs::read(&invocation.payload_path).await?;
    let public_key = tokio::fs::read_to_string(&invocation.public_key_path).await?;
    let parsed_public_key = ssh::parse_public_key_line(&public_key)?;
    let configured_public_key = signer.get_public_key().await?;
    if parsed_public_key.public_key != configured_public_key {
        return Err(anyhow!(
            "requested SSH public key does not match the configured Turnkey key"
        ));
    }
    let signed_data = ssh::build_signed_data("git", &payload);
    let signature = signer.sign_ed25519(&signed_data).await?;
    let armored = ssh::encode_armored_signature(
        &parsed_public_key.public_key_blob,
        "git",
        ssh::DEFAULT_HASH_ALGORITHM,
        &signature,
    )?;

    let signature_path = invocation.signature_path();
    tokio::fs::write(&signature_path, armored).await?;
    Ok(())
}
