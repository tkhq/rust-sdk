//! Cryptographic validation of manifest approvals against a manifest set.
//!
//! Mirrors the semantics QOS enforces at enclave boot
//! (`VersionedManifestEnvelope::check_approvals`): every approval signature
//! must verify against the manifest hash, the signer must be a manifest set
//! member, and each member counts at most once toward the threshold. Unlike
//! QOS, validation here classifies every approval instead of failing fast, so
//! callers can report problems before they surface at boot time.

use anyhow::bail;
use qos_core::protocol::services::boot::{Approval, VersionedManifest};
use qos_p256::P256Public;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt;
use turnkey_client::generated::external::data::v1::TvcOperatorApproval;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ApprovalVerdict {
    Valid,
    InvalidSignature,
    NotInManifestSet,
    /// Valid, but the member already counted toward the threshold.
    Duplicate,
    MissingOperator,
    MalformedPublicKey,
}

impl fmt::Display for ApprovalVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Valid => "valid",
            Self::InvalidSignature => "invalid signature",
            Self::NotInManifestSet => "not in manifest set",
            Self::Duplicate => "duplicate",
            Self::MissingOperator => "missing operator",
            Self::MalformedPublicKey => "malformed public key",
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValidatedApproval {
    pub operator_id: String,
    pub operator_name: String,
    pub verdict: ApprovalVerdict,
}

impl ValidatedApproval {
    /// "name (id)", or a placeholder when the API omitted the operator.
    pub(crate) fn operator_label(&self) -> String {
        if self.operator_name.is_empty() {
            "<unknown operator>".to_string()
        } else {
            format!("{} ({})", self.operator_name, self.operator_id)
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApprovalValidation {
    pub approvals: Vec<ValidatedApproval>,
    /// Number of distinct manifest set members with a valid approval.
    pub valid_count: usize,
    pub threshold: u32,
}

impl ApprovalValidation {
    pub(crate) fn quorum_reached(&self) -> bool {
        self.threshold > 0 && self.valid_count >= self.threshold as usize
    }
}

/// Classify one signature + public key against the manifest hash and manifest
/// set. Checks run in the same order as QOS: signature, then membership.
fn classify(manifest: &VersionedManifest, pub_key: &[u8], signature: &[u8]) -> ApprovalVerdict {
    let Ok(public) = P256Public::from_bytes(pub_key) else {
        return ApprovalVerdict::MalformedPublicKey;
    };

    if public.verify(&manifest.manifest_hash(), signature).is_err() {
        return ApprovalVerdict::InvalidSignature;
    }

    if !manifest
        .manifest_set()
        .members
        .iter()
        .any(|member| member.pub_key == pub_key)
    {
        return ApprovalVerdict::NotInManifestSet;
    }

    ApprovalVerdict::Valid
}

/// Hard check for a freshly generated approval before it is written anywhere.
pub(crate) fn verify_own_approval(
    manifest: &VersionedManifest,
    approval: &Approval,
) -> anyhow::Result<()> {
    match classify(manifest, &approval.member.pub_key, &approval.signature) {
        ApprovalVerdict::Valid => Ok(()),
        verdict => bail!("approval from {} is {verdict}", approval.member.alias),
    }
}

/// Validate a deployment's posted approvals against the manifest set. Every
/// approval gets a verdict; this never fails.
pub(crate) fn validate_deployment_approvals(
    manifest: &VersionedManifest,
    approvals: &[TvcOperatorApproval],
) -> ApprovalValidation {
    let mut counted_members: HashSet<Vec<u8>> = HashSet::new();
    let mut validated = Vec::with_capacity(approvals.len());

    for approval in approvals {
        let (operator_id, operator_name, verdict) = match &approval.operator {
            None => (
                String::new(),
                String::new(),
                ApprovalVerdict::MissingOperator,
            ),
            Some(operator) => {
                let verdict = match hex::decode(&operator.public_key) {
                    Err(_) => ApprovalVerdict::MalformedPublicKey,
                    Ok(pub_key) => match classify(manifest, &pub_key, &approval.approval) {
                        ApprovalVerdict::Valid if !counted_members.insert(pub_key) => {
                            ApprovalVerdict::Duplicate
                        }
                        verdict => verdict,
                    },
                };
                (operator.id.clone(), operator.name.clone(), verdict)
            }
        };

        validated.push(ValidatedApproval {
            operator_id,
            operator_name,
            verdict,
        });
    }

    let valid_count = validated
        .iter()
        .filter(|approval| approval.verdict == ApprovalVerdict::Valid)
        .count();

    ApprovalValidation {
        approvals: validated,
        valid_count,
        threshold: manifest.manifest_set().threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qos_core::protocol::services::boot::QuorumMember;
    use qos_p256::P256Pair;
    use serde_json::json;
    use turnkey_client::generated::external::data::v1::TvcOperator;
    use zeroize::Zeroizing;

    fn test_pair(seed_byte: u8) -> P256Pair {
        P256Pair::from_master_seed(&Zeroizing::new([seed_byte; 32])).unwrap()
    }

    fn test_manifest(members: &[(&str, &P256Pair)], threshold: u32) -> VersionedManifest {
        let members: Vec<_> = members
            .iter()
            .map(|(alias, pair)| {
                json!({
                    "alias": alias,
                    "pubKey": hex::encode(pair.public_key().to_bytes()),
                })
            })
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

    fn signed_approval(
        manifest: &VersionedManifest,
        operator_id: &str,
        pair: &P256Pair,
    ) -> TvcOperatorApproval {
        api_approval(
            operator_id,
            hex::encode(pair.public_key().to_bytes()),
            pair.sign(&manifest.manifest_hash()).unwrap(),
        )
    }

    fn api_approval(
        operator_id: &str,
        public_key: String,
        signature: Vec<u8>,
    ) -> TvcOperatorApproval {
        TvcOperatorApproval {
            id: format!("approval-{operator_id}"),
            manifest_id: "manifest-123".to_string(),
            operator: Some(TvcOperator {
                id: operator_id.to_string(),
                name: format!("operator-{operator_id}"),
                public_key,
                created_at: None,
                updated_at: None,
            }),
            approval: signature,
            created_at: None,
            updated_at: None,
        }
    }

    fn own_approval(manifest: &VersionedManifest, alias: &str, pair: &P256Pair) -> Approval {
        Approval {
            signature: pair.sign(&manifest.manifest_hash()).unwrap(),
            member: QuorumMember {
                alias: alias.to_string(),
                pub_key: pair.public_key().to_bytes(),
            },
        }
    }

    #[test]
    fn two_valid_approvals_reach_quorum() {
        let (alice, bob) = (test_pair(1), test_pair(2));
        let manifest = test_manifest(&[("alice", &alice), ("bob", &bob)], 2);

        let validation = validate_deployment_approvals(
            &manifest,
            &[
                signed_approval(&manifest, "alice", &alice),
                signed_approval(&manifest, "bob", &bob),
            ],
        );

        assert!(
            validation
                .approvals
                .iter()
                .all(|approval| approval.verdict == ApprovalVerdict::Valid)
        );
        assert_eq!(validation.valid_count, 2);
        assert_eq!(validation.threshold, 2);
        assert!(validation.quorum_reached());
    }

    #[test]
    fn tampered_signature_is_invalid_and_not_counted() {
        let (alice, bob) = (test_pair(1), test_pair(2));
        let manifest = test_manifest(&[("alice", &alice), ("bob", &bob)], 2);

        let mut tampered = signed_approval(&manifest, "bob", &bob);
        tampered.approval[0] ^= 0xff;

        let validation = validate_deployment_approvals(
            &manifest,
            &[signed_approval(&manifest, "alice", &alice), tampered],
        );

        assert_eq!(validation.approvals[0].verdict, ApprovalVerdict::Valid);
        assert_eq!(
            validation.approvals[1].verdict,
            ApprovalVerdict::InvalidSignature
        );
        assert_eq!(validation.valid_count, 1);
        assert!(!validation.quorum_reached());
    }

    #[test]
    fn signer_outside_manifest_set_is_rejected() {
        let (alice, mallory) = (test_pair(1), test_pair(9));
        let manifest = test_manifest(&[("alice", &alice)], 1);

        let validation = validate_deployment_approvals(
            &manifest,
            &[signed_approval(&manifest, "mallory", &mallory)],
        );

        assert_eq!(
            validation.approvals[0].verdict,
            ApprovalVerdict::NotInManifestSet
        );
        assert_eq!(validation.valid_count, 0);
    }

    #[test]
    fn same_member_counts_once() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 2);

        let validation = validate_deployment_approvals(
            &manifest,
            &[
                signed_approval(&manifest, "alice", &alice),
                signed_approval(&manifest, "alice", &alice),
            ],
        );

        assert_eq!(validation.approvals[0].verdict, ApprovalVerdict::Valid);
        assert_eq!(validation.approvals[1].verdict, ApprovalVerdict::Duplicate);
        assert_eq!(validation.valid_count, 1);
        assert!(!validation.quorum_reached());
    }

    #[test]
    fn missing_operator_does_not_poison_other_approvals() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 1);

        let mut missing = signed_approval(&manifest, "ghost", &alice);
        missing.operator = None;

        let validation = validate_deployment_approvals(
            &manifest,
            &[missing, signed_approval(&manifest, "alice", &alice)],
        );

        assert_eq!(
            validation.approvals[0].verdict,
            ApprovalVerdict::MissingOperator
        );
        assert_eq!(validation.approvals[1].verdict, ApprovalVerdict::Valid);
        assert_eq!(validation.valid_count, 1);
        assert!(validation.quorum_reached());
    }

    #[test]
    fn malformed_public_key_is_rejected() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 1);

        let not_hex = api_approval("alice", "not-hex".to_string(), vec![1, 2, 3]);
        let wrong_length = api_approval("alice", "aabbcc".to_string(), vec![1, 2, 3]);

        let validation = validate_deployment_approvals(&manifest, &[not_hex, wrong_length]);

        assert_eq!(
            validation.approvals[0].verdict,
            ApprovalVerdict::MalformedPublicKey
        );
        assert_eq!(
            validation.approvals[1].verdict,
            ApprovalVerdict::MalformedPublicKey
        );
        assert_eq!(validation.valid_count, 0);
    }

    #[test]
    fn zero_threshold_never_reaches_quorum() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 0);

        let validation = validate_deployment_approvals(
            &manifest,
            &[signed_approval(&manifest, "alice", &alice)],
        );

        assert_eq!(validation.valid_count, 1);
        assert!(!validation.quorum_reached());
    }

    #[test]
    fn own_approval_verifies() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 1);

        verify_own_approval(&manifest, &own_approval(&manifest, "alice", &alice)).unwrap();
    }

    #[test]
    fn own_approval_with_tampered_signature_errors() {
        let alice = test_pair(1);
        let manifest = test_manifest(&[("alice", &alice)], 1);

        let mut approval = own_approval(&manifest, "alice", &alice);
        approval.signature[0] ^= 0xff;

        let err = verify_own_approval(&manifest, &approval).unwrap_err();
        assert!(err.to_string().contains("invalid signature"));
    }

    #[test]
    fn own_approval_from_non_member_errors() {
        let (alice, mallory) = (test_pair(1), test_pair(9));
        let manifest = test_manifest(&[("alice", &alice)], 1);

        let err = verify_own_approval(&manifest, &own_approval(&manifest, "mallory", &mallory))
            .unwrap_err();
        assert!(err.to_string().contains("not in manifest set"));
    }
}
