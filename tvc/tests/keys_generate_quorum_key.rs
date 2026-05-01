use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_p256::{P256Pair, P256Public, MASTER_SEED_LEN};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuorumKeyMetadata {
    quorum_key_public: String,
    threshold: u32,
    shares: Vec<EncryptedShare>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedShare {
    operator_public_key: String,
    share: String,
}

#[test]
fn generate_quorum_key_writes_metadata() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("quorum_key.json");
    let metadata_path = temp.path().join("quorum_key_metadata.json");

    let operator_pairs = (0..3)
        .map(|_| P256Pair::generate().unwrap())
        .collect::<Vec<_>>();
    let operator_public_keys = operator_pairs
        .iter()
        .map(|pair| hex::encode(pair.public_key().to_bytes()))
        .collect::<Vec<_>>();

    fs::write(
        &config_path,
        serde_json::to_vec_pretty(&json!({
            "shares": 3,
            "threshold": 2,
            "operatorPublicKeys": [
                operator_public_keys[0].to_uppercase(),
                operator_public_keys[1].clone(),
                operator_public_keys[2].clone(),
            ],
        }))
        .unwrap(),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("generate-quorum-key")
        .arg(&config_path)
        .arg("--quorum-key-out")
        .arg(&metadata_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Quorum Public Key:"))
        .stdout(predicate::str::contains("Threshold: 2"));

    let metadata: QuorumKeyMetadata =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();

    assert_eq!(metadata.threshold, 2);
    assert_eq!(metadata.shares.len(), 3);
    assert_eq!(
        metadata
            .shares
            .iter()
            .map(|share| share.operator_public_key.clone())
            .collect::<Vec<_>>(),
        operator_public_keys
    );
    P256Public::from_bytes(&hex::decode(&metadata.quorum_key_public).unwrap()).unwrap();

    let decrypted_shares = metadata
        .shares
        .iter()
        .zip(operator_pairs.iter())
        .map(|(share, pair)| {
            let encrypted_share = hex::decode(&share.share).unwrap();
            pair.decrypt(&encrypted_share).unwrap()
        })
        .collect::<Vec<_>>();

    let reconstructed = qos_crypto::shamir::shares_reconstruct(&decrypted_shares[..2])
        .unwrap()
        .try_into()
        .map(|seed: [u8; MASTER_SEED_LEN]| P256Pair::from_master_seed(&seed).unwrap())
        .unwrap();
    assert_eq!(
        hex::encode(reconstructed.public_key().to_bytes()),
        metadata.quorum_key_public
    );
}

#[test]
fn generate_quorum_key_rejects_invalid_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("quorum_key.json");

    fs::write(
        &config_path,
        serde_json::to_vec_pretty(&json!({
            "shares": 2,
            "threshold": 2,
            "operatorPublicKeys": ["not-hex", "also-not-hex"],
        }))
        .unwrap(),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("generate-quorum-key")
        .arg(&config_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid operator public key at index 0",
        ));
}

#[test]
fn keys_help_lists_generate_quorum_key() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("generate-quorum-key"))
        .stdout(predicate::str::contains(
            "Generate and shamir-split a quorum key, encrypting each share to an operator key",
        ));
}
