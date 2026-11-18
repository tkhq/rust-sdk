use serde_json::{Value, json};
use turnkey_client::generated::external::data::v1::{
    AppProofPayload, AppProofRequestContext, AppProofType, BootProof,
    TvcQuorumKeyCreateProofPayload, app_proof_payload::ProofPayload,
};

#[test]
fn app_proof_context_accepts_missing_and_null_and_round_trips_when_present() {
    let legacy = json!({
        "type": "APP_PROOF_TYPE_ADDRESS_DERIVATION",
        "timestampMs": "100",
    });
    let mut explicit_null = legacy.clone();
    explicit_null["requestContext"] = Value::Null;
    for input in [legacy, explicit_null] {
        let payload: AppProofPayload = serde_json::from_value(input).unwrap();
        assert_eq!(payload.request_context, None);
        let encoded = serde_json::to_value(&payload).unwrap();
        assert_eq!(encoded.get("requestContext"), Some(&Value::Null));
    }

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
fn boot_endorsement_accepts_missing_and_null_and_round_trips_when_present() {
    let legacy = json!({
        "ephemeralPublicKeyHex":"ephemeral", "awsAttestationDocB64":"AAECAw==",
        "qosManifestB64":"manifest", "qosManifestEnvelopeB64":"envelope",
        "deploymentLabel":"historical", "enclaveApp":"signer", "owner":"turnkey"
    });
    let mut explicit_null = legacy.clone();
    explicit_null["quorumKeyAttestationSignatureHex"] = Value::Null;
    for input in [legacy.clone(), explicit_null] {
        let boot: BootProof = serde_json::from_value(input).unwrap();
        assert_eq!(boot.quorum_key_attestation_signature_hex, None);
        let encoded = serde_json::to_value(&boot).unwrap();
        assert_eq!(
            encoded.get("quorumKeyAttestationSignatureHex"),
            Some(&Value::Null)
        );
    }

    let boot = BootProof {
        quorum_key_attestation_signature_hex: Some("ab".repeat(64)),
        ..serde_json::from_value(legacy).unwrap()
    };
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
