use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_BASE_URL: &str = "TVC_API_BASE_URL";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";
const ORG_ID: &str = "org-test";
const APP_ID: &str = "11111111-1111-1111-1111-111111111111";
const ACTIVITY_ID: &str = "activity-pending";
const FINGERPRINT: &str = "activity-fingerprint";

fn generated_api_key() -> (String, String) {
    let stamper = TurnkeyP256ApiKey::generate();
    (
        hex::encode(stamper.compressed_public_key()),
        hex::encode(stamper.private_key()),
    )
}

fn authed_tvc_cmd(server: &MockServer) -> assert_cmd::Command {
    let (public_key, private_key) = generated_api_key();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .env(ENV_ORG_ID, ORG_ID)
        .env(ENV_API_BASE_URL, server.uri())
        .env(ENV_API_KEY_PUBLIC, public_key)
        .env(ENV_API_KEY_PRIVATE, private_key);
    cmd
}

async fn mount_tvc_app_lookup(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_tvc_app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tvcApp": {
                "id": APP_ID,
                "organizationId": ORG_ID,
                "name": "test app",
                "quorumPublicKey": "quorum-public-key",
                "publicDomain": "example.com"
            }
        })))
        .mount(server)
        .await;
}

fn consensus_needed_activity(activity_type: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "activity": {
            "type": activity_type,
            "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            "id": ACTIVITY_ID,
            "organizationId": ORG_ID,
            "fingerprint": FINGERPRINT
        }
    }))
}

#[tokio::test]
async fn deploy_create_reports_consensus_needed_without_generic_failure() {
    let server = MockServer::start().await;
    mount_tvc_app_lookup(&server).await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/validate_tvc_image"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resolvedImageDigest": "sha256:resolved"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_deployment"))
        .respond_with(consensus_needed_activity(
            "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
        ))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("--non-interactive")
        .arg("deploy")
        .arg("create")
        .arg("--app-id")
        .arg(APP_ID)
        .arg("--qos-version")
        .arg("0.12.1")
        .arg("--pivot-image-url")
        .arg("ghcr.io/team/app:latest")
        .arg("--expected-pivot-digest")
        .arg("sha256:resolved")
        .arg("--pivot-path")
        .arg("/app")
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Consensus needed"))
        .stdout(predicate::str::contains(ACTIVITY_ID))
        .stdout(predicate::str::contains(FINGERPRINT))
        .stdout(predicate::str::contains(
            "tvc activity approve --fingerprint",
        ))
        .stderr(predicate::str::contains("failed to create TVC deployment").not());
}

#[tokio::test]
async fn deploy_approve_reports_consensus_needed_without_generic_failure() {
    let server = MockServer::start().await;
    let approval_out = NamedTempFile::new().unwrap();

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_manifest_approvals"))
        .respond_with(consensus_needed_activity(
            "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS",
        ))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("--non-interactive")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--manifest-id")
        .arg("22222222-2222-2222-2222-222222222222")
        .arg("--operator-id")
        .arg("33333333-3333-3333-3333-333333333333")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--approval-out")
        .arg(approval_out.path())
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Consensus needed"))
        .stdout(predicate::str::contains(ACTIVITY_ID))
        .stdout(predicate::str::contains(FINGERPRINT))
        .stdout(predicate::str::contains(
            "tvc activity approve --fingerprint",
        ))
        .stderr(predicate::str::contains("failed to post manifest approval").not());
}
