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

const ORG_DUP: &str = "11111111-2222-4333-8444-555555555555";
const ORG_E2E: &str = "44444444-4444-4444-8444-444444444444";
const ORG_SOLO: &str = "55555555-5555-4555-8555-555555555555";
const ORG_BACKUP: &str = "66666666-6666-4666-8666-666666666666";
const ORG_OTHER: &str = "77777777-7777-4777-8777-777777777777";

fn spawn(args: &str) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} {args}");
    rexpect::spawn(&cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"))
}

/// Expect `text` while tolerating the PTY's hard wrap: output longer than the
/// terminal width gets a line break injected at an arbitrary point, so the
/// text is matched as a regex that accepts a break between any two characters.
fn exp_wrapped(session: &mut PtySession, text: &str) {
    let pattern: String = text
        .chars()
        .map(|c| {
            let escaped = if r"\.+*?()|[]{}^$".contains(c) {
                format!(r"\{c}")
            } else {
                c.to_string()
            };
            format!("{escaped}[\r\n]*")
        })
        .collect();
    session
        .exp_regex(&pattern)
        .unwrap_or_else(|e| panic!("expected (wrap-tolerant) {text:?}: {e}"));
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

/// Drive `tvc login` through the interactive new-org flow (empty config, so
/// no org selector appears) against a dead-port API base URL. Login persists
/// the profile and its generated API key before the final whoami request, so
/// the profile exists on disk even though the command exits nonzero when that
/// request fails.
fn pty_create_profile(home: &Path, org_id: &str, alias: &str) {
    let mut session = spawn_with_home(home, &["login", "--api-base-url", "http://127.0.0.1:1"]);

    session.exp_string("Organization ID").unwrap();
    session.send_line(org_id).unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line(alias).unwrap();

    session
        .exp_string(&format!("Selected org: {alias} ({org_id})"))
        .unwrap();
    session.exp_string("API Key Generated!").unwrap();
    session.exp_string("Press Enter when done...").unwrap();
    session.send_line("").unwrap();

    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();
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

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Organization ID").unwrap();
    session.send_line("").unwrap();
    session.exp_string("Organization ID is required").unwrap();
    session.exp_eof().unwrap();
}

/// Organization IDs are UUIDs; anything else is rejected at the prompt
/// boundary before any profile is created.
#[test]
fn login_with_non_uuid_org_id_bails() {
    let temp = tempfile::TempDir::new().unwrap();

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Organization ID").unwrap();
    session.send_line("not-a-uuid").unwrap();
    session
        .exp_string("Organization ID must be a UUID")
        .unwrap();
    session.exp_eof().unwrap();

    assert!(
        !temp.path().join(".config/turnkey/tvc.config.toml").exists(),
        "no config may be written for a rejected organization ID"
    );
}

/// TVC-159: interactive `tvc login` folds duplicate profiles down to one per
/// organization before proceeding: prompt for the keeper, one confirmation,
/// full profile-delete cleanup for the losers, and active-profile repair onto
/// the keeper. Duplicate profiles can no longer be created through the CLI,
/// so this seeds the legacy on-disk state directly.
#[test]
fn login_consolidates_duplicate_profiles_interactively() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-b"),
        &[],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["login"]);

    exp_wrapped(
        &mut session,
        &format!("Select the profile to keep for organization '{ORG_DUP}'"),
    );
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Permanently delete 'alias-b' and the key files on disk?")
        .unwrap();
    session.send_line("y").unwrap();

    session
        .exp_string(&format!("Deleted login profile 'alias-b' ({ORG_DUP})."))
        .unwrap();
    session.exp_string("Removed key directory").unwrap();
    session
        .exp_string("IMPORTANT: The API key may still be registered")
        .unwrap();

    // Login proceeds against the consolidated config.
    session.exp_string("Select organization").unwrap();
    session.send_line("alias-a").unwrap();
    session
        .exp_string(&format!("Selected org: alias-a ({ORG_DUP})"))
        .unwrap();
    session.exp_string("Using existing API key.").unwrap();
    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();

    assert!(!temp.path().join(".config/turnkey/orgs/alias-b").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("alias-b"));
    assert!(saved.contains(r#"active_org = "alias-a""#));
    assert!(!saved.contains("default_alias"));
}

/// Declining the consolidation confirmation cancels login and leaves both
/// profiles and their key files untouched (nothing is mutated before the
/// single consent point).
#[test]
fn login_consolidation_decline_cancels_login() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-b"),
        &[],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["login"]);

    exp_wrapped(
        &mut session,
        &format!("Select the profile to keep for organization '{ORG_DUP}'"),
    );
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Permanently delete 'alias-b' and the key files on disk?")
        .unwrap();
    session.send_line("n").unwrap();

    session
        .exp_string("operation cancelled by user: profile consolidation")
        .unwrap();
    session.exp_eof().unwrap();

    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-b").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("alias-a"));
    assert!(saved.contains("alias-b"));
}

/// `profile delete --org <org-id>` with several profiles registered for that
/// organization ID prompts for which profile to delete instead of deleting an
/// arbitrary one (TVC-159), and only the chosen profile is removed.
#[test]
fn profile_delete_with_duplicate_org_id_prompts_for_profile() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &[],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["profile", "delete", "--org", ORG_DUP]);

    session.exp_string("Select profile to delete").unwrap();
    session.send_line("alias-b").unwrap();

    session
        .exp_string(&format!("Permanently delete profile 'alias-b' ({ORG_DUP})"))
        .unwrap();
    session.send_line("y").unwrap();

    session
        .exp_string(&format!("Deleted login profile 'alias-b' ({ORG_DUP})."))
        .unwrap();
    session.exp_eof().unwrap();

    assert!(!temp.path().join(".config/turnkey/orgs/alias-b").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("alias-b"));
    assert!(saved.contains("alias-a"));
}

/// The interactive new-org flow (the only way to create a profile) works end
/// to end, keys its directory by org ID (TVC-55), and persists the profile
/// even though the final whoami request fails against the dead-port URL.
#[test]
fn login_creates_first_profile_and_persists_it() {
    let temp = tempfile::TempDir::new().unwrap();
    pty_create_profile(temp.path(), ORG_SOLO, "solo");

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("[orgs.solo]"));
    assert!(saved.contains(&format!(r#"id = "{ORG_SOLO}""#)));

    let org_dir = temp.path().join(".config/turnkey/orgs").join(ORG_SOLO);
    assert!(org_dir.join("api_key.json").exists());
    assert!(!temp.path().join(".config/turnkey/orgs/solo").exists());
}

/// Entering an organization ID that is already configured refuses to create a
/// second profile for it (one profile per organization, TVC-159) and names
/// the existing alias.
#[test]
fn login_new_org_refuses_already_configured_org_id() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_DUP)], Some("alias-a"), &[]);

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Select organization").unwrap();
    session.send_line("new").unwrap();

    session.exp_string("Organization ID").unwrap();
    session.send_line(ORG_DUP).unwrap();

    exp_wrapped(
        &mut session,
        &format!("Organization '{ORG_DUP}' is already configured as profile 'alias-a'."),
    );
    session
        .exp_string("tvc profile delete --org alias-a")
        .unwrap();
    session.exp_eof().unwrap();

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert_eq!(saved.matches(&format!(r#"id = "{ORG_DUP}""#)).count(), 1);
}

/// TVC-53: generating a fresh operator key during login offers a backup;
/// accepting prompts for a destination, writes the copy, and login still
/// succeeds. The mock whoami server carries login past its network step.
#[test]
fn login_fresh_operator_key_offers_backup() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_E2E)], Some("alias-a"), &[]);
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
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_E2E)], Some("alias-a"), &[]);
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

/// TVC-53: re-logins with an existing operator key get a single backup tip,
/// no prompts.
#[test]
fn login_existing_operator_key_prints_backup_tip() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_E2E)], Some("alias-a"), &[]);
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

/// Interactive `keys backup-operator-key` prompts for the destination and
/// reports the copy.
#[test]
fn keys_backup_operator_key_prompts_for_destination() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");

    let mut session = spawn_with_home(temp.path(), &["keys", "backup-operator-key"]);

    session.exp_string("Backup file path").unwrap();
    session.send_line(destination.to_str().unwrap()).unwrap();

    session.exp_string("Operator key backed up!").unwrap();
    session.exp_eof().unwrap();

    assert!(destination.exists());
}

/// Reusing an existing profile alias for a different organization refuses
/// instead of silently overwriting the profile.
#[test]
fn login_new_org_refuses_alias_already_in_use() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_OTHER)], Some("alias-a"), &[]);

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Select organization").unwrap();
    session.send_line("new").unwrap();

    session.exp_string("Organization ID").unwrap();
    session.send_line(ORG_SOLO).unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line("alias-a").unwrap();

    exp_wrapped(
        &mut session,
        &format!("Profile alias 'alias-a' is already in use for organization '{ORG_OTHER}'."),
    );
    session.exp_eof().unwrap();

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains(&format!(r#"id = "{ORG_OTHER}""#)));
    assert!(!saved.contains(ORG_SOLO));
}
