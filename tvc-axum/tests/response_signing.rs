#![allow(missing_docs, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use qos_nsm::NsmProvider;
use qos_nsm::mock::{MOCK_ROOT_CERT_DER, MOCK_SECONDS_SINCE_EPOCH, MockNsm};
use qos_nsm::nitro::{self, AttestError};
use qos_nsm::types::{NsmRequest, NsmResponse};
use qos_p256::{P256Pair, P256Public};
use sha2::{Digest, Sha256};
use tower::{ServiceBuilder, ServiceExt, service_fn};
use tvc_axum::{ATTESTATION_DOC_HEADER, MANIFEST_ENVELOPE_HEADER, ResponseSigningLayer};
use tvc_utils::{
    FakeManifestBuilder, VerificationExpectations, fake_keyed_member, fake_manifest_envelope,
    verify_attestation_and_manifest,
};

const SIGNATURE_COMPONENTS: &str = "(\"@method\" \"@path\" \"@status\" \"content-digest\")";
const SIGNATURE_ALG: &str = "ecdsa-p256-sha256";

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

fn signed_layer(
    ephemeral_key: &Arc<P256Pair>,
    quorum_key: Option<&Arc<P256Pair>>,
    envelope: &VersionedManifestEnvelope,
) -> ResponseSigningLayer {
    let mut builder = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(ephemeral_key))
        .nsm(Arc::new(MockNsm::new()))
        .manifest_envelope(envelope.clone());
    if let Some(quorum_key) = quorum_key {
        builder = builder.quorum_key(Arc::clone(quorum_key));
    }
    builder.build().expect("layer should build")
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

fn label_value<'a>(header: &'a str, label: &str) -> &'a str {
    header
        .split(", ")
        .find_map(|value| value.strip_prefix(&format!("{label}=")))
        .unwrap_or_else(|| panic!("{label} value should exist"))
}

fn created_from_signature_input(input: &str, label: &str) -> u64 {
    let value = label_value(input, label);
    let created = value
        .strip_prefix(&format!(r#"{SIGNATURE_COMPONENTS};created="#))
        .and_then(|value| value.split_once(';').map(|(created, _)| created))
        .expect("created parameter should exist");
    created.parse().expect("created should be a unix timestamp")
}

fn signature_bytes(signature_header: &str, label: &str) -> Vec<u8> {
    let signature = label_value(signature_header, label)
        .strip_prefix(':')
        .and_then(|value| value.strip_suffix(':'))
        .expect("signature should be an RFC byte sequence");
    STANDARD
        .decode(signature)
        .expect("signature should be base64")
}

fn signature_input(label: &str, created: u64) -> String {
    format!(r#"{SIGNATURE_COMPONENTS};created={created};keyid="{label}";alg="{SIGNATURE_ALG}""#)
}

fn signature_base(
    method: &str,
    path: &str,
    status: StatusCode,
    digest: &str,
    label: &str,
    created: u64,
) -> Vec<u8> {
    format!(
        "\"@method\": {method}\n\"@path\": {path}\n\"@status\": {}\n\"content-digest\": {digest}\n\"@signature-params\": {}",
        status.as_u16(),
        signature_input(label, created)
    )
    .into_bytes()
}

fn content_digest(body: &[u8]) -> String {
    format!("sha-256=:{}:", STANDARD.encode(Sha256::digest(body)))
}

#[tokio::test]
async fn full_trust_chain_verifies_from_attestation_doc() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let envelope = fake_manifest_envelope();
    let layer = signed_layer(&ephemeral_key, None, &envelope);

    let response = oneshot_response(&layer, "GET", "/chain?ignored=true").await;

    // 1. The manifest envelope header carries the exact canonical bytes the
    // enclave booted with.
    let manifest_bytes = STANDARD
        .decode(header_str(&response, MANIFEST_ENVELOPE_HEADER))
        .expect("manifest header should be base64");
    assert_eq!(
        manifest_bytes,
        envelope
            .to_storage_vec()
            .expect("envelope should serialize")
    );
    let decoded_envelope = VersionedManifestEnvelope::try_from_slice_compat(&manifest_bytes)
        .expect("manifest header should decode");

    // 2. The attestation document verifies up to the NSM root and binds the
    // manifest hash as user_data.
    let attestation_bytes = STANDARD
        .decode(header_str(&response, ATTESTATION_DOC_HEADER))
        .expect("attestation header should be base64");
    let attestation_doc = nitro::attestation_doc_from_der(
        &attestation_bytes,
        MOCK_ROOT_CERT_DER,
        MOCK_SECONDS_SINCE_EPOCH,
    )
    .expect("attestation doc should verify against the NSM root");
    assert_eq!(
        attestation_doc.user_data.as_deref().map(Vec::as_slice),
        Some(decoded_envelope.manifest_hash().as_slice()),
        "attestation user_data should be the manifest hash"
    );

    // 3. The ephemeral public key extracted from the attestation document
    // (never from a header) verifies the response signature.
    let attested_key_bytes = attestation_doc
        .public_key
        .expect("attestation doc should bind the ephemeral public key")
        .into_vec();
    assert_eq!(attested_key_bytes, ephemeral_key.public_key().to_bytes());
    let attested_key =
        P256Public::from_bytes(&attested_key_bytes).expect("attested key should decode");

    let digest = header_str(&response, "content-digest").to_owned();
    let signature_input_header = header_str(&response, "signature-input").to_owned();
    let signature_header = header_str(&response, "signature").to_owned();
    let created = created_from_signature_input(&signature_input_header, "ephemeral");
    assert_eq!(
        signature_input_header,
        format!("ephemeral={}", signature_input("ephemeral", created))
    );

    let body = body_bytes(response).await;
    assert_eq!(body, b"signed body");
    assert_eq!(digest, content_digest(&body));

    attested_key
        .verify(
            &signature_base(
                "GET",
                "/chain",
                StatusCode::OK,
                &digest,
                "ephemeral",
                created,
            ),
            &signature_bytes(&signature_header, "ephemeral"),
        )
        .expect("ephemeral signature should verify with the attested key");
}

#[tokio::test]
async fn tvc_utils_verifier_accepts_the_emitted_trust_chain() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let (member, pair) = fake_keyed_member("member");
    let envelope = FakeManifestBuilder::new().build_envelope_approved_by(&[(member, pair)]);
    let manifest = envelope.clone().manifest();
    // NSM whose PCR bank matches the manifest measurements.
    let nsm = MockNsm::new()
        .with_pcr(0, manifest.enclave().pcr0.clone())
        .with_pcr(1, manifest.enclave().pcr1.clone())
        .with_pcr(2, manifest.enclave().pcr2.clone())
        .with_pcr(3, manifest.enclave().pcr3.clone());
    let layer = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .nsm(Arc::new(nsm))
        .manifest_envelope(envelope.clone())
        .build()
        .expect("layer should build");

    let response = oneshot_response(&layer, "GET", "/verified").await;

    let attested_key = verify_attestation_and_manifest(
        &STANDARD
            .decode(header_str(&response, ATTESTATION_DOC_HEADER))
            .expect("attestation header should be base64"),
        &STANDARD
            .decode(header_str(&response, MANIFEST_ENVELOPE_HEADER))
            .expect("manifest header should be base64"),
        MOCK_ROOT_CERT_DER,
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new()
            .namespace_name("fake-namespace")
            .nonce(1)
            .quorum_key(manifest.namespace().quorum_key.clone())
            .qos_commit("fake-qos-commit")
            .pcrs(
                manifest.enclave().pcr0.clone(),
                manifest.enclave().pcr1.clone(),
                manifest.enclave().pcr2.clone(),
                manifest.enclave().pcr3.clone(),
            )
            .pivot_hash(*manifest.pivot_hash())
            .manifest_hash(envelope.manifest_hash()),
    )
    .unwrap_or_else(|e| panic!("emitted trust chain should verify: {e}"));

    // The returned ephemeral key verifies the response signature.
    let digest = header_str(&response, "content-digest").to_owned();
    let signature_input_header = header_str(&response, "signature-input").to_owned();
    let signature_header = header_str(&response, "signature").to_owned();
    let created = created_from_signature_input(&signature_input_header, "ephemeral");
    attested_key
        .verify(
            &signature_base(
                "GET",
                "/verified",
                StatusCode::OK,
                &digest,
                "ephemeral",
                created,
            ),
            &signature_bytes(&signature_header, "ephemeral"),
        )
        .expect("ephemeral signature should verify with the attested key");
}

#[tokio::test]
async fn quorum_signature_included_and_verifies_when_configured() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("ephemeral key should generate"));
    let quorum_key = Arc::new(P256Pair::generate().expect("quorum key should generate"));
    let envelope = fake_manifest_envelope();
    let layer = signed_layer(&ephemeral_key, Some(&quorum_key), &envelope);

    let response = oneshot_response(&layer, "GET", "/quorum").await;

    let digest = header_str(&response, "content-digest").to_owned();
    let signature_input_header = header_str(&response, "signature-input").to_owned();
    let signature_header = header_str(&response, "signature").to_owned();
    let created = created_from_signature_input(&signature_input_header, "ephemeral");
    assert_eq!(
        created_from_signature_input(&signature_input_header, "quorum"),
        created
    );
    assert_eq!(
        signature_input_header,
        format!(
            "ephemeral={}, quorum={}",
            signature_input("ephemeral", created),
            signature_input("quorum", created)
        )
    );

    let quorum_public = P256Public::from_bytes(&quorum_key.public_key().to_bytes())
        .expect("quorum public key should decode");
    quorum_public
        .verify(
            &signature_base("GET", "/quorum", StatusCode::OK, &digest, "quorum", created),
            &signature_bytes(&signature_header, "quorum"),
        )
        .expect("quorum signature should verify");
}

#[tokio::test]
async fn no_quorum_entries_without_quorum_key() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let envelope = fake_manifest_envelope();
    let layer = signed_layer(&ephemeral_key, None, &envelope);

    let response = oneshot_response(&layer, "GET", "/no-quorum").await;

    assert!(!header_str(&response, "signature-input").contains("quorum="));
    assert!(!header_str(&response, "signature").contains("quorum="));
}

#[tokio::test]
async fn no_public_key_header_is_emitted() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let envelope = fake_manifest_envelope();
    let layer = signed_layer(&ephemeral_key, None, &envelope);

    let response = oneshot_response(&layer, "GET", "/no-pk-header").await;

    assert!(
        !response
            .headers()
            .contains_key("x-tvc-ephemeral-public-key")
    );
    assert!(!response.headers().contains_key("x-tvc-ephemeral-signature"));
    assert!(!response.headers().contains_key("x-tvc-quorum-signature"));
    assert!(!response.headers().contains_key("x-tvc-signature-timestamp"));
}

#[tokio::test]
async fn tampered_signature_bases_fail_verification() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let public_key = P256Public::from_bytes(&ephemeral_key.public_key().to_bytes())
        .expect("public key should decode");
    let envelope = fake_manifest_envelope();
    let layer = signed_layer(&ephemeral_key, None, &envelope);

    let response = oneshot_response(&layer, "POST", "/tamper").await;

    let digest = header_str(&response, "content-digest").to_owned();
    let signature_input_header = header_str(&response, "signature-input").to_owned();
    let signature_header = header_str(&response, "signature").to_owned();
    let created = created_from_signature_input(&signature_input_header, "ephemeral");
    let body = body_bytes(response).await;
    let signature = signature_bytes(&signature_header, "ephemeral");

    public_key
        .verify(
            &signature_base(
                "POST",
                "/tamper",
                StatusCode::OK,
                &digest,
                "ephemeral",
                created,
            ),
            &signature,
        )
        .expect("signature should verify over expected signing payload");
    for tampered_base in [
        signature_base(
            "GET",
            "/tamper",
            StatusCode::OK,
            &digest,
            "ephemeral",
            created,
        ),
        signature_base(
            "POST",
            "/other",
            StatusCode::OK,
            &digest,
            "ephemeral",
            created,
        ),
        signature_base(
            "POST",
            "/tamper",
            StatusCode::NOT_FOUND,
            &digest,
            "ephemeral",
            created,
        ),
        signature_base(
            "POST",
            "/tamper",
            StatusCode::OK,
            &content_digest(b"tampered body"),
            "ephemeral",
            created,
        ),
        signature_base(
            "POST",
            "/tamper",
            StatusCode::OK,
            &content_digest(&body),
            "ephemeral",
            created + 1,
        ),
    ] {
        assert!(
            public_key.verify(&tampered_base, &signature).is_err(),
            "tampered signature base should fail verification"
        );
    }
}

#[tokio::test]
async fn attestation_doc_is_cached_across_responses() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));
    let nsm = Arc::new(CountingNsm::new());
    let layer = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .nsm(Arc::clone(&nsm) as Arc<dyn NsmProvider + Send + Sync>)
        .manifest_envelope(fake_manifest_envelope())
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

#[tokio::test]
async fn builder_requires_ephemeral_key_nsm_and_manifest() {
    let ephemeral_key = Arc::new(P256Pair::generate().expect("key should generate"));

    let missing_ephemeral = ResponseSigningLayer::builder()
        .nsm(Arc::new(MockNsm::new()))
        .manifest_envelope(fake_manifest_envelope())
        .build();
    assert!(matches!(
        missing_ephemeral,
        Err(tvc_axum::Error::MissingEphemeralKey)
    ));

    let missing_nsm = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .manifest_envelope(fake_manifest_envelope())
        .build();
    assert!(matches!(missing_nsm, Err(tvc_axum::Error::MissingNsm)));

    let missing_manifest = ResponseSigningLayer::builder()
        .ephemeral_key(Arc::clone(&ephemeral_key))
        .nsm(Arc::new(MockNsm::new()))
        .build();
    assert!(matches!(
        missing_manifest,
        Err(tvc_axum::Error::MissingManifestEnvelope)
    ));
}
