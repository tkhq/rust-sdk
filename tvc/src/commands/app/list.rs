//! App list command.

use anyhow::Context;
use clap::Args as ClapArgs;
use turnkey_client::generated::external::data::v1::TvcApp;
use turnkey_client::generated::GetTvcAppsRequest;

/// List apps.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Filter by app name.
    #[arg(short, long, env = "TVC_APP_NAME")]
    pub name: Option<String>,
}

/// Run the app list command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let response = auth
        .client
        .get_tvc_apps(GetTvcAppsRequest {
            organization_id: auth.org_id.clone(),
        })
        .await
        .context("failed to list TVC apps")?;

    let mut apps = response.tvc_apps;

    if let Some(ref name_filter) = args.name {
        apps.retain(|app| app.name.contains(name_filter.as_str()));
    }

    if apps.is_empty() {
        println!("No apps found.");
        return Ok(());
    }

    for app in &apps {
        render_app(app);
    }

    Ok(())
}

fn render_app(app: &TvcApp) {
    println!("Name: {}", app.name);
    println!("ID: {}", app.id);
    println!("Quorum Public Key: {}", app.quorum_public_key);
    let live = app.live_deployment_id.as_deref().unwrap_or("(none)");
    println!("Live Deployment: {live}");
    if !app.public_domain.is_empty() {
        println!("Public Domain: {}", app.public_domain);
    }
    println!("{}", "─".repeat(40));
}
