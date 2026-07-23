//! Axum adapters for Turnkey Verifiable Cloud applications.
//!
//! [`ResponseSigningLayer`] signs every HTTP response with the enclave's
//! ephemeral key (and optionally the quorum key) using RFC 9421 HTTP Message
//! Signatures with an RFC 9530 `Content-Digest`, and attaches the NSM
//! attestation document and QOS manifest envelope as response headers.
//!
//! A client can verify the full trust chain for a response:
//!
//! 1. Decode the manifest envelope from [`MANIFEST_ENVELOPE_HEADER`] and
//!    check it against known-good measurements.
//! 2. Verify the attestation document from [`ATTESTATION_DOC_HEADER`] up to
//!    the AWS Nitro root and check that its `user_data` equals the manifest
//!    hash. The document is requested from the NSM with `user_data` set to
//!    the manifest hash and `public_key` set to the ephemeral public key.
//! 3. Extract the ephemeral public key from the attestation document's
//!    `public_key` field and verify the response signature with it. The
//!    ephemeral public key is intentionally never sent as its own header:
//!    verifiers must take it from the attestation document.
//!
//! [`verify_response_trust_chain`] and [`verify_quorum_signature`] implement
//! the client side of this scheme.
//!
//! [`QosJson`] is a response adapter that serializes bodies with `qos_json`
//! canonical JSON so signed bytes are reproducible.

#![deny(missing_docs)]

mod verify;

pub use verify::{
    ManifestCommitmentKind, SignedResponseParts, VerificationExpectations, VerifyError,
    VerifyResponseError, verify_quorum_signature, verify_response_trust_chain,
};

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::body::{Body, Bytes, HttpBody};
use axum::http::{HeaderValue, Request, Response, StatusCode, header};
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use http_body_util::BodyExt;
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use qos_nsm::NsmProvider;
use qos_nsm::types::{NsmRequest, NsmResponse};
use qos_p256::P256Pair;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Response header carrying the base64 (standard alphabet) encoded NSM
/// attestation document. Its `user_data` is the manifest hash and its
/// `public_key` is the enclave's ephemeral public key.
pub const ATTESTATION_DOC_HEADER: &str = "x-tvc-attestation-doc";

/// Response header carrying the base64 (standard alphabet) encoded QOS
/// manifest envelope in its canonical storage encoding
/// ([`VersionedManifestEnvelope::to_storage_vec`]).
pub const MANIFEST_ENVELOPE_HEADER: &str = "x-tvc-manifest-envelope";

/// How long a fetched attestation document is reused before requesting a
/// fresh one from the NSM. The document binds only values that are fixed for
/// the lifetime of the layer (manifest hash and ephemeral public key), so
/// re-requesting only refreshes its timestamp.
const ATTESTATION_DOC_TTL: Duration = Duration::from_secs(300);

const SIGNATURE_COMPONENTS: &str =
    "(\"@method\";req \"@path\";req \"@query\";req \"@status\" \"content-digest\")";
const SIGNATURE_ALG: &str = "ecdsa-p256-sha256";

/// Errors from building a [`ResponseSigningLayer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The builder was not given an ephemeral key.
    #[error("an ephemeral key is required to sign responses")]
    MissingEphemeralKey,
    /// The builder was not given an NSM provider.
    #[error("an NSM provider is required to fetch attestation documents")]
    MissingNsm,
    /// The builder was not given a manifest envelope.
    #[error("a manifest envelope is required to build attestation headers")]
    MissingManifestEnvelope,
    /// The manifest envelope could not be serialized.
    #[error("failed to serialize manifest envelope: {0}")]
    ManifestSerialization(String),
    /// The NSM returned an unexpected response to an attestation request.
    #[error("failed to fetch attestation document: {0}")]
    Attestation(String),
}

fn unix_timestamp() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .ok()
}

pub(crate) fn content_digest(body: &[u8]) -> String {
    format!("sha-256=:{}:", STANDARD.encode(Sha256::digest(body)))
}

fn signature_input(label: &str, created: u64) -> String {
    format!(r#"{SIGNATURE_COMPONENTS};created={created};keyid="{label}";alg="{SIGNATURE_ALG}""#)
}

pub(crate) fn signature_base(
    method: &str,
    path: &str,
    query: Option<&str>,
    status: StatusCode,
    digest: &str,
    params: &str,
) -> Vec<u8> {
    let query = query.map_or_else(|| "?".to_owned(), |query| format!("?{query}"));
    format!(
        "\"@method\";req: {method}\n\"@path\";req: {path}\n\"@query\";req: {query}\n\"@status\": {}\n\"content-digest\": {digest}\n\"@signature-params\": {params}",
        status.as_u16(),
    )
    .into_bytes()
}

fn internal_error_response(message: &'static str) -> Response<Body> {
    let mut response = Response::new(Body::from(format!(r#"{{"error":"{message}"}}"#)));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

/// Axum response adapter that serializes response bodies with `qos_json`
/// canonical JSON.
pub struct QosJson<T>(pub T);

impl<T> IntoResponse for QosJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match qos_json::to_vec(&self.0) {
            Ok(bytes) => {
                let mut response = Response::new(Body::from(bytes));
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                response
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"error":"serialization failed"}"#,
            )
                .into_response(),
        }
    }
}

/// Request an attestation document binding `user_data` and `public_key`.
fn request_attestation_doc(
    nsm: &dyn NsmProvider,
    user_data: Vec<u8>,
    public_key: Vec<u8>,
) -> Result<Vec<u8>, Error> {
    match nsm.nsm_process_request(NsmRequest::Attestation {
        user_data: Some(user_data),
        nonce: None,
        public_key: Some(public_key),
    }) {
        NsmResponse::Attestation { document } => Ok(document),
        other => Err(Error::Attestation(format!(
            "unexpected NSM response: {other:?}"
        ))),
    }
}

struct AttestationCache {
    header: HeaderValue,
    fetched_at: Instant,
}

struct Shared {
    ephemeral_key: Arc<P256Pair>,
    quorum_key: Option<Arc<P256Pair>>,
    nsm: Arc<dyn NsmProvider + Send + Sync>,
    manifest_hash: [u8; 32],
    ephemeral_public_key: Vec<u8>,
    manifest_envelope_header: HeaderValue,
    attestation_cache: Mutex<AttestationCache>,
}

impl Shared {
    /// Return the attestation document header, requesting a fresh document
    /// from the NSM when the cached one is older than
    /// [`ATTESTATION_DOC_TTL`].
    fn attestation_header(&self) -> Result<HeaderValue, Error> {
        let mut cache = self
            .attestation_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.fetched_at.elapsed() >= ATTESTATION_DOC_TTL {
            cache.header = fetch_attestation_header(
                &*self.nsm,
                &self.manifest_hash,
                &self.ephemeral_public_key,
            )?;
            cache.fetched_at = Instant::now();
        }
        Ok(cache.header.clone())
    }

    async fn sign_response<ResBody>(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        response: Response<ResBody>,
    ) -> Result<Response<Body>, &'static str>
    where
        ResBody: HttpBody<Data = Bytes>,
        ResBody::Error: std::fmt::Display,
    {
        let (mut parts, body) = response.into_parts();
        let body = body
            .collect()
            .await
            .map_err(|_| "failed to read response body")?
            .to_bytes();
        let created = unix_timestamp().ok_or("failed to read system time")?;
        let digest = content_digest(&body);
        let sign = |label: &str, key: &P256Pair| {
            let params = signature_input(label, created);
            let base = signature_base(method, path, query, parts.status, &digest, &params);
            let signature = key.sign(&base).ok()?;
            Some((
                format!("{label}={params}"),
                format!("{label}=:{}:", STANDARD.encode(signature)),
            ))
        };

        let ephemeral = sign("ephemeral", &self.ephemeral_key)
            .ok_or("failed to sign response with ephemeral key")?;
        let mut signature_inputs = vec![ephemeral.0];
        let mut signatures = vec![ephemeral.1];

        if let Some(quorum_key) = &self.quorum_key {
            let quorum =
                sign("quorum", quorum_key).ok_or("failed to sign response with quorum key")?;
            signature_inputs.push(quorum.0);
            signatures.push(quorum.1);
        }

        parts.headers.insert(
            "content-digest",
            HeaderValue::from_str(&digest).map_err(|_| "failed to encode content digest")?,
        );
        parts.headers.insert(
            "signature-input",
            HeaderValue::from_str(&signature_inputs.join(", "))
                .map_err(|_| "failed to encode signature input")?,
        );
        parts.headers.insert(
            "signature",
            HeaderValue::from_str(&signatures.join(", "))
                .map_err(|_| "failed to encode signature")?,
        );
        parts.headers.insert(
            ATTESTATION_DOC_HEADER,
            self.attestation_header()
                .map_err(|_| "failed to fetch attestation document")?,
        );
        parts.headers.insert(
            MANIFEST_ENVELOPE_HEADER,
            self.manifest_envelope_header.clone(),
        );

        Ok(Response::from_parts(parts, Body::from(body)))
    }
}

fn fetch_attestation_header(
    nsm: &dyn NsmProvider,
    manifest_hash: &[u8; 32],
    ephemeral_public_key: &[u8],
) -> Result<HeaderValue, Error> {
    let document =
        request_attestation_doc(nsm, manifest_hash.to_vec(), ephemeral_public_key.to_vec())?;
    HeaderValue::from_str(&STANDARD.encode(document))
        .map_err(|e| Error::Attestation(format!("failed to encode attestation header: {e}")))
}

/// Tower layer that signs responses with the enclave's TVC P-256 keys and
/// attaches the attestation document and manifest envelope headers.
#[derive(Clone)]
pub struct ResponseSigningLayer {
    shared: Arc<Shared>,
}

impl ResponseSigningLayer {
    /// Create a builder for response signing middleware.
    #[must_use]
    pub fn builder() -> ResponseSigningLayerBuilder {
        ResponseSigningLayerBuilder::default()
    }
}

/// Builder for [`ResponseSigningLayer`].
#[derive(Default)]
pub struct ResponseSigningLayerBuilder {
    ephemeral_key: Option<Arc<P256Pair>>,
    quorum_key: Option<Arc<P256Pair>>,
    nsm: Option<Arc<dyn NsmProvider + Send + Sync>>,
    manifest_envelope: Option<VersionedManifestEnvelope>,
}

impl ResponseSigningLayerBuilder {
    /// Sign responses with the enclave's ephemeral key. Required.
    #[must_use]
    pub fn ephemeral_key(mut self, key: Arc<P256Pair>) -> Self {
        self.ephemeral_key = Some(key);
        self
    }

    /// Additionally sign responses with the quorum key. Optional.
    #[must_use]
    pub fn quorum_key(mut self, key: Arc<P256Pair>) -> Self {
        self.quorum_key = Some(key);
        self
    }

    /// NSM used to request attestation documents. Required.
    #[must_use]
    pub fn nsm(mut self, nsm: Arc<dyn NsmProvider + Send + Sync>) -> Self {
        self.nsm = Some(nsm);
        self
    }

    /// Manifest envelope the enclave booted with. Required. Its hash is
    /// bound into attestation documents as `user_data` and its canonical
    /// bytes are attached to responses.
    #[must_use]
    pub fn manifest_envelope(mut self, envelope: VersionedManifestEnvelope) -> Self {
        self.manifest_envelope = Some(envelope);
        self
    }

    /// Build the response signing layer. Requests an initial attestation
    /// document from the NSM so misconfiguration fails here rather than on
    /// the first request.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if a required input is missing, the manifest
    /// envelope cannot be serialized, or the NSM attestation request fails.
    pub fn build(self) -> Result<ResponseSigningLayer, Error> {
        let ephemeral_key = self.ephemeral_key.ok_or(Error::MissingEphemeralKey)?;
        let nsm = self.nsm.ok_or(Error::MissingNsm)?;
        let manifest_envelope = self
            .manifest_envelope
            .ok_or(Error::MissingManifestEnvelope)?;

        let manifest_bytes = manifest_envelope
            .to_storage_vec()
            .map_err(|e| Error::ManifestSerialization(e.to_string()))?;
        let manifest_envelope_header = HeaderValue::from_str(&STANDARD.encode(manifest_bytes))
            .map_err(|e| Error::ManifestSerialization(e.to_string()))?;
        let manifest_hash = manifest_envelope.manifest_hash();
        let ephemeral_public_key = ephemeral_key.public_key().to_bytes();

        let attestation_header =
            fetch_attestation_header(&*nsm, &manifest_hash, &ephemeral_public_key)?;

        Ok(ResponseSigningLayer {
            shared: Arc::new(Shared {
                ephemeral_key,
                quorum_key: self.quorum_key,
                nsm,
                manifest_hash,
                ephemeral_public_key,
                manifest_envelope_header,
                attestation_cache: Mutex::new(AttestationCache {
                    header: attestation_header,
                    fetched_at: Instant::now(),
                }),
            }),
        })
    }
}

impl<S> tower::Layer<S> for ResponseSigningLayer {
    type Service = ResponseSigningService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseSigningService {
            inner,
            shared: Arc::clone(&self.shared),
        }
    }
}

/// Tower service produced by [`ResponseSigningLayer`].
#[derive(Clone)]
pub struct ResponseSigningService<S> {
    inner: S,
    shared: Arc<Shared>,
}

impl<S, ReqBody, ResBody> tower::Service<Request<ReqBody>> for ResponseSigningService<S>
where
    S: tower::Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: std::fmt::Display,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let method = request.method().as_str().to_owned();
        let path = request.uri().path().to_owned();
        let query = request.uri().query().map(str::to_owned);
        let future = self.inner.call(request);
        let shared = Arc::clone(&self.shared);

        Box::pin(async move {
            let response = future.await?;
            Ok(shared
                .sign_response(&method, &path, query.as_deref(), response)
                .await
                .unwrap_or_else(internal_error_response))
        })
    }
}
