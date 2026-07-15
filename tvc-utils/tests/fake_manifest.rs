use qos_core::protocol::services::boot::{
    ManifestVersion, VersionedManifest, VersionedManifestEnvelope,
};
use qos_p256::P256Pair;
use tvc_utils::{FakeManifestBuilder, fake_manifest, fake_manifest_envelope};

#[test]
fn default_fake_manifest_is_v2_with_default_values() {
    let manifest = fake_manifest();

    assert_eq!(manifest.version, ManifestVersion::V2);
    assert_eq!(manifest.namespace.name, "fake-namespace");
    assert_eq!(manifest.namespace.nonce, 1);
    assert_eq!(manifest.namespace.quorum_key.len(), 130);
    assert_eq!(manifest.manifest_set.threshold, 1);
    assert_eq!(manifest.manifest_set.members.len(), 1);
    assert_eq!(manifest.share_set.threshold, 1);
    assert_eq!(manifest.share_set.members.len(), 1);
    assert_eq!(manifest.enclave.pcr0, vec![0; 48]);
    assert_eq!(manifest.enclave.pcr1, vec![1; 48]);
    assert_eq!(manifest.enclave.pcr2, vec![2; 48]);
    assert_eq!(manifest.enclave.pcr3, vec![3; 48]);
    assert_eq!(manifest.enclave.qos_commit, "fake-qos-commit");
    assert_eq!(manifest.pivot.hash, [7; 32]);
    assert!(manifest.pivot.args.is_empty());
}

#[test]
fn default_fake_manifest_envelope_has_no_approvals() {
    let envelope = fake_manifest_envelope();

    assert!(matches!(envelope, VersionedManifestEnvelope::V2(_)));
    assert!(envelope.manifest_set_approvals().is_empty());
    assert!(envelope.share_set_approvals().is_empty());
}

#[test]
fn builder_configures_all_fields() {
    let quorum_key = P256Pair::generate()
        .expect("key should generate")
        .public_key()
        .to_bytes();
    let manifest_members = vec![
        tvc_utils::fake_member("alice"),
        tvc_utils::fake_member("bob"),
    ];
    let share_members = vec![tvc_utils::fake_member("carol")];

    let manifest = FakeManifestBuilder::new()
        .namespace_name("custom-namespace")
        .nonce(42)
        .quorum_key(quorum_key.clone())
        .manifest_set(2, manifest_members.clone())
        .share_set(1, share_members.clone())
        .pcrs(vec![9; 48], vec![8; 48], vec![7; 48], vec![6; 48])
        .qos_commit("custom-commit")
        .pivot_hash([1; 32])
        .pivot_args(vec!["--flag".to_string()])
        .build();

    assert_eq!(manifest.namespace.name, "custom-namespace");
    assert_eq!(manifest.namespace.nonce, 42);
    assert_eq!(manifest.namespace.quorum_key, quorum_key);
    assert_eq!(manifest.manifest_set.threshold, 2);
    assert_eq!(manifest.manifest_set.members, manifest_members);
    assert_eq!(manifest.share_set.threshold, 1);
    assert_eq!(manifest.share_set.members, share_members);
    assert_eq!(manifest.enclave.pcr0, vec![9; 48]);
    assert_eq!(manifest.enclave.pcr1, vec![8; 48]);
    assert_eq!(manifest.enclave.pcr2, vec![7; 48]);
    assert_eq!(manifest.enclave.pcr3, vec![6; 48]);
    assert_eq!(manifest.enclave.qos_commit, "custom-commit");
    assert_eq!(manifest.pivot.hash, [1; 32]);
    assert_eq!(manifest.pivot.args, vec!["--flag".to_string()]);
}

#[test]
fn fake_member_generates_distinct_p256_keys() {
    let alice = tvc_utils::fake_member("alice");
    let bob = tvc_utils::fake_member("bob");

    assert_eq!(alice.alias, "alice");
    assert_eq!(alice.pub_key.len(), 130);
    assert_ne!(alice.pub_key, bob.pub_key);
}

#[test]
fn envelope_round_trips_through_storage_encoding() {
    let envelope = FakeManifestBuilder::new()
        .namespace_name("round-trip")
        .nonce(7)
        .build_envelope();

    let bytes = envelope
        .to_storage_vec()
        .expect("envelope should serialize");
    let decoded =
        VersionedManifestEnvelope::try_from_slice_compat(&bytes).expect("envelope should decode");

    assert_eq!(decoded, envelope);
    assert!(matches!(decoded, VersionedManifestEnvelope::V2(_)));
    assert_eq!(decoded.manifest_hash(), envelope.manifest_hash());
}

#[test]
fn manifest_round_trips_through_storage_encoding() {
    let manifest = VersionedManifest::V2(fake_manifest());

    let bytes = manifest
        .to_storage_vec()
        .expect("manifest should serialize");
    let decoded = VersionedManifest::try_from_slice_compat(&bytes).expect("manifest should decode");

    assert_eq!(decoded, manifest);
    assert_eq!(decoded.manifest_hash(), manifest.manifest_hash());
}

#[test]
fn envelope_storage_encoding_is_canonical_qos_json() {
    let envelope = fake_manifest_envelope();

    let storage = envelope
        .to_storage_vec()
        .expect("envelope should serialize");
    let VersionedManifestEnvelope::V2(inner) = &envelope else {
        panic!("fake envelope should be v2");
    };
    let canonical = qos_json::to_vec(inner).expect("qos_json should serialize");

    assert_eq!(storage, canonical);
}
