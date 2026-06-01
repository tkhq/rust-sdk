use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_core::protocol::services::boot::{
    Approval, Manifest, ManifestEnvelope, ManifestSet, Namespace, NitroConfig, PatchSet,
    PivotConfig, QuorumMember, RestartPolicy, ShareSet,
};
use qos_core::protocol::QosHash;
use qos_p256::{P256Pair, P256Public};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReEncryptedShareOutput {
    deployment_id: String,
    re_encrypted_share: String,
    share_approval: Approval,
}

fn sample_manifest_envelope(
    quorum_key: Vec<u8>,
    share_set_members: Vec<QuorumMember>,
) -> ManifestEnvelope {
    ManifestEnvelope {
        manifest: Manifest {
            namespace: Namespace {
                name: "test-namespace".to_string(),
                nonce: 7,
                quorum_key,
            },
            pivot: PivotConfig {
                hash: [0; 32],
                restart: RestartPolicy::Never,
                bridge_config: vec![],
                debug_mode: false,
                args: vec![],
            },
            manifest_set: ManifestSet {
                threshold: 0,
                members: vec![],
            },
            share_set: ShareSet {
                threshold: share_set_members.len() as u32,
                members: share_set_members,
            },
            enclave: NitroConfig {
                pcr0: vec![0; 48],
                pcr1: vec![1; 48],
                pcr2: vec![2; 48],
                pcr3: vec![3; 48],
                aws_root_certificate: vec![],
                qos_commit: "test-commit".to_string(),
            },
            patch_set: PatchSet {
                threshold: 0,
                members: vec![],
            },
        },
        manifest_set_approvals: vec![],
        share_set_approvals: vec![],
    }
}

/// On-disk inputs plus the secrets needed to validate the command's output.
struct Fixture {
    metadata_path: PathBuf,
    provision_bundle_path: PathBuf,
    operator_seed_path: PathBuf,
    ephemeral_pair: P256Pair,
    operator_public_key: Vec<u8>,
    plaintext_share: Vec<u8>,
    manifest_envelope: ManifestEnvelope,
}

/// Write a valid metadata / provision-bundle / operator-seed trio into `temp`
/// and return the paths plus the secrets a test needs to verify the result.
fn write_fixture(temp: &TempDir) -> Fixture {
    let metadata_path = temp.path().join("quorum_key_metadata.json");
    let provision_bundle_path = temp.path().join("provision_bundle.json");
    let operator_seed_path = temp.path().join("operator_seed.txt");

    let quorum_pair = P256Pair::generate().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let operator_public_key = operator_pair.public_key().to_bytes();
    let ephemeral_pair = P256Pair::generate().unwrap();
    let plaintext_share = b"operator quorum key share".to_vec();
    let encrypted_share = operator_pair.public_key().encrypt(&plaintext_share).unwrap();
    let manifest_envelope = sample_manifest_envelope(
        quorum_pair.public_key().to_bytes(),
        vec![QuorumMember {
            alias: "operator-1".to_string(),
            pub_key: operator_public_key.clone(),
        }],
    );

    fs::write(
        &operator_seed_path,
        String::from_utf8(operator_pair.to_master_seed_hex()).unwrap(),
    )
    .unwrap();
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&json!({
            "quorumKeyPublic": hex::encode(quorum_pair.public_key().to_bytes()),
            "threshold": 1,
            "shares": [{
                "operatorPublicKey": hex::encode(&operator_public_key),
                "share": hex::encode(&encrypted_share),
            }],
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &provision_bundle_path,
        serde_json::to_vec_pretty(&json!({
            "attestationDocumentCoseSign1Base64": "not parsed when verification is skipped",
            "manifestEnvelope": manifest_envelope,
            "fetchedAtUnixMs": 1_712_345_678_901_u64,
            "deploymentId": "deploy-123",
            "ephemeralPublicKeyHex": hex::encode(ephemeral_pair.public_key().to_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();

    Fixture {
        metadata_path,
        provision_bundle_path,
        operator_seed_path,
        ephemeral_pair,
        operator_public_key,
        plaintext_share,
        manifest_envelope,
    }
}

/// Base command (without an output destination or `--format`) for `fixture`.
fn re_encrypt_cmd(fixture: &Fixture) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.arg("keys")
        .arg("re-encrypt-share")
        .arg("--quorum-key-metadata")
        .arg(&fixture.metadata_path)
        .arg("--provision-bundle")
        .arg(&fixture.provision_bundle_path)
        .arg("--operator-seed")
        .arg(&fixture.operator_seed_path)
        .arg("--dangerous-skip-verification");
    cmd
}

/// Assert a re-encrypted-share JSON value uses camelCase, round-trips the share
/// back to its plaintext, and carries an approval that verifies.
fn assert_valid_share(value: &Value, fixture: &Fixture) {
    assert_eq!(value["deploymentId"], json!("deploy-123"));
    assert_eq!(
        value["ephemeralPublicKeyHex"],
        json!(hex::encode(fixture.ephemeral_pair.public_key().to_bytes()))
    );
    assert!(value.get("reEncryptedShare").is_some());
    assert!(value.get("shareApproval").is_some());
    assert!(value.get("re_encrypted_share").is_none());
    assert!(value.get("share_approval").is_none());

    let output: ReEncryptedShareOutput = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(output.deployment_id, "deploy-123");
    let re_encrypted_share = hex::decode(&output.re_encrypted_share).unwrap();
    let decrypted_share = fixture.ephemeral_pair.decrypt(&re_encrypted_share).unwrap();
    assert_eq!(decrypted_share, fixture.plaintext_share);
    assert_eq!(
        output.share_approval.member.pub_key,
        fixture.operator_public_key
    );

    let approval_public_key =
        P256Public::from_bytes(&output.share_approval.member.pub_key).unwrap();
    approval_public_key
        .verify(
            &fixture.manifest_envelope.manifest.qos_hash(),
            &output.share_approval.signature,
        )
        .unwrap();
}

#[test]
fn root_help_does_not_list_re_encrypt_share_command() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypt-share").not());
}

#[test]
fn keys_help_lists_re_encrypt_share_command() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypt-share"))
        .stdout(predicate::str::contains(
            "Re-encrypt a share for enclave provisioning",
        ));
}

#[test]
fn re_encrypt_share_help_lists_expected_flags() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("re-encrypt-share")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--quorum-key-metadata <PATH>"))
        .stdout(predicate::str::contains("--provision-bundle <PATH>"))
        .stdout(predicate::str::contains("--operator-seed <PATH>"))
        .stdout(predicate::str::contains("--dangerous-skip-verification"))
        .stdout(predicate::str::contains("--re-encrypted-out <PATH>"));
}

#[test]
fn re_encrypt_share_requires_metadata_and_provision_bundle() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("re-encrypt-share")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--quorum-key-metadata <PATH>"))
        .stderr(predicate::str::contains("--provision-bundle <PATH>"));
}

#[test]
fn re_encrypt_share_round_trips_metadata_share() {
    let temp = TempDir::new().unwrap();
    let fixture = write_fixture(&temp);
    let output_path = temp.path().join("re_encrypted_share.json");

    re_encrypt_cmd(&fixture)
        .arg("--re-encrypted-out")
        .arg(&output_path)
        .assert()
        .success();

    let value: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
    assert_valid_share(&value, &fixture);
}

/// `--format json` with no output file emits the share artifact as the sole
/// JSON document on stdout (matching the historical inline behavior).
#[test]
fn re_encrypt_share_json_inline_emits_share_on_stdout() {
    let temp = TempDir::new().unwrap();
    let fixture = write_fixture(&temp);

    let output = re_encrypt_cmd(&fixture)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout must be a JSON share, got {stdout:?}: {e}"));
    assert_valid_share(&value, &fixture);
}

/// `--format json` with `--re-encrypted-out` writes the artifact to the file and
/// emits a `{ reEncryptedSharePath }` envelope on stdout pointing at it.
#[test]
fn re_encrypt_share_json_to_file_emits_path_envelope() {
    let temp = TempDir::new().unwrap();
    let fixture = write_fixture(&temp);
    let output_path = temp.path().join("re_encrypted_share.json");

    let output = re_encrypt_cmd(&fixture)
        .arg("--format")
        .arg("json")
        .arg("--re-encrypted-out")
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let envelope: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        envelope["reEncryptedSharePath"],
        json!(output_path.display().to_string())
    );

    // The artifact itself lands in the file and round-trips.
    let file_value: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
    assert_valid_share(&file_value, &fixture);
}
