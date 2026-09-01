//! PTY-based integration tests.
//!
//! Drives the real `tvc` binary through a pseudo-terminal so `inquire`'s TTY
//! code path is exercised end-to-end.
//!
//! Gated `#[cfg(unix)]` because `rexpect` uses Unix PTYs; Windows users hit
//! inquire via ConPTY in production, but we don't test that surface here.

#![cfg(unix)]

mod common;

use qos_p256::P256Pair;
use rexpect::session::PtySession;
use std::collections::{BTreeSet, HashMap};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::thread::{self, JoinHandle};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::app::KNOWN_QUORUM_KEY;
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, KeyCurve, OperatorKind, OperatorRecord, OperatorRecordKind,
    OrgConfig, StoredApiKey, YubiKeyOperatorRecord, YubiKeySerial,
};

/// Default per-step timeout. Generous enough for CI-runner cold cargo builds
/// of the binary; tight enough to fail fast if an `exp_string` mismatches.
const TIMEOUT_MS: u64 = 10_000;

const ORG_HOSTED: &str = "88888888-8888-4888-8888-888888888888";

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
        .env_remove("TVC_NON_INTERACTIVE")
        .env_remove("TVC_ORG_ID")
        .env_remove("TVC_API_KEY_PUBLIC")
        .env_remove("TVC_API_KEY_PRIVATE");

    rexpect::session::spawn_command(cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {bin} {}", args.join(" ")))
}

/// Drive `tvc login` through the interactive new-org flow against a dead-port
/// API base URL. Login persists the profile and its generated API key before
/// the final whoami request, so the profile exists on disk even though the
/// command exits nonzero when that request fails.
///
/// `pick_new_in_selector` is required once profiles exist: login then opens an
/// org selector first, and typing "new" filters it down to the
/// "[new] Add a new organization" entry (no fixture string here contains
/// "new") which Enter selects.
fn pty_create_profile(home: &Path, org_id: &str, alias: &str, pick_new_in_selector: bool) {
    let mut session = spawn_with_home(home, &["login", "--api-base-url", "http://127.0.0.1:1"]);

    if pick_new_in_selector {
        session.exp_string("Select organization").unwrap();
        session.send_line("new").unwrap();
    }

    session.exp_string("Organization ID").unwrap();
    session.send_line(org_id).unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line(alias).unwrap();

    // Enter accepts the highlighted "Local key file" entry.
    session.exp_string("Operator key type").unwrap();
    session.send_line("").unwrap();

    session
        .exp_string(&format!("Selected org: {alias} ({org_id})"))
        .unwrap();
    session.exp_string("API Key Generated!").unwrap();
    session.exp_string("Press Enter when done...").unwrap();
    session.send_line("").unwrap();

    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();
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

fn answer_confirmation(session: &mut PtySession, question: &str, answer: &str) {
    session.exp_string(question).unwrap();
    session.exp_string("(y/N)").unwrap();
    session.send_line(answer).unwrap();
}

/// `tvc deploy approve` walks all six section confirmations in order and
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
    session.exp_string("MANIFEST SCHEMA").unwrap();
    session.exp_string("Version:       v1 (legacy)").unwrap();
    answer_confirmation(&mut session, "Approve manifest schema and DNS?", "y");

    session.exp_string("NAMESPACE").unwrap();
    session.exp_string("turnkey-prod").unwrap();
    answer_confirmation(&mut session, "Approve namespace?", "y");

    session.exp_string("ENCLAVE (AWS Nitro)").unwrap();
    answer_confirmation(&mut session, "Approve enclave configuration?", "y");

    session.exp_string("PIVOT BINARY").unwrap();
    session.exp_string("Restart Policy: Never").unwrap();
    session.exp_string("Debug Mode: disabled").unwrap();
    answer_confirmation(&mut session, "Approve pivot binary?", "y");

    session.exp_string("MANIFEST SET").unwrap();
    session.exp_string("operator-alice").unwrap();
    answer_confirmation(&mut session, "Approve manifest set?", "y");

    session.exp_string("SHARE SET").unwrap();
    answer_confirmation(&mut session, "Approve share set?", "y");

    session.exp_string("ALL SECTIONS APPROVED").unwrap();
    session.exp_string(r#""signature""#).unwrap();
    session.exp_eof().unwrap();
}

/// Rejecting at the fourth section (pivot) bails immediately with the exact
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

    answer_confirmation(&mut session, "Approve manifest schema and DNS?", "y");
    answer_confirmation(&mut session, "Approve namespace?", "y");
    answer_confirmation(&mut session, "Approve enclave configuration?", "y");
    answer_confirmation(&mut session, "Approve pivot binary?", "n");

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

/// TVC-159: when one organization ID is registered under multiple aliases,
/// `login --org <org-id>` resolves to an arbitrary alias, because resolution
/// falls back to `HashMap` iteration order, which is randomized per process.
///
/// The duplicate state is created through the CLI itself (two logins to the
/// same organization under different aliases), then resolution runs in fresh
/// processes until both aliases have been observed. With the bug present the
/// expected number of attempts is 2-3; all 40 agreeing has probability 2⁻³⁹.
#[test]
fn login_with_duplicate_org_id_selects_alias_nondeterministically() {
    let temp = tempfile::TempDir::new().unwrap();
    pty_create_profile(temp.path(), "org-dup-test", "alias-a", false);
    pty_create_profile(temp.path(), "org-dup-test", "alias-b", true);

    let bin = env!("CARGO_BIN_EXE_tvc");
    let mut seen = BTreeSet::new();

    for _ in 0..40 {
        let output = Command::new(bin)
            .args(["login", "--org", "org-dup-test"])
            .env("HOME", temp.path())
            .env("TVC_NON_INTERACTIVE", "1")
            .env_remove("TVC_ORG")
            .env_remove("TVC_API_BASE_URL")
            .env_remove("TVC_ORG_ID")
            .env_remove("TVC_API_KEY_PUBLIC")
            .env_remove("TVC_API_KEY_PRIVATE")
            .output()
            .unwrap();

        // Selection happens (and the config is saved) before the whoami
        // request, which fails against the dead-port base URL on the profile.
        assert!(!output.status.success());

        let stdout = String::from_utf8(output.stdout).unwrap();
        let alias = stdout
            .lines()
            .find_map(|line| line.strip_prefix("Selected org: "))
            .and_then(|rest| rest.strip_suffix(" (org-dup-test)"))
            .unwrap_or_else(|| panic!("no selected-org line in output:\n{stdout}"));
        seen.insert(alias.to_string());

        if seen.len() == 2 {
            break;
        }
    }

    let expected = ["alias-a", "alias-b"]
        .into_iter()
        .map(String::from)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        seen, expected,
        "resolution by org ID should be nondeterministic across processes (TVC-159)"
    );
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

/// Write a v1 config whose active org holds one hosted operator plus an
/// optional additional hosted identity for each ID passed in. All identities
/// use the same real composite public key, so each is a proven reuse
/// candidate; the extra IDs are also saved as last-app IDs to exercise
/// deduplication. Returns the composite.
fn write_hosted_org_config(home: &Path, saved_operator_ids: &[&str]) -> String {
    let turnkey_dir = home.join(".config/turnkey");
    std::fs::create_dir_all(&turnkey_dir).unwrap();

    let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
    let (encrypt_public_key, sign_public_key) = composite.split_at(composite.len() / 2);

    let mut operators = vec![OperatorRecord {
        name: "hosted-op".to_string(),
        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
            operator_id: "11111111-1111-4111-8111-111111111111".parse().unwrap(),
            wallet_id: "22222222-2222-4222-8222-222222222222".parse().unwrap(),
            path: "m/5527107'/0'/0'".to_string(),
            encrypt_public_key: encrypt_public_key.to_string(),
            sign_public_key: sign_public_key.to_string(),
            extra: toml::Table::new(),
        }),
    }];
    operators.extend(
        saved_operator_ids
            .iter()
            .enumerate()
            .map(|(index, id)| OperatorRecord {
                name: format!("hosted-op-{}", index + 2),
                kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                    operator_id: id.parse().unwrap(),
                    wallet_id: "55555555-5555-4555-8555-555555555555".parse().unwrap(),
                    path: "m/5527107'/0'/0'".to_string(),
                    encrypt_public_key: encrypt_public_key.to_string(),
                    sign_public_key: sign_public_key.to_string(),
                    extra: toml::Table::new(),
                }),
            }),
    );

    let config = Config {
        active_org: Some("hosted-org".to_string()),
        orgs: HashMap::from([(
            "hosted-org".to_string(),
            OrgConfig {
                id: ORG_HOSTED.to_string(),
                api_key_path: turnkey_dir.join("orgs/hosted-org/api_key.json"),
                api_base_url: common::LOCAL_API_BASE_URL.to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators,
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: Default::default(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::from([(
            "hosted-org".to_string(),
            saved_operator_ids.iter().map(|id| id.to_string()).collect(),
        )]),
        extra: toml::Table::new(),
    };
    std::fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    composite
}

/// The explicit minting path for an org whose operator is hosted: with
/// `--no-operator-reuse`, `app create` offers the registered hosted
/// operator's public key as the fill default, exactly as it offers a local
/// key file's key. Both are the same qos composite — the local file stores
/// encrypt ‖ sign concatenated, and the hosted registry record stores the
/// two points separately — so the org's one operator must be one Enter away
/// in both worlds. (Without the flag, the default flow reuses the registered
/// identity by ID instead of minting; see the sibling test below.)
///
/// The command exits nonzero after the fill — creating the app needs
/// credentials this fixture doesn't have — but that is outside this test's
/// scope.
#[test]
fn app_create_offers_the_hosted_operator_key_as_the_fill_default() {
    let temp = tempfile::TempDir::new().unwrap();
    let composite = write_hosted_org_config(temp.path(), &[]);

    // An app config whose only placeholder is the operator public key.
    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "operator-1",
            "publicKey": "<FILL_IN_OPERATOR_PUBLIC_KEY>"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    let mut session = spawn_with_home(
        temp.path(),
        &[
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
            "--no-operator-reuse",
        ],
    );

    // The prompt offers the hosted operator's composite key as its default;
    // Enter accepts it.
    session
        .exp_string("Operator 'operator-1' public key")
        .unwrap();
    exp_wrapped(&mut session, &composite);
    session.send_line("").unwrap();

    session.exp_string("Save filled config").unwrap();
    session.send_line("y").unwrap();
    exp_wrapped(&mut session, "Wrote ");
    session.exp_eof().unwrap();

    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&app_config_path).unwrap()).unwrap();
    assert_eq!(
        saved["manifestSetParams"]["newOperators"][0]["publicKey"], composite,
        "accepting the default must land the hosted operator's key in the saved config"
    );
}

/// The default flow for an org with a registered hosted operator reuses that
/// identity via `existingOperatorIds` instead of minting a duplicate operator
/// around the same key material.
///
/// The command exits nonzero afterwards — creating the app needs credentials
/// this fixture doesn't have — but the reuse announcement precedes that.
#[test]
fn app_create_reuses_the_registered_hosted_operator_by_default() {
    let temp = tempfile::TempDir::new().unwrap();
    let composite = write_hosted_org_config(temp.path(), &[]);

    // A complete app config: nothing to fill, so the run goes straight to
    // the reuse decision.
    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "operator-1",
            "publicKey": "{composite}"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    let mut session = spawn_with_home(
        temp.path(),
        &[
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
        ],
    );

    exp_wrapped(
        &mut session,
        "Reusing operator hosted-op (11111111-1111-4111-8111-111111111111)",
    );
    session.exp_eof().unwrap();
}

/// With several registered identities proven to use the requested key, the
/// default flow prompts to pick one, and the chosen candidate (here the
/// second, selected with a down-arrow) is what gets reused.
///
/// The command exits nonzero afterwards — creating the app needs credentials
/// this fixture doesn't have — but the selection and reuse announcement
/// precede that.
#[test]
fn app_create_prompts_to_pick_among_multiple_reuse_candidates() {
    let temp = tempfile::TempDir::new().unwrap();
    let composite = write_hosted_org_config(temp.path(), &["33333333-3333-4333-8333-333333333333"]);

    // A complete app config: nothing to fill, so the run goes straight to
    // the reuse decision.
    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "operator-1",
            "publicKey": "{composite}"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    let mut session = spawn_with_home(
        temp.path(),
        &[
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
        ],
    );

    // Both matching registered identities are offered. The duplicate saved
    // ID is deduplicated against the second registry record.
    session.exp_string("Select operator to reuse").unwrap();
    exp_wrapped(
        &mut session,
        "hosted-op (11111111-1111-4111-8111-111111111111)",
    );
    exp_wrapped(
        &mut session,
        "hosted-op-2 (33333333-3333-4333-8333-333333333333)",
    );

    // Down-arrow to the saved ID, Enter to confirm.
    session.send("\x1b[B").unwrap();
    session.send_line("").unwrap();

    exp_wrapped(
        &mut session,
        "Reusing operator hosted-op-2 (33333333-3333-4333-8333-333333333333)",
    );
    session.exp_eof().unwrap();
}

/// Write a v1 config holding only a YubiKey registry entry (no orgs), so an
/// interactive login goes straight to new-organization setup with a
/// registered serial to offer. Returns the cached composite key hex.
fn write_registry_only_config(home: &Path) -> String {
    let turnkey_dir = home.join(".config/turnkey");
    std::fs::create_dir_all(&turnkey_dir).unwrap();

    let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
    let mut config = Config::default();
    config
        .yubikeys
        .register(YubiKeySerial::from(0x01c9_5c1f), composite.parse().unwrap());
    std::fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    composite
}

/// A new organization can be set up with an already-registered YubiKey as
/// its operator, entirely from the registry cache: no device is needed,
/// no device operation occurs, and the saved config defaults to the yubikey
/// backend with a serial-only operator record.
#[test]
fn login_creates_a_new_org_with_the_explicit_registered_yubikey() {
    let temp = tempfile::TempDir::new().unwrap();
    let composite = write_registry_only_config(temp.path());
    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &[
            "login",
            "--api-base-url",
            &api_base_url,
            "--serial",
            "01c95c1f",
        ],
    );

    session.exp_string("No organization configured.").unwrap();
    session.exp_string("Organization ID").unwrap();
    session.send_line("org-e2e").unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line("").unwrap();

    // Down-arrow from "Local key file" to "YubiKey". The explicit serial
    // resolves the registered source without another selection prompt.
    session.exp_string("Operator key type").unwrap();
    session.send("\x1b[B").unwrap();
    session.send_line("").unwrap();

    session.exp_string("Operator public key:").unwrap();
    session
        .exp_string("Make sure to register this as an operator in your organization.")
        .unwrap();

    // API key generation, manual dashboard registration, then verification
    // against the mock server.
    session.exp_string("API Key Generated!").unwrap();
    session.exp_string("Press Enter when done...").unwrap();
    session.send_line("").unwrap();

    exp_wrapped(
        &mut session,
        "Using YubiKey operator 'yubikey-01c95c1f' (serial 01c95c1f).",
    );
    let output = session.exp_eof().unwrap();
    server.join().unwrap();

    assert!(output.contains("Successfully logged in!"), "{output}");
    assert!(!output.contains("Generating operator key"), "{output}");
    assert!(
        !output.contains("YubiKey to use as the operator"),
        "{output}"
    );
    assert!(output.contains("YubiKey operator:"), "{output}");

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(
        saved.contains("default_operator_kind = \"yubikey\""),
        "{saved}"
    );
    assert!(saved.contains("name = \"yubikey-01c95c1f\""), "{saved}");
    assert!(saved.contains(&composite), "{saved}");
}

/// A yubikey-default org with several YubiKey operators makes an interactive
/// login select one; the record prompt is a config-level choice served
/// entirely from the registry cache, so no device is needed.
#[test]
fn login_selects_among_multiple_yubikey_operators() {
    let temp = tempfile::TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    std::fs::create_dir_all(&turnkey_dir).unwrap();

    let yubikey_record = |name: &str, serial: u32| OperatorRecord {
        name: name.to_string(),
        kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
            serial: YubiKeySerial::from(serial),
            extra: toml::Table::new(),
        }),
    };
    let registry_key = || {
        hex::encode(P256Pair::generate().unwrap().public_key().to_bytes())
            .parse()
            .unwrap()
    };
    let mut config = Config {
        active_org: Some("yk-org".to_string()),
        orgs: HashMap::from([(
            "yk-org".to_string(),
            OrgConfig {
                id: "org-e2e".to_string(),
                api_key_path: turnkey_dir.join("orgs/yk-org/api_key.json"),
                api_base_url: common::LOCAL_API_BASE_URL.to_string(),
                default_operator_kind: OperatorKind::Yubikey,
                operators: vec![
                    yubikey_record("first", 0x01c9_5c1f),
                    yubikey_record("second", 0xdead_beef),
                ],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: Default::default(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };
    config
        .yubikeys
        .register(YubiKeySerial::from(0x01c9_5c1f), registry_key());
    config
        .yubikeys
        .register(YubiKeySerial::from(0xdead_beef), registry_key());
    std::fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    // An existing API key so login skips generation and goes straight to
    // verification.
    let api_key_dir = temp.path().join(".config/turnkey/orgs/yk-org");
    std::fs::create_dir_all(&api_key_dir).unwrap();
    let stamper = TurnkeyP256ApiKey::generate();
    let api_key = StoredApiKey {
        public_key: hex::encode(stamper.compressed_public_key()),
        private_key: hex::encode(stamper.private_key()),
        curve: KeyCurve::P256,
    };
    std::fs::write(
        api_key_dir.join("api_key.json"),
        serde_json::to_string_pretty(&api_key).unwrap(),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &["login", "--org", "yk-org", "--api-base-url", &api_base_url],
    );

    session.exp_string("Select YubiKey operator").unwrap();
    exp_wrapped(&mut session, "first (serial 01c95c1f)");
    exp_wrapped(&mut session, "second (serial deadbeef)");

    // Down-arrow to the second record, Enter to confirm.
    session.send("\x1b[B").unwrap();
    session.send_line("").unwrap();

    exp_wrapped(
        &mut session,
        "Using YubiKey operator 'second' (serial deadbeef).",
    );
    let output = session.exp_eof().unwrap();
    server.join().unwrap();

    assert!(output.contains("Successfully logged in!"), "{output}");
}

/// Logging in to an org whose default backend is hosted needs no local key:
/// login reports the registered hosted operator and generates nothing.
#[test]
fn login_reports_the_hosted_operator_for_a_hosted_default_org() {
    let temp = tempfile::TempDir::new().unwrap();
    write_hosted_org_config(temp.path(), &[]);

    // An existing API key so login skips generation and goes straight to
    // verification.
    let api_key_dir = temp.path().join(".config/turnkey/orgs/hosted-org");
    std::fs::create_dir_all(&api_key_dir).unwrap();
    let stamper = TurnkeyP256ApiKey::generate();
    let api_key = StoredApiKey {
        public_key: hex::encode(stamper.compressed_public_key()),
        private_key: hex::encode(stamper.private_key()),
        curve: KeyCurve::P256,
    };
    std::fs::write(
        api_key_dir.join("api_key.json"),
        serde_json::to_string_pretty(&api_key).unwrap(),
    )
    .unwrap();

    let (api_base_url, server) = spawn_whoami_server();

    let mut session = spawn_with_home(
        temp.path(),
        &[
            "login",
            "--org",
            "hosted-org",
            "--api-base-url",
            &api_base_url,
        ],
    );

    session.exp_string("Using existing API key.").unwrap();
    exp_wrapped(
        &mut session,
        "Using hosted operator 'hosted-op' (11111111-1111-4111-8111-111111111111).",
    );
    let output = session.exp_eof().unwrap();
    server.join().unwrap();

    assert!(output.contains("Successfully logged in!"), "{output}");
    assert!(!output.contains("Generating operator key"), "{output}");
    assert!(output.contains("Hosted operator:"), "{output}");
}
