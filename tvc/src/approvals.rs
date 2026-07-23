//! Cryptographic validation of manifest approvals against a manifest set.
//!
//! API approvals are parsed once at the fetch boundary into
//! [`OperatorApproval`] — construction fails on a missing operator or a
//! malformed public key, so validation only ever sees well-formed approvals.
//! [`ValidatedManifest`] then classifies every approval instead of failing
//! fast, so callers can report problems before they surface at enclave boot:
//! an approval counts toward the threshold if it is signed over the manifest
//! hash by a distinct manifest set member — the same acceptance rule QOS
//! enforces at boot. Unlike QOS, member keys are validated eagerly, at
//! [`ValidatedManifest`] construction: a manifest set containing an
//! unparseable key is a hard error even when no approval uses it, because
//! such a manifest signals broken tooling that needs human attention rather
//! than a bad approval.

use crate::commands::app_status::TimestampPayload;
use crate::errors::MissingResource;
use qos_core::protocol::services::boot::{Approval, VersionedManifest};
use qos_p256::{P256Error, P256Public};
use serde::ser::Serializer;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    num::TryFromIntError,
    ops::Deref,
};
use turnkey_client::generated::external::data::v1::{TvcOperator, TvcOperatorApproval};
use uuid::Uuid;

/// A posted manifest approval, parsed at the API boundary: the operator is
/// present and its public key is a valid P256 key. Existence of this type is
/// proof of well-formedness; it says nothing about signature validity or
/// manifest set membership — that is [`ValidatedManifest::validate_approvals`].
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OperatorApproval {
    pub id: Uuid,
    pub operator_id: Uuid,
    operator_name: String,
    /// Never serialized: the composite qos_p256 key (encryption + signing)
    /// has no canonical byte rendering, and `operator_id` already names the
    /// key owner.
    #[serde(skip)]
    pub public_key: P256Public,
    #[serde(serialize_with = "hex_signature")]
    signature: Vec<u8>,
    created_at: Option<TimestampPayload>,
}

/// Serialize signature bytes as lowercase hex, matching the wire encoding.
fn hex_signature<S: Serializer>(signature: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&hex::encode(signature))
}

/// Failures parsing a posted API approval into [`OperatorApproval`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum OperatorApprovalError {
    #[error("approval operator is missing")]
    MissingOperator(#[from] MissingResource),
    #[error("approval ID '{id}' is not a UUID")]
    ApprovalIdNotUuid { id: String, source: uuid::Error },
    #[error("operator ID '{id}' is not a UUID")]
    OperatorIdNotUuid { id: String, source: uuid::Error },
    #[error("operator {operator_id} public key is not valid hex")]
    PublicKeyNotHex {
        operator_id: Uuid,
        source: hex::FromHexError,
    },
    #[error("operator {operator_id} public key is not a valid P256 key: {error:?}")]
    PublicKeyNotP256 { operator_id: Uuid, error: P256Error },
}

impl TryFrom<TvcOperatorApproval> for OperatorApproval {
    type Error = OperatorApprovalError;

    fn try_from(approval: TvcOperatorApproval) -> Result<Self, OperatorApprovalError> {
        let TvcOperatorApproval {
            id,
            manifest_id: _,
            operator,
            approval: signature,
            created_at,
            updated_at: _,
        } = approval;

        let Some(operator) = operator else {
            return Err(MissingResource::new("approval operator", id).into());
        };

        let id = id
            .parse::<Uuid>()
            .map_err(|source| OperatorApprovalError::ApprovalIdNotUuid { id, source })?;

        let TvcOperator {
            id: operator_id,
            name: operator_name,
            public_key,
            created_at: _,
            updated_at: _,
        } = operator;

        let operator_id = operator_id.parse::<Uuid>().map_err(|source| {
            OperatorApprovalError::OperatorIdNotUuid {
                id: operator_id,
                source,
            }
        })?;

        let public_key =
            hex::decode(&public_key).map_err(|source| OperatorApprovalError::PublicKeyNotHex {
                operator_id,
                source,
            })?;

        let public_key = P256Public::from_bytes(&public_key)
            .map_err(|error| OperatorApprovalError::PublicKeyNotP256 { operator_id, error })?;

        Ok(Self {
            id,
            operator_id,
            operator_name,
            public_key,
            signature,
            created_at: created_at.map(Into::into),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, displaydoc::Display, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ApprovalVerdict {
    /// valid
    Valid,
    /// invalid signature
    InvalidSignature,
    /// not in manifest set
    NotInManifestSet,
    /// duplicate
    Duplicate,
}

/// "name (id)" for human-facing approval lines.
impl fmt::Display for OperatorApproval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.operator_name, self.operator_id)
    }
}

#[derive(serde::Serialize)]
pub(crate) struct ValidatedApproval {
    #[serde(flatten)]
    pub approval: OperatorApproval,
    pub verdict: ApprovalVerdict,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApprovalValidation {
    pub approvals: Vec<ValidatedApproval>,
    pub threshold: usize,
}

/// Quorum facts derived from a validation, computed once at packaging.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApprovalValidationMeta {
    pub valid_count: usize,
    pub quorum_reached: bool,
}

/// A validation packaged with its derived quorum facts, for outcome
/// payloads: both halves flatten into one JSON object.
#[derive(serde::Serialize)]
pub(crate) struct ApprovalValidationWithMeta {
    #[serde(flatten)]
    pub validation: ApprovalValidation,
    #[serde(flatten)]
    pub meta: ApprovalValidationMeta,
}

impl ApprovalValidation {
    /// Number of distinct manifest set members with a valid approval.
    pub(crate) fn valid_count(&self) -> usize {
        self.approvals
            .iter()
            .filter(|validated| validated.verdict == ApprovalVerdict::Valid)
            .count()
    }

    pub(crate) fn quorum_reached(&self) -> bool {
        self.valid_count() >= self.threshold
    }

    pub(crate) fn with_meta(self) -> ApprovalValidationWithMeta {
        let meta = ApprovalValidationMeta {
            valid_count: self.valid_count(),
            quorum_reached: self.quorum_reached(),
        };

        ApprovalValidationWithMeta {
            validation: self,
            meta,
        }
    }
}

/// Approval verification failures; the data rides in the variant and the
/// message is rendered where the error is actually delivered.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ApprovalVerificationError {
    #[error("approval member {0} is not part of the manifest set")]
    NotInManifestSet(String),
    #[error(
        "freshly generated approval from {alias} failed verification ({error:?}); \
         check that the operator key matches the manifest set member key"
    )]
    VerificationFailed { alias: String, error: P256Error },
}

/// A broken manifest set, discovered while constructing [`ValidatedManifest`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum ManifestSetError {
    #[error("manifest set contains an invalid public key for member {alias}: {error:?}")]
    InvalidMemberKey { alias: String, error: P256Error },
    #[error("manifest set threshold does not fit in usize")]
    ThresholdTooLarge(#[from] TryFromIntError),
}

/// A manifest whose manifest set survived validation: every member key
/// parses and the threshold fits `usize`. Construction is the proof, so
/// classifying approvals can no longer fail on a broken manifest. Borrows
/// the manifest — the owner lives at the top of the command flow and
/// outlives every consumer.
pub(crate) struct ValidatedManifest<'a> {
    manifest: &'a VersionedManifest,
    /// Member key bytes → parsed key. Byte-keyed because enclave-boot
    /// membership is byte equality, not point equality.
    member_keys: HashMap<&'a [u8], P256Public>,
    threshold: usize,
}

impl<'a> TryFrom<&'a VersionedManifest> for ValidatedManifest<'a> {
    type Error = ManifestSetError;

    fn try_from(manifest: &'a VersionedManifest) -> Result<Self, ManifestSetError> {
        let manifest_set = manifest.manifest_set();

        // Member keys parse eagerly: an unusable key is a broken manifest
        // even when no approval references it.
        let member_keys = manifest_set
            .members
            .iter()
            .map(|member| {
                P256Public::from_bytes(&member.pub_key)
                    .map(|key| (member.pub_key.as_slice(), key))
                    .map_err(|error| ManifestSetError::InvalidMemberKey {
                        alias: member.alias.clone(),
                        error,
                    })
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let threshold = usize::try_from(manifest_set.threshold)?;

        Ok(Self {
            manifest,
            member_keys,
            threshold,
        })
    }
}

impl Deref for ValidatedManifest<'_> {
    type Target = VersionedManifest;

    fn deref(&self) -> &Self::Target {
        self.manifest
    }
}

impl ValidatedManifest<'_> {
    /// Classify every posted approval against this manifest's set and hash.
    /// Every approval gets a verdict instead of failing fast, so callers can
    /// report problems before they surface at enclave boot.
    pub(crate) fn validate(&self, mut approvals: Vec<OperatorApproval>) -> ApprovalValidation {
        // Deterministic processing order: a verdict must be a fact about the
        // data, not about the API's response ordering. Time-ordered so the
        // earliest approval per member wins `Valid`; missing or unparseable
        // stamps sort last; approval ID breaks ties.
        approvals.sort_by_key(|approval| {
            let stamp = approval.created_at.as_ref().and_then(|created_at| {
                Some((
                    created_at.seconds.parse::<i64>().ok()?,
                    created_at.nanos.parse::<u32>().ok()?,
                ))
            });

            (stamp.is_none(), stamp, approval.id)
        });

        let manifest_hash = self.manifest_hash();
        let mut counted = HashSet::new();

        let approvals = approvals
            .into_iter()
            .map(|approval| {
                let key_bytes = approval.public_key.to_bytes();

                let verdict = 'verdict: {
                    if !self.member_keys.contains_key(key_bytes.as_slice()) {
                        break 'verdict ApprovalVerdict::NotInManifestSet;
                    }

                    let signature_valid = approval
                        .public_key
                        .verify(&manifest_hash, &approval.signature)
                        .is_ok();

                    if !signature_valid {
                        break 'verdict ApprovalVerdict::InvalidSignature;
                    }

                    if counted.contains(&key_bytes) {
                        break 'verdict ApprovalVerdict::Duplicate;
                    }

                    counted.insert(key_bytes);
                    ApprovalVerdict::Valid
                };

                ValidatedApproval { approval, verdict }
            })
            .collect();

        ApprovalValidation {
            approvals,
            threshold: self.threshold,
        }
    }

    /// Verify a freshly generated approval: the member must be in this
    /// manifest's set and the signature must cover the manifest hash.
    pub(crate) fn verify_approval(
        &self,
        approval: &Approval,
    ) -> Result<(), ApprovalVerificationError> {
        let member_key = self
            .member_keys
            .get(approval.member.pub_key.as_slice())
            .ok_or_else(|| {
                ApprovalVerificationError::NotInManifestSet(approval.member.alias.clone())
            })?;

        member_key
            .verify(&self.manifest_hash(), &approval.signature)
            .map_err(|error| ApprovalVerificationError::VerificationFailed {
                alias: approval.member.alias.clone(),
                error,
            })
    }
}

#[cfg(test)]
pub(crate) use tests::{test_uuid, validated_approval};

#[cfg(test)]
mod tests {
    use super::*;
    use qos_p256::P256Pair;
    use serde_json::json;
    use zeroize::Zeroizing;

    fn pair(seed_byte: u8) -> P256Pair {
        P256Pair::from_master_seed(&Zeroizing::new([seed_byte; 32])).unwrap()
    }

    fn key_hex(pair: &P256Pair) -> String {
        hex::encode(pair.public_key().to_bytes())
    }

    /// Deterministic test UUID: `name` as the leading raw bytes.
    pub(crate) fn test_uuid(name: &str) -> Uuid {
        let mut bytes = [0; 16];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Uuid::from_bytes(bytes)
    }

    /// A [`ValidatedApproval`] fixture for other modules' rendering tests;
    /// construction lives here so the approval fields stay private.
    pub(crate) fn validated_approval(
        name: &str,
        operator_name: &str,
        verdict: ApprovalVerdict,
    ) -> ValidatedApproval {
        ValidatedApproval {
            approval: OperatorApproval {
                id: test_uuid(&format!("approval-{name}")),
                operator_id: test_uuid(name),
                operator_name: operator_name.to_string(),
                public_key: pair(1).public_key(),
                signature: vec![],
                created_at: None,
            },
            verdict,
        }
    }

    fn test_manifest(members: &[(&str, String)], threshold: u32) -> VersionedManifest {
        let members: Vec<_> = members
            .iter()
            .map(|(alias, pub_key)| json!({ "alias": alias, "pubKey": pub_key }))
            .collect();

        let manifest = json!({
            "namespace": {
                "name": "test-namespace",
                "nonce": 7,
                "quorumKey": "0102"
            },
            "pivot": {
                "hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "restart": "Never",
                "bridgeConfig": [],
                "debugMode": false,
                "args": []
            },
            "manifestSet": {
                "threshold": threshold,
                "members": members
            },
            "shareSet": {
                "threshold": 0,
                "members": []
            },
            "enclave": {
                "pcr0": "00",
                "pcr1": "11",
                "pcr2": "22",
                "pcr3": "33",
                "awsRootCertificate": "44",
                "qosCommit": "test-commit"
            },
            "patchSet": {
                "threshold": 0,
                "members": []
            }
        });

        VersionedManifest::try_from_slice_compat(&serde_json::to_vec(&manifest).unwrap()).unwrap()
    }

    fn approval(name: &str, pair: &P256Pair, manifest: &VersionedManifest) -> OperatorApproval {
        OperatorApproval {
            id: test_uuid(&format!("approval-{name}")),
            operator_id: test_uuid(name),
            operator_name: format!("operator-{name}"),
            public_key: pair.public_key(),
            signature: pair.sign(&manifest.manifest_hash()).unwrap(),
            created_at: None,
        }
    }

    fn api_approval(name: &str, public_key: String, signature: Vec<u8>) -> TvcOperatorApproval {
        TvcOperatorApproval {
            id: test_uuid(&format!("approval-{name}")).to_string(),
            manifest_id: "manifest-123".to_string(),
            operator: Some(TvcOperator {
                id: test_uuid(name).to_string(),
                name: format!("operator-{name}"),
                public_key,
                created_at: None,
                updated_at: None,
            }),
            approval: signature,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn two_valid_approvals_reach_quorum() {
        let (alice, bob) = (pair(1), pair(2));
        let manifest = test_manifest(&[("alice", key_hex(&alice)), ("bob", key_hex(&bob))], 2);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let validation = manifest.validate(vec![
            approval("alice", &alice, &manifest),
            approval("bob", &bob, &manifest),
        ]);

        assert!(
            validation
                .approvals
                .iter()
                .all(|validated| validated.verdict == ApprovalVerdict::Valid)
        );
        assert_eq!(validation.valid_count(), 2);
        assert_eq!(validation.threshold, 2);
        assert!(validation.quorum_reached());
    }

    #[test]
    fn tampered_signature_is_invalid_and_not_counted() {
        let (alice, bob) = (pair(1), pair(2));
        let manifest = test_manifest(&[("alice", key_hex(&alice)), ("bob", key_hex(&bob))], 2);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let mut tampered = approval("bob", &bob, &manifest);
        tampered.signature[0] ^= 0xff;

        let validation = manifest.validate(vec![approval("alice", &alice, &manifest), tampered]);

        assert_eq!(validation.approvals[0].verdict, ApprovalVerdict::Valid);
        assert_eq!(
            validation.approvals[1].verdict,
            ApprovalVerdict::InvalidSignature
        );
        assert_eq!(validation.valid_count(), 1);
        assert!(!validation.quorum_reached());
    }

    #[test]
    fn signer_outside_manifest_set_is_rejected() {
        let (alice, mallory) = (pair(1), pair(9));
        let manifest = test_manifest(&[("alice", key_hex(&alice))], 1);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let validation = manifest.validate(vec![approval("mallory", &mallory, &manifest)]);

        assert_eq!(
            validation.approvals[0].verdict,
            ApprovalVerdict::NotInManifestSet
        );
        assert_eq!(validation.valid_count(), 0);
    }

    #[test]
    fn same_member_counts_once() {
        let alice = pair(1);
        let manifest = test_manifest(&[("alice", key_hex(&alice))], 2);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let validation = manifest.validate(vec![
            approval("alice", &alice, &manifest),
            approval("alice", &alice, &manifest),
        ]);

        assert_eq!(validation.approvals[0].verdict, ApprovalVerdict::Valid);
        assert_eq!(validation.approvals[1].verdict, ApprovalVerdict::Duplicate);
        assert_eq!(validation.valid_count(), 1);
        assert!(!validation.quorum_reached());
    }

    /// Fed in reverse order — and with second counts whose string forms sort
    /// the wrong way ("999999999" > "1700000000" lexicographically) — the
    /// earlier approval still wins `Valid` and the output is time-ordered.
    #[test]
    fn duplicate_verdicts_follow_timestamps_not_input_order() {
        let alice = pair(1);
        let manifest = test_manifest(&[("alice", key_hex(&alice))], 2);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let stamp = |seconds: &str| {
            Some(TimestampPayload {
                seconds: seconds.to_string(),
                nanos: "0".to_string(),
            })
        };

        let mut earlier = approval("alice", &alice, &manifest);
        earlier.id = test_uuid("earlier");
        earlier.created_at = stamp("999999999");

        let mut later = approval("alice", &alice, &manifest);
        later.id = test_uuid("later");
        later.created_at = stamp("1700000000");

        let validation = manifest.validate(vec![later, earlier]);

        assert_eq!(validation.approvals[0].approval.id, test_uuid("earlier"));
        assert_eq!(validation.approvals[0].verdict, ApprovalVerdict::Valid);
        assert_eq!(validation.approvals[1].approval.id, test_uuid("later"));
        assert_eq!(validation.approvals[1].verdict, ApprovalVerdict::Duplicate);
        assert_eq!(validation.valid_count(), 1);
    }

    #[test]
    fn zero_threshold_requires_no_approvals() {
        let alice = pair(1);
        let manifest = test_manifest(&[("alice", key_hex(&alice))], 0);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let validation = manifest.validate(vec![]);

        assert_eq!(validation.valid_count(), 0);
        assert!(validation.quorum_reached());
    }

    #[test]
    fn invalid_member_key_is_a_hard_error() {
        let alice = pair(1);

        let manifest = test_manifest(
            &[("alice", key_hex(&alice)), ("borked", "aabbcc".to_string())],
            1,
        );

        let Err(error) = ValidatedManifest::try_from(&manifest) else {
            panic!("expected the invalid member key to fail validation");
        };

        assert!(
            error
                .to_string()
                .contains("manifest set contains an invalid public key for member borked")
        );
    }

    #[test]
    fn validation_serializes_verdicts_and_computed_counts() {
        let (alice, bob) = (pair(1), pair(2));
        let manifest = test_manifest(&[("alice", key_hex(&alice)), ("bob", key_hex(&bob))], 2);
        let manifest = ValidatedManifest::try_from(&manifest).unwrap();

        let mut alice_approval = approval("alice", &alice, &manifest);
        alice_approval.created_at = Some(TimestampPayload {
            seconds: "1700000000".to_string(),
            nanos: "5".to_string(),
        });
        let alice_signature = hex::encode(&alice_approval.signature);

        let mut tampered = approval("bob", &bob, &manifest);
        tampered.signature[0] ^= 0xff;
        let tampered_signature = hex::encode(&tampered.signature);

        let validation = manifest.validate(vec![alice_approval, tampered]);

        assert_eq!(
            serde_json::to_value(validation.with_meta()).unwrap(),
            json!({
                "approvals": [
                    {
                        "id": test_uuid("approval-alice"),
                        "operatorId": test_uuid("alice"),
                        "operatorName": "operator-alice",
                        "signature": alice_signature,
                        "createdAt": { "seconds": "1700000000", "nanos": "5" },
                        "verdict": "valid",
                    },
                    {
                        "id": test_uuid("approval-bob"),
                        "operatorId": test_uuid("bob"),
                        "operatorName": "operator-bob",
                        "signature": tampered_signature,
                        "createdAt": null,
                        "verdict": "invalid-signature",
                    },
                ],
                "validCount": 1,
                "threshold": 2,
                "quorumReached": false,
            })
        );
    }

    #[test]
    fn approval_without_operator_is_missing_data() {
        let mut api = api_approval("ghost", "aabbcc".to_string(), vec![1, 2, 3]);
        api.operator = None;

        let Err(error) = OperatorApproval::try_from(api) else {
            panic!("expected the missing operator to fail parsing");
        };

        assert!(matches!(error, OperatorApprovalError::MissingOperator(_)));
    }

    #[test]
    fn non_hex_operator_key_is_malformed_data() {
        let api = api_approval("alice", "not-hex".to_string(), vec![1, 2, 3]);

        let Err(error) = OperatorApproval::try_from(api) else {
            panic!("expected the non-hex key to fail parsing");
        };

        assert!(error.to_string().contains(&format!(
            "operator {} public key is not valid hex",
            test_uuid("alice")
        )));
    }

    #[test]
    fn non_p256_operator_key_is_malformed_data() {
        let api = api_approval("alice", "aabbcc".to_string(), vec![1, 2, 3]);

        let Err(error) = OperatorApproval::try_from(api) else {
            panic!("expected the non-P256 key to fail parsing");
        };

        assert!(error.to_string().contains(&format!(
            "operator {} public key is not a valid P256 key",
            test_uuid("alice")
        )));
    }
}
