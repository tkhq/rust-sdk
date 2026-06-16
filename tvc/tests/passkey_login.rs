use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, OrgConfig, StoredPasskeySession};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_ASSERTION: &str = r#"{
  "credentialId": "credential-id",
  "clientDataJson": "client-data-json",
  "authenticatorData": "authenticator-data",
  "signature": "signature"
}"#;

#[tokio::test]
async fn login_passkey_stores_session_from_stamp_login() {
    let home = TempDir::new().unwrap();
    let turnkey_dir = home.path().join(".config").join("turnkey");
    let org_dir = turnkey_dir.join("orgs").join("test");
    let session_path = org_dir.join("passkey_session.json");
    fs::create_dir_all(&org_dir).unwrap();

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/stamp_login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_json(serde_json::json!({
                    "activity": {
                        "type": "ACTIVITY_TYPE_STAMP_LOGIN",
                        "status": "ACTIVITY_STATUS_COMPLETED",
                        "id": "activity-id",
                        "fingerprint": "activity-fingerprint",
                        "organizationId": "org-test",
                        "result": {
                            "stampLoginResult": {
                                "session": "session-jwt"
                            }
                        }
                    }
                })),
        )
        .mount(&server)
        .await;

    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: org_dir.join("api_key.json"),
                operator_key_path: org_dir.join("operator.json"),
                api_base_url: server.uri(),
                passkey_session_path: None,
            },
        )]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
    };
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        toml::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", home.path())
        .env("TVC_PASSKEY_FIXTURE_ASSERTION", FIXTURE_ASSERTION)
        .arg("login")
        .arg("--passkey")
        .arg("--org")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Passkey session stored"));

    let stored: StoredPasskeySession =
        serde_json::from_str(&fs::read_to_string(session_path).unwrap()).unwrap();
    assert_eq!(stored.session, "session-jwt");
}
