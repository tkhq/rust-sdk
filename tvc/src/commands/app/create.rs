//! App create command.

use clap::Args as ClapArgs;

/// Create a new app.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Name for the app.
    #[arg(short, long)]
    pub name: String,
}

/// Run the app create command.
pub async fn run(args: Args, _config: &crate::cli::GlobalConfig) -> anyhow::Result<()> {
    println!("Creating app: {}", args.name);
    todo!("app create not yet implemented")
}
