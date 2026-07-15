//! Pre-submit dedup: `tvc deploy create` and `tvc deploy approve` must vote
//! on an existing pending activity with an identical intent instead of
//! creating a duplicate.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_BASE_URL: &str = "TVC_API_BASE_URL";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";
const ORG_ID: &str = "org-test";
const APP_ID: &str = "11111111-1111-1111-1111-111111111111";
const EXISTING_ACTIVITY_ID: &str = "activity-existing";
const EXISTING_FINGERPRINT: &str = "fp-existing";
const CURRENT_USER_ID: &str = "user-current";

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

async fn mount_image_validation(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/validate_tvc_image"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resolvedImageDigest": "sha256:resolved"
        })))
        .mount(server)
        .await;
}

/// Intent JSON matching exactly what the CLI builds from `create_args`.
fn matching_intent_json(app_id: &str) -> serde_json::Value {
    serde_json::json!({
        "createTvcDeploymentIntent": {
            "appId": app_id,
            "qosVersion": "0.12.1",
            "pivotContainerImageUrl": "ghcr.io/team/app:latest@sha256:resolved",
            "pivotPath": "/app",
            "pivotArgs": [],
            "expectedPivotDigest": "sha256:resolved",
            "debugMode": false,
            "healthCheckType": "TVC_HEALTH_CHECK_TYPE_HTTP",
            "healthCheckPort": 3000,
            "publicIngressPort": 3000
        }
    })
}

async fn mount_whoami(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/whoami"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organizationId": ORG_ID,
            "organizationName": "Test Org",
            "userId": CURRENT_USER_ID,
            "username": "root-user"
        })))
        .mount(server)
        .await;
}

fn approval_vote(user_id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("vote-{user_id}"),
        "userId": user_id,
        "activityId": EXISTING_ACTIVITY_ID,
        "selection": "VOTE_SELECTION_APPROVED",
        "message": "{}",
        "publicKey": "public-key",
        "signature": "signature",
        "scheme": "SIGNATURE_SCHEME_TK_API_P256"
    })
}

async fn mount_pending_activities_with_votes(
    server: &MockServer,
    intent: serde_json::Value,
    votes: Vec<serde_json::Value>,
) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activities": [
                {
                    "id": EXISTING_ACTIVITY_ID,
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                    "fingerprint": EXISTING_FINGERPRINT,
                    "intent": intent,
                    "votes": votes
                }
            ]
        })))
        .mount(server)
        .await;
}

async fn mount_pending_activities(server: &MockServer, intent: serde_json::Value) {
    mount_pending_activities_with_votes(server, intent, vec![]).await;
}

fn create_args(cmd: &mut assert_cmd::Command) -> &mut assert_cmd::Command {
    cmd.arg("--non-interactive")
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
}

#[tokio::test]
async fn deploy_create_votes_on_existing_matching_activity() {
    let server = MockServer::start().await;
    mount_tvc_app_lookup(&server).await;
    mount_image_validation(&server).await;
    mount_whoami(&server).await;
    mount_pending_activities(&server, matching_intent_json(APP_ID)).await;

    // Voting on the existing activity; quorum still not reached afterwards.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": EXISTING_ACTIVITY_ID,
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": EXISTING_FINGERPRINT
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // The dedup path must NOT create a new activity.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_deployment"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let mut cmd = authed_tvc_cmd(&server);
    create_args(&mut cmd)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("already pending"))
        .stdout(predicate::str::contains(EXISTING_ACTIVITY_ID))
        .stdout(predicate::str::contains(EXISTING_FINGERPRINT))
        .stdout(predicate::str::contains("Vote recorded"))
        .stdout(predicate::str::contains("still pending quorum"));
}

#[tokio::test]
async fn deploy_create_vote_completing_quorum_succeeds() {
    let server = MockServer::start().await;
    mount_tvc_app_lookup(&server).await;
    mount_image_validation(&server).await;
    mount_whoami(&server).await;
    mount_pending_activities(&server, matching_intent_json(APP_ID)).await;

    // Voting on the existing activity completes quorum.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": EXISTING_ACTIVITY_ID,
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": EXISTING_FINGERPRINT
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_deployment"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let mut cmd = authed_tvc_cmd(&server);
    create_args(&mut cmd)
        .assert()
        .success()
        .stdout(predicate::str::contains("already pending"))
        .stdout(predicate::str::contains("quorum reached"));
}

#[tokio::test]
async fn deploy_create_skips_vote_when_user_already_approved() {
    let server = MockServer::start().await;
    mount_tvc_app_lookup(&server).await;
    mount_image_validation(&server).await;
    mount_whoami(&server).await;
    // The current user already approved the pending activity; one other
    // quorum member also approved.
    mount_pending_activities_with_votes(
        &server,
        matching_intent_json(APP_ID),
        vec![approval_vote(CURRENT_USER_ID), approval_vote("user-other")],
    )
    .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_organization_configs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "configs": {
                "features": [],
                "quorum": { "threshold": 3, "userIds": ["u1", "u2", "u3"] }
            }
        })))
        .mount(&server)
        .await;

    // No new vote and no new activity may be submitted.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_deployment"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let mut cmd = authed_tvc_cmd(&server);
    create_args(&mut cmd)
        .assert()
        .success()
        .stdout(predicate::str::contains("already approved"))
        .stdout(predicate::str::contains(EXISTING_ACTIVITY_ID))
        .stdout(predicate::str::contains("2 of 3"));
}

/// Generate the (deterministic) operator approval offline so the test knows
/// the exact signature the CLI will post, then serve a pending activity with
/// that same intent.
fn captured_manifest_approval_signature() -> String {
    let approval_out = tempfile::NamedTempFile::new().unwrap();
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .arg("--approval-out")
        .arg(approval_out.path())
        .assert()
        .success();

    let approval: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(approval_out.path()).unwrap()).unwrap();
    approval["signature"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn deploy_approve_votes_on_existing_matching_activity() {
    const MANIFEST_ID: &str = "22222222-2222-2222-2222-222222222222";
    const OPERATOR_ID: &str = "33333333-3333-3333-3333-333333333333";

    let signature = captured_manifest_approval_signature();

    let server = MockServer::start().await;
    mount_whoami(&server).await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activities": [
                {
                    "id": EXISTING_ACTIVITY_ID,
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS",
                    "fingerprint": EXISTING_FINGERPRINT,
                    "intent": {
                        "createTvcManifestApprovalsIntent": {
                            "manifestId": MANIFEST_ID,
                            "approvals": [
                                { "operatorId": OPERATOR_ID, "signature": signature }
                            ]
                        }
                    }
                }
            ]
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
                "type": "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS",
                "fingerprint": EXISTING_FINGERPRINT
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // The dedup path must NOT post a duplicate manifest-approvals activity.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_manifest_approvals"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("--non-interactive")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--manifest-id")
        .arg(MANIFEST_ID)
        .arg("--operator-id")
        .arg(OPERATOR_ID)
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--approval-out")
        .arg(tempfile::NamedTempFile::new().unwrap().path())
        .assert()
        .code(2)
        .stdout(predicate::str::contains("already pending"))
        .stdout(predicate::str::contains(EXISTING_ACTIVITY_ID))
        .stdout(predicate::str::contains("Vote recorded"))
        .stdout(predicate::str::contains("still pending quorum"));
}

#[tokio::test]
async fn deploy_create_proceeds_when_no_pending_activity_matches() {
    let server = MockServer::start().await;
    mount_tvc_app_lookup(&server).await;
    mount_image_validation(&server).await;
    // Pending activity exists but for a different app: no dedup match.
    mount_pending_activities(
        &server,
        matching_intent_json("99999999-9999-9999-9999-999999999999"),
    )
    .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/create_tvc_deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-new",
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": "fp-new"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = authed_tvc_cmd(&server);
    create_args(&mut cmd)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Consensus needed"))
        .stdout(predicate::str::contains("activity-new"))
        .stdout(predicate::str::contains("fp-new"));
}
