//! Deploy create command.

use clap::Args as ClapArgs;

/// Create a new deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Name for the deployment.
    #[arg(short, long)]
    pub name: Option<String>,
}

/// Run the deploy create command.
pub async fn run(args: Args, _config: &crate::cli::GlobalConfig) -> anyhow::Result<()> {
    println!("Creating deploy: {:?}", args.name);
    todo!("deploy create not yet implemented")
}
