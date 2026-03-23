use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::tempdir;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_auth::ssh;
use turnkey_auth::ssh::protocol;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TURNKEY_TEST_PUBLIC_KEY: [u8; 32] = [0x66; 32];
const TURNKEY_TEST_SIGNATURE: [u8; 64] = [0x22; 64];
const TURNKEY_TEST_V: &str = "00";
const CLIENT_RESPONSE_TIMEOUT: Duration = Duration::from_millis(600);
const HELD_CONNECTION_DURATION: Duration = Duration::from_millis(800);
const OVERSIZED_FRAME_LENGTH: usize = 1 << 20;

#[tokio::test]
async fn ssh_agent_lists_identity_and_signs_for_configured_key() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");
    let public_key_blob = public_key_blob(&TURNKEY_TEST_PUBLIC_KEY);

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;
    mount_sign_raw_payload_mock(&server, &TURNKEY_TEST_SIGNATURE).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    let identities = exchange_frame(
        &socket_path,
        &protocol::encode_agent_frame(protocol::SSH_AGENTC_REQUEST_IDENTITIES, &[]),
    )
    .await;
    assert_eq!(identities[4], protocol::SSH_AGENT_IDENTITIES_ANSWER);
    assert_eq!(
        identities,
        protocol::encode_request_identities_response(&public_key_blob).unwrap()
    );

    let challenge = b"ssh-agent-challenge";
    let sign_response =
        exchange_frame(&socket_path, &sign_request(&public_key_blob, challenge)).await;
    assert_eq!(sign_response[4], protocol::SSH_AGENT_SIGN_RESPONSE);
    assert_eq!(
        sign_response,
        protocol::encode_sign_response(&TURNKEY_TEST_SIGNATURE).unwrap()
    );

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 2);
    let sign_request_body: serde_json::Value = requests[1]
        .body_json()
        .expect("sign request body should be valid JSON");
    assert_eq!(
        sign_request_body["parameters"]["payload"],
        hex::encode(challenge)
    );
    assert_eq!(
        sign_request_body["parameters"]["encoding"],
        "PAYLOAD_ENCODING_HEXADECIMAL"
    );
    assert_eq!(
        sign_request_body["parameters"]["hashFunction"],
        "HASH_FUNCTION_NOT_APPLICABLE"
    );
}

#[tokio::test]
async fn ssh_agent_rejects_other_keys_and_unsupported_messages() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");
    let other_public_key_blob = public_key_blob(&[0x11; 32]);

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    let sign_failure = exchange_frame(
        &socket_path,
        &sign_request(&other_public_key_blob, b"ssh-agent-challenge"),
    )
    .await;
    assert_eq!(
        sign_failure,
        protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
    );

    let unsupported = exchange_frame(&socket_path, &protocol::encode_agent_frame(99, &[])).await;
    assert_eq!(
        unsupported,
        protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
    );

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn ssh_agent_contains_malformed_clients_and_keeps_serving() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");
    let public_key_blob = public_key_blob(&TURNKEY_TEST_PUBLIC_KEY);

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    send_partial_frame_then_disconnect(&socket_path).await;
    sleep(Duration::from_millis(150)).await;
    child
        .assert_running("ssh-agent should survive malformed clients")
        .await;

    let identities = exchange_frame(
        &socket_path,
        &protocol::encode_agent_frame(protocol::SSH_AGENTC_REQUEST_IDENTITIES, &[]),
    )
    .await;
    assert_eq!(
        identities,
        protocol::encode_request_identities_response(&public_key_blob).unwrap()
    );

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn ssh_agent_rejects_oversized_frames_and_keeps_serving() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");
    let public_key_blob = public_key_blob(&TURNKEY_TEST_PUBLIC_KEY);

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    let oversized_socket_path = socket_path.clone();
    let oversized_client = tokio::spawn(async move {
        let mut stream = UnixStream::connect(&oversized_socket_path)
            .await
            .expect("ssh-agent socket should accept");
        stream
            .write_all(&((OVERSIZED_FRAME_LENGTH as u32).to_be_bytes()))
            .await
            .expect("oversized frame header should write");
        sleep(HELD_CONNECTION_DURATION).await;
    });

    sleep(Duration::from_millis(300)).await;

    let identities = recv_frame_result(
        spawn_frame_request(
            socket_path.clone(),
            protocol::encode_agent_frame(protocol::SSH_AGENTC_REQUEST_IDENTITIES, &[]),
        ),
        CLIENT_RESPONSE_TIMEOUT,
    )
    .await;
    assert_eq!(
        identities,
        protocol::encode_request_identities_response(&public_key_blob).unwrap()
    );

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 1);

    oversized_client
        .await
        .expect("oversized client thread should finish");
}

#[tokio::test]
async fn ssh_agent_times_out_stalled_clients_and_keeps_serving() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");
    let public_key_blob = public_key_blob(&TURNKEY_TEST_PUBLIC_KEY);

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    let stalled_socket_path = socket_path.clone();
    let stalled_client = tokio::spawn(async move {
        let mut stream = UnixStream::connect(&stalled_socket_path)
            .await
            .expect("ssh-agent socket should accept");
        stream
            .write_all(&[0, 0, 0, 8, protocol::SSH_AGENTC_SIGN_REQUEST, 0, 0])
            .await
            .expect("partial frame should write");
        sleep(HELD_CONNECTION_DURATION).await;
    });

    sleep(Duration::from_millis(300)).await;

    let identities = recv_frame_result(
        spawn_frame_request(
            socket_path.clone(),
            protocol::encode_agent_frame(protocol::SSH_AGENTC_REQUEST_IDENTITIES, &[]),
        ),
        CLIENT_RESPONSE_TIMEOUT,
    )
    .await;
    assert_eq!(
        identities,
        protocol::encode_request_identities_response(&public_key_blob).unwrap()
    );

    let requests = server
        .received_requests()
        .await
        .expect("request recording should be enabled");
    assert_eq!(requests.len(), 1);

    stalled_client
        .await
        .expect("stalled client thread should finish");
}

#[tokio::test]
async fn ssh_agent_exits_on_sigterm_and_removes_socket() {
    let temp = tempdir().expect("temp dir should exist");
    let socket_path = temp.path().join("auth.sock");

    let server = MockServer::start().await;
    mount_get_private_key_mock(&server, &hex::encode(TURNKEY_TEST_PUBLIC_KEY)).await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut child = spawn_auth_ssh_agent(&socket_path, &server, &api_key);
    wait_for_socket(&socket_path, &mut child).await;

    let status = Command::new("kill")
        .arg("-TERM")
        .arg(child.pid().to_string())
        .status()
        .await
        .expect("kill should run");
    assert!(status.success());

    let exit_status = child.wait_for_exit().await;
    assert!(exit_status.success(), "ssh-agent should exit cleanly");
    assert!(
        !fs::try_exists(&socket_path)
            .await
            .expect("socket path should be readable"),
        "socket should be removed on shutdown"
    );
}

fn public_key_blob(public_key: &[u8]) -> Vec<u8> {
    let public_key_line =
        ssh::encode_public_key_line(public_key, None).expect("public key line should encode");
    ssh::parse_public_key_line(&public_key_line)
        .expect("public key should parse")
        .public_key_blob
}

fn sign_request(public_key_blob: &[u8], challenge: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    encode_ssh_string(public_key_blob, &mut payload);
    encode_ssh_string(challenge, &mut payload);
    payload.extend_from_slice(&0u32.to_be_bytes());
    protocol::encode_agent_frame(protocol::SSH_AGENTC_SIGN_REQUEST, &payload)
}

fn encode_ssh_string(bytes: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    output.extend_from_slice(bytes);
}

fn spawn_auth_ssh_agent(
    socket_path: &Path,
    server: &MockServer,
    api_key: &TurnkeyP256ApiKey,
) -> ChildGuard {
    let child = Command::new(env!("CARGO_BIN_EXE_auth"))
        .arg("ssh-agent")
        .arg("--socket")
        .arg(socket_path)
        .env("TURNKEY_ORGANIZATION_ID", "org-id")
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(api_key.compressed_public_key()),
        )
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            hex::encode(api_key.private_key()),
        )
        .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
        .env("TURNKEY_API_BASE_URL", server.uri())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("auth ssh-agent should spawn");

    ChildGuard { child }
}

async fn wait_for_socket(socket_path: &Path, child: &mut ChildGuard) {
    for _ in 0..100 {
        if fs::try_exists(socket_path)
            .await
            .expect("socket path should be readable")
        {
            return;
        }

        if let Some(status) = child
            .child
            .try_wait()
            .expect("child status should be readable")
        {
            let stderr = child.read_stderr().await;
            panic!("auth ssh-agent exited before binding socket: {status}\n{stderr}");
        }

        sleep(Duration::from_millis(20)).await;
    }

    let stderr = child.read_stderr().await;
    panic!(
        "timed out waiting for ssh-agent socket at {}: {stderr}",
        socket_path.display()
    );
}

async fn exchange_frame(socket_path: &Path, frame: &[u8]) -> Vec<u8> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("ssh-agent socket should accept");
    stream.write_all(frame).await.expect("frame should write");

    let mut length = [0u8; 4];
    stream
        .read_exact(&mut length)
        .await
        .expect("frame length should be readable");
    let length = u32::from_be_bytes(length) as usize;
    let mut body = vec![0u8; length];
    stream
        .read_exact(&mut body)
        .await
        .expect("frame body should be readable");

    let mut response = (length as u32).to_be_bytes().to_vec();
    response.extend_from_slice(&body);
    response
}

fn spawn_frame_request(socket_path: PathBuf, frame: Vec<u8>) -> JoinHandle<Vec<u8>> {
    tokio::spawn(async move { exchange_frame(&socket_path, &frame).await })
}

async fn recv_frame_result(handle: JoinHandle<Vec<u8>>, timeout_duration: Duration) -> Vec<u8> {
    timeout(timeout_duration, handle)
        .await
        .expect("frame request should complete before timeout")
        .expect("frame request task should complete")
}

async fn send_partial_frame_then_disconnect(socket_path: &Path) {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("ssh-agent socket should accept");
    stream
        .write_all(&[0, 0, 0, 8, protocol::SSH_AGENTC_SIGN_REQUEST, 0, 0])
        .await
        .expect("partial frame should write");
}

async fn mount_get_private_key_mock(server: &MockServer, public_key: &str) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_private_key"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "privateKey": {
                "privateKeyId": "pk-id",
                "publicKey": public_key,
                "privateKeyName": "ssh agent signer",
                "curve": "CURVE_ED25519",
                "addresses": [],
                "privateKeyTags": [],
                "createdAt": null,
                "updatedAt": null,
                "exported": false,
                "imported": false
            }
        })))
        .mount(server)
        .await;
}

async fn mount_sign_raw_payload_mock(server: &MockServer, signature: &[u8]) {
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/sign_raw_payload"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "result": {
                    "signRawPayloadResult": {
                        "r": hex::encode(&signature[..32]),
                        "s": hex::encode(&signature[32..]),
                        "v": TURNKEY_TEST_V
                    }
                }
            }
        })))
        .mount(server)
        .await;
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    async fn assert_running(&mut self, context: &str) {
        if let Some(status) = self
            .child
            .try_wait()
            .expect("child status should be readable")
        {
            let stderr = self.read_stderr().await;
            panic!("{context}: {status}\n{stderr}");
        }
    }

    fn pid(&self) -> u32 {
        self.child.id().expect("child pid should be available")
    }

    async fn read_stderr(&mut self) -> String {
        let mut stderr = String::new();
        if let Some(mut reader) = self.child.stderr.take() {
            let _ = reader.read_to_string(&mut stderr).await;
        }
        stderr
    }

    async fn wait_for_exit(&mut self) -> std::process::ExitStatus {
        for _ in 0..100 {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("child status should be readable")
            {
                return status;
            }

            sleep(Duration::from_millis(20)).await;
        }

        let stderr = self.read_stderr().await;
        panic!("timed out waiting for ssh-agent to exit: {stderr}");
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
