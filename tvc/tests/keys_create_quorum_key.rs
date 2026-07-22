use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_p256::P256Pair;

const FIRST_OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
const SECOND_OPERATOR_ID: &str = "22222222-2222-4222-8222-222222222222";

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
        .stdout(predicate::str::contains("--operator-encrypt-keys <HEX>"))
        .stdout(predicate::str::contains("TVC_OPERATOR_ENCRYPT_KEYS"))
        .stdout(predicate::str::contains("--operator-ids <UUID>"))
        .stdout(predicate::str::contains("TVC_OPERATOR_IDS"));
}

#[test]
fn create_quorum_key_requires_threshold_and_one_operator_source() {
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
            "<--operator-encrypt-keys <HEX>|--operator-ids <UUID>>",
        ));
}

#[test]
fn create_quorum_key_reads_explicit_inputs_from_env() {
    let first = operator_encrypt_key();
    let second = operator_encrypt_key();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("TVC_QUORUM_KEY_THRESHOLD", "3")
        .env("TVC_OPERATOR_ENCRYPT_KEYS", format!("{first},{second}"))
        .arg("keys")
        .arg("create-quorum-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "threshold (3) cannot exceed operator encryption public key count (2)",
        ))
        .stderr(
            predicate::str::contains("the following required arguments were not provided").not(),
        );
}

#[test]
fn create_quorum_key_reads_operator_ids_from_env() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("TVC_QUORUM_KEY_THRESHOLD", "3")
        .env(
            "TVC_OPERATOR_IDS",
            format!("{FIRST_OPERATOR_ID},{SECOND_OPERATOR_ID}"),
        )
        .arg("keys")
        .arg("create-quorum-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "threshold (3) cannot exceed operator encryption public key count (2)",
        ));
}

#[test]
fn create_quorum_key_operator_sources_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .arg("keys")
        .arg("create-quorum-key")
        .arg("--threshold")
        .arg("2")
        .arg("--operator-encrypt-keys")
        .arg(format!(
            "{},{}",
            operator_encrypt_key(),
            operator_encrypt_key()
        ))
        .arg("--operator-ids")
        .arg(format!("{FIRST_OPERATOR_ID},{SECOND_OPERATOR_ID}"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
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
        .stderr(predicate::str::contains(
            "out of range integral type conversion attempted",
        ));
}
