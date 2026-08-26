//! Secret list command - lists secret metadata for the active organization.

use super::list_all_secrets;
use crate::client::build_client;
use crate::config::turnkey::Config;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use anyhow::Result;
use chrono::DateTime;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use tracing::instrument;
use turnkey_client::generated::SecretMetadata;
use turnkey_client::generated::immutable::models::v1::KeyValue;

pub const LONG_ABOUT: &str = r#"List secret metadata for the active organization.

Only metadata is shown: IDs, names, static properties, and creation times.
Secret values never leave storage on this path - use `tvc secrets export`."#;

/// List secret metadata for the active organization.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {}

/// Run the secret list command.
#[instrument(skip_all)]
pub async fn run(_ctx: &mut StdCtx, args: Args, config: Config) -> Result<Outcome> {
    let Args {} = args;
    let auth = build_client(&config).await?;
    let secrets = list_all_secrets(&auth)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Outcome::SecretsListed(SecretsListed { secrets }))
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecretsListed {
    secrets: Vec<ListedSecret>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListedSecret {
    id: String,
    name: Option<String>,
    static_properties: BTreeMap<String, String>,
    /// RFC 3339 creation time, or the raw unix-milliseconds value when out of
    /// range.
    created_at: String,
}

impl From<SecretMetadata> for ListedSecret {
    fn from(metadata: SecretMetadata) -> Self {
        // Destructure exhaustively (rather than `..`) so that adding a field
        // to the generated `SecretMetadata` forces a compile error here.
        let SecretMetadata {
            secret_id: id,
            name,
            static_properties,
            created_at_unix_ms,
        } = metadata;

        let created_at = DateTime::from_timestamp_millis(created_at_unix_ms as i64)
            .map(|created| created.to_rfc3339())
            .unwrap_or_else(|| format!("{created_at_unix_ms} (unix ms)"));

        Self {
            id,
            name,
            static_properties: static_properties
                .into_iter()
                .map(|KeyValue { key, value }| (key, value))
                .collect(),
            created_at,
        }
    }
}

impl Display for SecretsListed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.secrets.is_empty() {
            return f.write_str("No secrets found.");
        }

        let body = self
            .secrets
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n\n");
        f.write_str(&body)
    }
}

impl Display for ListedSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Name: {}
ID: {}
Created: {}"#,
            self.name.as_deref().unwrap_or("(none)"),
            self.id,
            self.created_at
        )?;
        if !self.static_properties.is_empty() {
            let properties = self
                .static_properties
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "\nProperties: {properties}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::secrets::test_support::{test_config, test_ctx};
    use tempfile::TempDir;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn metadata(id: &str, name: &str) -> serde_json::Value {
        serde_json::json!({
            "secretId": id,
            "name": name,
            "staticProperties": [{"key": "environment", "value": "demo"}],
            "createdAtUnixMs": "1756100000000",
        })
    }

    #[tokio::test]
    async fn run_lists_metadata_across_pages() {
        let server = MockServer::start().await;

        // A full first page (100 items) forces a second request that
        // continues after the last returned ID.
        let full_page: Vec<serde_json::Value> = (0..100)
            .map(|i| metadata(&format!("secret-{i:03}"), &format!("name-{i:03}")))
            .collect();
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_secrets"))
            .and(body_partial_json(
                serde_json::json!({"paginationOptions": {"after": ""}}),
            ))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"secrets": full_page})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_secrets"))
            .and(body_partial_json(
                serde_json::json!({"paginationOptions": {"after": "secret-099"}}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"secrets": [metadata("secret-100", "name-100")]}),
            ))
            .expect(1)
            .mount(&server)
            .await;

        let dir = TempDir::new().unwrap();
        let outcome = run(&mut test_ctx(), Args {}, test_config(&dir, &server.uri()))
            .await
            .unwrap();

        let Outcome::SecretsListed(listed) = outcome else {
            panic!("expected SecretsListed");
        };
        assert_eq!(listed.secrets.len(), 101);
        assert_eq!(listed.secrets[0].id, "secret-000");
        assert_eq!(listed.secrets[100].id, "secret-100");
    }

    #[test]
    fn an_empty_listing_renders_a_message() {
        assert_eq!(SecretsListed::default().to_string(), "No secrets found.");
    }

    #[test]
    fn listed_secrets_serialize_metadata_only() {
        let listed = SecretsListed {
            secrets: vec![ListedSecret {
                id: "secret-abc".to_string(),
                name: Some("db-password".to_string()),
                static_properties: BTreeMap::from([(
                    "environment".to_string(),
                    "demo".to_string(),
                )]),
                created_at: "2026-08-25T00:00:00+00:00".to_string(),
            }],
        };
        assert_eq!(
            serde_json::to_value(&listed).unwrap(),
            serde_json::json!({
                "secrets": [{
                    "id": "secret-abc",
                    "name": "db-password",
                    "staticProperties": {"environment": "demo"},
                    "createdAt": "2026-08-25T00:00:00+00:00",
                }]
            })
        );
    }

    #[test]
    fn listed_secret_renders_name_id_created_and_properties() {
        let listed: ListedSecret = SecretMetadata {
            secret_id: "secret-abc".to_string(),
            name: Some("db-password".to_string()),
            static_properties: vec![KeyValue {
                key: "environment".to_string(),
                value: "demo".to_string(),
            }],
            created_at_unix_ms: 1_756_100_000_000,
        }
        .into();
        assert_eq!(
            listed.to_string(),
            r#"Name: db-password
ID: secret-abc
Created: 2025-08-25T05:33:20+00:00
Properties: environment=demo"#
        );
    }
}
