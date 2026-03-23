use clap::{Args as ClapArgs, Subcommand};

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

/// Runs the `auth config` subcommand.
pub async fn run(args: Args) -> anyhow::Result<()> {
    match args.command {
        Command::Get(args) => {
            let key = ConfigKey::parse(&args.key)?;
            println!("{}", config::get_resolved_config_value(key)?);
        }
        Command::Set(args) => {
            let key = ConfigKey::parse(&args.key)?;
            config::set_config_value(key, &args.value)?;
        }
        Command::List => {
            print!("{}", config::render_config()?);
        }
    }

    Ok(())
}
