//! PTY-based integration tests.
//!
//! Drives the real `tvc` binary through a pseudo-terminal so `inquire`'s TTY
//! code path is exercised end-to-end.
//!
//! Gated `#[cfg(unix)]` because `rexpect` uses Unix PTYs; Windows users hit
//! inquire via ConPTY in production, but we don't test that surface here.

#![cfg(unix)]

mod common;

use rexpect::session::PtySession;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::thread::{self, JoinHandle};

/// Default per-step timeout. Generous enough for CI-runner cold cargo builds
/// of the binary; tight enough to fail fast if an `exp_string` mismatches.
const TIMEOUT_MS: u64 = 10_000;

fn spawn(args: &str) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} {args}");
    rexpect::spawn(&cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"))
}

/// Spawn the binary in a PTY with `HOME` pointed at an isolated directory and
/// ambient `TVC_*` variables scrubbed so developer shells can't leak into the
/// prompts under test.
fn spawn_with_home(home: &Path, args: &[&str]) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");

    let mut cmd = Command::new(bin);
    cmd.args(args)
        .env("HOME", home)
        .env_remove("TVC_ORG")
        .env_remove("TVC_API_BASE_URL")
        .env_remove("TVC_NON_INTERACTIVE");

    rexpect::session::spawn_command(cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {bin} {}", args.join(" ")))
}

/// One-shot mock Turnkey API that answers the whoami query, enough to carry
/// login past its credential verification and into the operator-key flow.
/// Same shape as `tests/error_output.rs::spawn_json_server`.
fn spawn_whoami_server() -> (String, JoinHandle<()>) {
    // Port 0 requests an available ephemeral port instead of sharing a fixed
    // test port that could collide with another process or parallel test.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        let actual_path = request_line
            .split_whitespace()
            .nth(1)
            .expect("request line should contain a path");
        assert_eq!(actual_path, "/public/v1/query/whoami");

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
        drop(reader);

        let body = r#"{"organizationId":"org-e2e","organizationName":"E2E Org","userId":"user-1","username":"e2e"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    (format!("http://{address}"), handle)
}

/// `tvc deploy approve` walks all five section confirmations in order and
/// emits the signed approval JSON when the user accepts every section.
///
/// Replaces the deleted `tests/deploy_approve.rs::approve_interactive_prompts`
/// integration test which used piped stdin.
#[test]
fn approve_walks_all_sections_with_yeses() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed-path fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("MANIFEST APPROVAL").unwrap();
    session.exp_string("NAMESPACE").unwrap();
    session.exp_string("turnkey-prod").unwrap();
    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ENCLAVE (AWS Nitro)").unwrap();
    session
        .exp_string("Approve enclave configuration?")
        .unwrap();
    session.send_line("y").unwrap();

    session.exp_string("PIVOT BINARY").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("MANIFEST SET").unwrap();
    session.exp_string("operator-alice").unwrap();
    session.exp_string("Approve manifest set?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("SHARE SET").unwrap();
    session.exp_string("Approve share set?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ALL SECTIONS APPROVED").unwrap();
    session.exp_string(r#""signature""#).unwrap();
    session.exp_eof().unwrap();
}

/// Rejecting at the third section (pivot) bails immediately with the exact
/// "operation cancelled by user: approval" string and never reaches the
/// manifest-set section.
#[test]
fn approve_bails_when_user_rejects_pivot() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed-path fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();
    session
        .exp_string("Approve enclave configuration?")
        .unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("n").unwrap();

    session
        .exp_string("operation cancelled by user: approval")
        .unwrap();
    session.exp_eof().unwrap();
}

/// Submitting an empty Organization ID at the new-org prompt errors with the
/// exact bail string.
///
/// Replaces the deleted `tests/login.rs::login_empty_org_id_fails`.
#[test]
fn login_with_empty_org_id_bails() {
    let temp = tempfile::TempDir::new().unwrap();

    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} login");

    let mut session = rexpect::session::spawn_command(
        {
            let mut c = std::process::Command::new(bin);
            c.arg("login").env("HOME", temp.path());
            c
        },
        Some(TIMEOUT_MS),
    )
    .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"));

    session.exp_string("Organization ID").unwrap();
    session.send_line("").unwrap();
    session.exp_string("Organization ID is required").unwrap();
    session.exp_eof().unwrap();
}

/// Interactive `keys backup-operator-key` prompts for the destination and
/// reports the copy.
#[test]
fn keys_backup_operator_key_prompts_for_destination() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");

    let mut session = spawn_with_home(temp.path(), &["keys", "backup-operator-key"]);

    session.exp_string("Backup file path").unwrap();
    session.send_line(destination.to_str().unwrap()).unwrap();

    session.exp_string("Operator key backed up!").unwrap();
    session.exp_eof().unwrap();

    assert!(destination.exists());
}

/// TVC-53: generating a fresh operator key during login offers a backup;
/// accepting prompts for a destination, writes the copy, and login still
/// succeeds. The mock whoami server carries login past its network step.
#[test]
fn login_fresh_operator_key_offers_backup() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-e2e")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    std::fs::remove_file(
        temp.path()
            .join(".config/turnkey/orgs/alias-a/operator.json"),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();
    let destination = temp.path().join("operator-backup.json");

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "alias-a", "--api-base-url", &api_base_url],
    );

    session.exp_string("Verifying credentials...").unwrap();
    session.exp_string("Operator Key Generated!").unwrap();
    session
        .exp_string("Back up your operator key now?")
        .unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Backup file path").unwrap();
    session.send_line(destination.to_str().unwrap()).unwrap();

    session.exp_string("Operator key backed up!").unwrap();
    session.exp_string("Successfully logged in!").unwrap();
    session.exp_eof().unwrap();
    server.join().unwrap();

    assert_eq!(
        std::fs::read(&destination).unwrap(),
        std::fs::read(
            temp.path()
                .join(".config/turnkey/orgs/alias-a/operator.json")
        )
        .unwrap()
    );
}

/// TVC-53: declining the backup nudge points at the standalone command and
/// login still succeeds.
#[test]
fn login_backup_decline_points_at_command() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-e2e")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    std::fs::remove_file(
        temp.path()
            .join(".config/turnkey/orgs/alias-a/operator.json"),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "alias-a", "--api-base-url", &api_base_url],
    );

    session
        .exp_string("Back up your operator key now?")
        .unwrap();
    session.send_line("n").unwrap();

    session
        .exp_string("You can back up any time with `tvc keys backup-operator-key`.")
        .unwrap();
    session.exp_string("Successfully logged in!").unwrap();
    session.exp_eof().unwrap();
    server.join().unwrap();
}

/// Cancelling the backup confirm prompt (Ctrl-D; Ctrl-C and Esc reach the
/// same `InquireError` path) degrades to a warning: the config and key files
/// are already saved by the time the nudge runs, so login must still succeed.
#[test]
fn login_backup_confirm_cancel_still_succeeds() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-e2e")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    std::fs::remove_file(
        temp.path()
            .join(".config/turnkey/orgs/alias-a/operator.json"),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "alias-a", "--api-base-url", &api_base_url],
    );

    session
        .exp_string("Back up your operator key now?")
        .unwrap();
    session.send_control('d').unwrap();

    session.exp_string("WARNING: backup skipped:").unwrap();
    session.exp_string("Successfully logged in!").unwrap();
    session.exp_eof().unwrap();
    server.join().unwrap();
}

/// Cancelling at the destination prompt has the same contract as cancelling
/// at the confirm: warning, then a successful login.
#[test]
fn login_backup_destination_cancel_still_succeeds() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-e2e")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    std::fs::remove_file(
        temp.path()
            .join(".config/turnkey/orgs/alias-a/operator.json"),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "alias-a", "--api-base-url", &api_base_url],
    );

    session
        .exp_string("Back up your operator key now?")
        .unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Backup file path").unwrap();
    session.send_control('d').unwrap();

    session.exp_string("WARNING: backup skipped:").unwrap();
    session.exp_string("Successfully logged in!").unwrap();
    session.exp_eof().unwrap();
    server.join().unwrap();
}

/// TVC-53: re-logins with an existing operator key get a single backup tip,
/// no prompts.
#[test]
fn login_existing_operator_key_prints_backup_tip() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-e2e")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "alias-a", "--api-base-url", &api_base_url],
    );

    session.exp_string("Using existing operator key.").unwrap();
    session
        .exp_string("Tip: back it up with `tvc keys backup-operator-key`.")
        .unwrap();
    session.exp_string("Successfully logged in!").unwrap();
    session.exp_eof().unwrap();
    server.join().unwrap();
}
