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

#[test]
fn activity_help_lists_subcommands() {
    cargo_bin_cmd!("tvc")
        .arg("activity")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("approve"))
        .stdout(predicate::str::contains("reject"));
}

#[test]
fn activity_approve_requires_fingerprint() {
    cargo_bin_cmd!("tvc")
        .arg("activity")
        .arg("approve")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--fingerprint"));
}

#[test]
fn activity_reject_requires_fingerprint() {
    cargo_bin_cmd!("tvc")
        .arg("activity")
        .arg("reject")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--fingerprint"));
}

#[test]
fn activity_list_rejects_unknown_activity_type() {
    cargo_bin_cmd!("tvc")
        .arg("activity")
        .arg("list")
        .arg("--activity-type")
        .arg("NOT_A_REAL_ACTIVITY_TYPE")
        .assert()
        .failure()
        .stderr(predicate::str::contains("NOT_A_REAL_ACTIVITY_TYPE"));
}

#[tokio::test]
async fn activity_list_shows_pending_activities() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activities": [
                {
                    "id": "activity-1",
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                    "fingerprint": "fp-1",
                    "createdAt": { "seconds": "1752570000", "nanos": "0" }
                },
                {
                    "id": "activity-2",
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS",
                    "fingerprint": "fp-2"
                }
            ]
        })))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("activity-1"))
        .stdout(predicate::str::contains("fp-1"))
        .stdout(predicate::str::contains(
            "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
        ))
        .stdout(predicate::str::contains("ACTIVITY_STATUS_CONSENSUS_NEEDED"))
        .stdout(predicate::str::contains("2025-07-15"))
        .stdout(predicate::str::contains("activity-2"))
        .stdout(predicate::str::contains("fp-2"));
}

#[tokio::test]
async fn activity_list_reports_when_nothing_matches() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "activities": [] })),
        )
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching activities"));
}

async fn mount_no_pending_activities(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "activities": [] })),
        )
        .mount(server)
        .await;
}

async fn mount_whoami(server: &MockServer, user_id: &str) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/whoami"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organizationId": ORG_ID,
            "organizationName": "Test Org",
            "userId": user_id,
            "username": "root-user"
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn activity_approve_skips_when_user_already_voted() {
    let server = MockServer::start().await;
    mount_whoami(&server, "user-current").await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_activities"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activities": [
                {
                    "id": "activity-1",
                    "organizationId": ORG_ID,
                    "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                    "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                    "fingerprint": "fp-1",
                    "votes": [
                        {
                            "id": "vote-1",
                            "userId": "user-current",
                            "activityId": "activity-1",
                            "selection": "VOTE_SELECTION_APPROVED",
                            "message": "{}",
                            "publicKey": "public-key",
                            "signature": "signature",
                            "scheme": "SIGNATURE_SCHEME_TK_API_P256"
                        }
                    ]
                }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_organization_configs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "configs": {
                "features": [],
                "quorum": { "threshold": 2, "userIds": ["u1", "u2"] }
            }
        })))
        .mount(&server)
        .await;

    // Already voted: no new approval may be submitted.
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("approve")
        .arg("--fingerprint")
        .arg("fp-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("already approved"))
        .stdout(predicate::str::contains("1 of 2"));
}

#[tokio::test]
async fn activity_approve_reports_vote_recorded_when_quorum_still_pending() {
    let server = MockServer::start().await;
    mount_no_pending_activities(&server).await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-1",
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": "fp-1"
            }
        })))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("approve")
        .arg("--fingerprint")
        .arg("fp-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vote recorded"))
        .stdout(predicate::str::contains("still pending quorum"));
}

#[tokio::test]
async fn activity_approve_reports_completion_when_quorum_reached() {
    let server = MockServer::start().await;
    mount_no_pending_activities(&server).await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/approve_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-1",
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": "fp-1"
            }
        })))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("approve")
        .arg("--fingerprint")
        .arg("fp-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Approval submitted"))
        .stdout(predicate::str::contains("activity-1"))
        .stdout(predicate::str::contains("completed"));
}

#[tokio::test]
async fn activity_reject_reports_rejection() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/public/v1/submit/reject_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-1",
                "organizationId": ORG_ID,
                "status": "ACTIVITY_STATUS_REJECTED",
                "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
                "fingerprint": "fp-1"
            }
        })))
        .mount(&server)
        .await;

    authed_tvc_cmd(&server)
        .arg("activity")
        .arg("reject")
        .arg("--fingerprint")
        .arg("fp-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rejection submitted"))
        .stdout(predicate::str::contains("fp-1"));
}
