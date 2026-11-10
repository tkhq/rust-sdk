use super::*;
mod fixture;
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{sync::Mutex, time::Duration};
use turnkey_client::{
    RetryConfig, TurnkeyClient, TurnkeyP256ApiKey,
    generated::{
        CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent,
        external::data::v1::{AppProof, BootProof, SignatureScheme},
    },
};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate, matchers::method};

fn point(seed: u8) -> String {
    hex::encode(
        SigningKey::from_slice(&[seed; 32])
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes(),
    )
}
fn pin() -> [u8; 130] {
    hex::decode(format!("{}{}", point(8), point(9)))
        .unwrap()
        .try_into()
        .unwrap()
}
fn verifier() -> TvcProvenance {
    TvcProvenance {
        endorsement: endorsement::SelectedEndorsement::with_test_roots(
            fixture::quorum_key(),
            fixture::ROOT.to_vec(),
        ),
    }
}
fn proof(payload: String) -> AppProof {
    let signature: Signature = SigningKey::from_slice(&[9; 32])
        .unwrap()
        .sign(payload.as_bytes());
    AppProof {
        scheme: SignatureScheme::EphemeralKeyP256,
        public_key: hex::encode(pin()),
        proof_payload: payload,
        signature: hex::encode(signature.to_bytes()),
    }
}
fn operator_payload(role: usize, digests: Vec<String>) -> Value {
    json!({"type":"APP_PROOF_TYPE_ADDRESS_DERIVATION", "timestampMs":"1793845997611", "requestContext":{"version":1,"verifiedSignatureDigests":digests}, "addressDerivationProof":{"organizationId":"org-id","walletId":"wallet-1","derivationPath":format!("m/0/{role}"),"address":point(role as u8 + 1)}})
}
fn quorum_payload(digests: Vec<String>) -> Value {
    json!({"type":"APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE", "timestampMs":"1793845997611", "requestContext":{"version":1,"verifiedSignatureDigests":digests}, "tvcQuorumKeyCreateProof":{"organizationId":"org-id","quorumKeyId":"quorum-1","quorumPublicKey":format!("{}{}",point(3),point(4)),"threshold":2,"operatorEncryptKeys":[point(1),point(2)]}})
}
fn digest(request: &Request) -> String {
    let bytes = base64::decode_config(
        request.headers.get("x-stamp").unwrap().to_str().unwrap(),
        base64::URL_SAFE_NO_PAD,
    )
    .unwrap();
    let stamp: Value = serde_json::from_slice(&bytes).unwrap();
    let mut hash = Sha256::new();
    hash.update(1i32.to_le_bytes());
    for value in [
        stamp["publicKey"].as_str().unwrap(),
        std::str::from_utf8(&request.body).unwrap(),
        stamp["signature"].as_str().unwrap(),
    ] {
        hash.update((value.len() as u32).to_le_bytes());
        hash.update(value.as_bytes());
    }
    hex::encode(hash.finalize())
}

#[test]
fn independently_selected_roots_support_production_preprod_and_dev() {
    assert!(TvcProvenance::for_api_url("https://api.turnkey.com").is_ok());
    assert!(TvcProvenance::for_api_url("https://api.preprod.turnkey.engineering").is_ok());
    assert!(TvcProvenance::for_api_url("https://api.dev.turnkey.engineering").is_ok());
    for url in [
        "http://localhost:8081",
        "https://evil.test",
        "https://api.turnkey.com.evil.test",
    ] {
        assert!(matches!(
            TvcProvenance::for_api_url(url),
            Err(Error::UnsupportedEnvironment)
        ));
    }
}

#[test]
fn actual_tkms_claim_bytes_decode_and_verify_under_their_fixture_ephemeral_key() {
    let proofs: Vec<AppProof> = serde_json::from_str(include_str!(
        "../../tests/fixtures/tvc_request_bound_proofs.json"
    ))
    .unwrap();
    assert_eq!(proofs.len(), 3);
    for p in &proofs {
        crate::verify_app_proof_signature(p).unwrap();
        assert!(claims::decode(&p.proof_payload).is_ok());
    }
    let mut changed = proofs[0].clone();
    changed.proof_payload.push(' ');
    assert!(crate::verify_app_proof_signature(&changed).is_err());
}

#[test]
fn authentication_uses_original_bytes_and_both_attested_ephemeral_halves() {
    let original =
        proof(serde_json::to_string_pretty(&operator_payload(0, vec!["ab".repeat(32)])).unwrap());
    assert!(
        verifier()
            .endorsement
            .authenticate(&original, &fixture::boot())
            .is_ok()
    );
    let mut mutations = vec![];
    let mut changed = original.clone();
    changed.proof_payload.push(' ');
    mutations.push(changed);
    let mut changed = original.clone();
    changed.proof_payload =
        serde_json::to_string(&serde_json::from_str::<Value>(&changed.proof_payload).unwrap())
            .unwrap();
    mutations.push(changed);
    let mut changed = original.clone();
    changed.signature.replace_range(..2, "00");
    mutations.push(changed);
    let mut changed = original.clone();
    changed.public_key.replace_range(..130, &point(7));
    mutations.push(changed);
    let mut changed = original.clone();
    changed.public_key.replace_range(130.., &point(7));
    mutations.push(changed);
    let mut changed = original.clone();
    changed.public_key = point(9);
    mutations.push(changed);
    for changed in mutations {
        assert!(
            verifier()
                .endorsement
                .authenticate(&changed, &fixture::boot())
                .is_err()
        );
    }
}

#[test]
fn strict_wire_mutation_matrix() {
    let baseline = operator_payload(0, vec!["ab".repeat(32)]);
    let mut cases = Vec::new();
    for field in [
        "type",
        "timestampMs",
        "requestContext",
        "addressDerivationProof",
    ] {
        let mut v = baseline.clone();
        v.as_object_mut().unwrap().remove(field);
        cases.push(v);
        let mut v = baseline.clone();
        v[field] = Value::Null;
        cases.push(v);
    }
    for field in ["organizationId", "walletId", "derivationPath", "address"] {
        for wrong in [Value::Null, json!(""), json!(2), json!([])] {
            let mut v = baseline.clone();
            v["addressDerivationProof"][field] = wrong;
            cases.push(v);
        }
        let mut v = baseline.clone();
        v["addressDerivationProof"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push(v);
    }
    for wrong in [
        json!(1),
        json!(""),
        json!("+1"),
        json!("1e2"),
        json!("18446744073709551616"),
    ] {
        let mut v = baseline.clone();
        v["timestampMs"] = wrong;
        cases.push(v);
    }
    for wrong in [json!("1"), json!(1.0), json!(-1), Value::Null, json!(2)] {
        let mut v = baseline.clone();
        v["requestContext"]["version"] = wrong;
        cases.push(v);
    }
    for wrong in [
        json!([]),
        json!([null]),
        json!(["AB".repeat(32)]),
        json!(["00"]),
        json!(["gg".repeat(32)]),
        json!(vec!["ab".repeat(32); 1025]),
    ] {
        let mut v = baseline.clone();
        v["requestContext"]["verifiedSignatureDigests"] = wrong;
        cases.push(v);
    }
    let mut v = baseline.clone();
    v["request_context"] = json!({});
    cases.push(v);
    let mut v = baseline.clone();
    v["requestContext"]["unknown"] = json!(true);
    cases.push(v);
    let mut v = baseline.clone();
    v["addressDerivationProof"]["wallet_id"] = json!("wallet-1");
    cases.push(v);
    let mut v = baseline.clone();
    v["tvcQuorumKeyCreateProof"] = json!({});
    cases.push(v);
    let mut v = baseline.clone();
    v["type"] = json!("APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE");
    cases.push(v);
    for (index, v) in cases.into_iter().enumerate() {
        assert!(
            claims::decode(&v.to_string()).is_err(),
            "accepted mutation {index}: {v}"
        );
    }
    let raw = baseline.to_string();
    for invalid in [
        format!("{raw} true"),
        raw.replacen(
            "\"timestampMs\":",
            "\"timestampMs\":\"1\",\"timestampMs\":",
            1,
        ),
        raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1),
        raw.replacen("\"walletId\":", "\"walletId\":\"other\",\"walletId\":", 1),
        raw.replacen("\"version\":1", "\"version\":1e0", 1),
    ] {
        assert!(claims::decode(&invalid).is_err(), "accepted {invalid}");
    }
}

#[tokio::test]
async fn canonical_points_composites_and_maximum_claim_boundaries() {
    for invalid in [
        point(1).to_uppercase(),
        format!(" {}", point(1)),
        format!("0x{}", point(1)),
        format!("02{}", &point(1)[2..66]),
        "00".repeat(65),
        format!("04{}", "ff".repeat(64)),
        format!("{}00", point(1)),
    ] {
        assert!(RolePublicKey::parse(&invalid).is_err());
    }
    for invalid in [
        point(1),
        format!("{}{}", point(1), "00".repeat(65)),
        format!("{}{}", "00".repeat(65), point(1)),
        "é".repeat(130),
        format!("{}{}00", point(1), point(2)),
    ] {
        assert!(claims::composite(&invalid).is_err());
    }
    let mut largest = quorum_payload(vec!["ab".repeat(32); 1024]);
    let recipients: Vec<String> = (1u32..=255)
        .map(|i| {
            let mut scalar = [0; 32];
            scalar[28..].copy_from_slice(&i.to_be_bytes());
            hex::encode(
                SigningKey::from_slice(&scalar)
                    .unwrap()
                    .verifying_key()
                    .to_encoded_point(false)
                    .as_bytes(),
            )
        })
        .collect();
    largest["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] = json!(recipients);
    largest["tvcQuorumKeyCreateProof"]["threshold"] = json!(255);
    assert!(claims::decode(&largest.to_string()).is_ok());
    assert!(largest.to_string().len() < 256 * 1024);
    largest["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"]
        .as_array_mut()
        .unwrap()
        .push(json!(point(7)));
    assert!(claims::decode(&largest.to_string()).is_err());
    let p = proof(format!(
        "{}{}",
        operator_payload(0, vec!["ab".repeat(32)]),
        " ".repeat(256 * 1024)
    ));
    assert!(
        verifier()
            .claims(
                &capture_client("http://127.0.0.1:1".into()),
                "org-id",
                &[p],
                &mut empty_report(1)
            )
            .await
            .is_err()
    );
    assert!(
        verifier()
            .claims(
                &capture_client("http://127.0.0.1:1".into()),
                "org-id",
                &vec![proof(operator_payload(0, vec!["ab".repeat(32)]).to_string()); 65],
                &mut empty_report(65)
            )
            .await
            .is_err()
    );
}

#[derive(Clone, Copy)]
enum Mode {
    Operator,
    Quorum,
    QuorumThreeRecipients,
    Split,
    Common,
    SubstituteWallet,
    SubstituteKey,
    WrongOrg,
    WrongWallet,
    WrongPath,
    SameRoles,
    Duplicate,
    MissingRole,
    Empty,
    PolicyOnly,
    MissingContext,
    WrongType,
    WrongQuorumId,
    QuorumSubstitutedKey,
    ReorderedRecipients,
    Replay,
    EarlierClaim,
}
struct Producer {
    mode: Mode,
    first: Mutex<Option<String>>,
}
impl Respond for Producer {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let sent = digest(request);
        let mut first = self.first.lock().unwrap();
        let pending = matches!(self.mode, Mode::Split | Mode::Common) && first.is_none();
        if first.is_none() {
            *first = Some(sent.clone());
        }
        let first_digest = first.clone().unwrap_or_else(|| sent.clone());
        let mut roles = vec![
            operator_payload(0, vec![sent.clone()]),
            operator_payload(1, vec![sent.clone()]),
        ];
        let mut result = json!({"walletId":"wallet-1","operatorId":"routing-label","encryptPublicKey":point(1),"signPublicKey":point(2)});
        let quorum = matches!(
            self.mode,
            Mode::Quorum
                | Mode::QuorumThreeRecipients
                | Mode::WrongQuorumId
                | Mode::QuorumSubstitutedKey
                | Mode::ReorderedRecipients
        );
        match self.mode {
            Mode::Split => {
                roles[0]["requestContext"]["verifiedSignatureDigests"] = json!([first_digest]);
            }
            Mode::Common => {
                for role in &mut roles {
                    role["requestContext"]["verifiedSignatureDigests"] =
                        json!([first_digest, sent]);
                }
            }
            Mode::SubstituteWallet => result["walletId"] = json!("other-wallet"),
            Mode::SubstituteKey => result["encryptPublicKey"] = json!(point(7)),
            Mode::WrongOrg => roles[0]["addressDerivationProof"]["organizationId"] = json!("other"),
            Mode::WrongWallet => roles[0]["addressDerivationProof"]["walletId"] = json!("other"),
            Mode::WrongPath => {
                roles[0]["addressDerivationProof"]["derivationPath"] = json!("m/0/2")
            }
            Mode::SameRoles => roles[1]["addressDerivationProof"]["address"] = json!(point(1)),
            Mode::Duplicate => roles.push(roles[0].clone()),
            Mode::MissingRole => {
                roles.pop();
            }
            Mode::Empty | Mode::PolicyOnly => roles.clear(),
            Mode::MissingContext => {
                roles[0].as_object_mut().unwrap().remove("requestContext");
            }
            Mode::Replay => {
                for role in &mut roles {
                    role["requestContext"]["verifiedSignatureDigests"] = json!(["00".repeat(32)]);
                }
            }
            Mode::EarlierClaim => {
                for role in &mut roles {
                    role["requestContext"]["verifiedSignatureDigests"] = json!([first_digest]);
                }
            }
            _ => {}
        }
        let mut proofs: Vec<_> = roles.into_iter().map(|v| proof(v.to_string())).collect();
        if quorum {
            let mut payload = quorum_payload(vec![digest(request)]);
            if matches!(self.mode, Mode::QuorumThreeRecipients) {
                payload["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] =
                    json!([point(1), point(2), point(3)]);
            }
            if matches!(self.mode, Mode::ReorderedRecipients) {
                payload["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] =
                    json!([point(2), point(1)]);
            }
            proofs = vec![proof(payload.to_string())];
            let share_ids = if matches!(self.mode, Mode::QuorumThreeRecipients) {
                vec!["s1", "s2", "s3"]
            } else {
                vec!["s1", "s2"]
            };
            result = json!({"quorumKeyId":if matches!(self.mode,Mode::WrongQuorumId){"substituted"}else{"quorum-1"},"quorumPublicKey":if matches!(self.mode,Mode::QuorumSubstitutedKey){format!("{}{}",point(5),point(6))}else{format!("{}{}",point(3),point(4))},"shareIds":share_ids});
        }
        proofs.push(AppProof {
            scheme: SignatureScheme::Unspecified,
            public_key: "unrelated-ump-key".into(),
            proof_payload: r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#.into(),
            signature: String::new(),
        });
        if matches!(self.mode, Mode::Empty) {
            proofs.clear();
        }
        ResponseTemplate::new(200).set_body_json(json!({"activity":{"id":"activity-1","organizationId":"org-id","type":if matches!(self.mode,Mode::WrongType){"ACTIVITY_TYPE_CREATE_WALLET"}else if quorum{"ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY"}else{"ACTIVITY_TYPE_CREATE_TVC_OPERATOR"},"status":if pending{"ACTIVITY_STATUS_PENDING"}else{"ACTIVITY_STATUS_COMPLETED"},"fingerprint":"untrusted","appProofs":proofs,"result":if quorum {json!({"createTvcQuorumKeyResult":result})}else{json!({"createTvcOperatorResult":result})}}}))
    }
}
async fn receipt(mode: Mode) -> ActivityReceipt {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(Producer {
            mode,
            first: Mutex::new(None),
        })
        .mount(&server)
        .await;
    let client = TurnkeyClient::builder()
        .api_key(TurnkeyP256ApiKey::from_bytes([6; 32], None).unwrap())
        .base_url(server.uri())
        .retry_config(RetryConfig {
            initial_delay: Duration::from_millis(1),
            multiplier: 1.0,
            max_delay: Duration::from_millis(1),
            max_retries: 3,
        })
        .build()
        .unwrap()
        .with_app_proofs();
    match mode {
        Mode::Quorum
        | Mode::QuorumThreeRecipients
        | Mode::WrongQuorumId
        | Mode::QuorumSubstitutedKey
        | Mode::ReorderedRecipients => {
            let operator_encrypt_keys = if matches!(mode, Mode::QuorumThreeRecipients) {
                vec![point(1), point(2), point(3)]
            } else {
                vec![point(1), point(2)]
            };
            client
                .create_tvc_quorum_key_with_receipt(
                    "org-id".into(),
                    100,
                    CreateTvcQuorumKeyIntent {
                        threshold: 2,
                        operator_encrypt_keys,
                    },
                )
                .await
                .unwrap()
                .into_parts()
                .1
        }
        _ => {
            client
                .create_tvc_operator_with_receipt(
                    "org-id".into(),
                    100,
                    CreateTvcOperatorIntent {
                        wallet_name: Some("wallet".into()),
                        wallet_id: None,
                        path: "m/0".into(),
                        operator_name: "synthetic-predicate-test".into(),
                    },
                )
                .await
                .unwrap()
                .into_parts()
                .1
        }
    }
}
async fn boot_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/public/v1/query/get_boot_proof"))
        .and(wiremock::matchers::body_json(
            json!({"organizationId":"org-id", "ephemeralKey":hex::encode(pin())}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"bootProof":fixture::boot()})),
        )
        .mount(&server)
        .await;
    server
}
async fn accept_operator(
    expected: &OperatorExpectation,
    receipt: ActivityReceipt,
) -> Result<VerifiedOperatorKeys, Error> {
    let server = boot_server().await;
    verifier()
        .accept_operator(&capture_client(server.uri()), expected, receipt)
        .await
}
async fn accept_quorum(
    expected: &QuorumExpectation,
    receipt: ActivityReceipt,
) -> Result<VerifiedQuorumKey, Error> {
    let server = boot_server().await;
    verifier()
        .accept_quorum(&capture_client(server.uri()), expected, receipt)
        .await
}
fn expected() -> OperatorExpectation {
    OperatorExpectation::parse("org-id".into(), Some("wallet-1".into()), "m/0".into()).unwrap()
}
#[tokio::test]
async fn freshly_signed_capture_accepts_common_intersection_and_rejects_split() {
    for mode in [Mode::Operator, Mode::Common] {
        let accepted = accept_operator(&expected(), receipt(mode).await)
            .await
            .unwrap();
        assert_eq!(accepted.encrypt_key().to_string(), point(1));
        assert_eq!(accepted.sign_key().to_string(), point(2));
    }
    assert!(matches!(
        accept_operator(&expected(), receipt(Mode::Split).await).await,
        Err(Error::ReceiptMismatch)
    ));
}
#[tokio::test]
async fn signed_claims_reject_unsigned_substitutions_conflicts_and_replay() {
    for mode in [
        Mode::SubstituteWallet,
        Mode::SubstituteKey,
        Mode::WrongOrg,
        Mode::WrongWallet,
        Mode::WrongPath,
        Mode::SameRoles,
        Mode::Duplicate,
        Mode::MissingRole,
        Mode::Empty,
        Mode::PolicyOnly,
        Mode::MissingContext,
        Mode::WrongType,
        Mode::Replay,
    ] {
        assert!(
            accept_operator(&expected(), receipt(mode).await)
                .await
                .is_err()
        );
    }
    let wrong_wallet =
        OperatorExpectation::parse("org-id".into(), Some("other".into()), "m/0".into()).unwrap();
    assert!(
        accept_operator(&wrong_wallet, receipt(Mode::Operator).await)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn fresh_quorum_claim_binds_signed_id_complete_key_and_recipient_order() {
    let expected = QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2)]).unwrap();
    let accepted = accept_quorum(&expected, receipt(Mode::Quorum).await)
        .await
        .unwrap();
    assert_eq!(
        accepted.into_parts(),
        (
            "quorum-1".into(),
            format!("{}{}", point(3), point(4)),
            vec!["s1".into(), "s2".into()]
        )
    );
    for mode in [Mode::WrongQuorumId, Mode::ReorderedRecipients] {
        assert!(accept_quorum(&expected, receipt(mode).await).await.is_err());
    }
}

fn capture_client(url: String) -> TurnkeyClient<TurnkeyP256ApiKey> {
    TurnkeyClient::builder()
        .api_key(TurnkeyP256ApiKey::from_bytes([6; 32], None).unwrap())
        .base_url(url)
        .build()
        .unwrap()
}
async fn same_body_receipt(url: String) -> ActivityReceipt {
    capture_client(url)
        .with_app_proofs()
        .create_tvc_operator_with_receipt(
            "org-id".into(),
            100,
            CreateTvcOperatorIntent {
                wallet_name: Some("wallet".into()),
                wallet_id: None,
                path: "m/0".into(),
                operator_name: "same-body-with-clock-rollback".into(),
            },
        )
        .await
        .unwrap()
        .into_parts()
        .1
}
#[tokio::test]
async fn earlier_signed_claim_rejects_a_new_submission_of_the_same_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(Producer {
            mode: Mode::EarlierClaim,
            first: Mutex::new(None),
        })
        .mount(&server)
        .await;
    let first = same_body_receipt(server.uri()).await;
    assert!(accept_operator(&expected(), first).await.is_ok());
    assert!(matches!(
        accept_operator(&expected(), same_body_receipt(server.uri()).await).await,
        Err(Error::ReceiptMismatch)
    ));
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let digests = requests.iter().map(digest).collect::<HashSet<_>>();
    assert_eq!(digests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.body == requests[0].body)
    );
}
#[tokio::test]
async fn mutating_the_separate_result_cannot_substitute_the_receipts_result() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(Producer {
            mode: Mode::Operator,
            first: Mutex::new(None),
        })
        .mount(&server)
        .await;
    let submitted = capture_client(server.uri())
        .with_app_proofs()
        .create_tvc_operator_with_receipt(
            "org-id".into(),
            100,
            CreateTvcOperatorIntent {
                wallet_name: Some("wallet".into()),
                wallet_id: None,
                path: "m/0".into(),
                operator_name: "operator".into(),
            },
        )
        .await
        .unwrap();
    let (mut separate, receipt) = submitted.into_parts();
    separate.result.encrypt_public_key = point(7);
    let accepted = accept_operator(&expected(), receipt).await.unwrap();
    assert_ne!(
        accepted.encrypt_key().to_string(),
        separate.result.encrypt_public_key
    );
    assert_eq!(accepted.encrypt_key().to_string(), point(1));
}

#[test]
fn strict_quorum_required_fields_numeric_forms_aliases_and_duplicates() {
    let baseline = quorum_payload(vec!["ab".repeat(32)]);
    let mut cases = Vec::new();
    for field in [
        "organizationId",
        "quorumKeyId",
        "quorumPublicKey",
        "threshold",
        "operatorEncryptKeys",
    ] {
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push(value);
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"][field] = Value::Null;
        cases.push(value);
    }
    for wrong in [json!("2"), json!(2.0), json!(1), json!(256), json!(-2)] {
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"]["threshold"] = wrong;
        cases.push(value);
    }
    for wrong in [
        json!([]),
        json!([null]),
        json!([point(1), point(1)]),
        json!("keys"),
        json!([point(1), point(2).to_uppercase()]),
    ] {
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] = wrong;
        cases.push(value);
    }
    let mut value = baseline.clone();
    value["tvcQuorumKeyCreateProof"]["quorum_key_id"] = json!("alias");
    cases.push(value);
    let mut value = baseline.clone();
    value["addressDerivationProof"] =
        operator_payload(0, vec!["ab".repeat(32)])["addressDerivationProof"].clone();
    cases.push(value);
    let mut value = baseline.clone();
    value["tvcQuorumKeyCreateProof"]["quorumKeyId"] = json!("");
    cases.push(value);
    for value in cases {
        assert!(
            claims::decode(&value.to_string()).is_err(),
            "accepted {value}"
        );
    }
    let raw = baseline.to_string();
    for invalid in [
        raw.replacen("\"threshold\":2", "\"threshold\":2e0", 1),
        raw.replacen("\"threshold\":2", "\"threshold\":2,\"threshold\":2", 1),
        raw.replacen(
            "\"type\":",
            "\"type\":\"APP_PROOF_TYPE_POLICY_OUTCOME\",\"type\":",
            1,
        ),
    ] {
        assert!(claims::decode(&invalid).is_err());
    }
}

#[test]
fn real_signer_endorsement_vector_fixes_purpose_bytes_and_raw_signature_encoding() {
    let v: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/boot_endorsement_v1.json"
    ))
    .unwrap();
    let q = claims::composite(v["quorumPublicKeyHex"].as_str().unwrap()).unwrap();
    let verifier = endorsement::SelectedEndorsement::with_test_roots(q, fixture::ROOT.to_vec());
    let document = hex::decode(v["documentHex"].as_str().unwrap()).unwrap();
    assert_eq!(hex::encode(endorsement::ENDORSEMENT_PREFIX), v["prefixHex"]);
    let mut message = endorsement::ENDORSEMENT_PREFIX.to_vec();
    message.extend_from_slice(&document);
    assert_eq!(hex::encode(message), v["messageHex"]);
    verifier
        .verify_document_signature(&document, v["signatureHex"].as_str().unwrap())
        .unwrap();
    for (_, signature) in v["negativeSignatures"].as_object().unwrap() {
        assert!(
            verifier
                .verify_document_signature(&document, signature.as_str().unwrap())
                .is_err()
        );
    }
    let mut changed = document.clone();
    changed[10] ^= 1;
    assert!(
        verifier
            .verify_document_signature(&changed, v["signatureHex"].as_str().unwrap())
            .is_err()
    );
}

#[test]
fn complete_synthetic_chain_rejects_each_broken_link_and_uses_signed_historical_time() {
    let original = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
    let verifier = verifier();
    let good = fixture::boot();
    assert!(verifier.endorsement.authenticate(&original, &good).is_ok());
    let document = base64::decode(&good.aws_attestation_doc_b64).unwrap();
    assert!(
        crate::parse_and_verify_der_attestation(&document, fixture::ROOT, 1_793_904_000).is_err()
    );
    let mut changed = good.clone();
    changed.created_at = Some(turnkey_client::generated::external::data::v1::Timestamp {
        seconds: "0".into(),
        nanos: "999999999999".into(),
    });
    assert!(
        verifier
            .endorsement
            .authenticate(&original, &changed)
            .is_ok()
    );
    let mut mutations = vec![];
    let mut changed = good.clone();
    changed.quorum_key_attestation_signature_hex = None;
    mutations.push(changed);
    for signature in [
        String::new(),
        "00".repeat(64),
        "AA".repeat(64),
        "00".repeat(65),
    ] {
        let mut changed = good.clone();
        changed.quorum_key_attestation_signature_hex = Some(signature);
        mutations.push(changed);
    }
    let mut changed = good.clone();
    changed
        .ephemeral_public_key_hex
        .replace_range(..130, &point(12));
    mutations.push(changed);
    let mut bytes = document.clone();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    let mut changed = good.clone();
    changed.aws_attestation_doc_b64 = base64::encode(&bytes);
    mutations.push(changed.clone());
    changed.quorum_key_attestation_signature_hex = Some(fixture::sign_document(
        &bytes,
        endorsement::ENDORSEMENT_PREFIX,
    ));
    mutations.push(changed);
    for mutate in 0..8 {
        let mut manifest = fixture::manifest();
        let mut attestation = fixture::attestation(&manifest);
        match mutate {
            0 => {
                manifest.namespace.quorum_key =
                    hex::decode(format!("{}{}", point(12), point(11))).unwrap();
                attestation = fixture::attestation(&manifest);
            }
            1 => {
                attestation.public_key = Some(serde_bytes::ByteBuf::from(
                    hex::decode(format!("{}{}", point(12), point(9))).unwrap(),
                ))
            }
            2 => attestation.pcrs.get_mut(&17).unwrap()[0] ^= 1,
            3 => attestation.pcrs.get_mut(&0).unwrap()[0] ^= 1,
            4 => attestation.user_data.as_mut().unwrap()[0] ^= 1,
            5 => attestation.timestamp = 1_735_776_000_000,
            6 => {
                attestation.pcrs.remove(&31);
            }
            7 => attestation.nonce = Some(serde_bytes::ByteBuf::from(vec![1])),
            _ => panic!("unexpected test mutation"),
        }
        mutations.push(fixture::boot_from(manifest, &attestation));
    }
    let mut changed = good.clone();
    let mut manifest = fixture::manifest();
    manifest.namespace.name = "substituted".into();
    changed.qos_manifest_b64 = base64::encode(borsh::to_vec(&manifest).unwrap());
    mutations.push(changed);
    let mut changed = good.clone();
    let mut envelope = fixture::envelope(fixture::manifest());
    envelope.manifest_set_approvals[0].signature[0] ^= 1;
    changed.qos_manifest_envelope_b64 = base64::encode(borsh::to_vec(&envelope).unwrap());
    mutations.push(changed);
    for (index, changed) in mutations.into_iter().enumerate() {
        assert!(
            verifier
                .endorsement
                .authenticate(&original, &changed)
                .is_err(),
            "accepted chain mutation {index}"
        );
    }
    let wrong_q = endorsement::SelectedEndorsement::with_test_roots(pin(), fixture::ROOT.to_vec());
    assert!(wrong_q.authenticate(&original, &good).is_err());
    let wrong_root = endorsement::SelectedEndorsement::with_test_roots(
        fixture::quorum_key(),
        fixture::LEAF.to_vec(),
    );
    assert!(wrong_root.authenticate(&original, &good).is_err());
}

#[tokio::test]
async fn boot_lookup_failure_is_reported_without_resubmitting_creation() {
    let receipt = receipt(Mode::Operator).await;
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/public/v1/query/get_boot_proof"))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_string("private request body x-stamp proof material"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let error = verifier()
        .accept_operator(&capture_client(server.uri()), &expected(), receipt)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::BootProofLookup(BootProofLookupError::HttpStatus(503))
    ));
    assert!(!format!("{error:?} {error}").contains("private request body"));
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(
        requests
            .iter()
            .all(|r| r.url.path() == "/public/v1/query/get_boot_proof")
    );
    assert!(
        requests
            .iter()
            .all(|r| serde_json::from_slice::<Value>(&r.body).unwrap()
                == json!({"organizationId":"org-id", "ephemeralKey":hex::encode(pin())}))
    );
}

fn empty_report(count: usize) -> ProofReport {
    ProofReport::new(
        "synthetic test policy",
        hex::encode(pin()),
        AcceptanceEvidence::Operator {
            organization: "org-id".into(),
            wallet: None,
            base_path: "m/0".into(),
            returned: None,
        },
        Vec::new(),
        count,
    )
}

fn states(checks: &[Check]) -> Vec<(ProofCheck, CheckState)> {
    checks
        .iter()
        .map(|check| (check.kind, check.state))
        .collect()
}

#[tokio::test]
async fn reports_actual_operator_and_quorum_acceptance_with_full_public_operands() {
    let server = boot_server().await;
    for existing_wallet in [None, Some("wallet-1".to_owned())] {
        let expected =
            OperatorExpectation::parse("org-id".into(), existing_wallet, "m/0".into()).unwrap();
        let (result, report) = verifier()
            .accept_operator_reported(
                &capture_client(server.uri()),
                &expected,
                receipt(Mode::Operator).await,
            )
            .await
            .into_parts();
        result.unwrap();
        assert!(report.checks().iter().all(|check| check.state == Pass));
        assert_eq!(
            report.signer_quorum_key(),
            hex::encode(fixture::quorum_key())
        );
        assert_eq!(
            report.matched_fingerprint(),
            Some(report.sent_fingerprints()[0].as_str())
        );
        for envelope in &report.envelopes()[..2] {
            assert!(envelope.checks().iter().all(|check| check.state == Pass));
            assert_eq!(envelope.ephemeral_key(), Some(hex::encode(pin()).as_str()));
            let attestation = envelope.attestation().unwrap();
            assert_eq!(attestation.issued_at_ms, fixture::ISSUED_AT);
            assert_eq!(attestation.public_key, Some(hex::encode(pin())));
            assert!(
                attestation
                    .pcrs
                    .iter()
                    .any(|(index, pcr)| *index == 17 && pcr.len() == 96)
            );
            assert_eq!(
                envelope.boot().unwrap().manifest_hash,
                envelope.boot().unwrap().envelope_manifest_hash
            );
            let Some(ClaimEvidence::Operator {
                public_key,
                digests,
                ..
            }) = envelope.claim()
            else {
                panic!("operator claim evidence");
            };
            assert_eq!(public_key.len(), 130);
            assert_eq!(digests, report.sent_fingerprints());
        }
        let policy = &report.envelopes()[2];
        assert!(policy.policy_excluded());
        assert_eq!(policy.checks()[0].state, Pass);
        assert!(
            policy.checks()[1..]
                .iter()
                .all(|check| check.state == CheckState::NotRun)
        );
        assert!(policy.claim().is_none());
    }
    let expected = QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2)]).unwrap();
    let (result, report) = verifier()
        .accept_quorum_reported(
            &capture_client(server.uri()),
            &expected,
            receipt(Mode::Quorum).await,
        )
        .await
        .into_parts();
    result.unwrap();
    assert!(report.checks().iter().all(|check| check.state == Pass));
    let Some(ClaimEvidence::Quorum {
        public_key,
        recipients,
        ..
    }) = report.envelopes()[0].claim()
    else {
        panic!("quorum claim evidence");
    };
    assert_eq!(public_key, &format!("{}{}", point(3), point(4)));
    assert_eq!(recipients, &vec![point(1), point(2)]);
    let debug = format!("{report:?}");
    assert!(!debug.contains("unrelated-ump-key"));
    assert!(!debug.contains("signature:"));
    assert!(!debug.contains("synthetic-predicate-test"));
}

#[test]
fn authentication_failure_reports_stop_at_the_actual_failed_operation() {
    let original = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
    let mut bad_endorsement = fixture::boot();
    bad_endorsement.quorum_key_attestation_signature_hex = Some("00".repeat(64));
    let manifest = fixture::manifest();
    let mut doc = fixture::attestation(&manifest);
    doc.timestamp = 1;
    let bad_aws = fixture::boot_from(manifest, &doc);
    let mut bad_boot = fixture::boot();
    bad_boot.ephemeral_public_key_hex = format!("{}{}", point(7), point(9));
    let mut malformed_boot = fixture::boot();
    malformed_boot.qos_manifest_b64 = "not-base64".into();
    let malformed_envelope = BootProof {
        qos_manifest_envelope_b64: base64::encode([0]),
        ..fixture::boot()
    };
    let malformed_key = BootProof {
        ephemeral_public_key_hex: "00".repeat(130),
        ..fixture::boot()
    };
    let mut bad_app = original.clone();
    bad_app.signature = "00".repeat(64);
    let operations = [
        DocumentEndorsement,
        AwsAttestationAtIssuance,
        BootBinding,
        ApplicationSignature,
    ];
    for (failed, app, boot, has_boot_evidence) in [
        (DocumentEndorsement, &original, bad_endorsement, false),
        (AwsAttestationAtIssuance, &original, bad_aws, false),
        (BootBinding, &original, bad_boot, true),
        (BootBinding, &original, malformed_boot, false),
        (BootBinding, &original, malformed_envelope, false),
        (BootBinding, &original, malformed_key, false),
        (ApplicationSignature, &bad_app, fixture::boot(), true),
    ] {
        let mut report = EnvelopeReport::new();
        assert!(matches!(
            verifier()
                .endorsement
                .authenticate_reported(app, &boot, &mut report),
            Err(Error::EndorsementFailure)
        ));
        let index = operations
            .iter()
            .position(|operation| *operation == failed)
            .unwrap();
        let expected = operations
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                (
                    *kind,
                    if i < index {
                        Pass
                    } else if i == index {
                        Fail
                    } else {
                        CheckState::NotRun
                    },
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(states(&report.checks()[3..7]), expected);
        assert_eq!(report.checks().last().unwrap().state, CheckState::NotRun);
        assert_eq!(report.attestation().is_some(), index >= 2);
        assert_eq!(report.boot().is_some(), has_boot_evidence);
        assert!(report.claim().is_none());
    }
}

#[tokio::test]
async fn acceptance_failure_reports_retain_original_errors_and_no_later_passes() {
    let server = boot_server().await;
    for (mode, failed) in [
        (Mode::Replay, CommonSubmission),
        (Mode::SubstituteKey, ResultBinding),
        (Mode::MissingContext, ClaimDecode),
        (Mode::Empty, ClaimSetAndIntent),
        (Mode::WrongType, ActivityShape),
    ] {
        let (result, report) = verifier()
            .accept_operator_reported(
                &capture_client(server.uri()),
                &expected(),
                receipt(mode).await,
            )
            .await
            .into_parts();
        match failed {
            CommonSubmission => assert!(matches!(result, Err(Error::ReceiptMismatch))),
            ResultBinding | ActivityShape => {
                assert!(matches!(result, Err(Error::ClaimMismatch(_))))
            }
            ClaimDecode => assert!(matches!(result, Err(Error::MalformedEvidence(_)))),
            ClaimSetAndIntent => assert!(matches!(result, Err(Error::ProofUnavailable))),
            _ => panic!("unexpected test stage"),
        }
        if failed == ClaimDecode {
            assert_eq!(report.envelopes()[0].checks().last().unwrap().state, Fail);
            assert!(
                report.envelopes()[1]
                    .checks()
                    .iter()
                    .all(|check| check.state == CheckState::NotRun)
            );
            assert!(
                report.checks()[2..]
                    .iter()
                    .all(|check| check.state == CheckState::NotRun)
            );
        } else {
            let failed_index = report
                .checks()
                .iter()
                .position(|check| check.kind == failed)
                .unwrap();
            assert_eq!(report.checks()[failed_index].state, Fail);
            assert!(
                report.checks()[failed_index + 1..]
                    .iter()
                    .all(|check| check.state == CheckState::NotRun)
            );
        }
    }
}

#[tokio::test]
async fn envelope_bounds_routing_and_key_errors_are_failed_not_unrun() {
    let client = capture_client("http://127.0.0.1:1".into());
    let valid = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
    let mut malformed_routing = valid.clone();
    malformed_routing.proof_payload = "private-sentinel-invalid-json".into();
    let mut oversized = valid.clone();
    oversized.proof_payload = "x".repeat(256 * 1024 + 1);
    let mut bad_key = valid.clone();
    bad_key.public_key = "secret-sentinel-not-a-key".into();
    for (proofs, failed) in [
        (vec![valid; 65], EnvelopeCount),
        (vec![malformed_routing], EnvelopeRouting),
        (vec![oversized], EnvelopeRouting),
        (vec![bad_key], EphemeralKeyFormat),
    ] {
        let mut report = empty_report(proofs.len());
        assert!(matches!(
            verifier()
                .claims(&client, "org-id", &proofs, &mut report)
                .await,
            Err(Error::MalformedEvidence(_))
        ));
        let checks = if failed == EnvelopeCount {
            report.checks()
        } else {
            report.envelopes()[0].checks()
        };
        let i = checks
            .iter()
            .position(|check| check.kind == failed)
            .unwrap();
        assert_eq!(checks[i].state, Fail);
        assert!(
            checks[i + 1..]
                .iter()
                .all(|check| check.state == CheckState::NotRun)
        );
        let diagnostic = format!("{report:?}");
        assert!(!diagnostic.contains("private-sentinel"));
        assert!(!diagnostic.contains("secret-sentinel"));
    }
}

#[tokio::test]
async fn quorum_failure_reports_bind_the_complete_key_and_ordered_intent() {
    let server = boot_server().await;
    let expected =
        QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2), point(3)]).unwrap();
    let (result, report) = verifier()
        .accept_quorum_reported(
            &capture_client(server.uri()),
            &expected,
            receipt(Mode::QuorumThreeRecipients).await,
        )
        .await
        .into_parts();
    result.unwrap();
    assert!(report.checks().iter().all(|check| check.state == Pass));
    for (mode, expected, failed) in [
        (
            Mode::WrongQuorumId,
            QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2)]).unwrap(),
            ResultBinding,
        ),
        (
            Mode::QuorumSubstitutedKey,
            QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2)]).unwrap(),
            ResultBinding,
        ),
        (
            Mode::ReorderedRecipients,
            QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(2)]).unwrap(),
            ClaimSetAndIntent,
        ),
        (
            Mode::QuorumThreeRecipients,
            QuorumExpectation::parse("org-id".into(), 3, vec![point(1), point(2), point(3)])
                .unwrap(),
            ClaimSetAndIntent,
        ),
        (
            Mode::Quorum,
            QuorumExpectation::parse("org-id".into(), 2, vec![point(1), point(3)]).unwrap(),
            ClaimSetAndIntent,
        ),
    ] {
        let (result, report) = verifier()
            .accept_quorum_reported(
                &capture_client(server.uri()),
                &expected,
                receipt(mode).await,
            )
            .await
            .into_parts();
        assert!(matches!(result, Err(Error::ClaimMismatch(_))));
        assert!(
            report.envelopes()[0]
                .checks()
                .iter()
                .all(|check| check.state == Pass)
        );
        let i = report
            .checks()
            .iter()
            .position(|check| check.kind == failed)
            .unwrap();
        assert_eq!(report.checks()[i].state, Fail);
        assert!(
            report.checks()[i + 1..]
                .iter()
                .all(|check| check.state == CheckState::NotRun)
        );
        if matches!(mode, Mode::QuorumSubstitutedKey) {
            let AcceptanceEvidence::Quorum {
                returned: Some(returned),
                ..
            } = report.acceptance()
            else {
                panic!("returned quorum evidence");
            };
            assert_eq!(returned.public_key, format!("{}{}", point(5), point(6)));
            let Some(ClaimEvidence::Quorum { public_key, .. }) = report.envelopes()[0].claim()
            else {
                panic!("signed quorum evidence");
            };
            assert_eq!(public_key, &format!("{}{}", point(3), point(4)));
        }
    }
}

#[tokio::test]
async fn distinct_ephemeral_keys_stay_scoped_to_each_authenticated_envelope() {
    use qos_core::protocol::QosHash;
    let second_key = format!("{}{}", point(12), point(13));
    let manifest = fixture::manifest();
    let mut doc = fixture::attestation(&manifest);
    let key_bytes = hex::decode(&second_key).unwrap();
    doc.public_key = Some(serde_bytes::ByteBuf::from(key_bytes.clone()));
    let pcr17 = qos_nsm::nitro::expected_manifest_commitment_pcr::<qos_nsm::nitro::Live>(
        &manifest.qos_hash(),
        &key_bytes,
    )
    .unwrap();
    doc.pcrs
        .insert(17, serde_bytes::ByteBuf::from(pcr17.to_vec()));
    let mut second_boot = fixture::boot_from(manifest, &doc);
    second_boot.ephemeral_public_key_hex = second_key.clone();
    let server = boot_server().await;
    Mock::given(wiremock::matchers::body_json(
        json!({"organizationId":"org-id", "ephemeralKey":second_key}),
    ))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({"bootProof":second_boot})))
    .mount(&server)
    .await;
    let first = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
    let mut second = proof(operator_payload(1, vec!["ab".repeat(32)]).to_string());
    second.public_key = second_key.clone();
    let signature: Signature = SigningKey::from_slice(&[13; 32])
        .unwrap()
        .sign(second.proof_payload.as_bytes());
    second.signature = hex::encode(signature.to_bytes());
    let mut report = empty_report(2);
    verifier()
        .claims(
            &capture_client(server.uri()),
            "org-id",
            &[first, second],
            &mut report,
        )
        .await
        .unwrap();
    assert_eq!(
        report.envelopes()[0].ephemeral_key(),
        Some(hex::encode(pin()).as_str())
    );
    assert_eq!(
        report.envelopes()[1].ephemeral_key(),
        Some(second_key.as_str())
    );
    assert!(
        report
            .envelopes()
            .iter()
            .all(|entry| entry.checks().iter().all(|check| check.state == Pass))
    );
}

#[tokio::test]
async fn boot_lookup_failure_is_fail_and_leaves_later_envelopes_unrun() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string("private-backend-sentinel"))
        .mount(&server)
        .await;
    let (result, report) = verifier()
        .accept_operator_reported(
            &capture_client(server.uri()),
            &expected(),
            receipt(Mode::Operator).await,
        )
        .await
        .into_parts();
    assert!(matches!(
        result,
        Err(Error::BootProofLookup(BootProofLookupError::HttpStatus(
            503
        )))
    ));
    let checks = report.envelopes()[0].checks();
    let i = checks
        .iter()
        .position(|check| check.kind == BootProofLookup)
        .unwrap();
    assert_eq!(checks[i].state, Fail);
    assert!(
        checks[i + 1..]
            .iter()
            .all(|check| check.state == CheckState::NotRun)
    );
    assert!(
        report.envelopes()[1]
            .checks()
            .iter()
            .all(|check| check.state == CheckState::NotRun)
    );
    assert!(
        report.checks()[2..]
            .iter()
            .all(|check| check.state == CheckState::NotRun)
    );
    assert!(!format!("{report:?}").contains("private-backend-sentinel"));
}
