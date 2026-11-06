mod common;
use assert_cmd::cargo::cargo_bin_cmd;
use qos_p256::P256Pair;
use serde_json::Value;
use std::{fs, net::TcpListener};

fn recipients() -> String {
    (0..2)
        .map(|_| hex::encode(&P256Pair::generate().unwrap().public_key().to_bytes()[..65]))
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn both_creation_commands_refuse_with_zero_posts_and_unchanged_or_absent_config() {
    for existing in [false, true] {
        for operator in [false, true] {
            let home = tempfile::TempDir::new().unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            if existing {
                common::write_profiles_config(home.path(), &[("active", "org")], Some("active"));
            }
            let path = home.path().join(".config/turnkey/tvc.config.toml");
            let baseline = fs::read(&path).ok();
            let api_key = turnkey_api_key_stamper::TurnkeyP256ApiKey::generate();
            let mut command = cargo_bin_cmd!("tvc");
            command
                .env_clear()
                .env("HOME", home.path())
                .env("TVC_API_BASE_URL", url)
                .env("TVC_ORG_ID", "org")
                .env(
                    "TVC_API_KEY_PUBLIC",
                    hex::encode(api_key.compressed_public_key()),
                )
                .env("TVC_API_KEY_PRIVATE", hex::encode(api_key.private_key()))
                .args(["--message-format", "json"]);
            if operator {
                command.args(["operator", "create"]);
            } else {
                command.args([
                    "keys",
                    "create-quorum-key",
                    "--threshold",
                    "2",
                    "--operator-encrypt-keys",
                    &recipients(),
                ]);
            }
            let output = command.output().unwrap();
            assert_eq!(output.status.code(), Some(1));
            let lines = std::str::from_utf8(&output.stdout)
                .unwrap()
                .lines()
                .collect::<Vec<_>>();
            assert_eq!(lines.len(), 1);
            let error: Value = serde_json::from_str(lines[0]).unwrap();
            assert_eq!(error["reason"], "command_error");
            assert_eq!(error["code"], "provenance_unsupported_environment");
            assert!(error.get("quorumPublicKey").is_none());
            assert!(error.get("encryptPublicKey").is_none());
            assert!(output.stderr.is_empty());
            assert_eq!(fs::read(&path).ok(), baseline);
            if !existing {
                assert!(!home.path().join(".config").exists());
            }
            assert_eq!(
                listener.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }
    }
}
