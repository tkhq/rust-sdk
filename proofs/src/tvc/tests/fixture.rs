use super::{pin, point};
use crate::{CoseError, CoseSign1, MessageDigest, Sha2, SignatureAlgorithm, SigningPublicKey};
use aws_nitro_enclaves_cose::{crypto::SigningPrivateKey, header_map::HeaderMap};
use aws_nitro_enclaves_nsm_api::api::{AttestationDoc, Digest};
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use p384::ecdsa::signature::hazmat::PrehashSigner;
use qos_core::protocol::{
    QosHash,
    services::boot::{
        Approval, Manifest, ManifestEnvelope, ManifestSet, MemberPubKey, Namespace, NitroConfig,
        PatchSet, PivotConfig, QuorumMember, RestartPolicy, ShareSet,
    },
};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
use turnkey_client::generated::external::data::v1::BootProof;

pub(super) const ROOT: &[u8] =
    include_bytes!("../../../tests/fixtures/synthetic-attestation/root.der");
pub(super) const LEAF: &[u8] =
    include_bytes!("../../../tests/fixtures/synthetic-attestation/leaf.der");
pub(super) const ISSUED_AT: u64 = 1_717_200_000_000;

pub(super) fn quorum_key() -> [u8; 130] {
    hex::decode(format!("{}{}", point(10), point(11)))
        .unwrap()
        .try_into()
        .unwrap()
}
pub(super) fn sign_document(document: &[u8], prefix: &[u8]) -> String {
    let mut message = prefix.to_vec();
    message.extend_from_slice(document);
    let signature: Signature = SigningKey::from_slice(&[11; 32]).unwrap().sign(&message);
    hex::encode(signature.to_bytes())
}
pub(super) fn manifest() -> Manifest {
    let member = QuorumMember {
        alias: "fixture-approver".into(),
        pub_key: pin().to_vec(),
    };
    Manifest {
        namespace: Namespace {
            name: "fixture".into(),
            nonce: 1,
            quorum_key: quorum_key().to_vec(),
        },
        pivot: PivotConfig {
            hash: [9; 32],
            restart: RestartPolicy::Never,
            bridge_config: vec![],
            debug_mode: false,
            args: vec![],
        },
        manifest_set: ManifestSet {
            threshold: 1,
            members: vec![member.clone()],
        },
        share_set: ShareSet {
            threshold: 1,
            members: vec![member.clone()],
        },
        enclave: NitroConfig {
            pcr0: vec![1; 48],
            pcr1: vec![2; 48],
            pcr2: vec![3; 48],
            pcr3: vec![4; 48],
            aws_root_certificate: ROOT.to_vec(),
            qos_commit: "fixture".into(),
        },
        patch_set: PatchSet {
            threshold: 1,
            members: vec![MemberPubKey {
                pub_key: member.pub_key,
            }],
        },
    }
}
pub(super) fn envelope(manifest: Manifest) -> ManifestEnvelope {
    let signature: Signature = SigningKey::from_slice(&[9; 32])
        .unwrap()
        .sign(&manifest.qos_hash());
    ManifestEnvelope {
        manifest_set_approvals: vec![Approval {
            signature: signature.to_bytes().to_vec(),
            member: manifest.manifest_set.members[0].clone(),
        }],
        share_set_approvals: vec![],
        manifest,
    }
}
pub(super) fn attestation(manifest: &Manifest) -> AttestationDoc {
    let hash = manifest.qos_hash();
    let pcr17 =
        qos_nsm::nitro::expected_manifest_commitment_pcr::<qos_nsm::nitro::Live>(&hash, &pin())
            .unwrap();
    let mut pcrs = (0..32)
        .map(|index| (index, ByteBuf::from(vec![0; 48])))
        .collect::<BTreeMap<_, _>>();
    pcrs.extend(BTreeMap::from([
        (0, ByteBuf::from(manifest.enclave.pcr0.clone())),
        (1, ByteBuf::from(manifest.enclave.pcr1.clone())),
        (2, ByteBuf::from(manifest.enclave.pcr2.clone())),
        (3, ByteBuf::from(manifest.enclave.pcr3.clone())),
        (17, ByteBuf::from(pcr17.to_vec())),
    ]));
    AttestationDoc {
        module_id: "fixture-nsm".into(),
        digest: Digest::SHA384,
        timestamp: ISSUED_AT,
        pcrs,
        certificate: ByteBuf::from(LEAF.to_vec()),
        cabundle: vec![ByteBuf::from(ROOT.to_vec())],
        public_key: Some(ByteBuf::from(pin().to_vec())),
        user_data: Some(ByteBuf::from(hash.to_vec())),
        nonce: None,
    }
}
struct AttestationKey(p384::ecdsa::SigningKey);
impl SigningPublicKey for AttestationKey {
    fn get_parameters(&self) -> Result<(SignatureAlgorithm, MessageDigest), CoseError> {
        Ok((SignatureAlgorithm::ES384, MessageDigest::Sha384))
    }
    fn verify(&self, digest: &[u8], signature: &[u8]) -> Result<bool, CoseError> {
        SigningPublicKey::verify(
            &crate::P384PubKey(p384::PublicKey::from(self.0.verifying_key())),
            digest,
            signature,
        )
    }
}
impl SigningPrivateKey for AttestationKey {
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, CoseError> {
        let signature: p384::ecdsa::Signature = self.0.sign_prehash(digest).unwrap();
        Ok(signature.to_bytes().to_vec())
    }
}
pub(super) fn document(doc: &AttestationDoc) -> Vec<u8> {
    CoseSign1::new::<Sha2>(
        &doc.to_binary(),
        &HeaderMap::new(),
        &AttestationKey(p384::ecdsa::SigningKey::from_slice(&[42; 48]).unwrap()),
    )
    .unwrap()
    .as_bytes(true)
    .unwrap()
}
pub(super) fn boot_from(manifest: Manifest, doc: &AttestationDoc) -> BootProof {
    let bytes = document(doc);
    BootProof {
        ephemeral_public_key_hex: hex::encode(pin()),
        aws_attestation_doc_b64: base64::encode(&bytes),
        qos_manifest_b64: base64::encode(borsh::to_vec(&manifest).unwrap()),
        qos_manifest_envelope_b64: base64::encode(borsh::to_vec(&envelope(manifest)).unwrap()),
        deployment_label: "synthetic-test".into(),
        enclave_app: "signer".into(),
        owner: String::new(),
        created_at: None,
        qos_manifest_version: Some("1".into()),
        quorum_key_attestation_signature_hex: Some(sign_document(
            &bytes,
            super::endorsement::ENDORSEMENT_PREFIX,
        )),
    }
}
pub(super) fn boot() -> BootProof {
    let manifest = manifest();
    let doc = attestation(&manifest);
    boot_from(manifest, &doc)
}
