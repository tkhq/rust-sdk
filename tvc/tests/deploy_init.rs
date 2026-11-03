//! Clone initialization keeps imported images out of reusable configuration.

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use std::{fs, process::Command};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::deploy::DeployConfig;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::path};

#[tokio::test]
async fn imported_image_placeholder_is_written_and_prompted_for() {
    let server = MockServer::start().await;
    let manifest = STANDARD.encode(include_bytes!("../fixtures/manifest.json"));
    Mock::given(path("/public/v1/query/get_tvc_deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tvcDeployment": {
                "id": "deploy-123",
                "organizationId": "org-1",
                "appId": "651b573c-861b-4f10-a478-cbcfe0c226af",
                "qosVersion": "0.15.0",
                "manifest": { "id": "manifest-1", "manifest": manifest },
                "pivotContainer": {
                    "containerUrl": "957814223021.dkr.ecr.us-east-1.amazonaws.com/tvc-customers/org-1/pivot:latest@sha256:abc",
                    "path": "/bin/pivot",
                    "args": [],
                    "hasPullSecret": false,
                    "healthCheckType": "TVC_HEALTH_CHECK_TYPE_HTTP",
                    "healthCheckPort": 3000,
                    "publicIngressPort": 3000
                },
                "instanceSizeCpus": 2,
                "instanceSizeRam": 1
            }
        })))
        .expect(if cfg!(unix) { 2 } else { 1 })
        .mount(&server)
        .await;
    let url = server.uri();
    tokio::task::spawn_blocking(move || {
        let home = TempDir::new().unwrap();
        let output = home.path().join("deploy.json");
        let key = TurnkeyP256ApiKey::generate();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tvc"));
        cmd.env_clear()
            .env("HOME", home.path())
            .env("TVC_ORG_ID", "org-1")
            .env(
                "TVC_API_KEY_PUBLIC",
                hex::encode(key.compressed_public_key()),
            )
            .env("TVC_API_KEY_PRIVATE", hex::encode(key.private_key()))
            .env("TVC_API_BASE_URL", &url)
            .args([
                "deploy",
                "init",
                "--from-deployment",
                "deploy-123",
                "--output",
            ])
            .arg(&output);
        let mut noninteractive = assert_cmd::Command::from_std(cmd);
        noninteractive
            .arg("--message-format")
            .arg("json")
            .assert()
            .success();
        let config: DeployConfig = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert_eq!(
            config.pivot_container_image_url,
            "<FILL_IN_PIVOT_CONTAINER_IMAGE_URL>"
        );
        assert_eq!(config.missing_required_fields(), vec!["--pivot-image-url"]);

        #[cfg(unix)]
        {
            let interactive_output = home.path().join("interactive.json");
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_tvc"));
            cmd.env_clear()
                .env("HOME", home.path())
                .env("TERM", "xterm")
                .env("TVC_ORG_ID", "org-1")
                .env(
                    "TVC_API_KEY_PUBLIC",
                    hex::encode(key.compressed_public_key()),
                )
                .env("TVC_API_KEY_PRIVATE", hex::encode(key.private_key()))
                .env("TVC_API_BASE_URL", &url)
                .args([
                    "deploy",
                    "init",
                    "--from-deployment",
                    "deploy-123",
                    "--interactive",
                    "--output",
                ])
                .arg(&interactive_output);
            let mut session = rexpect::session::spawn_command(cmd, Some(10_000)).unwrap();
            session.exp_string("Pivot container image URL").unwrap();
            session
                .send_line("ghcr.io/customer/pivot@sha256:abc")
                .unwrap();
            session.exp_string("Created deployment config:").unwrap();
            session.exp_eof().unwrap();
            let config: DeployConfig =
                serde_json::from_slice(&fs::read(interactive_output).unwrap()).unwrap();
            assert_eq!(
                config.pivot_container_image_url,
                "ghcr.io/customer/pivot@sha256:abc"
            );
            assert!(!config.has_placeholders());
        }
    })
    .await
    .unwrap();
}
