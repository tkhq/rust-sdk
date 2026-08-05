#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleNode {
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub id: u64,
    /// Contains a digest (i.e. hash)
    /// For a leaf node, this would be organization_digest from from the associated notarization.
    /// For a none-leaf node (i.e. an intermediate node or the root), this would be the computed digest of the children.
    #[serde(default)]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleRootPayload {
    #[serde(default)]
    pub node: ::core::option::Option<MerkleNode>,
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub timestamp: u64,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleRoot {
    #[serde(default)]
    pub payload: ::core::option::Option<MerkleRootPayload>,
    /// Notarizer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleLeafPayload {
    #[serde(default)]
    pub node: ::core::option::Option<MerkleNode>,
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub timestamp: u64,
    /// The UUID of the organization
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// NOTE: The MerkleLeaf itself needs to be signed, rather than simply relying on the signature of its assocaited Notarization.
/// The purpose of this is to make the `MerkleLeafPayload.timestamp` immutable, so that it can be used by the Ump to verify the relative age of leaf and notarization.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleLeaf {
    #[serde(default)]
    pub payload: ::core::option::Option<MerkleLeafPayload>,
    /// Notarizer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleProof {
    #[serde(default)]
    pub root: ::core::option::Option<MerkleRoot>,
    #[serde(default)]
    pub nodes: ::prost::alloc::vec::Vec<MerkleNode>,
    #[serde(default)]
    pub leaves: ::prost::alloc::vec::Vec<MerkleLeaf>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MerkleLeafNotarization {
    #[serde(default)]
    pub leaf: ::core::option::Option<MerkleLeaf>,
    #[serde(default)]
    pub notarization: ::core::option::Option<Notarization>,
}
#[derive(Debug)]
/// The original version of AccountPayload used for signature verification.
/// This version did not contain the `exported` or `public_key` fields.
/// All signatures created using this version must continue to verify using only these fields.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AccountPayloadV0 {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
    pub curve: super::super::common::v1::Curve,
    pub path_format: super::super::common::v1::PathFormat,
    pub path: ::prost::alloc::string::String,
    pub address_format: super::super::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Introduced the `exported` field to indicate whether the account was externally exported.
/// This broke compatibility with signatures generated using V0, so we retain both versions.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AccountPayloadV1 {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
    pub curve: super::super::common::v1::Curve,
    pub path_format: super::super::common::v1::PathFormat,
    pub path: ::prost::alloc::string::String,
    pub address_format: super::super::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
    #[serde(default)]
    pub exported: bool,
}
#[derive(Debug)]
/// Introduced `public_key` (optional) field to support embedding the public key.
/// This is the canonical and current version for new signatures.
/// Older versions (V0, V1) are still supported for signature verification fallback.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AccountPayloadV2 {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
    pub curve: super::super::common::v1::Curve,
    pub path_format: super::super::common::v1::PathFormat,
    pub path: ::prost::alloc::string::String,
    pub address_format: super::super::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
    #[serde(default)]
    pub exported: bool,
    #[serde(default)]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// The signed wrapper containing a payload (currently AccountPayloadV2)
/// and its corresponding cryptographic signature.
/// Signature is created over the digest of the payload message.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Account {
    #[serde(default)]
    pub payload: ::core::option::Option<AccountPayloadV2>,
    /// Signer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct NotarizationPayload {
    /// Hash of the organization data
    pub organization_digest: ::prost::alloc::string::String,
    /// Hash of the previous notarization; used to verify notarization digest
    pub previous_notarization_digest: ::prost::alloc::string::String,
    /// Used to verifiy notarization hash and recency requirements
    /// This timestamp is in ms, in UTC. It comes directly from the NSM.
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub timestamp: u64,
    /// The serialized organization version used to calculate the digest
    pub organization_data_version: ::prost::alloc::string::String,
    /// The UUID of the organization
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Notarization {
    #[serde(default)]
    pub payload: ::core::option::Option<NotarizationPayload>,
    /// Notarizer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(Debug)]
/// while not technically immutable, this felt like the logical place to put this for now
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct NotarizedOrganization {
    #[serde(default)]
    pub data_bytes: ::prost::alloc::vec::Vec<u8>,
    #[serde(default)]
    pub notarization: ::core::option::Option<Notarization>,
    #[serde(default)]
    pub merkle_proof: ::core::option::Option<MerkleProof>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Signature {
    pub scheme: SignatureScheme,
    pub public_key: ::prost::alloc::string::String,
    pub message: ::prost::alloc::string::String,
    pub signature: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SmartContractInterfacePayload {
    pub smart_contract_interface_id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    /// for smart contract addresses and program IDs
    pub smart_contract_address: ::prost::alloc::string::String,
    /// JSON string for an ABI or IDL
    pub smart_contract_interface: ::prost::alloc::string::String,
    pub r#type: super::super::common::v1::SmartContractInterfaceType,
    pub label: ::prost::alloc::string::String,
    pub notes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SmartContractInterface {
    #[serde(default)]
    pub payload: ::core::option::Option<SmartContractInterfacePayload>,
    /// Signer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct PolicyBudgetOverride {
    #[serde(default)]
    pub max_recursion_depth: u32,
    #[serde(default)]
    pub max_evaluation_steps: u32,
    #[serde(default)]
    pub timeout_ms: u32,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct PolicyEvaluationMetrics {
    #[serde(default)]
    pub max_recursion_depth_reached: u32,
    #[serde(default)]
    pub max_evaluation_steps_used: u32,
    #[serde(default)]
    pub max_elapsed_time_ms: u32,
}
#[derive(Debug)]
#[serde_with::serde_as]
/// Secret signing payload. This is immutable and gets signed by the signers
/// quorum key to prove authenticity.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SecretPayloadV1 {
    /// UUID for this secret
    pub secret_id: ::prost::alloc::string::String,
    /// UUID for the organization this secret belongs to
    pub organization_id: ::prost::alloc::string::String,
    /// Human readable name - helpful for things like displaying the secret in the
    /// Turnkey dashboard.
    #[serde(default)]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// Ciphertext produced by at-rest encryption inside the signer enclave.
    #[serde(default)]
    pub ciphertext: ::prost::alloc::vec::Vec<u8>,
    /// Policy visible static properties bound to the secret at creation time.
    #[serde(default)]
    pub static_properties: ::prost::alloc::vec::Vec<KeyValue>,
    /// 12 byte random nonce used for at-rest AES-256-GCM encryption.
    /// Must be securely generated in the enclave.
    #[serde(default)]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
    /// 32 byte random value bound into HKDF info during key derivation for the AES-256-GCM cipher key.
    /// Must be securely generated in the enclave.
    #[serde(default)]
    pub derivation_salt: ::prost::alloc::vec::Vec<u8>,
    /// At-rest encryption suite used for the ciphertext. This is persisted with the
    /// immutable payload and is included in HKDF info and AAD to force suite migration is modeled as secret rotation rather than
    /// runtime cipher agility.
    pub at_rest_encryption_suite: AtRestEncryptionSuite,
    /// Unix timestamp for when this key was created. In case we ever want to add TTL.
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub created_at_unix_ms: u64,
}
#[derive(Debug)]
#[serde_with::serde_as]
/// An Ingress Target Key is a type created in the signer enclave and the reason
/// it is referred to as a payload is because the signer signs over this exact type.
/// It is intended to be the encryption target for transport encryption protocols
/// that facilitate ingress into our signer enclave. As an encryption target key,
/// it is meant to be only used once; in other terms this can be referred to as
/// an "ephemeral key layer" - ensuring that transport ciphertexts are each
/// encrypted to unique keys and that a compromise of a single key has a blast
/// radius limited to just one cipher text.
///
/// The type has been initially developed for use with the import secrets API,
/// but the intention is that this may be used in other APIs that require an
/// "ephemeral key layer" for ingress encryption.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct IngressTargetKeyPayloadV1 {
    /// UUID for this key
    pub ingress_target_key_id: ::prost::alloc::string::String,
    /// UUID organization this belongs to
    pub organization_id: ::prost::alloc::string::String,
    /// Server target message specific to transport encryption suite.
    /// This is what gets returned to the user by InitImportSecretsResult.
    /// For TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1 this is a JSON-encoded
    /// enclave_encrypt ServerTargetMsgV1.
    pub enclave_target_message: ::prost::alloc::string::String,
    /// Ciphertext of the target key
    #[serde(default)]
    pub ciphertext: ::prost::alloc::vec::Vec<u8>,
    /// Transport encryption suite the key is meant to be used with. This determines
    /// the target message format, target public key encoding, and import ciphertext
    /// envelope expected from the caller.
    pub encryption_suite: TransportEncryptionSuite,
    /// At-rest encryption suite used for the ciphertext that stores the target key.
    pub at_rest_encryption_suite: AtRestEncryptionSuite,
    /// Unix timestamp of when this was created at. This may be useful in the future if we want to make these expire after a certain point.
    /// Since this is signed over by the signer this can be used to enforce an expiration in the enclave.
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub created_at_unix_ms: u64,
    /// 32 byte random value bound into HKDF info during key derivation for the AES-256-GCM cipher key.
    #[serde(default)]
    pub derivation_salt: ::prost::alloc::vec::Vec<u8>,
    /// 12 byte random nonce used for at-rest AES-256-GCM encryption.
    #[serde(default)]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
    /// Encoded transport target public key, broken out from `target_public_key_data` so it can
    /// be indexed and looked up directly by the coordinator when consuming the key during import.
    /// Encoding is dictated by `encryption_suite`. For TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1
    /// this is a hex-encoded uncompressed P-256 public key.
    pub target_public_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Type to represent arbitrary key-value pairs evaluated in the policy engine.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct KeyValue {
    /// Policy visible property name.
    pub key: ::prost::alloc::string::String,
    /// Policy visible property value.
    pub value: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SessionProfilePayload {
    pub id: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    pub scope: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub notes: ::core::option::Option<::prost::alloc::string::String>,
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignedSessionProfile {
    #[serde(default)]
    pub payload: ::core::option::Option<SessionProfilePayload>,
    /// Notarizer signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<Signature>,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignatureScheme {
    #[serde(rename = "SIGNATURE_SCHEME_UNSPECIFIED")]
    Unspecified = 0,
    /// Scheme used for Turnkey's public API
    #[serde(rename = "SIGNATURE_SCHEME_TK_API_P256")]
    TkApiP256 = 1,
    /// Scheme used on our UI when users sign with Webauthn
    /// Public keys are encoded using COSE (<https://www.w3.org/TR/webauthn-2/#sctn-encoded-credPubKey-examples>)
    #[serde(rename = "SIGNATURE_SCHEME_TK_WEBAUTHN")]
    TkWebauthn = 2,
    /// Scheme used by our enclave applications
    #[serde(rename = "SIGNATURE_SCHEME_TK_QUORUM_P256")]
    TkQuorumP256 = 3,
    /// Scheme used for Turnkey's public API
    #[serde(rename = "SIGNATURE_SCHEME_TK_API_SECP256K1")]
    TkApiSecp256k1 = 4,
    /// Scheme used for Turnkey's public API
    #[serde(rename = "SIGNATURE_SCHEME_TK_API_ED25519")]
    TkApiEd25519 = 5,
    /// Scheme used for Ethereum wallet signatures
    #[serde(rename = "SIGNATURE_SCHEME_TK_API_SECP256K1_EIP191")]
    TkApiSecp256k1Eip191 = 6,
    #[serde(rename = "SIGNATURE_SCHEME_TK_ATTESTED_P256")]
    TkAttestedP256 = 7,
}
impl SignatureScheme {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "SIGNATURE_SCHEME_UNSPECIFIED",
            Self::TkApiP256 => "SIGNATURE_SCHEME_TK_API_P256",
            Self::TkWebauthn => "SIGNATURE_SCHEME_TK_WEBAUTHN",
            Self::TkQuorumP256 => "SIGNATURE_SCHEME_TK_QUORUM_P256",
            Self::TkApiSecp256k1 => "SIGNATURE_SCHEME_TK_API_SECP256K1",
            Self::TkApiEd25519 => "SIGNATURE_SCHEME_TK_API_ED25519",
            Self::TkApiSecp256k1Eip191 => "SIGNATURE_SCHEME_TK_API_SECP256K1_EIP191",
            Self::TkAttestedP256 => "SIGNATURE_SCHEME_TK_ATTESTED_P256",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGNATURE_SCHEME_UNSPECIFIED" => Some(Self::Unspecified),
            "SIGNATURE_SCHEME_TK_API_P256" => Some(Self::TkApiP256),
            "SIGNATURE_SCHEME_TK_WEBAUTHN" => Some(Self::TkWebauthn),
            "SIGNATURE_SCHEME_TK_QUORUM_P256" => Some(Self::TkQuorumP256),
            "SIGNATURE_SCHEME_TK_API_SECP256K1" => Some(Self::TkApiSecp256k1),
            "SIGNATURE_SCHEME_TK_API_ED25519" => Some(Self::TkApiEd25519),
            "SIGNATURE_SCHEME_TK_API_SECP256K1_EIP191" => {
                Some(Self::TkApiSecp256k1Eip191)
            }
            "SIGNATURE_SCHEME_TK_ATTESTED_P256" => Some(Self::TkAttestedP256),
            _ => None,
        }
    }
}
/// At-rest encryption suite for internal ciphertexts.
///
/// IMPORTANT: Each variant identifies the full encryption contract for persisted encrypted
/// resources: cipher, key derivation, nonce handling, AAD, HKDF inputs, etc. Changes
/// to any of those claims should introduce a new variant here so stored payloads can be
/// decoded and verified unambiguously; reducing chances of confused deputy attacks.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtRestEncryptionSuite {
    #[serde(rename = "AT_REST_ENCRYPTION_SUITE_UNSPECIFIED")]
    Unspecified = 0,
    /// AES-256-GCM using a per-resource HKDF-SHA256 derived key.
    ///
    /// The derived key is bound to the persisted resource type, organization, and
    /// resource identifiers through HKDF info. The ciphertext is authenticated with
    /// AAD containing the same context as the HKDF info.
    #[serde(rename = "AT_REST_ENCRYPTION_SUITE_AES256_GCM_HKDF_SHA256_V1")]
    Aes256GcmHkdfSha256V1 = 1,
}
impl AtRestEncryptionSuite {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "AT_REST_ENCRYPTION_SUITE_UNSPECIFIED",
            Self::Aes256GcmHkdfSha256V1 => {
                "AT_REST_ENCRYPTION_SUITE_AES_256_GCM_HKDF_SHA256_V1"
            }
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AT_REST_ENCRYPTION_SUITE_UNSPECIFIED" => Some(Self::Unspecified),
            "AT_REST_ENCRYPTION_SUITE_AES_256_GCM_HKDF_SHA256_V1" => {
                Some(Self::Aes256GcmHkdfSha256V1)
            }
            _ => None,
        }
    }
}
/// Stable internal resource type labels used when deriving and authenticating
/// at-rest ciphertexts.
///
/// If any resource type is updated, it should result in a new resource type
/// and thus a new variant in this enum
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtRestResourceType {
    #[serde(rename = "AT_REST_RESOURCE_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "AT_REST_RESOURCE_TYPE_INGRESS_TARGET_KEY_PAYLOAD_V1")]
    IngressTargetKeyPayloadV1 = 1,
    #[serde(rename = "AT_REST_RESOURCE_TYPE_SECRET_PAYLOAD_V1")]
    SecretPayloadV1 = 2,
}
impl AtRestResourceType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "AT_REST_RESOURCE_TYPE_UNSPECIFIED",
            Self::IngressTargetKeyPayloadV1 => {
                "AT_REST_RESOURCE_TYPE_INGRESS_TARGET_KEY_PAYLOAD_V1"
            }
            Self::SecretPayloadV1 => "AT_REST_RESOURCE_TYPE_SECRET_PAYLOAD_V1",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AT_REST_RESOURCE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "AT_REST_RESOURCE_TYPE_INGRESS_TARGET_KEY_PAYLOAD_V1" => {
                Some(Self::IngressTargetKeyPayloadV1)
            }
            "AT_REST_RESOURCE_TYPE_SECRET_PAYLOAD_V1" => Some(Self::SecretPayloadV1),
            _ => None,
        }
    }
}
/// Transport encryption suite for enclave ingress and egress.
///
/// IMPORTANT: Each value identifies the full protocol contract for moving encrypted material
/// across the enclave boundary, including key agreement, public key encoding,
/// target message format, ciphertext envelope shape, and authenticated context.
/// If any aspect changes a new variant should be introduced here so we can eliminate all
/// ambiguity when interpreting
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransportEncryptionSuite {
    #[serde(rename = "TRANSPORT_ENCRYPTION_SUITE_UNSPECIFIED")]
    Unspecified = 0,
    /// HPKE suite from by Turnkey's enclave_encrypt crate.
    ///
    /// The target public key is a hex-encoded uncompressed P-256 public key, and the
    /// target message is a JSON-encoded enclave_encrypt ServerTargetMsgV1.
    #[serde(rename = "TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1")]
    EnclaveEncryptV1 = 1,
}
impl TransportEncryptionSuite {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "TRANSPORT_ENCRYPTION_SUITE_UNSPECIFIED",
            Self::EnclaveEncryptV1 => "TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TRANSPORT_ENCRYPTION_SUITE_UNSPECIFIED" => Some(Self::Unspecified),
            "TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1" => {
                Some(Self::EnclaveEncryptV1)
            }
            _ => None,
        }
    }
}
