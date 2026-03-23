use std::fs;
use std::fs::File;
use std::process::Command;

use turnkey_auth::ssh;
use predicates::prelude::*;
use tempfile::tempdir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TURNKEY_TEST_V: &str = "00";

#[tokio::test]
async fn git_sign_writes_verifiable_sshsig_file() {
    let temp = tempdir().expect("temp dir should exist");
    let key_path = temp.path().join("id_ed25519");
    let public_key_path = temp.path().join("id_ed25519.pub");
    let payload_path = temp.path().join("payload.txt");
    let allowed_signers_path = temp.path().join("allowed_signers");

    let status = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .expect("ssh-keygen should run");
    assert!(status.success());

    fs::write(&payload_path, b"hello world").expect("payload should be written");

    let raw_signature = extract_raw_signature(&key_path, &payload_path);
    let public_key_line = fs::read_to_string(&public_key_path).expect("public key should exist");
    let parsed_public_key =
        ssh::parse_public_key_line(&public_key_line).expect("public key should parse");

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(&parsed_public_key.public_key)).await;
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/sign_raw_payload"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "result": {
                    "signRawPayloadResult": {
                        "r": hex::encode(&raw_signature[..32]),
                        "s": hex::encode(&raw_signature[32..]),
                        "v": TURNKEY_TEST_V
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut cmd = assert_cmd::Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("git-sign")
        .arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&public_key_path)
        .arg(&payload_path)
        .env("TURNKEY_ORGANIZATION_ID", "org-id")
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(api_key.compressed_public_key()),
        )
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            hex::encode(api_key.private_key()),
        )
        .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
        .env("TURNKEY_API_BASE_URL", server.uri());

    cmd.assert().success();

    let signature_path = payload_path.with_extension("txt.sig");
    assert!(signature_path.exists(), "signature file should be created");

    fs::write(
        &allowed_signers_path,
        format!("git {}", public_key_line.trim()),
    )
    .expect("allowed signers should be written");

    let status = Command::new("ssh-keygen")
        .args(["-Y", "verify", "-n", "git", "-I", "git", "-f"])
        .arg(&allowed_signers_path)
        .arg("-s")
        .arg(&signature_path)
        .stdin(File::open(&payload_path).expect("payload should open"))
        .status()
        .expect("ssh-keygen verify should run");

    assert!(status.success(), "ssh-keygen should verify auth output");
}

#[tokio::test]
async fn direct_ssh_signer_invocation_writes_verifiable_sshsig_file() {
    let temp = tempdir().expect("temp dir should exist");
    let key_path = temp.path().join("id_ed25519");
    let public_key_path = temp.path().join("id_ed25519.pub");
    let payload_path = temp.path().join("payload.txt");
    let allowed_signers_path = temp.path().join("allowed_signers");

    let status = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .expect("ssh-keygen should run");
    assert!(status.success());

    fs::write(&payload_path, b"hello world").expect("payload should be written");

    let raw_signature = extract_raw_signature(&key_path, &payload_path);
    let public_key_line = fs::read_to_string(&public_key_path).expect("public key should exist");
    let parsed_public_key =
        ssh::parse_public_key_line(&public_key_line).expect("public key should parse");

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(&parsed_public_key.public_key)).await;
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/sign_raw_payload"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "result": {
                    "signRawPayloadResult": {
                        "r": hex::encode(&raw_signature[..32]),
                        "s": hex::encode(&raw_signature[32..]),
                        "v": TURNKEY_TEST_V
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut cmd = assert_cmd::Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&public_key_path)
        .arg(&payload_path)
        .env("TURNKEY_ORGANIZATION_ID", "org-id")
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(api_key.compressed_public_key()),
        )
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            hex::encode(api_key.private_key()),
        )
        .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
        .env("TURNKEY_API_BASE_URL", server.uri());

    cmd.assert().success();

    let signature_path = payload_path.with_extension("txt.sig");
    assert!(signature_path.exists(), "signature file should be created");

    fs::write(
        &allowed_signers_path,
        format!("git {}", public_key_line.trim()),
    )
    .expect("allowed signers should be written");

    let status = Command::new("ssh-keygen")
        .args(["-Y", "verify", "-n", "git", "-I", "git", "-f"])
        .arg(&allowed_signers_path)
        .arg("-s")
        .arg(&signature_path)
        .stdin(File::open(&payload_path).expect("payload should open"))
        .status()
        .expect("ssh-keygen verify should run");

    assert!(status.success(), "ssh-keygen should verify auth output");
}

#[tokio::test]
async fn git_sign_rejects_public_key_that_does_not_match_configured_turnkey_key() {
    let temp = tempdir().expect("temp dir should exist");
    let key_path = temp.path().join("id_ed25519");
    let public_key_path = temp.path().join("id_ed25519.pub");
    let payload_path = temp.path().join("payload.txt");

    let status = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .expect("ssh-keygen should run");
    assert!(status.success());

    fs::write(&payload_path, b"hello world").expect("payload should be written");

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_private_key"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "privateKey": {
                "privateKeyId": "pk-id",
                "publicKey": "1111111111111111111111111111111111111111111111111111111111111111",
                "privateKeyName": "git signer",
                "curve": "CURVE_ED25519",
                "addresses": [],
                "privateKeyTags": [],
                "createdAt": null,
                "updatedAt": null,
                "exported": false,
                "imported": false
            }
        })))
        .mount(&server)
        .await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut cmd = assert_cmd::Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("git-sign")
        .arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&public_key_path)
        .arg(&payload_path)
        .env("TURNKEY_ORGANIZATION_ID", "org-id")
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(api_key.compressed_public_key()),
        )
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            hex::encode(api_key.private_key()),
        )
        .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
        .env("TURNKEY_API_BASE_URL", server.uri());

    cmd.assert().failure().stderr(predicate::str::contains(
        "does not match the configured Turnkey key",
    ));

    let signature_path = payload_path.with_extension("txt.sig");
    assert!(
        !signature_path.exists(),
        "signature file should not be created"
    );
}

async fn mount_get_private_key_mock(server: &MockServer, public_key: &str) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_private_key"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "privateKey": {
                "privateKeyId": "pk-id",
                "publicKey": public_key,
                "privateKeyName": "git signer",
                "curve": "CURVE_ED25519",
                "addresses": [],
                "privateKeyTags": [],
                "createdAt": null,
                "updatedAt": null,
                "exported": false,
                "imported": false
            }
        })))
        .mount(server)
        .await;
}

fn extract_raw_signature(key_path: &std::path::Path, payload_path: &std::path::Path) -> Vec<u8> {
    let status = Command::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "git", "-f"])
        .arg(key_path)
        .arg(payload_path)
        .status()
        .expect("ssh-keygen sign should run");
    assert!(status.success());

    let signature_path = payload_path.with_extension("txt.sig");
    let armored = fs::read_to_string(signature_path).expect("signature should exist");
    parse_raw_signature_from_armored(&armored)
}

fn parse_raw_signature_from_armored(armored: &str) -> Vec<u8> {
    use base64::Engine;

    let base64 = armored
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    let blob = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .expect("signature body should decode");

    let mut cursor = blob.as_slice();
    assert_eq!(&cursor[..6], b"SSHSIG");
    cursor = &cursor[6 + 4..];

    let _public_key = read_ssh_bytes(&mut cursor);
    let _namespace = read_ssh_bytes(&mut cursor);
    let _reserved = read_ssh_bytes(&mut cursor);
    let _hash_algorithm = read_ssh_bytes(&mut cursor);
    let signature_blob = read_ssh_bytes(&mut cursor);

    let mut signature_cursor = signature_blob.as_slice();
    let algorithm = read_ssh_bytes(&mut signature_cursor);
    assert_eq!(std::str::from_utf8(&algorithm).unwrap(), "ssh-ed25519");
    read_ssh_bytes(&mut signature_cursor)
}

fn read_ssh_bytes(cursor: &mut &[u8]) -> Vec<u8> {
    let length = u32::from_be_bytes(cursor[..4].try_into().expect("ssh length should exist"));
    *cursor = &cursor[4..];
    let length = length as usize;
    let value = cursor[..length].to_vec();
    *cursor = &cursor[length..];
    value
}
