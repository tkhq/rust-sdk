//! App list command.

use anyhow::Context;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use turnkey_client::generated::GetTvcAppsRequest;
use turnkey_client::generated::external::data::v1::TvcApp;

use crate::commands::display::{format_egress_enabled, yes_no};
use crate::outcome::Outcome;
use crate::output::StdCtx;

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
pub async fn run(_ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
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

    Ok(Outcome::AppList(AppsListed {
        apps: apps.into_iter().map(AppSummary::from).collect(),
    }))
}

fn filter_by_name(apps: &mut Vec<TvcApp>, name: Option<&str>) {
    if let Some(filter) = name {
        apps.retain(|app| app.name.contains(filter));
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppsListed {
    apps: Vec<AppSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSummary {
    id: String,
    name: String,
    quorum_public_key: String,
    live_deployment_id: Option<String>,
    egress_enabled: bool,
    debug_mode_deployments_enabled: bool,
    /// Empty when the app has no public domain configured.
    #[serde(skip_serializing_if = "String::is_empty")]
    public_domain: String,
}

impl From<TvcApp> for AppSummary {
    fn from(app: TvcApp) -> Self {
        // Destructure exhaustively (rather than `..`) so that adding a field to
        // the generated `TvcApp` forces a compile error here
        let TvcApp {
            id,
            name,
            quorum_public_key,
            live_deployment_id,
            public_domain,
            enable_egress: egress_enabled,
            enable_debug_mode_deployments: debug_mode_deployments_enabled,
            organization_id: _,
            manifest_set: _,
            share_set: _,
            created_at: _,
            updated_at: _,
        } = app;

        Self {
            id,
            name,
            quorum_public_key,
            live_deployment_id,
            egress_enabled,
            debug_mode_deployments_enabled,
            public_domain,
        }
    }
}

impl Display for AppsListed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.apps.is_empty() {
            return f.write_str("No apps found.");
        }

        let body = self
            .apps
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        f.write_str(&body)
    }
}

impl Display for AppSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let live = self.live_deployment_id.as_deref().unwrap_or("(none)");
        write!(
            f,
            r#"Name: {}
ID: {}
Quorum Public Key: {}
Live Deployment: {live}
{}
Debug Mode Deployments: {}"#,
            self.name,
            self.id,
            self.quorum_public_key,
            format_egress_enabled(self.egress_enabled),
            yes_no(self.debug_mode_deployments_enabled),
        )?;

        if !self.public_domain.is_empty() {
            write!(f, "\nPublic Domain: {}", self.public_domain)?;
        }

        write!(f, "\n{}", "─".repeat(SEPARATOR_WIDTH))
    }
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
            format_egress_enabled(app.enable_egress),
            "Egress Enabled: yes"
        );
    }

    fn full_summary() -> AppSummary {
        AppSummary {
            id: "app-1".to_string(),
            name: "my-app".to_string(),
            quorum_public_key: "04abcd".to_string(),
            live_deployment_id: Some("dep-9".to_string()),
            egress_enabled: true,
            debug_mode_deployments_enabled: true,
            public_domain: "my-app.example.com".to_string(),
        }
    }

    fn minimal_summary() -> AppSummary {
        AppSummary {
            id: "app-2".to_string(),
            name: "bare".to_string(),
            quorum_public_key: "04ef".to_string(),
            live_deployment_id: None,
            egress_enabled: false,
            debug_mode_deployments_enabled: false,
            public_domain: String::new(),
        }
    }

    #[test]
    fn render_full_golden() {
        assert_eq!(
            full_summary().to_string(),
            r#"Name: my-app
ID: app-1
Quorum Public Key: 04abcd
Live Deployment: dep-9
Egress Enabled: yes
Debug Mode Deployments: yes
Public Domain: my-app.example.com
────────────────────────────────────────"#
        );
    }

    #[test]
    fn render_minimal_golden() {
        assert_eq!(
            minimal_summary().to_string(),
            r#"Name: bare
ID: app-2
Quorum Public Key: 04ef
Live Deployment: (none)
Egress Enabled: no
Debug Mode Deployments: no
────────────────────────────────────────"#
        );
    }
}
