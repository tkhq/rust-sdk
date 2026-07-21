use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_p256::P256Pair;

fn operator_encrypt_key() -> String {
    let public_key = P256Pair::generate().unwrap().public_key().to_bytes();
    hex::encode(&public_key[..65])
}

#[test]
fn create_quorum_key_help_lists_required_explicit_inputs() {
    cargo_bin_cmd!("tvc")
        .arg("keys")
        .arg("create-quorum-key")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--threshold <THRESHOLD>"))
        .stdout(predicate::str::contains("TVC_QUORUM_KEY_THRESHOLD"))
        .stdout(predicate::str::contains(
            "--operator-encrypt-keys <HEX,HEX,...>",
        ))
        .stdout(predicate::str::contains("TVC_OPERATOR_ENCRYPT_KEYS"))
        .stdout(predicate::str::contains("--operators").not());
}

#[test]
fn create_quorum_key_requires_both_explicit_inputs() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .arg("keys")
        .arg("create-quorum-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--threshold <THRESHOLD>"))
        .stderr(predicate::str::contains(
            "--operator-encrypt-keys <HEX,HEX,...>",
        ));
}

#[test]
fn create_quorum_key_reads_explicit_inputs_from_env() {
    let first = operator_encrypt_key();
    let second = operator_encrypt_key();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("TVC_QUORUM_KEY_THRESHOLD", "1")
        .env("TVC_OPERATOR_ENCRYPT_KEYS", format!("{first},{second}"))
        .arg("keys")
        .arg("create-quorum-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("threshold must be at least 2"))
        .stderr(
            predicate::str::contains("the following required arguments were not provided").not(),
        );
}

#[test]
fn create_quorum_key_rejects_threshold_above_u8_range() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .arg("keys")
        .arg("create-quorum-key")
        .arg("--threshold")
        .arg("256")
        .arg("--operator-encrypt-keys")
        .arg(format!(
            "{},{}",
            operator_encrypt_key(),
            operator_encrypt_key()
        ))
        .assert()
        .failure()
        .stderr(predicate::str::contains("256 is not in 0..=255"));
}
