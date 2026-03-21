use clap::Args as ClapArgs;

use crate::cli::GlobalArgs;
use crate::ssh::encode_public_key_line;
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args;

pub async fn run(_args: &Args, global: &GlobalArgs) -> anyhow::Result<()> {
    let signer = TurnkeySigner::from_env()?;
    let public_key = signer.get_public_key().await?;
    let key_line = encode_public_key_line(&public_key, None)?;

    if global.json {
        let output = serde_json::json!({
            "publicKey": key_line,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{key_line}");
    }

    Ok(())
}
