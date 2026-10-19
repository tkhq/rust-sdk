//! `deploy create` runs against a local mock of the Turnkey API.

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::{Value, json};
use std::process::Output;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::deploy::DEFAULT_QOS_VERSION;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

const ORG_ID: &str = "org-test";
const APP_ID: &str = "7d4ea3b5-3f5c-4d7b-9f26-51d4d2c1c7a2";
const PIVOT_DIGEST: &str = "dba4f54cd573b81d724b0002c4e761c76eb21f881f31d48b972b3e4c101f2098";
const IMAGE_DIGEST: &str =
    "sha256:1f6a0c2766a300b348f141fc11a379bac84d9c78854afdc106697a266631622c";

const GET_APP_PATH: &str = "/public/v1/query/get_tvc_app";
const VALIDATE_IMAGE_PATH: &str = "/public/v1/query/validate_tvc_image";
const CREATE_DEPLOYMENT_PATH: &str = "/public/v1/submit/create_tvc_deployment";

fn app_response() -> Value {
    let operator = |name: &str, public_key: &str| {
        json!({
            "id": format!("{name}-id"),
            "name": name,
            "publicKey": public_key,
            "encryptPublicKey": "",
            "signPublicKey": "",
        })
    };
    let set = |threshold: u32, operators: Vec<Value>| {
        json!({
            "id": "set-id",
            "name": "set",
            "organizationId": ORG_ID,
            "operators": operators,
            "threshold": threshold,
        })
    };
    json!({
        "tvcApp": {
            "id": APP_ID,
            "organizationId": ORG_ID,
            "name": "app",
            "quorumPublicKey": "04aa01",
            "manifestSet": set(2, vec![operator("alice", "0a0b"), operator("bob", "0c0d")]),
            "shareSet": set(1, vec![operator("carol", "0e0f")]),
            "enableEgress": true,
            "publicDomain": "app.example",
            "enableDebugModeDeployments": false,
        }
    })
}

fn create_deployment_response() -> Value {
    json!({
        "activity": {
            "id": "activity-1",
            "organizationId": ORG_ID,
            "status": "ACTIVITY_STATUS_COMPLETED",
            "type": "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
            "result": {
                "createTvcDeploymentResult": {
                    "deploymentId": "deployment-1",
                    "manifestId": "manifest-1",
                }
            },
            "fingerprint": "",
        }
    })
}

async fn mount(server: &MockServer, route: &str, response: impl Respond + 'static, calls: u64) {
    Mock::given(method("POST"))
        .and(path(route))
        .respond_with(response)
        .expect(calls)
        .named(route)
        .mount(server)
        .await;
}

fn json_response(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

/// Run `tvc deploy create` in flag-only, non-interactive JSON mode against
/// `server`, with env-var auth and a throwaway HOME.
async fn run_deploy_create(server: &MockServer, extra_args: &[&str]) -> Output {
    let home = TempDir::new().unwrap();
    let stamper = TurnkeyP256ApiKey::generate();
    let mut command = cargo_bin_cmd!("tvc");
    command
        .env_clear()
        .env("HOME", home.path())
        .env("TVC_ORG_ID", ORG_ID)
        .env(
            "TVC_API_KEY_PUBLIC",
            hex::encode(stamper.compressed_public_key()),
        )
        .env("TVC_API_KEY_PRIVATE", hex::encode(stamper.private_key()))
        .env("TVC_API_BASE_URL", server.uri())
        // Via env so a test can override it with the flag.
        .env("TVC_EXPECTED_PIVOT_DIGEST", PIVOT_DIGEST)
        .args([
            "--message-format",
            "json",
            "deploy",
            "create",
            "--app-id",
            APP_ID,
            "--pivot-image-url",
            "ghcr.io/example/app:latest",
            "--pivot-path",
            "/app/pivot",
            "--pivot-args=--host,0.0.0.0",
            "--health-check-port",
            "3000",
            "--public-ingress-port",
            "4000",
        ])
        .args(extra_args);

    // The mock server runs on this test's runtime, so the blocking child
    // process must run off it.
    tokio::task::spawn_blocking(move || command.output().unwrap())
        .await
        .unwrap()
}

fn last_json_message(output: &Output) -> Value {
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let line = stdout.lines().last().unwrap_or_else(|| {
        panic!(
            "no stdout; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    serde_json::from_str(line).unwrap()
}

async fn request_paths(server: &MockServer) -> Vec<String> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .map(|request| request.url.path().to_string())
        .collect()
}

async fn request_body(server: &MockServer, route: &str) -> Value {
    let request = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .find(|request| request.url.path() == route)
        .unwrap_or_else(|| panic!("no request to {route}"));
    serde_json::from_slice(&request.body).unwrap()
}

/// Without the opt-in the request is the one the coordinator has always
/// received: the configured QOS version, no nonce, no manifest, and none of the
/// extra queries. This holds whether or not `manifest-v3` is compiled in.
#[tokio::test]
async fn coordinator_generated_manifest_request_is_unchanged_by_default() {
    let server = MockServer::start().await;
    mount(&server, GET_APP_PATH, json_response(app_response()), 1).await;
    mount(
        &server,
        VALIDATE_IMAGE_PATH,
        json_response(json!({"resolvedImageDigest": IMAGE_DIGEST})),
        1,
    )
    .await;
    mount(
        &server,
        CREATE_DEPLOYMENT_PATH,
        json_response(create_deployment_response()),
        1,
    )
    .await;

    let output = run_deploy_create(&server, &[]).await;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(last_json_message(&output)["reason"], "deployment_created");
    assert_eq!(
        request_paths(&server).await,
        [GET_APP_PATH, VALIDATE_IMAGE_PATH, CREATE_DEPLOYMENT_PATH]
    );

    let create = request_body(&server, CREATE_DEPLOYMENT_PATH).await;
    let parameters = &create["parameters"];
    assert_eq!(parameters["qosVersion"], DEFAULT_QOS_VERSION);
    assert_eq!(parameters["nonce"], Value::Null);
    assert_eq!(parameters["qosManifest"], Value::Null);
}

#[cfg(feature = "manifest-v3")]
mod client_generated_manifest {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use qos_core::protocol::services::boot::VersionedManifest;
    use std::sync::{Arc, Mutex};

    const NONCE: u32 = 1_790_000_000;
    const PCR: &str = "50a89cedc3c4ebd18f55544173b3002013b0fa8009c9558819bfad1fd16ac07ebe0a8d21dffb3360e589db3ff629a5dd";

    const GENERATION_INFO_PATH: &str = "/public/v1/query/get_tvc_manifest_generation_info";
    const GET_DEPLOYMENT_PATH: &str = "/public/v1/query/get_tvc_deployment";

    /// Answers `create_tvc_deployment` and remembers the manifest it was sent,
    /// as the base64 string on the wire.
    struct RecordManifest(Arc<Mutex<Option<String>>>);

    impl Respond for RecordManifest {
        fn respond(&self, request: &wiremock::Request) -> ResponseTemplate {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            *self.0.lock().unwrap() = body["parameters"]["qosManifest"]
                .as_str()
                .map(str::to_string);
            json_response(create_deployment_response())
        }
    }

    /// Answers `get_tvc_deployment` with the given base64 manifest, or with the
    /// one `RecordManifest` saw when it is `None`.
    struct StoredManifest {
        sent: Arc<Mutex<Option<String>>>,
        stored: Option<String>,
    }

    impl Respond for StoredManifest {
        fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
            let manifest = self
                .stored
                .clone()
                .or_else(|| self.sent.lock().unwrap().clone())
                .unwrap();
            json_response(json!({
                "tvcDeployment": {
                    "id": "deployment-1",
                    "organizationId": ORG_ID,
                    "appId": APP_ID,
                    "qosVersion": DEFAULT_QOS_VERSION,
                    "manifest": {"id": "manifest-1", "manifest": manifest},
                }
            }))
        }
    }

    async fn mock_api(stored: Option<String>) -> MockServer {
        let sent = Arc::new(Mutex::new(None));
        let server = MockServer::start().await;
        mount(&server, GET_APP_PATH, json_response(app_response()), 1).await;
        mount(
            &server,
            VALIDATE_IMAGE_PATH,
            json_response(json!({"resolvedImageDigest": IMAGE_DIGEST})),
            1,
        )
        .await;
        mount(
            &server,
            GENERATION_INFO_PATH,
            json_response(json!({
                "nonce": NONCE,
                "namespace": format!("dev/tvc/{APP_ID}"),
                "pcr0": PCR,
                "pcr1": PCR,
                "pcr2": PCR,
                "pcr3": PCR,
                "awsRootCertificate": "3082021130820196a0",
                "defaultDnsResolvers": ["10.0.0.2"],
            })),
            1,
        )
        .await;
        mount(
            &server,
            CREATE_DEPLOYMENT_PATH,
            RecordManifest(sent.clone()),
            1,
        )
        .await;
        mount(
            &server,
            GET_DEPLOYMENT_PATH,
            StoredManifest { sent, stored },
            1,
        )
        .await;
        server
    }

    /// The CLI builds the manifest from the coordinator's inputs for the same
    /// QOS version as the default path, sends it with the nonce it was built
    /// for, and then finds that same manifest stored on the deployment.
    #[tokio::test]
    async fn manifest_is_built_locally_and_sent_with_its_nonce() {
        let server = mock_api(None).await;

        let output = run_deploy_create(&server, &["--client-generated-manifest"]).await;

        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(last_json_message(&output)["reason"], "deployment_created");
        assert_eq!(
            request_paths(&server).await,
            [
                GET_APP_PATH,
                VALIDATE_IMAGE_PATH,
                GENERATION_INFO_PATH,
                CREATE_DEPLOYMENT_PATH,
                GET_DEPLOYMENT_PATH,
            ]
        );
        assert_eq!(
            request_body(&server, GENERATION_INFO_PATH).await,
            json!({"organizationId": ORG_ID, "appId": APP_ID, "qosVersion": DEFAULT_QOS_VERSION})
        );

        let create = request_body(&server, CREATE_DEPLOYMENT_PATH).await;
        let parameters = &create["parameters"];
        assert_eq!(parameters["qosVersion"], DEFAULT_QOS_VERSION);
        assert_eq!(parameters["nonce"], NONCE);
        // `bytes` travel as standard base64, the proto3 JSON encoding.
        let manifest = STANDARD
            .decode(parameters["qosManifest"].as_str().unwrap())
            .unwrap();
        let VersionedManifest::V2(manifest) =
            VersionedManifest::try_from_slice_compat(&manifest).unwrap()
        else {
            panic!("expected a v2 manifest");
        };
        assert_eq!(manifest.namespace.nonce, NONCE);
        assert_eq!(hex::encode(manifest.pivot.hash), PIVOT_DIGEST);
    }

    /// A server release that predates the manifest field drops it and stores
    /// a manifest of its own; the CLI must notice rather than report success.
    #[tokio::test]
    async fn manifest_the_coordinator_did_not_store_is_an_error() {
        let coordinator_manifest = STANDARD.encode(br#"{"version":"v2"}"#);
        let server = mock_api(Some(coordinator_manifest)).await;

        let output = run_deploy_create(&server, &["--client-generated-manifest"]).await;

        assert!(!output.status.success());
        let message = last_json_message(&output);
        assert_eq!(message["reason"], "command_error", "{message}");
        let text = message["message"].as_str().unwrap();
        assert!(
            text.contains("deployment deployment-1 was created, but the server did not store the client-generated QOS manifest"),
            "{message}"
        );
    }

    /// An organization without client-generated manifests enabled is refused
    /// when the CLI asks for the generation info; nothing is created and the
    /// server's reason reaches the user.
    #[tokio::test]
    async fn organization_without_client_generated_manifests_is_refused_before_creating() {
        let reason =
            format!("client-generated TVC manifests are not enabled for organization {ORG_ID}");
        let server = MockServer::start().await;
        mount(&server, GET_APP_PATH, json_response(app_response()), 1).await;
        mount(
            &server,
            VALIDATE_IMAGE_PATH,
            json_response(json!({"resolvedImageDigest": IMAGE_DIGEST})),
            1,
        )
        .await;
        mount(
            &server,
            GENERATION_INFO_PATH,
            ResponseTemplate::new(403).set_body_json(json!({"code": 7, "message": reason})),
            1,
        )
        .await;
        mount(
            &server,
            CREATE_DEPLOYMENT_PATH,
            json_response(create_deployment_response()),
            0,
        )
        .await;

        let output = run_deploy_create(&server, &["--client-generated-manifest"]).await;

        assert!(!output.status.success());
        let message = last_json_message(&output);
        assert_eq!(message["code"], "unauthorized", "{message}");
        assert_eq!(message["httpStatus"], 403, "{message}");
        assert!(
            message["message"].as_str().unwrap().contains(&reason),
            "{message}"
        );
    }

    #[tokio::test]
    async fn pivot_digest_with_a_prefix_is_rejected_before_any_request() {
        let server = MockServer::start().await;

        let output = run_deploy_create(
            &server,
            &[
                "--client-generated-manifest",
                "--expected-pivot-digest",
                &format!("sha256:{PIVOT_DIGEST}"),
            ],
        )
        .await;

        assert!(!output.status.success());
        let message = last_json_message(&output);
        assert!(
            message["message"]
                .as_str()
                .unwrap()
                .contains("must be 64 hex characters with no `sha256:` prefix"),
            "{message}"
        );
        assert_eq!(request_paths(&server).await, Vec::<String>::new());
    }
}
