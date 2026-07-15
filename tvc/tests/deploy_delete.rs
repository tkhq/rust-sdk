use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ORG_ID: &str = "org-test";
const DEPLOY_ID: &str = "11111111-1111-1111-1111-111111111111";
const EXISTING_ACTIVITY_ID: &str = "activity-existing-delete";
const EXISTING_FINGERPRINT: &str = "fp-existing-delete";

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
        .env("TVC_ORG_ID", ORG_ID)
        .env("TVC_API_BASE_URL", server.uri())
        .env("TVC_API_KEY_PUBLIC", public_key)
        .env("TVC_API_KEY_PRIVATE", private_key);
    cmd
}

fn deploy_delete_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("delete");
    (temp, cmd)
}

#[test]
fn deploy_delete_help_lists_expected_flags() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn deploy_delete_requires_deploy_id() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[tokio::test]
async fn deploy_delete_votes_on_existing_matching_activity() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activities": [
                {
                    "id": EXISTING_ACTIVITY_ID,
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_DELETE_TVC_DEPLOYMENT",
                    "fingerprint": EXISTING_FINGERPRINT,
                    "intent": {
                        "deleteTvcDeploymentIntent": {
                            "deploymentId": DEPLOY_ID
                        }
                    },
                    "votes": []
                }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/whoami"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organizationId": ORG_ID,
            "organizationName": "Test Org",
            "userId": "user-current",
            "username": "root-user"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": EXISTING_ACTIVITY_ID,
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "type": "ACTIVITY_TYPE_DELETE_TVC_DEPLOYMENT",
                "fingerprint": EXISTING_FINGERPRINT
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/delete_tvc_deployment"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("deploy")
        .arg("delete")
        .arg("--deploy-id")
        .arg(DEPLOY_ID)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("already pending"))
        .stdout(predicate::str::contains(EXISTING_ACTIVITY_ID))
        .stdout(predicate::str::contains(EXISTING_FINGERPRINT))
        .stdout(predicate::str::contains("Vote recorded"));
}
