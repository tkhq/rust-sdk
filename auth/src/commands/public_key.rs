use clap::Args as ClapArgs;

use crate::config::Config;
use crate::ssh::encode_public_key_line;
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {}

/// Runs the `auth public-key` subcommand.
pub async fn run(_args: Args) -> anyhow::Result<()> {
    let signer = TurnkeySigner::new(Config::resolve()?)?;
    let public_key = signer.get_public_key().await?;
    println!("{}", encode_public_key_line(&public_key, None)?);
    Ok(())
}
