use clap::{Args as ClapArgs, Subcommand};

use crate::cli::GlobalArgs;
use crate::config::{self, ConfigKey};

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the resolved value for one config key.
    Get(GetArgs),
    /// Persist a config value to the global config file.
    Set(SetArgs),
    /// Print the resolved effective config.
    List,
}

#[derive(Debug, ClapArgs)]
struct GetArgs {
    key: String,
}

#[derive(Debug, ClapArgs)]
struct SetArgs {
    key: String,
    value: String,
}

pub async fn run(args: &Args, global: &GlobalArgs) -> anyhow::Result<()> {
    match &args.command {
        Command::Get(get_args) => {
            let key = ConfigKey::parse(&get_args.key)?;
            let value = config::get_config_value(key)?;
            if global.json {
                let output = serde_json::json!({
                    "key": get_args.key,
                    "value": value,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{value}");
            }
        }
        Command::Set(set_args) => {
            let key = ConfigKey::parse(&set_args.key)?;
            config::set_persisted_config_value(key, &set_args.value)?;
        }
        Command::List => {
            if global.json {
                let resolved = config::load_resolved_config()?;
                let output = serde_json::json!({
                    "turnkey": {
                        "organizationId": resolved.organization_id.unwrap_or_default(),
                        "apiPublicKey": resolved.api_public_key.unwrap_or_default(),
                        "apiPrivateKey": resolved.api_private_key.unwrap_or_default(),
                        "privateKeyId": resolved.private_key_id.unwrap_or_default(),
                        "apiBaseUrl": resolved.api_base_url,
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print!("{}", config::render_resolved_config()?);
            }
        }
    }

    Ok(())
}
