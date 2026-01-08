//! App list command.

use clap::Args as ClapArgs;

/// List apps.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Filter by app name.
    #[arg(short, long)]
    pub name: Option<String>,
}

/// Run the app list command.
pub async fn run(args: Args, _config: &crate::cli::GlobalConfig) -> anyhow::Result<()> {
    println!("Listing apps with filter: {:?}", args.name);
    todo!("app list not yet implemented")
}
