//! App list command.

use anyhow::Context;
use clap::Args as ClapArgs;
use turnkey_client::generated::external::data::v1::TvcApp;
use turnkey_client::generated::GetTvcAppsRequest;

const SEPARATOR_WIDTH: usize = 40;

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

    filter_by_name(&mut apps, args.name.as_deref());

    if apps.is_empty() {
        println!("No apps found.");
        return Ok(());
    }

    for app in &apps {
        render_app(app);
    }

    Ok(())
}

fn filter_by_name(apps: &mut Vec<TvcApp>, name: Option<&str>) {
    if let Some(filter) = name {
        apps.retain(|app| app.name.contains(filter));
    }
}

fn render_app(app: &TvcApp) {
    let live = app.live_deployment_id.as_deref().unwrap_or("(none)");
    let mut lines = vec![
        format!("Name: {}", app.name),
        format!("ID: {}", app.id),
        format!("Quorum Public Key: {}", app.quorum_public_key),
        format!("Live Deployment: {live}"),
        crate::commands::display::format_egress_enabled(app.enable_egress),
    ];

    if !app.public_domain.is_empty() {
        lines.push(format!("Public Domain: {}", app.public_domain));
    }

    lines.push("─".repeat(SEPARATOR_WIDTH));

    println!("{}", lines.join("\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app(name: &str) -> TvcApp {
        TvcApp {
            id: "test-id".to_string(),
            organization_id: "test-org".to_string(),
            name: name.to_string(),
            quorum_public_key: "test-key".to_string(),
            manifest_set: None,
            share_set: None,
            enable_egress: false,
            created_at: None,
            updated_at: None,
            live_deployment_id: None,
            public_domain: String::new(),
            enable_debug_mode_deployments: false,
        }
    }

    #[test]
    fn filter_by_name_no_filter_returns_all() {
        let mut apps = vec![make_app("alpha"), make_app("beta")];
        filter_by_name(&mut apps, None);
        assert_eq!(apps.len(), 2);
    }

    #[test]
    fn filter_by_name_exact_match_returns_one() {
        let mut apps = vec![make_app("alpha"), make_app("beta")];
        filter_by_name(&mut apps, Some("alpha"));
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "alpha");
    }

    #[test]
    fn filter_by_name_partial_match_returns_matching() {
        let mut apps = vec![
            make_app("my-app-prod"),
            make_app("my-app-staging"),
            make_app("other"),
        ];
        filter_by_name(&mut apps, Some("my-app"));
        assert_eq!(apps.len(), 2);
    }

    #[test]
    fn filter_by_name_no_match_returns_empty() {
        let mut apps = vec![make_app("alpha"), make_app("beta")];
        filter_by_name(&mut apps, Some("gamma"));
        assert!(apps.is_empty());
    }

    #[test]
    fn matched_app_has_all_rendered_fields() {
        let mut app = make_app("my-app");
        app.id = "app-uuid-123".to_string();
        app.quorum_public_key = "04abcdef".to_string();
        app.live_deployment_id = Some("deploy-uuid-456".to_string());
        app.public_domain = "my-app.example.com".to_string();

        let mut apps = vec![app];
        filter_by_name(&mut apps, Some("my-app"));

        assert_eq!(apps.len(), 1);
        let app = &apps[0];
        assert_eq!(app.name, "my-app");
        assert_eq!(app.id, "app-uuid-123");
        assert_eq!(app.quorum_public_key, "04abcdef");
        assert_eq!(
            app.live_deployment_id.as_deref().unwrap_or("(none)"),
            "deploy-uuid-456"
        );
        assert_eq!(app.public_domain, "my-app.example.com");
    }

    #[test]
    fn app_with_no_live_deployment_renders_none() {
        let app = make_app("my-app");
        assert_eq!(
            app.live_deployment_id.as_deref().unwrap_or("(none)"),
            "(none)"
        );
    }

    #[test]
    fn egress_enabled_line_reflects_app_setting() {
        let mut app = make_app("my-app");
        app.enable_egress = true;

        assert_eq!(
            crate::commands::display::format_egress_enabled(app.enable_egress),
            "Egress Enabled: yes"
        );
    }
}
