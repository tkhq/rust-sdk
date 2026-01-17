//! Deploy status command.

use clap::Args as ClapArgs;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy status command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    println!("Getting status for deploy: {}", args.deploy_id);
    todo!("deploy status not yet implemented")
}
