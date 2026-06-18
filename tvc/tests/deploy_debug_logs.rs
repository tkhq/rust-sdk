use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn debug_logs_cmd(home: &TempDir) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .env("HOME", home.path())
        .arg("deploy")
        .arg("debug-logs");
    cmd
}

#[test]
fn deploy_help_lists_debug_logs_subcommand() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("debug-logs"))
        .stdout(predicate::str::contains(
            "Fetch debug logs for a deployment",
        ));
}

#[test]
fn debug_logs_help_lists_expected_flags() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("--include-platform-timestamp"))
        .stdout(predicate::str::contains("--since-seconds <SINCE_SECONDS>"))
        .stdout(predicate::str::contains("--tail-lines <TAIL_LINES>"))
        .stdout(predicate::str::contains(
            "--dangerous-enable-debug-mode-deployments",
        ))
        .stdout(predicate::str::contains("--dangerous-deploy-debug-mode"));
}

#[test]
fn debug_logs_requires_deploy_id() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn debug_logs_accepts_flags_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("deploy-123")
        .arg("--follow")
        .arg("--tail-lines")
        .arg("10")
        .arg("--since-seconds")
        .arg("30")
        .arg("--include-platform-timestamp")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"));
}

#[test]
fn debug_logs_rejects_negative_tail_lines_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("deploy-123")
        .arg("--tail-lines")
        .arg("-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--tail-lines must be greater than or equal to 0",
        ));
}

#[test]
fn debug_logs_rejects_negative_since_seconds_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("deploy-123")
        .arg("--since-seconds")
        .arg("-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--since-seconds must be greater than or equal to 0",
        ));
}
