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
        .stdout(predicate::str::contains("--disable-dedupe"))
        .stdout(predicate::str::contains("--poll"))
        .stdout(predicate::str::contains(
            "--poll-interval-seconds <POLL_INTERVAL_SECONDS>",
        ))
        .stdout(predicate::str::contains("--include-platform-timestamp"))
        .stdout(predicate::str::contains(
            "--recent-line-capacity <RECENT_LINE_CAPACITY>",
        ))
        .stdout(predicate::str::contains("--since-seconds <SINCE_SECONDS>"))
        .stdout(predicate::str::contains("--tail-lines <TAIL_LINES>"))
        .stdout(predicate::str::contains(
            "This should only be increased if high-volume logs still show duplicates",
        ))
        .stdout(predicate::str::contains(
            "Limit raw pod log history requested per replica",
        ))
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
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--poll")
        .arg("--poll-interval-seconds")
        .arg("3")
        .arg("--tail-lines")
        .arg("10")
        .arg("--since-seconds")
        .arg("30")
        .arg("--include-platform-timestamp")
        .arg("--disable-dedupe")
        .arg("--recent-line-capacity")
        .arg("2000")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"));
}

#[test]
fn debug_logs_rejects_negative_tail_lines_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--tail-lines")
        .arg("-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '-1'"))
        .stderr(predicate::str::contains("--tail-lines <TAIL_LINES>"))
        .stderr(predicate::str::contains("is not in 0.."));
}

#[test]
fn debug_logs_rejects_zero_poll_interval_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--poll")
        .arg("--poll-interval-seconds")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '0'"))
        .stderr(predicate::str::contains(
            "--poll-interval-seconds <POLL_INTERVAL_SECONDS>",
        ))
        .stderr(predicate::str::contains("is not in 1.."));
}

#[test]
fn debug_logs_rejects_negative_poll_interval_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--poll")
        .arg("--poll-interval-seconds")
        .arg("-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '-1'"))
        .stderr(predicate::str::contains(
            "--poll-interval-seconds <POLL_INTERVAL_SECONDS>",
        ))
        .stderr(predicate::str::contains("is not in 1.."));
}

#[test]
fn debug_logs_rejects_zero_poll_interval_without_poll_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--poll-interval-seconds")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '0'"))
        .stderr(predicate::str::contains(
            "--poll-interval-seconds <POLL_INTERVAL_SECONDS>",
        ))
        .stderr(predicate::str::contains("is not in 1.."));
}

#[test]
fn debug_logs_rejects_negative_since_seconds_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--since-seconds")
        .arg("-1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '-1'"))
        .stderr(predicate::str::contains("--since-seconds <SINCE_SECONDS>"))
        .stderr(predicate::str::contains("is not in 0.."));
}

#[test]
fn debug_logs_rejects_zero_recent_line_capacity_before_authentication() {
    let temp = TempDir::new().unwrap();

    debug_logs_cmd(&temp)
        .arg("--deploy-id")
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--recent-line-capacity")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value '0'"))
        .stderr(predicate::str::contains(
            "--recent-line-capacity <RECENT_LINE_CAPACITY>",
        ))
        .stderr(predicate::str::contains("is not in 1.."));
}
