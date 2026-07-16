//! Fake QuorumOS manifest generation for tests.
//!
//! Use [`fake_manifest_envelope`] for a one-liner with sensible defaults, or
//! [`FakeManifestBuilder`] to configure the namespace, manifest/share sets,
//! quorum key, PCRs, and pivot configuration.
//!
//! **DO NOT USE IN PRODUCTION.** The generated manifests carry made-up
//! measurements and no approvals.

use qos_core::protocol::services::boot::{
    Approval, ManifestEnvelopeV2, ManifestSet, ManifestV2, ManifestVersion, Namespace, NitroConfig,
    PivotConfigV2, PivotEnv, QuorumMember, RestartPolicy, ShareSet, VersionedManifestEnvelope,
};
use qos_p256::P256Pair;

/// PCR register length used by AWS Nitro (SHA-384).
const PCR_LEN: usize = 48;

/// Generate a [`QuorumMember`] with the given alias and a freshly generated
/// P-256 key. Only for tests.
///
/// # Panics
///
/// Panics if key generation fails, which indicates a broken RNG. Only for
/// tests.
#[must_use]
pub fn fake_member(alias: &str) -> QuorumMember {
    QuorumMember {
        alias: alias.to_string(),
        pub_key: P256Pair::generate()
            .expect("test key generation should not fail")
            .public_key()
            .to_bytes(),
    }
}

/// Generate a [`QuorumMember`] with the given alias along with its P-256 key
/// pair, for signing manifest-set approvals. Only for tests.
///
/// # Panics
///
/// Panics if key generation fails, which indicates a broken RNG. Only for
/// tests.
#[must_use]
pub fn fake_keyed_member(alias: &str) -> (QuorumMember, P256Pair) {
    let pair = P256Pair::generate().expect("test key generation should not fail");
    let member = QuorumMember {
        alias: alias.to_string(),
        pub_key: pair.public_key().to_bytes(),
    };
    (member, pair)
}

/// Build a fake manifest (v2) with default values. Only for tests.
#[must_use]
pub fn fake_manifest() -> ManifestV2 {
    FakeManifestBuilder::new().build()
}

/// Build a fake [`VersionedManifestEnvelope`] (v2, no approvals) with default
/// values. Only for tests.
#[must_use]
pub fn fake_manifest_envelope() -> VersionedManifestEnvelope {
    FakeManifestBuilder::new().build_envelope()
}

/// Builder for fake QuorumOS manifests (schema v2). Only for tests.
///
/// Defaults: namespace `fake-namespace` with nonce 1 and a freshly generated
/// quorum key, one-member manifest and share sets with threshold 1, distinct
/// 48-byte PCR values, a non-zero pivot hash, and no pivot args.
#[derive(Debug, Clone)]
pub struct FakeManifestBuilder {
    namespace_name: String,
    nonce: u32,
    quorum_key: Option<Vec<u8>>,
    manifest_set: Option<ManifestSet>,
    share_set: Option<ShareSet>,
    pcr0: Vec<u8>,
    pcr1: Vec<u8>,
    pcr2: Vec<u8>,
    pcr3: Vec<u8>,
    qos_commit: String,
    pivot_hash: [u8; 32],
    pivot_args: Vec<String>,
}

impl Default for FakeManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeManifestBuilder {
    /// Create a builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            namespace_name: "fake-namespace".to_string(),
            nonce: 1,
            quorum_key: None,
            manifest_set: None,
            share_set: None,
            pcr0: vec![0; PCR_LEN],
            pcr1: vec![1; PCR_LEN],
            pcr2: vec![2; PCR_LEN],
            pcr3: vec![3; PCR_LEN],
            qos_commit: "fake-qos-commit".to_string(),
            pivot_hash: [7; 32],
            pivot_args: vec![],
        }
    }

    /// Set the namespace name.
    #[must_use]
    pub fn namespace_name(mut self, name: &str) -> Self {
        self.namespace_name = name.to_string();
        self
    }

    /// Set the namespace nonce.
    #[must_use]
    pub fn nonce(mut self, nonce: u32) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set the quorum public key bytes.
    #[must_use]
    pub fn quorum_key(mut self, quorum_key: Vec<u8>) -> Self {
        self.quorum_key = Some(quorum_key);
        self
    }

    /// Set the manifest set members and threshold.
    #[must_use]
    pub fn manifest_set(mut self, threshold: u32, members: Vec<QuorumMember>) -> Self {
        self.manifest_set = Some(ManifestSet { threshold, members });
        self
    }

    /// Set the share set members and threshold.
    #[must_use]
    pub fn share_set(mut self, threshold: u32, members: Vec<QuorumMember>) -> Self {
        self.share_set = Some(ShareSet { threshold, members });
        self
    }

    /// Set PCR0 through PCR3.
    #[must_use]
    pub fn pcrs(mut self, pcr0: Vec<u8>, pcr1: Vec<u8>, pcr2: Vec<u8>, pcr3: Vec<u8>) -> Self {
        self.pcr0 = pcr0;
        self.pcr1 = pcr1;
        self.pcr2 = pcr2;
        self.pcr3 = pcr3;
        self
    }

    /// Set the QOS commit reference.
    #[must_use]
    pub fn qos_commit(mut self, qos_commit: &str) -> Self {
        self.qos_commit = qos_commit.to_string();
        self
    }

    /// Set the pivot binary hash.
    #[must_use]
    pub fn pivot_hash(mut self, pivot_hash: [u8; 32]) -> Self {
        self.pivot_hash = pivot_hash;
        self
    }

    /// Set the pivot binary arguments.
    #[must_use]
    pub fn pivot_args(mut self, pivot_args: Vec<String>) -> Self {
        self.pivot_args = pivot_args;
        self
    }

    /// Build the fake manifest (v2).
    ///
    /// # Panics
    ///
    /// Panics if default key generation fails, which indicates a broken RNG.
    /// Only for tests.
    #[must_use]
    pub fn build(self) -> ManifestV2 {
        let quorum_key = self.quorum_key.unwrap_or_else(|| {
            P256Pair::generate()
                .expect("test key generation should not fail")
                .public_key()
                .to_bytes()
        });
        let manifest_set = self.manifest_set.unwrap_or_else(|| ManifestSet {
            threshold: 1,
            members: vec![fake_member("fake-manifest-set-member")],
        });
        let share_set = self.share_set.unwrap_or_else(|| ShareSet {
            threshold: 1,
            members: vec![fake_member("fake-share-set-member")],
        });

        ManifestV2 {
            version: ManifestVersion::V2,
            namespace: Namespace {
                name: self.namespace_name,
                nonce: self.nonce,
                quorum_key,
            },
            pivot: PivotConfigV2 {
                hash: self.pivot_hash,
                restart: RestartPolicy::Never,
                bridge_config: vec![],
                debug_mode: false,
                args: self.pivot_args,
                env: PivotEnv::new(),
            },
            manifest_set,
            share_set,
            enclave: NitroConfig {
                pcr0: self.pcr0,
                pcr1: self.pcr1,
                pcr2: self.pcr2,
                pcr3: self.pcr3,
                aws_root_certificate: vec![],
                qos_commit: self.qos_commit,
            },
            dns: None,
        }
    }

    /// Build a fake [`VersionedManifestEnvelope`] (v2) with no approvals.
    #[must_use]
    pub fn build_envelope(self) -> VersionedManifestEnvelope {
        VersionedManifestEnvelope::V2(ManifestEnvelopeV2 {
            manifest: self.build(),
            manifest_set_approvals: vec![],
            share_set_approvals: vec![],
        })
    }

    /// Build a fake [`VersionedManifestEnvelope`] (v2) with manifest-set
    /// approvals signed by the given members over the manifest hash. If no
    /// manifest set was configured, it is derived from the approvers with a
    /// threshold requiring all of them.
    ///
    /// # Panics
    ///
    /// Panics if signing fails, which indicates a broken key pair. Only for
    /// tests.
    #[must_use]
    pub fn build_envelope_approved_by(
        mut self,
        approvers: &[(QuorumMember, P256Pair)],
    ) -> VersionedManifestEnvelope {
        if self.manifest_set.is_none() {
            self.manifest_set = Some(ManifestSet {
                threshold: u32::try_from(approvers.len())
                    .expect("approver count should fit in u32"),
                members: approvers.iter().map(|(member, _)| member.clone()).collect(),
            });
        }

        let mut envelope = ManifestEnvelopeV2 {
            manifest: self.build(),
            manifest_set_approvals: vec![],
            share_set_approvals: vec![],
        };
        let manifest_hash = VersionedManifestEnvelope::V2(envelope.clone()).manifest_hash();
        envelope.manifest_set_approvals = approvers
            .iter()
            .map(|(member, pair)| Approval {
                signature: pair
                    .sign(&manifest_hash)
                    .expect("test approval signing should not fail"),
                member: member.clone(),
            })
            .collect();
        VersionedManifestEnvelope::V2(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_envelope_passes_check_approvals() {
        let (member_a, pair_a) = fake_keyed_member("member-a");
        let (member_b, pair_b) = fake_keyed_member("member-b");
        let envelope = FakeManifestBuilder::new()
            .build_envelope_approved_by(&[(member_a.clone(), pair_a), (member_b.clone(), pair_b)]);

        envelope
            .check_approvals()
            .expect("threshold approvals from generated members should verify");
        assert_eq!(envelope.manifest_set().threshold, 2);
        assert_eq!(
            envelope.manifest_set().members,
            vec![member_a.clone(), member_b.clone()]
        );
        assert_eq!(envelope.manifest_set_approvals().len(), 2);
        assert_eq!(envelope.manifest_set_approvals()[0].member, member_a);
        assert_eq!(envelope.manifest_set_approvals()[1].member, member_b);
    }

    #[test]
    fn approved_envelope_respects_explicit_manifest_set() {
        let (member, pair) = fake_keyed_member("member");
        let envelope = FakeManifestBuilder::new()
            .manifest_set(1, vec![member.clone(), fake_member("other")])
            .build_envelope_approved_by(&[(member, pair)]);

        envelope
            .check_approvals()
            .expect("a threshold-meeting subset of approvals should verify");
        assert_eq!(envelope.manifest_set().members.len(), 2);
        assert_eq!(envelope.manifest_set_approvals().len(), 1);
    }
}
