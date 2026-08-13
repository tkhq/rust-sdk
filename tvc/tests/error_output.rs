//! Process-boundary regression tests for the CLI error-output contract.
//!
//! These tests primarily exercise how upstream HTTP failures are parsed,
//! classified, and rendered. A one-shot local server supplies deterministic
//! status codes and response bodies without relying on the hosted API.
//! The server binds a loopback socket, so the test environment must permit
//! local networking. Binding to port `0` lets the OS select an available
//! ephemeral port, avoiding fixed-port collisions in parallel and CI runs.
//!
//! The suite also covers clap errors and the human/JSON stdout and stderr contract.

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::Output;
use std::thread::{self, JoinHandle};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;

const DEPLOYMENT_ID: &str = "00000000-0000-0000-0000-000000000000";
const GET_DEPLOYMENT_PATH: &str = "/public/v1/query/get_tvc_deployment";

fn tvc_command() -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("tvc");
    command.env_clear();
    command
}

// Config is built before dispatch, so the binary needs a HOME even on the
// env-auth path; the caller owns the temp dir so it outlives the command run.
fn authenticated_command(home: &TempDir, api_base_url: &str) -> assert_cmd::Command {
    let stamper = TurnkeyP256ApiKey::generate();
    let mut command = tvc_command();
    command
        .env("HOME", home.path())
        .env("TVC_ORG_ID", "org-test")
        .env(
            "TVC_API_KEY_PUBLIC",
            hex::encode(stamper.compressed_public_key()),
        )
        .env("TVC_API_KEY_PRIVATE", hex::encode(stamper.private_key()))
        .env("TVC_API_BASE_URL", api_base_url);
    command
}

fn command_output(mut command: assert_cmd::Command, expected_code: i32) -> Output {
    command.assert().code(expected_code).get_output().clone()
}

fn single_json_message(output: &Output) -> Value {
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "expected one newline-terminated NDJSON object, got {stdout:?}"
    );
    serde_json::from_str(stdout.trim_end()).expect("stdout should contain one JSON object")
}

fn spawn_json_server(
    status: u16,
    body: String,
    expected_path: &'static str,
) -> (String, JoinHandle<()>) {
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
        assert_eq!(actual_path, expected_path);

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

        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    (format!("http://{address}"), handle)
}

fn deploy_status_command(
    home: &TempDir,
    api_base_url: &str,
    message_format: Option<&str>,
) -> assert_cmd::Command {
    let mut command = authenticated_command(home, api_base_url);
    command
        .arg("deploy")
        .arg("status")
        .arg("--deploy-id")
        .arg(DEPLOYMENT_ID);
    if let Some(message_format) = message_format {
        command.arg("--message-format").arg(message_format);
    }
    command
}

#[test]
fn json_usage_errors_are_single_ndjson_objects_on_stdout() {
    let cases = [
        (
            vec!["deploy", "status", "--message-format", "json"],
            r#"error: the following required arguments were not provided:
  --deploy-id <DEPLOY_ID>

Usage: tvc deploy status --deploy-id <DEPLOY_ID> --message-format <MESSAGE_FORMAT>

For more information, try '--help'."#,
        ),
        (
            vec!["app", "list", "--message-format=json", "--bogus-flag"],
            r#"error: unexpected argument '--bogus-flag' found

Usage: tvc app list --message-format <MESSAGE_FORMAT>

For more information, try '--help'."#,
        ),
        (
            vec!["badcommand", "--message-format", "json"],
            r#"error: unrecognized subcommand 'badcommand'

Usage: tvc [OPTIONS] <COMMAND>

For more information, try '--help'."#,
        ),
    ];

    for (args, expected_message) in cases {
        let mut command = tvc_command();
        command.args(args);
        let output = command_output(command, 2);
        let message = single_json_message(&output);

        assert_eq!(
            message,
            serde_json::json!({
                "reason": "command_error",
                "code": "usage_error",
                "message": expected_message,
            })
        );
    }
}

#[test]
fn human_usage_errors_keep_clap_stderr_behavior() {
    let mut command = tvc_command();
    command.arg("deploy").arg("status");
    let output = command_output(command, 2);

    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--deploy-id"), "{stderr:?}");
    assert!(stderr.contains("Usage:"), "{stderr:?}");
    assert!(!stderr.contains(r#""code":"usage_error""#), "{stderr:?}");
}

#[test]
fn help_remains_plain_text() {
    for args in [vec!["--help"], vec!["deploy", "--help"]] {
        let mut command = tvc_command();
        command.args(args);
        let output = command_output(command, 0);

        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Usage:"), "{stdout:?}");
        assert!(!stdout.trim_start().starts_with('{'), "{stdout:?}");
    }
}

#[test]
fn http_status_errors_emit_stable_codes_status_and_full_chain() {
    let home = TempDir::new().unwrap();

    for (status, expected_code) in [
        (401, "unauthorized"),
        (403, "unauthorized"),
        (404, "not_found"),
        (500, "api_error"),
    ] {
        let server_message = format!("server failure {status}");
        let body = format!(r#"{{"code":{status},"message":"{server_message}"}}"#);
        let (api_base_url, server) = spawn_json_server(status, body, GET_DEPLOYMENT_PATH);
        let output = command_output(deploy_status_command(&home, &api_base_url, Some("json")), 1);
        server.join().unwrap();

        let message = single_json_message(&output);
        assert_eq!(message["reason"], "command_error");
        assert_eq!(message["code"], expected_code);
        assert_eq!(message["httpStatus"], status);
        let rendered = message["message"].as_str().unwrap();
        assert!(
            rendered.contains(&format!("failed to fetch deployment {DEPLOYMENT_ID}")),
            "{rendered:?}"
        );
        assert!(rendered.contains(&status.to_string()), "{rendered:?}");
        assert!(rendered.contains(&server_message), "{rendered:?}");
    }
}

#[test]
fn successful_response_with_missing_resource_has_no_http_status() {
    let home = TempDir::new().unwrap();
    let (api_base_url, server) = spawn_json_server(
        200,
        r#"{"tvcDeployment":null}"#.to_string(),
        GET_DEPLOYMENT_PATH,
    );
    let output = command_output(deploy_status_command(&home, &api_base_url, Some("json")), 1);
    server.join().unwrap();

    let message = single_json_message(&output);
    assert_eq!(message["reason"], "command_error");
    assert_eq!(message["code"], "not_found");
    assert!(message.get("httpStatus").is_none());
    assert_eq!(
        message["message"],
        format!("deployment not found: {DEPLOYMENT_ID}")
    );
}

#[test]
fn connection_failure_emits_network_error_without_http_status() {
    let home = TempDir::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let api_base_url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);

    let mut command = authenticated_command(&home, &api_base_url);
    command
        .arg("app")
        .arg("list")
        .arg("--message-format")
        .arg("json");
    let output = command_output(command, 1);
    let message = single_json_message(&output);

    assert_eq!(message["reason"], "command_error");
    assert_eq!(message["code"], "network_error");
    assert!(message.get("httpStatus").is_none());
    let rendered = message["message"].as_str().unwrap();
    assert!(rendered.contains("failed to list TVC apps"), "{rendered:?}");
    assert!(rendered.contains("HTTP request failed"), "{rendered:?}");
    assert_eq!(rendered.matches("error sending request").count(), 2);
}

#[test]
fn human_and_json_runtime_errors_render_identical_message_text() {
    let home = TempDir::new().unwrap();
    let body = r#"{"code":404,"message":"deployment absent"}"#.to_string();
    let (json_api_base_url, json_server) =
        spawn_json_server(404, body.clone(), GET_DEPLOYMENT_PATH);
    let json_output = command_output(
        deploy_status_command(&home, &json_api_base_url, Some("json")),
        1,
    );
    json_server.join().unwrap();
    let json_message = single_json_message(&json_output);

    let (human_api_base_url, human_server) = spawn_json_server(404, body, GET_DEPLOYMENT_PATH);
    let human_output = command_output(deploy_status_command(&home, &human_api_base_url, None), 1);
    human_server.join().unwrap();

    assert!(human_output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(human_output.stderr).unwrap(),
        format!(
            r#"error: {}
"#,
            json_message["message"].as_str().unwrap()
        )
    );
}

const CLIENT_VERSION_TOO_OLD_BODY: &str = r#"{"code":3,"message":"tvc 0.12.0 is older than the minimum version this backend supports (0.12.2); upgrade tvc to the latest release","details":[],"turnkeyErrorCode":"TVC_CLIENT_VERSION_TOO_OLD"}"#;

#[test]
fn client_version_rejection_emits_dedicated_code() {
    let home = TempDir::new().unwrap();
    let (api_base_url, server) = spawn_json_server(
        400,
        CLIENT_VERSION_TOO_OLD_BODY.to_string(),
        GET_DEPLOYMENT_PATH,
    );
    let output = command_output(deploy_status_command(&home, &api_base_url, Some("json")), 1);
    server.join().unwrap();

    let message = single_json_message(&output);
    assert_eq!(message["reason"], "command_error");
    assert_eq!(message["code"], "client_version_too_old");
    assert_eq!(message["httpStatus"], 400);
}

#[test]
fn client_version_rejection_renders_human_hint() {
    let home = TempDir::new().unwrap();
    let (api_base_url, server) = spawn_json_server(
        400,
        CLIENT_VERSION_TOO_OLD_BODY.to_string(),
        GET_DEPLOYMENT_PATH,
    );
    let output = command_output(deploy_status_command(&home, &api_base_url, None), 1);
    server.join().unwrap();

    assert!(output.stdout.is_empty());
    // The client-version rejection collapses the error line to a terse message;
    // the server's own text rides the unchanged `hint:` line.
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("is older than the minimum version the backend supports"),
    );
}
