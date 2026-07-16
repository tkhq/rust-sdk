//! Fake QuorumOS manifest generation for tests.
//!
//! Use [`fake_manifest_envelope`] for a one-liner with sensible defaults, or
//! [`FakeManifestBuilder`] to configure the namespace, manifest/share sets,
//! quorum key, PCRs, and pivot configuration.
//!
//! **DO NOT USE IN PRODUCTION.** The generated manifests carry made-up
//! measurements and no approvals.

use qos_core::protocol::services::boot::{
    ManifestEnvelopeV2, ManifestSet, ManifestV2, ManifestVersion, Namespace, NitroConfig,
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
}
