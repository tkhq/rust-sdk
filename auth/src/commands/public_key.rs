use clap::Args as ClapArgs;

use crate::ssh::encode_public_key_line;
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let signer = TurnkeySigner::from_env()?;
    let public_key = signer.get_public_key().await?;
    println!("{}", encode_public_key_line(&public_key, None)?);
    Ok(())
}
