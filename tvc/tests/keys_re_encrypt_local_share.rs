mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_core::protocol::services::boot::{
    Approval, Manifest, ManifestEnvelope, ManifestSet, Namespace, NitroConfig, PatchSet,
    PivotConfig, QuorumMember, RestartPolicy, ShareSet, VersionedManifestEnvelope,
};
use qos_p256::{P256Pair, P256Public};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReEncryptedShareOutput {
    deployment_id: String,
    ephemeral_public_key_hex: String,
    re_encrypted_share: String,
    share_approval: Approval,
}

fn sample_manifest_envelope(quorum_key: Vec<u8>, share_set: ShareSet) -> VersionedManifestEnvelope {
    let envelope = ManifestEnvelope {
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
            share_set,
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
    };
    envelope.into()
}

fn member(alias: &str, operator_pair: &P256Pair) -> QuorumMember {
    QuorumMember {
        alias: alias.to_string(),
        pub_key: operator_pair.public_key().to_bytes(),
    }
}

fn share_entry(operator_pair: &P256Pair, plaintext_share: &[u8]) -> Value {
    json!({
        "operatorPublicKey": hex::encode(operator_pair.public_key().to_bytes()),
        "share": hex::encode(operator_pair.public_key().encrypt(plaintext_share).unwrap()),
    })
}

struct ShareInputs {
    metadata: PathBuf,
    provision_bundle: PathBuf,
    operator_seed: PathBuf,
    output: PathBuf,
}

fn write_share_inputs(
    dir: &Path,
    operator_pair: &P256Pair,
    metadata: Value,
    manifest_envelope: &VersionedManifestEnvelope,
    ephemeral_public_key: &P256Public,
) -> ShareInputs {
    let inputs = ShareInputs {
        metadata: dir.join("quorum_key_metadata.json"),
        provision_bundle: dir.join("provision_bundle.json"),
        operator_seed: dir.join("operator_seed.txt"),
        output: dir.join("re_encrypted_share.json"),
    };

    fs::write(
        &inputs.operator_seed,
        String::from_utf8(operator_pair.to_master_seed_hex().to_vec()).unwrap(),
    )
    .unwrap();
    fs::write(
        &inputs.metadata,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    fs::write(
        &inputs.provision_bundle,
        serde_json::to_vec_pretty(&json!({
            "attestationDocumentCoseSign1Base64": "not parsed when verification is skipped",
            "manifestEnvelope": manifest_envelope,
            "fetchedAtUnixMs": 1_712_345_678_901_u64,
            "deploymentId": "deploy-123",
            "ephemeralPublicKeyHex": hex::encode(ephemeral_public_key.to_bytes()),
        }))
        .unwrap(),
    )
    .unwrap();

    inputs
}

fn assert_share_set_refused(
    dir: &Path,
    operator_pair: &P256Pair,
    metadata_threshold: u32,
    share_holders: &[&P256Pair],
    share_set: ShareSet,
    message: &str,
) {
    let quorum_pair = P256Pair::generate().unwrap();
    let inputs = write_share_inputs(
        dir,
        operator_pair,
        json!({
            "quorumKeyPublic": hex::encode(quorum_pair.public_key().to_bytes()),
            "threshold": metadata_threshold,
            "shares": share_holders
                .iter()
                .map(|holder| share_entry(holder, b"share"))
                .collect::<Vec<_>>(),
        }),
        &sample_manifest_envelope(quorum_pair.public_key().to_bytes(), share_set),
        &P256Pair::generate().unwrap().public_key(),
    );

    let assert = cargo_bin_cmd!("tvc")
        .env("HOME", dir)
        .env_remove("TVC_OPERATOR_SEED")
        .arg("--message-format=json")
        .args(["keys", "re-encrypt-local-share"])
        .arg("--quorum-key-metadata")
        .arg(&inputs.metadata)
        .arg("--provision-bundle")
        .arg(&inputs.provision_bundle)
        .arg("--operator-seed-path")
        .arg(&inputs.operator_seed)
        .args([
            "--dangerous-skip-verification",
            "--dangerous-skip-interactive",
        ])
        .arg("--re-encrypted-out")
        .arg(&inputs.output)
        .assert()
        .failure();

    assert_eq!(
        serde_json::from_slice::<Value>(&assert.get_output().stdout).unwrap(),
        json!({
            "reason": "command_error",
            "code": "command_error",
            "message": message,
        })
    );
    assert!(!inputs.output.exists());
}

#[test]
fn root_help_does_not_list_re_encrypt_local_share_command() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypt-local-share").not());
}

#[test]
fn keys_help_lists_re_encrypt_local_share_command_only() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypt-local-share"))
        .stdout(predicate::str::contains("re-encrypt-share").not())
        .stdout(predicate::str::contains(
            "Re-encrypt a local share for enclave provisioning",
        ));
}

#[test]
fn re_encrypt_local_share_help_lists_expected_flags() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("re-encrypt-local-share")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--quorum-key-metadata <PATH>"))
        .stdout(predicate::str::contains("--provision-bundle <PATH>"))
        .stdout(predicate::str::contains("--operator-seed <HEX_SEED>"))
        .stdout(predicate::str::contains("--operator-seed-path <PATH>"))
        .stdout(predicate::str::contains("--dangerous-skip-verification"))
        .stdout(predicate::str::contains("--dangerous-skip-interactive"))
        .stdout(predicate::str::contains("--re-encrypted-out <PATH>"));
}

#[test]
fn re_encrypt_local_share_requires_metadata_and_provision_bundle() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("re-encrypt-local-share")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--quorum-key-metadata <PATH>"))
        .stderr(predicate::str::contains("--provision-bundle <PATH>"));
}

/// Without explicit seed flags the operator comes from the active org's
/// registry, and a hosted-only org has no local key to decrypt with: the
/// refusal fires during backend selection, before the input files are even
/// read (they do not exist here), and points at the hosted counterpart.
#[test]
fn hosted_only_org_is_pointed_at_deploy_provision() {
    let temp = TempDir::new().unwrap();
    common::write_hosted_only_config(
        temp.path(),
        "hosted-org",
        "88888888-8888-4888-8888-888888888888",
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove("TVC_OPERATOR_SEED")
        .arg("keys")
        .arg("re-encrypt-local-share")
        .arg("--quorum-key-metadata")
        .arg(temp.path().join("does_not_exist_metadata.json"))
        .arg("--provision-bundle")
        .arg(temp.path().join("does_not_exist_bundle.json"))
        .arg("--dangerous-skip-verification")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "needs the local operator key the share was encrypted to",
        ))
        .stderr(predicate::str::contains("tvc deploy provision"));
}

/// A yubikey org in a non-interactive run refuses during backend selection:
/// the PIN can only be typed at a prompt. The refusal fires before the input
/// files are even read (they do not exist here) and no device is touched, so
/// this needs no USB.
#[test]
fn yubikey_org_without_a_prompt_reports_the_pin_requirement() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config(
        temp.path(),
        "yubikey-org",
        "99999999-9999-4999-8999-999999999999",
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove("TVC_OPERATOR_SEED")
        .arg("keys")
        .arg("re-encrypt-local-share")
        .arg("--quorum-key-metadata")
        .arg(temp.path().join("does_not_exist_metadata.json"))
        .arg("--provision-bundle")
        .arg(temp.path().join("does_not_exist_bundle.json"))
        .arg("--dangerous-skip-verification")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "a YubiKey operator needs its PIN typed at an interactive prompt",
        ));
}

#[test]
fn an_unknown_yubikey_serial_is_rejected_before_share_input_io() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config(
        temp.path(),
        "yubikey-org",
        "99999999-9999-4999-8999-999999999999",
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove("TVC_OPERATOR_SEED")
        .args(["keys", "re-encrypt-local-share"])
        .arg("--quorum-key-metadata")
        .arg(temp.path().join("does-not-exist-metadata.json"))
        .arg("--provision-bundle")
        .arg(temp.path().join("does-not-exist-bundle.json"))
        .args([
            "--dangerous-skip-verification",
            "--dangerous-skip-interactive",
            "--serial",
            "deadbeef",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no YubiKey operator has serial deadbeef",
        ))
        .stderr(predicate::str::contains("does-not-exist-metadata.json").not());
}

#[test]
fn non_interactive_run_requires_a_manifest_review_choice() {
    let temp = TempDir::new().unwrap();

    let assert = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--message-format=json")
        .args(["keys", "re-encrypt-local-share"])
        .arg("--quorum-key-metadata")
        .arg(temp.path().join("does-not-exist-metadata.json"))
        .arg("--provision-bundle")
        .arg(temp.path().join("does-not-exist-bundle.json"))
        .assert()
        .failure();

    assert_eq!(
        serde_json::from_slice::<Value>(&assert.get_output().stdout).unwrap(),
        json!({
            "reason": "missing_required_input",
            "code": "missing_required_input",
            "message": "--dangerous-skip-interactive is required in non-interactive mode (set --dangerous-skip-interactive or run in a TTY without --non-interactive / TVC_NON_INTERACTIVE=true)",
        })
    );
}

#[test]
fn re_encrypt_local_share_round_trips_metadata_share() {
    let temp = TempDir::new().unwrap();
    let quorum_pair = P256Pair::generate().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let operator_public_key = operator_pair.public_key().to_bytes();
    let ephemeral_pair = P256Pair::generate().unwrap();
    let plaintext_share = b"operator quorum key share";
    let manifest_envelope = sample_manifest_envelope(
        quorum_pair.public_key().to_bytes(),
        ShareSet {
            threshold: 1,
            members: vec![member("operator-1", &operator_pair)],
        },
    );
    let inputs = write_share_inputs(
        temp.path(),
        &operator_pair,
        json!({
            "quorumKeyPublic": hex::encode(quorum_pair.public_key().to_bytes()),
            "threshold": 1,
            "shares": [share_entry(&operator_pair, plaintext_share)],
        }),
        &manifest_envelope,
        &ephemeral_pair.public_key(),
    );

    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("re-encrypt-local-share")
        .arg("--quorum-key-metadata")
        .arg(&inputs.metadata)
        .arg("--provision-bundle")
        .arg(&inputs.provision_bundle)
        .arg("--operator-seed-path")
        .arg(&inputs.operator_seed)
        .arg("--dangerous-skip-verification")
        .arg("--dangerous-skip-interactive")
        .arg("--re-encrypted-out")
        .arg(&inputs.output)
        .assert()
        .success();

    let value: Value = serde_json::from_slice(&fs::read(&inputs.output).unwrap()).unwrap();
    assert_eq!(value["deploymentId"], json!("deploy-123"));
    assert_eq!(
        value["ephemeralPublicKeyHex"],
        json!(hex::encode(ephemeral_pair.public_key().to_bytes()))
    );
    assert!(value.get("reEncryptedShare").is_some());
    assert!(value.get("shareApproval").is_some());
    assert!(value.get("re_encrypted_share").is_none());
    assert!(value.get("share_approval").is_none());

    let output: ReEncryptedShareOutput = serde_json::from_value(value).unwrap();
    assert_eq!(output.deployment_id, "deploy-123");
    assert_eq!(
        output.ephemeral_public_key_hex,
        hex::encode(ephemeral_pair.public_key().to_bytes())
    );
    let re_encrypted_share = hex::decode(&output.re_encrypted_share).unwrap();
    let decrypted_share = ephemeral_pair.decrypt(&re_encrypted_share).unwrap();
    assert_eq!(decrypted_share.as_slice(), plaintext_share);
    assert_eq!(output.share_approval.member.pub_key, operator_public_key);

    let approval_public_key =
        P256Public::from_bytes(&output.share_approval.member.pub_key).unwrap();
    approval_public_key
        .verify(
            &manifest_envelope.manifest_hash(),
            &output.share_approval.signature,
        )
        .unwrap();
}

#[test]
fn re_encrypt_local_share_refuses_a_threshold_the_manifest_overstates() {
    let temp = TempDir::new().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let second_operator = P256Pair::generate().unwrap();
    let third_operator = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &operator_pair,
        2,
        &[&operator_pair, &second_operator, &third_operator],
        ShareSet {
            threshold: 3,
            members: vec![
                member("operator-1", &operator_pair),
                member("operator-2", &second_operator),
                member("operator-3", &third_operator),
            ],
        },
        "quorum key metadata threshold (2) does not match provision bundle manifest share set threshold (3)",
    );
}

#[test]
fn re_encrypt_local_share_refuses_a_share_holder_the_manifest_does_not_name() {
    let temp = TempDir::new().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let share_holder = P256Pair::generate().unwrap();
    let manifest_member = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &operator_pair,
        2,
        &[&operator_pair, &share_holder],
        ShareSet {
            threshold: 2,
            members: vec![
                member("operator-1", &operator_pair),
                member("operator-2", &manifest_member),
            ],
        },
        &format!(
            "quorum key metadata holds a share for operator {}, which is not a provision bundle manifest share set member",
            hex::encode(share_holder.public_key().to_bytes()),
        ),
    );
}

#[test]
fn re_encrypt_local_share_refuses_a_share_set_smaller_than_the_metadata() {
    let temp = TempDir::new().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let second_operator = P256Pair::generate().unwrap();
    let third_operator = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &operator_pair,
        2,
        &[&operator_pair, &second_operator, &third_operator],
        ShareSet {
            threshold: 2,
            members: vec![
                member("operator-1", &operator_pair),
                member("operator-2", &second_operator),
            ],
        },
        "quorum key metadata holds 3 shares but provision bundle manifest share set has 2 members",
    );
}

#[test]
fn re_encrypt_local_share_refuses_metadata_holding_two_shares_for_one_operator() {
    let temp = TempDir::new().unwrap();
    let repeated_operator = P256Pair::generate().unwrap();
    let shareless_operator = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &repeated_operator,
        2,
        &[&repeated_operator, &repeated_operator],
        ShareSet {
            threshold: 2,
            members: vec![
                member("operator-1", &repeated_operator),
                member("operator-2", &shareless_operator),
            ],
        },
        &format!(
            "quorum key metadata holds more than one share for operator {}",
            hex::encode(repeated_operator.public_key().to_bytes()),
        ),
    );
}

#[test]
fn re_encrypt_local_share_refuses_a_share_set_naming_one_operator_twice() {
    let temp = TempDir::new().unwrap();
    let repeated_operator = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &repeated_operator,
        2,
        &[&repeated_operator, &repeated_operator],
        ShareSet {
            threshold: 2,
            members: vec![
                member("operator-1", &repeated_operator),
                member("operator-2", &repeated_operator),
            ],
        },
        &format!(
            "provision bundle manifest share set names operator {} more than once",
            hex::encode(repeated_operator.public_key().to_bytes()),
        ),
    );
}

#[test]
fn re_encrypt_local_share_refuses_an_operator_seed_the_metadata_has_no_share_for() {
    let temp = TempDir::new().unwrap();
    let share_holder = P256Pair::generate().unwrap();
    let missing_operator = P256Pair::generate().unwrap();

    assert_share_set_refused(
        temp.path(),
        &missing_operator,
        1,
        &[&share_holder],
        ShareSet {
            threshold: 1,
            members: vec![member("operator-1", &share_holder)],
        },
        &format!(
            "operator ({}) not found in quorum key metadata shares",
            hex::encode(missing_operator.public_key().to_bytes()),
        ),
    );
}

#[test]
fn re_encrypt_local_share_refuses_a_quorum_key_the_manifest_does_not_name() {
    let temp = TempDir::new().unwrap();
    let quorum_pair = P256Pair::generate().unwrap();
    let manifest_quorum_pair = P256Pair::generate().unwrap();
    let operator_pair = P256Pair::generate().unwrap();
    let inputs = write_share_inputs(
        temp.path(),
        &operator_pair,
        json!({
            "quorumKeyPublic": hex::encode(quorum_pair.public_key().to_bytes()),
            "threshold": 1,
            "shares": [share_entry(&operator_pair, b"share")],
        }),
        &sample_manifest_envelope(
            manifest_quorum_pair.public_key().to_bytes(),
            ShareSet {
                threshold: 1,
                members: vec![member("operator-1", &operator_pair)],
            },
        ),
        &P256Pair::generate().unwrap().public_key(),
    );

    let assert = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove("TVC_OPERATOR_SEED")
        .arg("--message-format=json")
        .args(["keys", "re-encrypt-local-share"])
        .arg("--quorum-key-metadata")
        .arg(&inputs.metadata)
        .arg("--provision-bundle")
        .arg(&inputs.provision_bundle)
        .arg("--operator-seed-path")
        .arg(&inputs.operator_seed)
        .args([
            "--dangerous-skip-verification",
            "--dangerous-skip-interactive",
        ])
        .arg("--re-encrypted-out")
        .arg(&inputs.output)
        .assert()
        .failure();

    assert_eq!(
        serde_json::from_slice::<Value>(&assert.get_output().stdout).unwrap(),
        json!({
            "reason": "command_error",
            "code": "command_error",
            "message": format!(
                "quorum key metadata quorumKeyPublic ({}) does not match provision bundle manifest quorumKey ({})",
                hex::encode(quorum_pair.public_key().to_bytes()),
                hex::encode(manifest_quorum_pair.public_key().to_bytes()),
            ),
        })
    );
    assert!(!inputs.output.exists());
}
