use turnkey_client::generated::external::data::v1::{
    AppProofPayload, AppProofRequestContext, AppProofType, BootProof,
    TvcQuorumKeyCreateProofPayload, app_proof_payload::ProofPayload,
};
#[test]
fn absent_legacy_context_is_omitted_and_new_schema_fields_round_trip() {
    let legacy = AppProofPayload {
        r#type: AppProofType::AddressDerivation,
        timestamp_ms: "100".into(),
        request_context: None,
        proof_payload: None,
    };
    let encoded = serde_json::to_value(&legacy).unwrap();
    assert!(encoded.get("requestContext").is_none());
    assert_eq!(
        serde_json::from_value::<AppProofPayload>(encoded).unwrap(),
        legacy
    );
    let quorum = AppProofPayload {
        r#type: AppProofType::TvcQuorumKeyCreate,
        timestamp_ms: "100".into(),
        request_context: Some(AppProofRequestContext {
            version: 1,
            verified_signature_digests: vec!["ab".repeat(32)],
        }),
        proof_payload: Some(ProofPayload::TvcQuorumKeyCreateProof(
            TvcQuorumKeyCreateProofPayload {
                organization_id: "org".into(),
                quorum_key_id: "id".into(),
                quorum_public_key: "key".into(),
                threshold: 2,
                operator_encrypt_keys: vec!["one".into(), "two".into()],
            },
        )),
    };
    assert_eq!(
        serde_json::from_str::<AppProofPayload>(&serde_json::to_string(&quorum).unwrap()).unwrap(),
        quorum
    );
}

#[test]
fn historical_boot_proofs_remain_readable_and_optional_endorsements_round_trip() {
    let legacy = serde_json::json!({
        "ephemeralPublicKeyHex":"ephemeral", "awsAttestationDocB64":"AAECAw==",
        "qosManifestB64":"manifest", "qosManifestEnvelopeB64":"envelope",
        "deploymentLabel":"historical", "enclaveApp":"signer", "owner":"turnkey"
    });
    let mut boot: BootProof = serde_json::from_value(legacy).unwrap();
    assert_eq!(boot.quorum_key_attestation_signature_hex, None);
    assert!(
        serde_json::to_value(&boot)
            .unwrap()
            .get("quorumKeyAttestationSignatureHex")
            .is_none()
    );
    boot.quorum_key_attestation_signature_hex = Some("ab".repeat(64));
    let serialized = serde_json::to_value(&boot).unwrap();
    assert_eq!(
        serialized["quorumKeyAttestationSignatureHex"],
        "ab".repeat(64)
    );
    assert_eq!(
        serde_json::from_value::<BootProof>(serialized).unwrap(),
        boot
    );
}
