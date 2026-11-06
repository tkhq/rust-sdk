use super::*;
use crate::{RetryConfig, TurnkeyP256ApiKey, TurnkeySecp256k1ApiKey};
use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use turnkey_api_key_stamper::{Stamp, StamperError};
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{method, path},
};

fn response(status: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"activity": {
        "id":"synthetic-activity", "organizationId":"synthetic-org", "fingerprint":"server-value-is-untrusted",
        "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":status
    }}))
}

fn completed() -> ResponseTemplate {
    response("ACTIVITY_STATUS_COMPLETED")
}

fn client<S: Stamp>(server: &MockServer, key: S) -> TurnkeyClient<S> {
    TurnkeyClient::builder()
        .api_key(key)
        .base_url(server.uri())
        .retry_config(RetryConfig {
            initial_delay: Duration::from_millis(1),
            multiplier: 1.0,
            max_delay: Duration::from_millis(1),
            max_retries: 3,
        })
        .build()
        .unwrap()
}

fn captured(request: &Request) -> RequestFingerprint {
    let header = StampHeader {
        name: "x-stamp".into(),
        value: request
            .headers
            .get("x-stamp")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    };
    RequestFingerprint::from_stamp(&header, std::str::from_utf8(&request.body).unwrap()).unwrap()
}

#[tokio::test]
async fn exact_loopback_tuples_for_both_curves_and_equal_body_time() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let p256 = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let secp = client(
        &server,
        TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let body = json!({"timestampMs":"100", "text":"é🔑\"\\\n"});
    let mut receipts = Vec::new();
    receipts.push(
        p256.process_activity_with_receipt(&body, "/p256".into())
            .await
            .unwrap(),
    );
    receipts.push(
        p256.process_activity_with_receipt(&body, "/p256".into())
            .await
            .unwrap(),
    );
    receipts.push(
        secp.process_activity_with_receipt(&body, "/secp".into())
            .await
            .unwrap(),
    );
    receipts.push(
        secp.process_activity_with_receipt(&body, "/secp".into())
            .await
            .unwrap(),
    );
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 4);
    for (receipt, request) in receipts.iter().zip(&requests) {
        assert_eq!(request.body, serde_json::to_vec(&body).unwrap());
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(request)]
        );
        assert_eq!(receipt.activity().id, "synthetic-activity");
    }
    assert_ne!(captured(&requests[0]), captured(&requests[1]));
    assert_ne!(captured(&requests[2]), captured(&requests[3]));
    if let Ok(destination) = std::env::var("TVC572_CAPTURE_PATH") {
        let tuples: Vec<_> = requests.iter().map(|request| {
            let stamp: Value = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(request.headers["x-stamp"].as_bytes()).unwrap()).unwrap();
            json!({"scheme":stamp["scheme"], "publicKey":stamp["publicKey"], "signature":stamp["signature"], "message":std::str::from_utf8(&request.body).unwrap(), "digest":captured(request).to_hex()})
        }).collect();
        std::fs::write(destination, serde_json::to_vec_pretty(&tuples).unwrap()).unwrap();
    }
}

struct PendingTwice(AtomicUsize);
impl Respond for PendingTwice {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        if self.0.fetch_add(1, Ordering::SeqCst) < 2 {
            response("ACTIVITY_STATUS_PENDING")
        } else {
            completed()
        }
    }
}

struct CountSerialize(Arc<AtomicUsize>);
impl Serialize for CountSerialize {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        self.0.fetch_add(1, Ordering::SeqCst);
        json!({"timestampMs":"100", "message":"same"}).serialize(serializer)
    }
}

#[tokio::test]
async fn pending_posts_each_serialize_once_and_retain_all_fingerprints() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(PendingTwice(AtomicUsize::new(0)))
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let serializations = Arc::new(AtomicUsize::new(0));
    let receipt = client
        .process_activity_with_receipt(CountSerialize(serializations.clone()), "/submit".into())
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(serializations.load(Ordering::SeqCst), 3);
    assert_eq!(requests.len(), 3);
    assert_eq!(
        receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
        requests.iter().map(captured).collect::<Vec<_>>()
    );
    for request in &requests {
        assert_eq!(
            serde_json::from_slice::<Value>(&request.body).unwrap(),
            json!({"timestampMs":"100", "message":"same"})
        );
    }
    assert!(
        requests
            .windows(2)
            .all(|pair| captured(&pair[0]) != captured(&pair[1]))
    );
}

#[tokio::test]
async fn concurrent_invocations_and_unrelated_queries_remain_isolated() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let left = json!({"invocation":"left"});
    let right = json!({"invocation":"right"});
    let query_body = json!({"query":"unrelated"});
    let (left, right, query) = tokio::join!(
        client.process_activity_with_receipt(left, "/left".into()),
        client.process_activity_with_receipt(right, "/right".into()),
        client.process_request::<_, Value>(&query_body, "/query".into()),
    );
    query.unwrap();
    let requests = server.received_requests().await.unwrap();
    for (receipt, route) in [(left.unwrap(), "/left"), (right.unwrap(), "/right")] {
        let request = requests
            .iter()
            .find(|request| request.url.path() == route)
            .unwrap();
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(request)]
        );
    }
}

#[tokio::test]
async fn cancellation_timeout_and_failed_status_produce_no_accepted_receipt() {
    let server = MockServer::start().await;
    Mock::given(path("/slow"))
        .respond_with(completed().set_delay(Duration::from_millis(200)))
        .mount(&server)
        .await;
    Mock::given(path("/ok"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let result = tokio::time::timeout(
        Duration::from_millis(40),
        client.process_activity_with_receipt((), "/slow".into()),
    )
    .await;
    assert!(result.is_err());
    let receipt = client
        .process_activity_with_receipt((), "/ok".into())
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
        [captured(
            requests.iter().find(|r| r.url.path() == "/ok").unwrap()
        )]
    );
    let short = TurnkeyClient::builder()
        .api_key(TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap())
        .base_url(server.uri())
        .timeout(Duration::from_millis(40))
        .build()
        .unwrap();
    assert!(
        matches!(short.process_activity_with_receipt((), "/slow".into()).await,
            Err(TurnkeyClientError::SubmissionFailed(e))
                if e.submission_state() == SubmissionState::MayHaveSubmitted
                    && e.cause() == &SubmissionFailureCause::Transport)
    );
    Mock::given(path("/failed"))
        .respond_with(response("ACTIVITY_STATUS_FAILED"))
        .mount(&server)
        .await;
    assert!(matches!(
        client
            .process_activity_with_receipt((), "/failed".into())
            .await,
        Err(TurnkeyClientError::SubmissionFailed(e))
            if e.submission_state() == SubmissionState::MayHaveSubmitted
                && e.cause() == &SubmissionFailureCause::ActivityNotCompleted
    ));
}

#[tokio::test]
async fn malformed_or_failed_stamps_never_reach_http() {
    let server = MockServer::start().await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    type TestStamp = fn(&TurnkeyP256ApiKey, &[u8]) -> Result<StampHeader, StamperError>;
    let malformed: [TestStamp; 5] = [
        |_, _| Err(StamperError::EntropyUnavailable),
        |_, _| Err(StamperError::FreshSigningFailed),
        |_, _| {
            Ok(StampHeader {
                name: "custom".into(),
                value: "value".into(),
            })
        },
        |_, _| {
            Ok(StampHeader {
                name: "X-Stamp".into(),
                value: "bad!".into(),
            })
        },
        |_, _| {
            Ok(StampHeader { name:"X-Stamp".into(), value:BASE64_URL_SAFE_NO_PAD.encode(br#"{"scheme":"SIGNATURE_SCHEME_TK_API_P256","publicKey":"00","signature":"00"}"#) })
        },
    ];
    for stamp in malformed {
        let mut invocation = SubmissionInvocation::default();
        assert!(
            client
                .process_activity_with_stamper(&(), "/submit".into(), stamp, Some(&mut invocation))
                .await
                .is_err()
        );
        assert!(invocation.sent.is_empty());
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[test]
fn strict_stamp_parsing_and_exact_text() {
    let key = TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap();
    let body = "é🔑\"\\\n";
    let valid = key.stamp_fresh(body.as_bytes()).unwrap();
    let original = RequestFingerprint::from_stamp(&valid, body).unwrap();
    let mut json: Value =
        serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(&valid.value).unwrap()).unwrap();
    json["publicKey"] = json["publicKey"].as_str().unwrap().to_uppercase().into();
    json["signature"] = json["signature"].as_str().unwrap().to_uppercase().into();
    let header = |value: &Value| StampHeader {
        name: "x-sTaMp".into(),
        value: BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(value).unwrap()),
    };
    assert_ne!(
        RequestFingerprint::from_stamp(&header(&json), body).unwrap(),
        original
    );
    assert!(RequestFingerprint::from_stamp(&valid, "substitution").is_err());
    for value in [
        json!({}),
        json!({"scheme":"OTHER","publicKey":"00","signature":"00"}),
        json!({"scheme":null,"publicKey":"00","signature":"00"}),
    ] {
        assert!(RequestFingerprint::from_stamp(&header(&value), body).is_err());
    }
    let duplicate = StampHeader {
        name: "X-Stamp".into(),
        value: BASE64_URL_SAFE_NO_PAD
            .encode(br#"{"scheme":"a","scheme":"b","publicKey":"00","signature":"00"}"#),
    };
    assert!(RequestFingerprint::from_stamp(&duplicate, body).is_err());
}

#[tokio::test]
async fn generated_variants_share_ordinary_request_and_typed_result_extraction() {
    use crate::generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent};
    let server = MockServer::start().await;
    Mock::given(path("/public/v1/submit/create_tvc_operator")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
        "id":"operator-activity", "organizationId":"org", "fingerprint":"server", "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":"ACTIVITY_STATUS_COMPLETED",
        "result":{"createTvcOperatorResult":{"walletId":"wallet", "operatorId":"operator", "encryptPublicKey":"encrypt", "signPublicKey":"sign"}}
    }}))).mount(&server).await;
    Mock::given(path("/public/v1/submit/create_tvc_quorum_key")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
        "id":"quorum-activity", "organizationId":"org", "fingerprint":"server", "type":"ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY", "status":"ACTIVITY_STATUS_COMPLETED",
        "result":{"createTvcQuorumKeyResult":{"quorumKeyId":"quorum", "quorumPublicKey":"key", "shareIds":["share"]}}
    }}))).mount(&server).await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    )
    .with_app_proofs();
    let params = CreateTvcOperatorIntent {
        wallet_id: None,
        wallet_name: Some("wallet".into()),
        path: "p".into(),
        operator_name: "operator".into(),
    };
    let ordinary = client
        .create_tvc_operator("org".into(), 100, params.clone())
        .await
        .unwrap();
    let submitted = client
        .create_tvc_operator_with_receipt("org".into(), 100, params)
        .await
        .unwrap();
    assert_eq!(ordinary.result, submitted.activity_result().result);
    assert_eq!(submitted.receipt().activity().id, ordinary.activity_id);
    let params = CreateTvcQuorumKeyIntent {
        threshold: 2,
        operator_encrypt_keys: vec!["one".into(), "two".into()],
    };
    let ordinary = client
        .create_tvc_quorum_key("org".into(), 100, params.clone())
        .await
        .unwrap();
    let (submitted, receipt) = client
        .create_tvc_quorum_key_with_receipt("org".into(), 100, params)
        .await
        .unwrap()
        .into_parts();
    assert_eq!(ordinary.result, submitted.result);
    assert_eq!(receipt.activity().id, ordinary.activity_id);
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests[0].body, requests[1].body);
    assert_eq!(requests[2].body, requests[3].body);
    for request in &requests {
        assert_eq!(
            request.headers["x-tvc-client-version"],
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&request.body).unwrap()["generateAppProofs"],
            true
        );
    }
}

#[test]
fn mono_signature_hex_digest_vectors_and_all_field_mutations() {
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../tests/fixtures/signature_digest_vectors.json"
    ))
    .unwrap();
    for name in [
        "p256",
        "secp256k1",
        "captured-0",
        "captured-1",
        "captured-2",
        "captured-3",
    ] {
        assert!(
            vectors.iter().any(|vector| vector["name"] == name),
            "missing {name}"
        );
    }
    for vector in vectors {
        let scheme = vector["scheme"].as_i64().unwrap() as i32;
        let public_key = vector["publicKey"].as_str().unwrap();
        let message = vector["message"].as_str().unwrap();
        let signature = vector["signature"].as_str().unwrap();
        let actual = RequestFingerprint::signature(scheme, public_key, message, signature).unwrap();
        assert_eq!(
            actual.to_hex(),
            vector["digest"].as_str().unwrap(),
            "{}",
            vector["name"]
        );
        assert_eq!(actual.as_bytes().len(), 32);
        for changed in [
            RequestFingerprint::signature(
                if scheme == 1 { 4 } else { 1 },
                public_key,
                message,
                signature,
            ),
            RequestFingerprint::signature(scheme, &format!("{public_key}x"), message, signature),
            RequestFingerprint::signature(scheme, public_key, &format!("{message}x"), signature),
            RequestFingerprint::signature(scheme, public_key, message, &format!("{signature}x")),
        ] {
            assert_ne!(actual, changed.unwrap());
        }
    }
}

fn failure(error: TurnkeyClientError) -> SubmissionFailure {
    match error {
        TurnkeyClientError::SubmissionFailed(failure) => failure,
        other => panic!("expected submission failure, got {other:?}"),
    }
}

#[tokio::test]
async fn receipt_errors_preserve_submission_state_and_hide_response_contents() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(503).set_body_json(json!({"secret":"stamp-body-secret"})),
        )
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let error = failure(
        client
            .process_activity_with_receipt((), "/submit".into())
            .await
            .unwrap_err(),
    );
    assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
    assert_eq!(error.cause(), &SubmissionFailureCause::HttpStatus(503));
    assert_eq!(error.activity_id(), None);
    assert!(!format!("{error:?} {error}").contains("stamp-body-secret"));
    let mut source = std::error::Error::source(&error);
    while let Some(error) = source {
        assert!(!format!("{error:?} {error}").contains("stamp-body-secret"));
        source = error.source();
    }
}

#[tokio::test]
async fn generated_extraction_failure_retains_possible_submission_and_safe_label() {
    use crate::generated::CreateTvcOperatorIntent;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let error = failure(
        client
            .create_tvc_operator_with_receipt(
                "org".into(),
                100,
                CreateTvcOperatorIntent {
                    wallet_id: None,
                    wallet_name: Some("wallet".into()),
                    path: "path".into(),
                    operator_name: "operator".into(),
                },
            )
            .await
            .unwrap_err(),
    );
    assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
    assert_eq!(error.cause(), &SubmissionFailureCause::InvalidResponse);
    assert_eq!(error.activity_id(), Some("synthetic-activity"));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[test]
fn diagnostic_activity_labels_are_bounded_and_printable() {
    assert_eq!(
        safe_activity_id("untrusted [label] é"),
        Some("untrusted [label] é".into())
    );
    assert_eq!(safe_activity_id("bad\nlabel"), None);
    assert_eq!(safe_activity_id(&"x".repeat(129)), None);
    assert_eq!(safe_activity_id(""), None);
}

#[tokio::test]
#[ignore = "requires OS fault injection; run client/tests/entropy_failure.sh"]
async fn actual_os_entropy_failure_tracks_first_and_pending_sends_for_both_curves() {
    let marker = std::path::PathBuf::from(
        std::env::var("TVC572_ENTROPY_FAILURE_MARKER")
            .expect("run client/tests/entropy_failure.sh"),
    );
    let server = MockServer::start().await;
    Mock::given(path("/ok"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let arm = marker.clone();
    Mock::given(path("/pending"))
        .respond_with(move |_: &Request| {
            std::fs::write(&arm, b"armed").unwrap();
            response("ACTIVITY_STATUS_PENDING")
        })
        .mount(&server)
        .await;

    async fn check<S: FreshStamp>(server: &MockServer, key: S, marker: &std::path::Path) {
        let client = client(server, key);
        let before = server.received_requests().await.unwrap().len();
        std::fs::write(marker, b"armed").unwrap();
        let first = failure(
            client
                .process_activity_with_receipt((), "/ok".into())
                .await
                .unwrap_err(),
        );
        std::fs::remove_file(marker).unwrap();
        assert_eq!(first.submission_state(), SubmissionState::NeverSubmitted);
        assert_eq!(first.cause(), &SubmissionFailureCause::EntropyUnavailable);
        assert_eq!(first.activity_id(), None);
        assert_eq!(server.received_requests().await.unwrap().len(), before);

        let later = failure(
            client
                .process_activity_with_receipt((), "/pending".into())
                .await
                .unwrap_err(),
        );
        std::fs::remove_file(marker).unwrap();
        assert_eq!(later.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(later.cause(), &SubmissionFailureCause::EntropyUnavailable);
        assert_eq!(later.activity_id(), Some("synthetic-activity"));
        assert!(
            later
                .to_string()
                .contains("remote creation may have succeeded")
        );
        assert!(!later.to_string().contains("never submitted"));
        assert_eq!(server.received_requests().await.unwrap().len(), before + 1);

        let recovered = client
            .process_activity_with_receipt((), "/ok".into())
            .await
            .unwrap();
        assert_eq!(recovered.sent_fingerprints().count(), 1);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), before + 2);
        assert_eq!(
            *recovered.sent_fingerprints().next().unwrap(),
            captured(requests.last().unwrap())
        );
    }
    check(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        &marker,
    )
    .await;
    check(
        &server,
        TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
        &marker,
    )
    .await;
}

struct FailsAfterFirstSerialization(AtomicUsize);
impl Serialize for FailsAfterFirstSerialization {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            json!({"timestampMs":"100"}).serialize(serializer)
        } else {
            Err(serde::ser::Error::custom("private request secret"))
        }
    }
}

#[tokio::test]
async fn pending_then_serialization_failure_retains_prior_submission() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(response("ACTIVITY_STATUS_PENDING"))
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    let error = failure(
        client
            .process_activity_with_receipt(
                FailsAfterFirstSerialization(AtomicUsize::new(0)),
                "/submit".into(),
            )
            .await
            .unwrap_err(),
    );
    assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
    assert_eq!(error.activity_id(), Some("synthetic-activity"));
    assert_eq!(error.cause(), &SubmissionFailureCause::InvalidRequest);
    assert!(!format!("{error:?} {error}").contains("private request secret"));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn direct_tvc_creation_requests_include_sdk_version_without_changing_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(completed())
        .mount(&server)
        .await;
    let client = client(
        &server,
        TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
    );
    for proof_flag in [true, false] {
        for route in [
            "/public/v1/submit/create_tvc_operator",
            "/public/v1/submit/create_tvc_quorum_key",
        ] {
            let body = json!({"generateAppProofs": proof_flag});
            client.process_activity(&body, route.into()).await.unwrap();
        }
    }
    client
        .process_activity((), "/unrelated".into())
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 5);
    for (index, request) in requests[..4].iter().enumerate() {
        assert_eq!(
            request.headers["x-tvc-client-version"],
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&request.body).unwrap(),
            json!({"generateAppProofs": index < 2})
        );
    }
    assert!(!requests[4].headers.contains_key("x-tvc-client-version"));
}
