use qos_core::protocol::ProtocolError;
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use qos_nsm::NsmProvider;
use qos_nsm::mock::{MOCK_ROOT_CERT_DER, MOCK_SECONDS_SINCE_EPOCH, MockNsm};
use qos_nsm::types::{NsmRequest, NsmResponse};
use qos_p256::P256Pair;
use tvc_utils::{
    FakeManifestBuilder, VerificationExpectations, VerifyError, fake_keyed_member, fake_member,
    verify_attestation_and_manifest,
};

/// A fake trust chain: an approved manifest envelope, an attestation document
/// bound to its manifest hash and an ephemeral key, all backed by the NSM
/// mock PKI.
struct TrustChain {
    envelope: VersionedManifestEnvelope,
    envelope_bytes: Vec<u8>,
    attestation_doc: Vec<u8>,
    ephemeral_key: P256Pair,
}

impl TrustChain {
    fn new() -> Self {
        Self::with_builder(FakeManifestBuilder::new())
    }

    fn with_builder(builder: FakeManifestBuilder) -> Self {
        let (member, pair) = fake_keyed_member("member");
        let envelope = builder.build_envelope_approved_by(&[(member, pair)]);
        Self::with_envelope(envelope)
    }

    fn with_envelope(envelope: VersionedManifestEnvelope) -> Self {
        let ephemeral_key = P256Pair::generate().expect("key should generate");
        let user_data = envelope.manifest_hash().to_vec();
        let attestation_doc = attest(
            &envelope,
            Some(user_data),
            Some(ephemeral_key.public_key().to_bytes()),
        );
        let envelope_bytes = envelope
            .to_storage_vec()
            .expect("envelope should serialize");
        Self {
            envelope,
            envelope_bytes,
            attestation_doc,
            ephemeral_key,
        }
    }

    fn verify(&self, expectations: VerificationExpectations) -> Result<Vec<u8>, VerifyError> {
        verify_bytes(
            &self.attestation_doc,
            &self.envelope_bytes,
            MOCK_SECONDS_SINCE_EPOCH,
            &expectations,
        )
    }
}

/// Call [`verify_attestation_and_manifest`] against the NSM mock PKI root
/// and map the returned key to bytes ([`qos_p256::P256Public`] is not
/// `Debug`).
fn verify_bytes(
    attestation_doc: &[u8],
    manifest_envelope: &[u8],
    validation_time_secs: u64,
    expectations: &VerificationExpectations,
) -> Result<Vec<u8>, VerifyError> {
    verify_attestation_and_manifest(
        attestation_doc,
        manifest_envelope,
        MOCK_ROOT_CERT_DER,
        validation_time_secs,
        expectations,
    )
    .map(|key| key.to_bytes())
}

/// Request an attestation document over the envelope's manifest PCRs from a
/// mock NSM.
fn attest(
    envelope: &VersionedManifestEnvelope,
    user_data: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
) -> Vec<u8> {
    let manifest = envelope.clone().manifest();
    let enclave = manifest.enclave();
    let nsm = MockNsm::new()
        .with_pcr(0, enclave.pcr0.clone())
        .with_pcr(1, enclave.pcr1.clone())
        .with_pcr(2, enclave.pcr2.clone())
        .with_pcr(3, enclave.pcr3.clone());
    match nsm.nsm_process_request(NsmRequest::Attestation {
        user_data,
        nonce: None,
        public_key,
    }) {
        NsmResponse::Attestation { document } => document,
        other => panic!("unexpected NSM response: {other:?}"),
    }
}

#[test]
fn full_expectations_pass_and_return_the_ephemeral_key() {
    let chain = TrustChain::with_builder(
        FakeManifestBuilder::new()
            .namespace_name("verify-namespace")
            .nonce(7)
            .qos_commit("verify-qos-commit"),
    );
    let manifest = chain.envelope.clone().manifest();

    let key_bytes = chain
        .verify(
            VerificationExpectations::new()
                .namespace_name("verify-namespace")
                .nonce(7)
                .quorum_key(manifest.namespace().quorum_key.clone())
                .qos_commit("verify-qos-commit")
                .pcrs(
                    manifest.enclave().pcr0.clone(),
                    manifest.enclave().pcr1.clone(),
                    manifest.enclave().pcr2.clone(),
                    manifest.enclave().pcr3.clone(),
                )
                .pivot_hash(*manifest.pivot_hash())
                .manifest_hash(chain.envelope.manifest_hash()),
        )
        .expect("full expectations should verify");

    assert_eq!(key_bytes, chain.ephemeral_key.public_key().to_bytes());
}

#[test]
fn empty_expectations_still_verify_the_trust_chain() {
    let chain = TrustChain::new();

    let key_bytes = chain
        .verify(VerificationExpectations::new())
        .expect("trust chain should verify without expectations");

    assert_eq!(key_bytes, chain.ephemeral_key.public_key().to_bytes());
}

#[test]
fn each_supplied_expectation_is_checked() {
    let chain = TrustChain::new();

    for (expectations, name) in [
        (
            VerificationExpectations::new().namespace_name("other-namespace"),
            "namespace name",
        ),
        (VerificationExpectations::new().nonce(1337), "nonce"),
        (
            VerificationExpectations::new().quorum_key(vec![9; 65]),
            "quorum key",
        ),
        (
            VerificationExpectations::new().qos_commit("other-commit"),
            "qos commit",
        ),
        (VerificationExpectations::new().pcr0(vec![9; 48]), "pcr0"),
        (VerificationExpectations::new().pcr1(vec![9; 48]), "pcr1"),
        (VerificationExpectations::new().pcr2(vec![9; 48]), "pcr2"),
        (VerificationExpectations::new().pcr3(vec![9; 48]), "pcr3"),
        (
            VerificationExpectations::new().pivot_hash([9; 32]),
            "pivot hash",
        ),
        (
            VerificationExpectations::new().manifest_hash([9; 32]),
            "manifest hash",
        ),
    ] {
        let err = chain
            .verify(expectations)
            .expect_err(&format!("wrong {name} should fail"));
        let matched = matches!(
            (name, &err),
            ("namespace name", VerifyError::NamespaceNameMismatch { .. })
                | ("nonce", VerifyError::NonceMismatch { .. })
                | ("quorum key", VerifyError::QuorumKeyMismatch { .. })
                | ("qos commit", VerifyError::QosCommitMismatch { .. })
                | ("pcr0", VerifyError::PcrMismatch { index: 0, .. })
                | ("pcr1", VerifyError::PcrMismatch { index: 1, .. })
                | ("pcr2", VerifyError::PcrMismatch { index: 2, .. })
                | ("pcr3", VerifyError::PcrMismatch { index: 3, .. })
                | ("pivot hash", VerifyError::PivotHashMismatch { .. })
                | ("manifest hash", VerifyError::ManifestHashMismatch { .. })
        );
        assert!(matched, "wrong {name} should map to its own error: {err:?}");
    }
}

#[test]
fn attestation_doc_user_data_must_match_the_manifest_hash() {
    let (member, pair) = fake_keyed_member("member");
    let envelope = FakeManifestBuilder::new().build_envelope_approved_by(&[(member, pair)]);
    let ephemeral_key = P256Pair::generate().expect("key should generate");
    // A valid attestation document bound to a different manifest hash.
    let attestation_doc = attest(
        &envelope,
        Some(vec![9; 32]),
        Some(ephemeral_key.public_key().to_bytes()),
    );

    let err = verify_bytes(
        &attestation_doc,
        &envelope
            .to_storage_vec()
            .expect("envelope should serialize"),
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new(),
    )
    .expect_err("mismatched user data should fail");

    assert!(matches!(err, VerifyError::UserDataMismatch { .. }));
}

#[test]
fn attestation_doc_pcrs_must_match_the_manifest() {
    let (member, pair) = fake_keyed_member("member");
    let envelope = FakeManifestBuilder::new().build_envelope_approved_by(&[(member, pair)]);
    let ephemeral_key = P256Pair::generate().expect("key should generate");
    // Attestation document measured over a PCR bank that differs from the
    // manifest (PCR1 is not seeded and defaults to zeros).
    let nsm = MockNsm::new();
    let attestation_doc = match nsm.nsm_process_request(NsmRequest::Attestation {
        user_data: Some(envelope.manifest_hash().to_vec()),
        nonce: None,
        public_key: Some(ephemeral_key.public_key().to_bytes()),
    }) {
        NsmResponse::Attestation { document } => document,
        other => panic!("unexpected NSM response: {other:?}"),
    };

    let err = verify_bytes(
        &attestation_doc,
        &envelope
            .to_storage_vec()
            .expect("envelope should serialize"),
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new(),
    )
    .expect_err("attestation PCRs differing from the manifest should fail");

    assert!(matches!(
        err,
        VerifyError::AttestationPcrMismatch { index: 1, .. }
    ));
}

#[test]
fn attestation_doc_must_verify_against_the_root_ca() {
    let chain = TrustChain::new();

    // Validation time far past the mock PKI certificate expiry.
    let err = verify_bytes(
        &chain.attestation_doc,
        &chain.envelope_bytes,
        MOCK_SECONDS_SINCE_EPOCH + 60 * 60 * 24 * 365 * 100,
        &VerificationExpectations::new(),
    )
    .expect_err("expired certificate chain should fail");
    assert!(matches!(err, VerifyError::AttestationDoc(_)));

    let err = verify_bytes(
        &[0; 32],
        &chain.envelope_bytes,
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new(),
    )
    .expect_err("garbage attestation document should fail");
    assert!(matches!(err, VerifyError::AttestationDoc(_)));
}

#[test]
fn manifest_set_approvals_must_meet_the_threshold() {
    // No approvals at all.
    let chain = TrustChain::with_envelope(FakeManifestBuilder::new().build_envelope());
    let err = chain
        .verify(VerificationExpectations::new())
        .expect_err("envelope without approvals should fail");
    assert!(matches!(
        err,
        VerifyError::ManifestSetApprovals(ProtocolError::NotEnoughApprovals)
    ));

    // One approval where the manifest set requires two.
    let (member_a, pair_a) = fake_keyed_member("member-a");
    let (member_b, _) = fake_keyed_member("member-b");
    let chain = TrustChain::with_envelope(
        FakeManifestBuilder::new()
            .manifest_set(2, vec![member_a.clone(), member_b])
            .build_envelope_approved_by(&[(member_a, pair_a)]),
    );
    let err = chain
        .verify(VerificationExpectations::new())
        .expect_err("sub-threshold approvals should fail");
    assert!(matches!(
        err,
        VerifyError::ManifestSetApprovals(ProtocolError::NotEnoughApprovals)
    ));
}

#[test]
fn approvals_from_outside_the_manifest_set_fail() {
    let (outsider, outsider_pair) = fake_keyed_member("outsider");
    let chain = TrustChain::with_envelope(
        FakeManifestBuilder::new()
            .manifest_set(1, vec![fake_member("insider")])
            .build_envelope_approved_by(&[(outsider, outsider_pair)]),
    );

    let err = chain
        .verify(VerificationExpectations::new())
        .expect_err("approval from a non-member should fail");

    assert!(matches!(
        err,
        VerifyError::ManifestSetApprovals(ProtocolError::NotManifestSetMember)
    ));
}

#[test]
fn undecodable_manifest_envelope_fails() {
    let chain = TrustChain::new();

    let err = verify_bytes(
        &chain.attestation_doc,
        b"not a manifest envelope",
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new(),
    )
    .expect_err("undecodable envelope should fail");

    assert!(matches!(err, VerifyError::ManifestEnvelopeDecode(_)));
}

#[test]
fn attestation_doc_without_a_public_key_fails() {
    let (member, pair) = fake_keyed_member("member");
    let envelope = FakeManifestBuilder::new().build_envelope_approved_by(&[(member, pair)]);
    let attestation_doc = attest(&envelope, Some(envelope.manifest_hash().to_vec()), None);

    let err = verify_bytes(
        &attestation_doc,
        &envelope
            .to_storage_vec()
            .expect("envelope should serialize"),
        MOCK_SECONDS_SINCE_EPOCH,
        &VerificationExpectations::new(),
    )
    .expect_err("attestation document without a public key should fail");

    assert!(matches!(err, VerifyError::MissingPublicKey));
}
