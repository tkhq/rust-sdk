mod common;
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader},
    process::Stdio,
};
#[test]
fn profile_delete_conflict_preserves_key_files_and_the_other_writers_config() {
    let home = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(home.path(), &[("default", "org")], Some("default"));
    common::write_profile_key_files(home.path(), "default");
    let path = home.path().join(".config/turnkey/tvc.config.toml");
    let original = fs::read_to_string(&path).unwrap();
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path.with_file_name("tvc.config.toml.lock"))
        .unwrap();
    lock.lock().unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_tvc"));
    command
        .env_clear()
        .env("HOME", home.path())
        .env("RUST_LOG", "tvc=debug")
        .args([
            "--message-format",
            "json",
            "profile",
            "delete",
            "--org",
            "default",
            "--yes",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut diagnostics = BufReader::new(child.stderr.take().unwrap());
    loop {
        let mut line = String::new();
        assert_ne!(
            diagnostics.read_line(&mut line).unwrap(),
            0,
            "child ended before committing"
        );
        if line.contains("waiting for config commit lock") {
            break;
        }
    }
    let other = format!("{original}\n# another coordinated writer committed\n");
    fs::write(&path, &other).unwrap();
    drop(lock);
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["code"], "config_conflict");
    assert_eq!(fs::read_to_string(&path).unwrap(), other);
    assert!(
        home.path()
            .join(".config/turnkey/orgs/default/api_key.json")
            .exists()
    );
    assert!(
        home.path()
            .join(".config/turnkey/orgs/default/operator.json")
            .exists()
    );
}

#[cfg(target_os = "macos")]
#[test]
fn yubikey_save_recovery_respects_the_replacement_commit_point() {
    use std::process::Command;

    let support = tempfile::TempDir::new().unwrap();
    let library = support.path().join("fail_config_sync.dylib");
    let compile = Command::new("cc")
        .args(["-dynamiclib", "tests/support/fail_config_sync.c", "-o"])
        .arg(&library)
        .output()
        .unwrap();
    assert!(compile.status.success(), "{compile:?}");

    for (failure, expected_code, replaced) in [
        ("before_replacement", "config_save_failed", false),
        ("after_replacement", "config_durability_uncertain", true),
    ] {
        let home = tempfile::TempDir::new().unwrap();
        let path = home.path().join(".config/turnkey/tvc.config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let point = "046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5";
        let original = format!(
            r#"version = 1
active_org = "default"
[[yubikeys]]
serial = "01c95c1f"
public_key = "{point}{point}"
[orgs.default]
id = "org-123"
api_key_path = "/unused/api.json"
operators = []
"#
        );
        fs::write(&path, &original).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_tvc"))
            .env_clear()
            .env("HOME", home.path())
            .env("DYLD_INSERT_LIBRARIES", &library)
            .env("TVC_TEST_SYNC_FAILURE", failure)
            .args([
                "--message-format",
                "json",
                "operator",
                "create",
                "--kind",
                "yubikey",
                "--serial",
                "01c95c1f",
                "--default",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let error: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(error["code"], expected_code);
        let message = error["message"].as_str().unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        if replaced {
            let saved: toml::Value = toml::from_str(&saved).unwrap();
            assert_eq!(
                saved["orgs"]["default"]["operators"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                saved["orgs"]["default"]["operators"][0]["serial"].as_str(),
                Some("01c95c1f")
            );
            assert!(message.contains("replacement completed"), "{message}");
            assert!(!message.contains("or restore this"), "{message}");
            assert!(!message.contains("[[orgs."), "{message}");
        } else {
            assert_eq!(saved, original);
            assert!(message.contains("or restore this"), "{message}");
        }
    }
}
