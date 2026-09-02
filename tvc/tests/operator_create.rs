mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_p256::P256Pair;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread::{self, JoinHandle};

fn spawn_create_operator_server() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
    let (encrypt_public_key, sign_public_key) = composite.split_at(composite.len() / 2);
    let body = format!(
        r#"{{"activity":{{"id":"activity-1","organizationId":"org-123","status":"ACTIVITY_STATUS_COMPLETED","type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR","result":{{"createTvcOperatorResult":{{"walletId":"22222222-2222-4222-8222-222222222222","operatorId":"11111111-1111-4111-8111-111111111111","encryptPublicKey":"{encrypt_public_key}","signPublicKey":"{sign_public_key}"}}}},"fingerprint":""}}}}"#
    );
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        assert_eq!(
            request_line.split_whitespace().nth(1),
            Some("/public/v1/submit/create_tvc_operator")
        );

        let mut content_length = 0;

        loop {
            let mut header = String::new();
            reader.read_line(&mut header).unwrap();

            if header == "\r\n" {
                break;
            }

            if let Some(value) = header
                .strip_prefix("content-length:")
                .or_else(|| header.strip_prefix("Content-Length:"))
            {
                content_length = value.trim().parse().unwrap();
            }
        }

        let mut request_body = vec![0; content_length];
        reader.read_exact(&mut request_body).unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    (format!("http://{address}"), handle)
}

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
fn an_unregistered_serial_points_at_external_setup() {
    let temp = tempfile::TempDir::new().unwrap();
    write_local_org_with_registered_yubikey(temp.path());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .args(["operator", "create", "--kind", "yubikey"])
        .args(["--serial", "deadbeef"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "is not in the device registry; install its certificates",
        ))
        .stderr(predicate::str::contains(
            "tvc keys refresh-yubikey --serial deadbeef",
        ));
}

/// Write a v2 config with an active local-default org and one registered
/// YubiKey the org does not reference yet.
fn write_local_org_with_registered_yubikey(home: &Path) {
    let dir = home.join(".config/turnkey");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tvc.config.toml"),
        format!(
            r#"version = 2
active_org = "11111111-1111-4111-8111-111111111111"

[aliases]
default = "11111111-1111-4111-8111-111111111111"

[[yubikeys]]
serial = "01c95c1f"
public_key = "{key}"

[orgs.11111111-1111-4111-8111-111111111111]
api_key_path = "/keys/api.json"

[[orgs.11111111-1111-4111-8111-111111111111.operators]]
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
fn a_failed_hosted_default_save_reports_the_record_and_default_kind() {
    let temp = tempfile::TempDir::new().unwrap();
    let (api_base_url, server) = spawn_create_operator_server();
    common::write_profiles_config(
        temp.path(),
        &[("default", "11111111-1111-4111-8111-111111111111")],
        Some("default"),
    );
    common::write_profile_key_files(temp.path(), "default");

    let config_path = temp.path().join(".config/turnkey/tvc.config.toml");
    let config = std::fs::read_to_string(&config_path)
        .unwrap()
        .replace(common::LOCAL_API_BASE_URL, &api_base_url);
    std::fs::write(&config_path, config).unwrap();
    let mut permissions = std::fs::metadata(&config_path).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&config_path, permissions).unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args(["operator", "create", "--default"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "hosted operator 11111111-1111-4111-8111-111111111111 was created remotely",
        ))
        .stderr(predicate::str::contains(
            r#"[[orgs."11111111-1111-4111-8111-111111111111".operators]]"#,
        ))
        .stderr(predicate::str::contains("kind = \"hosted\""))
        .stderr(predicate::str::contains(
            "default_operator_kind = \"hosted\"",
        ));

    server.join().unwrap();
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
        .stderr(predicate::str::contains(
            r#"[[orgs."11111111-1111-4111-8111-111111111111".operators]]"#,
        ))
        .stderr(predicate::str::contains("kind = \"yubikey\""))
        .stderr(predicate::str::contains("default_operator_kind"));
}
