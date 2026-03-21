use clap::Args as ClapArgs;
use tracing::debug;

use crate::cli::GlobalArgs;
use crate::config::{self, Config};
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args;

pub async fn run(_args: &Args, global: &GlobalArgs) -> anyhow::Result<()> {
    let config = Config::from_env()?;
    debug!(
        api_base_url = %config.api_base_url,
        organization_id = %config.organization_id,
        private_key_id = %config.private_key_id,
        "resolving identity"
    );

    let signer = TurnkeySigner::new(config.clone())?;
    let public_key = signer.get_public_key().await?;
    let ssh_public_key = crate::ssh::encode_public_key_line(&public_key, None)?;
    let config_path = config::global_config_path()?;

    if global.json {
        let output = serde_json::json!({
            "organizationId": config.organization_id,
            "privateKeyId": config.private_key_id,
            "apiBaseUrl": config.api_base_url,
            "sshPublicKey": ssh_public_key,
            "configPath": config_path.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        eprintln!("Credentials verified successfully.\n");
        println!("Organization ID:  {}", config.organization_id);
        println!("Private Key ID:   {}", config.private_key_id);
        println!("API Base URL:     {}", config.api_base_url);
        println!("SSH Public Key:   {ssh_public_key}");
        println!("Config Path:      {}", config_path.display());
    }

    Ok(())
}
