#![allow(missing_docs, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use qos_core::protocol::services::boot::{
    Approval, ManifestEnvelopeV2, ManifestSet, ManifestV2, ManifestVersion, Namespace, NitroConfig,
    PivotConfigV2, PivotEnv, QuorumMember, RestartPolicy, ShareSet, VersionedManifestEnvelope,
};
use qos_nsm::NsmProvider;
use qos_nsm::mock::{MOCK_ROOT_CERT_DER, MOCK_SECONDS_SINCE_EPOCH, MockNsm};
use qos_nsm::nitro::{self, AttestError};
use qos_nsm::types::{NsmRequest, NsmResponse};
use qos_p256::{P256Pair, P256Public};
use tower::{ServiceBuilder, ServiceExt, service_fn};
use tvc_axum::{
    ATTESTATION_DOC_HEADER, MANIFEST_ENVELOPE_HEADER, ManifestCommitmentKind, ResponseSigningLayer,
    SignedResponseParts, VerificationExpectations, VerifyResponseError, verify_quorum_signature,
    verify_response_trust_chain,
};

/// NSM wrapper that counts attestation requests. Only for tests.
struct CountingNsm {
    inner: MockNsm,
    attestation_requests: AtomicUsize,
}

impl CountingNsm {
    fn new() -> Self {
        Self {
            inner: MockNsm::new(),
            attestation_requests: AtomicUsize::new(0),
        }
    }
}

impl NsmProvider for CountingNsm {
    fn nsm_process_request(&self, request: NsmRequest) -> NsmResponse {
        if matches!(request, NsmRequest::Attestation { .. }) {
            self.attestation_requests.fetch_add(1, Ordering::SeqCst);
        }
        self.inner.nsm_process_request(request)
    }

    fn timestamp_ms(&self) -> Result<u64, AttestError> {
        self.inner.timestamp_ms()
    }

    fn attestation_root_ca_der(&self) -> Vec<u8> {
        self.inner.attestation_root_ca_der()
    }
}

fn test_member(alias: &str) -> (QuorumMember, P256Pair) {
    let pair = P256Pair::generate().expect("key should generate");
    let member = QuorumMember {
        alias: alias.to_string(),
        pub_key: pair.public_key().to_bytes(),
    };
    (member, pair)
}

/// Minimal manifest envelope with one manifest-set member approving at
/// threshold 1, mirroring the qos_core verify tests.
fn approved_envelope_with_quorum(quorum_key: Vec<u8>) -> VersionedManifestEnvelope {
    let (member, pair) = test_member("member");
    let manifest = ManifestV2 {
        version: ManifestVersion::V2,
        namespace: Namespace {
            name: "test-namespace".to_string(),
            nonce: 1,
            quorum_key,
        },
        pivot: PivotConfigV2 {
            hash: [7; 32],
            restart: RestartPolicy::Never,
            bridge_config: vec![],
            debug_mode: false,
            args: vec![],
            env: PivotEnv::new(),
        },
        manifest_set: ManifestSet {
            threshold: 1,
            members: vec![member.clone()],
        },
        share_set: ShareSet {
            threshold: 1,
            members: vec![member.clone()],
        },
        enclave: NitroConfig {
            pcr0: vec![0; 48],
            pcr1: vec![1; 48],
            pcr2: vec![2; 48],
            pcr3: vec![3; 48],
            aws_root_certificate: vec![],
            qos_commit: "test-qos-commit".to_string(),
        },
        dns: None,
    };
    let mut envelope = ManifestEnvelopeV2 {
        manifest,
        manifest_set_approvals: vec![],
        share_set_approvals: vec![],
    };
    let manifest_hash = VersionedManifestEnvelope::V2(envelope.clone()).manifest_hash();
    envelope.manifest_set_approvals = vec![Approval {
        signature: pair
            .sign(&manifest_hash)
            .expect("approval signing should not fail"),
        member,
    }];
    VersionedManifestEnvelope::V2(envelope)
}

fn approved_envelope() -> VersionedManifestEnvelope {
    approved_envelope_with_quorum(
        P256Pair::generate()
            .expect("key should generate")
            .public_key()
            .to_bytes(),
    )
}

/// NSM whose PCR bank matches the manifest measurements and carries the
/// setup manifest commitment (PCR16) over the manifest hash and ephemeral
/// public key, so the emitted attestation documents pass full verification.
fn seeded_nsm(envelope: &VersionedManifestEnvelope, ephemeral_key: &P256Pair) -> MockNsm {
    let manifest = envelope.clone().manifest();
    let enclave = manifest.enclave();
    let commitment_pcr = nitro::expected_manifest_commitment_pcr(
        ManifestCommitmentKind::Setup,
        &envelope.manifest_hash(),
        &ephemeral_key.public_key().to_bytes(),
    )
    .expect("commitment PCR should compute");
    MockNsm::new()
        .with_pcr(0, enclave.pcr0.clone())
        .with_pcr(1, enclave.pcr1.clone())
        .with_pcr(2, enclave.pcr2.clone())
        .with_pcr(3, enclave.pcr3.clone())
        .with_pcr(
            ManifestCommitmentKind::Setup.pcr_index(),
            commitment_pcr.to_vec(),
        )
}

async fn oneshot_response(layer: &ResponseSigningLayer, method: &str, uri: &str) -> Response {
    let service = ServiceBuilder::new()
        .layer(layer.clone())
        .service(service_fn(|_request: Request<Body>| async {
            Ok::<_, std::convert::Infallible>(Response::new(Body::from("signed body")))
        }));
    service
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed")
}

async fn body_bytes(response: Response) -> Vec<u8> {
    use http_body_util::BodyExt;
    response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes()
        .to_vec()
}

fn header_str<'a>(response: &'a Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .unwrap_or_else(|| panic!("{name} header should exist"))
        .to_str()
        .unwrap_or_else(|_| panic!("{name} header should be ascii"))
}

/// A signed response and everything a client needs to verify it.
struct SignedFixture {
    envelope: VersionedManifestEnvelope,
    ephemeral_public: Vec<u8>,
    quorum_public: P256Public,
    method: &'static str,
    path: &'static str,
    query: Option<&'static str>,
    status: StatusCode,
    body: Vec<u8>,
    signature_input: String,
    signature: String,
    attestation_doc: String,
    manifest_envelope: String,
}

impl SignedFixture {
    fn parts(&self) -> SignedResponseParts<'_> {
        SignedResponseParts {
            method: self.method,
            path: self.path,
            query: self.query,
            status: self.status,
            body: &self.body,
            signature_input: &self.signature_input,
            signature: &self.signature,
        }
    }

    fn verify_chain(
        &self,
        parts: &SignedResponseParts<'_>,
        expectations: &VerificationExpectations,
    ) -> Result<P256Public, VerifyResponseError> {
        verify_response_trust_chain(
            parts,
            &self.attestation_doc,
            &self.manifest_envelope,
            MOCK_ROOT_CERT_DER,
            MOCK_SECONDS_SINCE_EPOCH,
            ManifestCommitmentKind::Setup,
            expectations,
        )
    }
}

/// Drive one request through a quorum-signing layer whose manifest carries
/// the quorum public key and whose NSM passes full verification.
async fn signed_fixture(method: &'static str, uri: &'static str) -> SignedFixture {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let quorum_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let envelope = approved_envelope_with_quorum(quorum_key.public_key().to_bytes());
    let layer = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .quorum_key(Arc::clone(&quorum_key))
        .nsm(Arc::new(seeded_nsm(&envelope, &ephemeral_key)))
        .manifest_envelope(envelope.clone())
        .build()
        .expect("layer should build");

    let response = oneshot_response(&layer, method, uri).await;
    let (path, query) = uri
        .split_once('?')
        .map_or((uri, None), |(path, query)| (path, Some(query)));
    let status = response.status();
    let signature_input = header_str(&response, "signature-input").to_owned();
    let signature = header_str(&response, "signature").to_owned();
    let attestation_doc = header_str(&response, ATTESTATION_DOC_HEADER).to_owned();
    let manifest_envelope = header_str(&response, MANIFEST_ENVELOPE_HEADER).to_owned();
    let body = body_bytes(response).await;

    SignedFixture {
        envelope,
        ephemeral_public: ephemeral_key.public_key().to_bytes(),
        quorum_public: P256Public::from_bytes(&quorum_key.public_key().to_bytes())
            .expect("quorum public key should decode"),
        method,
        path,
        query,
        status,
        body,
        signature_input,
        signature,
        attestation_doc,
        manifest_envelope,
    }
}

#[tokio::test]
async fn quorum_and_full_chain_verification_succeed() {
    let fixture = signed_fixture("GET", "/verified").await;
    let manifest = fixture.envelope.clone().manifest();

    // Full chain: attestation document, manifest expectations, approvals and
    // commitment PCR verify, and the attested ephemeral key verifies the
    // ephemeral response signature.
    let attested_key = fixture
        .verify_chain(
            &fixture.parts(),
            &VerificationExpectations::new()
                .namespace_name(&manifest.namespace().name)
                .quorum_key(manifest.namespace().quorum_key.clone())
                .manifest_hash(fixture.envelope.manifest_hash()),
        )
        .expect("trust chain should verify");
    assert_eq!(attested_key.to_bytes(), fixture.ephemeral_public);

    // The quorum key taken from the verified manifest verifies the quorum
    // response signature.
    let quorum_key = P256Public::from_bytes(&manifest.namespace().quorum_key)
        .expect("manifest quorum key should decode");
    verify_quorum_signature(&fixture.parts(), &quorum_key).expect("quorum signature should verify");
}

#[tokio::test]
async fn tampered_responses_and_wrong_keys_fail() {
    let fixture = signed_fixture("POST", "/tamper").await;
    let expectations = VerificationExpectations::new();

    let wrong_key = P256Pair::generate().expect("key should generate");
    let wrong_public =
        P256Public::from_bytes(&wrong_key.public_key().to_bytes()).expect("key should decode");
    assert!(matches!(
        verify_quorum_signature(&fixture.parts(), &wrong_public),
        Err(VerifyResponseError::InvalidSignature("quorum"))
    ));

    let tampered_body = SignedResponseParts {
        body: b"tampered body",
        ..fixture.parts()
    };
    assert!(matches!(
        verify_quorum_signature(&tampered_body, &fixture.quorum_public),
        Err(VerifyResponseError::InvalidSignature("quorum"))
    ));
    assert!(matches!(
        fixture.verify_chain(&tampered_body, &expectations),
        Err(VerifyResponseError::InvalidSignature("ephemeral"))
    ));

    let tampered_status = SignedResponseParts {
        status: StatusCode::NOT_FOUND,
        ..fixture.parts()
    };
    assert!(matches!(
        fixture.verify_chain(&tampered_status, &expectations),
        Err(VerifyResponseError::InvalidSignature("ephemeral"))
    ));
}

#[tokio::test]
async fn changed_query_fails_verification() {
    let fixture = signed_fixture("GET", "/query?account=a").await;
    verify_quorum_signature(&fixture.parts(), &fixture.quorum_public)
        .expect("original query should verify");

    let changed_query = SignedResponseParts {
        query: Some("account=b"),
        ..fixture.parts()
    };
    assert!(matches!(
        verify_quorum_signature(&changed_query, &fixture.quorum_public),
        Err(VerifyResponseError::InvalidSignature("quorum"))
    ));
}

#[tokio::test]
async fn no_quorum_signature_without_quorum_key() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let layer = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .nsm(Arc::new(MockNsm::new()))
        .manifest_envelope(approved_envelope())
        .build()
        .expect("layer should build");

    let response = oneshot_response(&layer, "GET", "/no-quorum").await;
    let signature_input = header_str(&response, "signature-input").to_owned();
    let signature = header_str(&response, "signature").to_owned();
    let body = body_bytes(response).await;

    let parts = SignedResponseParts {
        method: "GET",
        path: "/no-quorum",
        query: None,
        status: StatusCode::OK,
        body: &body,
        signature_input: &signature_input,
        signature: &signature,
    };
    let any_key =
        P256Public::from_bytes(&ephemeral_key.public_key().to_bytes()).expect("key should decode");
    assert!(matches!(
        verify_quorum_signature(&parts, &any_key),
        Err(VerifyResponseError::MissingSignatureInput("quorum"))
    ));
}

#[tokio::test]
async fn attestation_doc_is_cached_across_responses() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let nsm = Arc::new(CountingNsm::new());
    let layer = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .nsm(Arc::clone(&nsm) as Arc<dyn NsmProvider + Send + Sync>)
        .manifest_envelope(approved_envelope())
        .build()
        .expect("layer should build");
    assert_eq!(nsm.attestation_requests.load(Ordering::SeqCst), 1);

    let first = oneshot_response(&layer, "GET", "/cached").await;
    let second = oneshot_response(&layer, "GET", "/cached").await;

    assert_eq!(
        header_str(&first, ATTESTATION_DOC_HEADER),
        header_str(&second, ATTESTATION_DOC_HEADER)
    );
    assert_eq!(
        nsm.attestation_requests.load(Ordering::SeqCst),
        1,
        "responses within the TTL should reuse the attestation doc fetched at build time"
    );
}
