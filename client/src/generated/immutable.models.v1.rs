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
            _ => None,
        }
    }
}
