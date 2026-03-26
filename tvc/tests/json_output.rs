use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn app_init_json_output() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("app.json");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--json")
        .arg("app")
        .arg("init")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"config_file\""));
}

#[test]
fn app_init_json_output_is_valid_json() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("app.json");

    let output = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--json")
        .arg("app")
        .arg("init")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("not utf8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
    assert!(parsed.get("config_file").is_some());
}

#[test]
fn deploy_init_json_output() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("deploy.json");

    // Need a HOME with config dir for deploy init
    let config_dir = temp.path().join(".config").join("turnkey");
    std::fs::create_dir_all(&config_dir).unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--json")
        .arg("deploy")
        .arg("init")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"config_file\""));
}

#[test]
fn deploy_approve_json_output_with_skip_post() {
    cargo_bin_cmd!("tvc")
        .arg("--json")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--yes")
        .arg("--skip-post")
        .assert()
        .success()
        // The approval JSON is still on stdout even in --json mode
        .stdout(predicate::str::contains("\"signature\""));
}

#[test]
fn quiet_mode_suppresses_status() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("app.json");

    let output = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--quiet")
        .arg("app")
        .arg("init")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("not utf8");
    let stderr = String::from_utf8(output.stderr).expect("not utf8");

    // Quiet mode: no stdout or stderr output
    assert!(stdout.is_empty(), "stdout should be empty in quiet mode");
    assert!(stderr.is_empty(), "stderr should be empty in quiet mode");
}
