use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn operator_create_help_documents_defaults_and_env_inputs() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--kind"))
        .stdout(predicate::str::contains("TVC_OPERATOR_KIND"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("tvc-operator"))
        .stdout(predicate::str::contains("TVC_OPERATOR_NAME"))
        .stdout(predicate::str::contains("--wallet-name"))
        .stdout(predicate::str::contains("tvc-wallet"))
        .stdout(predicate::str::contains("TVC_OPERATOR_WALLET_NAME"))
        .stdout(predicate::str::contains("--wallet-id"))
        .stdout(predicate::str::contains("TVC_OPERATOR_WALLET_ID"))
        .stdout(predicate::str::contains("--account-path"))
        .stdout(predicate::str::contains("m/5527107'/0'/0'"))
        .stdout(predicate::str::contains("TVC_OPERATOR_ACCOUNT_PATH"))
        .stdout(predicate::str::contains("--serial"))
        .stdout(predicate::str::contains("--default"));
}

#[test]
fn operator_create_wallet_inputs_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--wallet-name")
        .arg("wallet")
        .arg("--wallet-id")
        .arg("11111111-1111-4111-8111-111111111111")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with '--wallet-id"));
}

#[test]
fn operator_create_rejects_malformed_wallet_uuid() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--wallet-id")
        .arg("not-a-uuid")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value 'not-a-uuid' for '--wallet-id <WALLET_ID>'",
        ));
}

#[test]
fn operator_create_accepts_wallet_uuid_with_default_wallet_name() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("operator")
        .arg("create")
        .arg("--wallet-id")
        .arg("11111111-1111-4111-8111-111111111111")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"))
        .stderr(predicate::str::contains("cannot be used with '--wallet-id").not());
}

#[test]
fn operator_create_rejects_empty_text_inputs() {
    for flag in ["--name", "--wallet-name", "--account-path"] {
        cargo_bin_cmd!("tvc")
            .arg("operator")
            .arg("create")
            .arg(flag)
            .arg("")
            .assert()
            .failure()
            .stderr(predicate::str::contains("a value is required"));
    }
}

#[test]
fn a_serial_conflicts_with_the_wallet_flags() {
    cargo_bin_cmd!("tvc")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "01c95c1f", "--wallet-name", "wallet"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn a_serial_is_rejected_for_the_hosted_kind() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args(["operator", "create", "--serial", "01c95c1f"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--serial is only valid with --kind yubikey",
        ));
}

#[test]
fn a_yubikey_operator_requires_a_serial_in_non_interactive_mode() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--serial is required in non-interactive mode",
        ));
}

#[test]
fn an_unregistered_serial_cannot_be_provisioned_non_interactively() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "deadbeef"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "is not in the device registry, and provisioning it is interactive",
        ));
}

/// Write a v1 config with an active local-default org and one registered
/// YubiKey the org does not reference yet.
fn write_local_org_with_registered_yubikey(home: &Path) {
    let dir = home.join(".config/turnkey");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tvc.config.toml"),
        format!(
            r#"version = 1
active_org = "default"

[[yubikeys]]
serial = "01c95c1f"
public_key = "{key}"

[orgs.default]
id = "org-123"
api_key_path = "/keys/api.json"

[[orgs.default.operators]]
name = "default"
kind = "local"
key_path = "/keys/operator.json"
"#,
            key = "07".repeat(130)
        ),
    )
    .unwrap();
}

/// The system-level zero-hardware proof: adding a REGISTERED serial succeeds
/// non-interactively on a machine with no device attached (any PC/SC call
/// would fail), and a second add of the same serial is refused.
#[test]
fn a_registered_serial_is_added_non_interactively_without_a_device() {
    let temp = tempfile::TempDir::new().unwrap();
    write_local_org_with_registered_yubikey(temp.path());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "01c95c1f", "--default"])
        .assert()
        .success()
        .stdout(predicate::str::contains("YubiKey operator added!"))
        .stdout(predicate::str::contains("yubikey-01c95c1f"))
        .stdout(predicate::str::contains(
            "It is now the organization's default operator.",
        ));

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(
        saved.contains("default_operator_kind = \"yubikey\""),
        "{saved}"
    );
    assert!(saved.contains("name = \"yubikey-01c95c1f\""), "{saved}");
    assert!(saved.contains("[[yubikeys]]"), "{saved}");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "01c95c1f"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "YubiKey 01c95c1f is already an operator of this organization",
        ));
}

#[test]
fn a_failed_save_reports_the_recovery_record() {
    let temp = tempfile::TempDir::new().unwrap();
    write_local_org_with_registered_yubikey(temp.path());

    let config_path = temp.path().join(".config/turnkey/tvc.config.toml");
    let mut permissions = std::fs::metadata(&config_path).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&config_path, permissions).unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "01c95c1f", "--default"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the YubiKey operator could not be saved",
        ))
        .stderr(predicate::str::contains(r#"[[orgs."default".operators]]"#))
        .stderr(predicate::str::contains("kind = \"yubikey\""))
        .stderr(predicate::str::contains("default_operator_kind"));
}
