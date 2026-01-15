#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Timestamp {
    /// Stringified int
    pub seconds: ::prost::alloc::string::String,
    /// Stringified int
    pub nanos: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// This proto definition is used in our external-facing APIs.
/// It's important to leverage annotations because they're used in our external interfaces.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OrganizationData {
    pub organization_id: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    #[serde(default)]
    pub users: ::prost::alloc::vec::Vec<User>,
    #[serde(default)]
    pub policies: ::prost::alloc::vec::Vec<Policy>,
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKey>,
    #[serde(default)]
    pub invitations: ::prost::alloc::vec::Vec<Invitation>,
    #[serde(default)]
    pub tags: ::prost::alloc::vec::Vec<Tag>,
    #[serde(default)]
    pub root_quorum: ::core::option::Option<Quorum>,
    #[serde(default)]
    pub features: ::prost::alloc::vec::Vec<
        super::super::super::immutable::data::v1::Feature,
    >,
    #[serde(default)]
    pub wallets: ::prost::alloc::vec::Vec<Wallet>,
    #[serde(default)]
    pub smart_contract_interface_references: ::prost::alloc::vec::Vec<
        super::super::super::immutable::data::v1::SmartContractInterfaceReference,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthProvider {
    pub provider_id: ::prost::alloc::string::String,
    pub provider_name: ::prost::alloc::string::String,
    pub issuer: ::prost::alloc::string::String,
    pub audience: ::prost::alloc::string::String,
    pub subject: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct User {
    pub user_id: ::prost::alloc::string::String,
    pub user_name: ::prost::alloc::string::String,
    /// some users do not have emails (programmatic users)
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub user_phone_number: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<Authenticator>,
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<ApiKey>,
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProvider>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApiKey {
    #[serde(default)]
    pub credential: ::core::option::Option<Credential>,
    pub api_key_id: ::prost::alloc::string::String,
    pub api_key_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub expiration_seconds: ::core::option::Option<u64>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Authenticator {
    #[serde(default)]
    pub transports: Vec<
        super::super::super::immutable::webauthn::v1::AuthenticatorTransport,
    >,
    pub attestation_type: ::prost::alloc::string::String,
    pub aaguid: ::prost::alloc::string::String,
    pub credential_id: ::prost::alloc::string::String,
    pub model: ::prost::alloc::string::String,
    #[serde(default)]
    pub credential: ::core::option::Option<Credential>,
    pub authenticator_id: ::prost::alloc::string::String,
    pub authenticator_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Credential {
    pub public_key: ::prost::alloc::string::String,
    /// To distinguish the credential type (webauthn, API key)
    pub r#type: super::super::super::immutable::common::v1::CredentialType,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Policy {
    pub policy_id: ::prost::alloc::string::String,
    pub policy_name: ::prost::alloc::string::String,
    pub effect: super::super::super::immutable::common::v1::Effect,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    pub notes: ::prost::alloc::string::String,
    #[serde(default)]
    pub consensus: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub condition: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PrivateKey {
    pub private_key_id: ::prost::alloc::string::String,
    pub public_key: ::prost::alloc::string::String,
    pub private_key_name: ::prost::alloc::string::String,
    pub curve: super::super::super::immutable::common::v1::Curve,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<Address>,
    #[serde(default)]
    pub private_key_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub exported: bool,
    #[serde(default)]
    pub imported: bool,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Address {
    pub format: super::super::super::immutable::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Invitation {
    pub invitation_id: ::prost::alloc::string::String,
    pub receiver_user_name: ::prost::alloc::string::String,
    pub receiver_email: ::prost::alloc::string::String,
    #[serde(default)]
    pub receiver_user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    pub access_type: super::super::super::immutable::common::v1::AccessType,
    pub status: InvitationStatus,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    pub sender_user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Tag {
    pub tag_id: ::prost::alloc::string::String,
    pub tag_name: ::prost::alloc::string::String,
    pub tag_type: TagType,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Quorum {
    #[serde(default)]
    pub threshold: i32,
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Wallet {
    pub wallet_id: ::prost::alloc::string::String,
    pub wallet_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub exported: bool,
    #[serde(default)]
    pub imported: bool,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub features: ::prost::alloc::vec::Vec<
        super::super::super::immutable::data::v1::Feature,
    >,
    #[serde(default)]
    pub quorum: ::core::option::Option<Quorum>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Oauth2Credential {
    pub oauth2_credential_id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub provider: super::super::super::immutable::common::v1::Oauth2Provider,
    pub client_id: ::prost::alloc::string::String,
    pub encrypted_client_secret: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct FiatOnRampCredential {
    pub fiat_onramp_credential_id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub onramp_provider: super::super::super::immutable::common::v1::FiatOnRampProvider,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub project_id: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    pub publishable_api_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encrypted_secret_api_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub encrypted_private_api_key: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub sandbox_mode: bool,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InvitationStatus {
    #[serde(rename = "INVITATION_STATUS_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "INVITATION_STATUS_CREATED")]
    Created = 1,
    #[serde(rename = "INVITATION_STATUS_ACCEPTED")]
    Accepted = 2,
    #[serde(rename = "INVITATION_STATUS_REVOKED")]
    Revoked = 3,
}
impl InvitationStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "INVITATION_STATUS_UNSPECIFIED",
            Self::Created => "INVITATION_STATUS_CREATED",
            Self::Accepted => "INVITATION_STATUS_ACCEPTED",
            Self::Revoked => "INVITATION_STATUS_REVOKED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "INVITATION_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "INVITATION_STATUS_CREATED" => Some(Self::Created),
            "INVITATION_STATUS_ACCEPTED" => Some(Self::Accepted),
            "INVITATION_STATUS_REVOKED" => Some(Self::Revoked),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TagType {
    #[serde(rename = "TAG_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "TAG_TYPE_USER")]
    User = 1,
    #[serde(rename = "TAG_TYPE_PRIVATE_KEY")]
    PrivateKey = 3,
}
impl TagType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "TAG_TYPE_UNSPECIFIED",
            Self::User => "TAG_TYPE_USER",
            Self::PrivateKey => "TAG_TYPE_PRIVATE_KEY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TAG_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "TAG_TYPE_USER" => Some(Self::User),
            "TAG_TYPE_PRIVATE_KEY" => Some(Self::PrivateKey),
            _ => None,
        }
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct BootProof {
    pub ephemeral_public_key_hex: ::prost::alloc::string::String,
    pub aws_attestation_doc_b64: ::prost::alloc::string::String,
    pub qos_manifest_b64: ::prost::alloc::string::String,
    pub qos_manifest_envelope_b64: ::prost::alloc::string::String,
    pub deployment_label: ::prost::alloc::string::String,
    pub enclave_app: ::prost::alloc::string::String,
    pub owner: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AppProof {
    pub scheme: SignatureScheme,
    pub public_key: ::prost::alloc::string::String,
    pub proof_payload: ::prost::alloc::string::String,
    pub signature: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AppProofPayload {
    pub r#type: AppProofType,
    pub timestamp_ms: ::prost::alloc::string::String,
    #[serde(default)]
    pub proof_payload: ::core::option::Option<app_proof_payload::ProofPayload>,
}
/// Nested message and enum types in `AppProofPayload`.
pub mod app_proof_payload {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum ProofPayload {
        #[serde(rename = "PROOF_PAYLOAD_ADDRESS_DERIVATION_PROOF")]
        AddressDerivationProof(super::AddressDerivationProofPayload),
        #[serde(rename = "PROOF_PAYLOAD_POLICY_OUTCOME_PROOF")]
        PolicyOutcomeProof(super::PolicyOutcomeProofPayload),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AddressDerivationProofPayload {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet_id: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub derivation_path: ::core::option::Option<::prost::alloc::string::String>,
    pub address: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PolicyOutcomeProofPayload {
    pub organization_id: ::prost::alloc::string::String,
    pub outcome: super::super::super::immutable::common::v1::Outcome,
    pub decision_context_digest: ::prost::alloc::string::String,
    pub organization_data_digest: ::prost::alloc::string::String,
    pub parent_organization_data_digest: ::prost::alloc::string::String,
    #[serde(default)]
    pub user_request_approvals: ::prost::alloc::vec::Vec<
        super::super::super::immutable::models::v1::Signature,
    >,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignatureScheme {
    #[serde(rename = "SIGNATURE_SCHEME_UNSPECIFIED")]
    Unspecified = 0,
    /// Scheme used by our enclave applications
    #[serde(rename = "SIGNATURE_SCHEME_EPHEMERAL_KEY_P256")]
    EphemeralKeyP256 = 1,
}
impl SignatureScheme {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "SIGNATURE_SCHEME_UNSPECIFIED",
            Self::EphemeralKeyP256 => "SIGNATURE_SCHEME_EPHEMERAL_KEY_P256",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGNATURE_SCHEME_UNSPECIFIED" => Some(Self::Unspecified),
            "SIGNATURE_SCHEME_EPHEMERAL_KEY_P256" => Some(Self::EphemeralKeyP256),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AppProofType {
    #[serde(rename = "APP_PROOF_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "APP_PROOF_TYPE_ADDRESS_DERIVATION")]
    AddressDerivation = 1,
    #[serde(rename = "APP_PROOF_TYPE_POLICY_OUTCOME")]
    PolicyOutcome = 2,
}
impl AppProofType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "APP_PROOF_TYPE_UNSPECIFIED",
            Self::AddressDerivation => "APP_PROOF_TYPE_ADDRESS_DERIVATION",
            Self::PolicyOutcome => "APP_PROOF_TYPE_POLICY_OUTCOME",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "APP_PROOF_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "APP_PROOF_TYPE_ADDRESS_DERIVATION" => Some(Self::AddressDerivation),
            "APP_PROOF_TYPE_POLICY_OUTCOME" => Some(Self::PolicyOutcome),
            _ => None,
        }
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SmartContractInterface {
    pub organization_id: ::prost::alloc::string::String,
    pub smart_contract_interface_id: ::prost::alloc::string::String,
    pub smart_contract_address: ::prost::alloc::string::String,
    pub smart_contract_interface: ::prost::alloc::string::String,
    pub r#type: ::prost::alloc::string::String,
    pub label: ::prost::alloc::string::String,
    pub notes: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcApp {
    pub id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    pub quorum_public_key: ::prost::alloc::string::String,
    pub manifest_set_id: ::prost::alloc::string::String,
    pub share_set_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub external_connectivity: bool,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcDeployment {
    pub id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub app_id: ::prost::alloc::string::String,
    pub manifest_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub manifest: ::prost::alloc::vec::Vec<u8>,
    pub qos_version: ::prost::alloc::string::String,
    #[serde(default)]
    pub pivot_container: ::core::option::Option<TvcContainerSpec>,
    #[serde(default)]
    pub host_container: ::core::option::Option<TvcContainerSpec>,
    pub stage: super::super::super::immutable::common::v1::TvcDeploymentStage,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcContainerSpec {
    pub container_url: ::prost::alloc::string::String,
    pub path: ::prost::alloc::string::String,
    #[serde(default)]
    pub args: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub has_pull_secret: bool,
}
#[derive(Debug)]
/// An account derived from a Wallet
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WalletAccount {
    pub wallet_account_id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
    pub curve: super::super::super::immutable::common::v1::Curve,
    pub path_format: super::super::super::immutable::common::v1::PathFormat,
    pub path: ::prost::alloc::string::String,
    pub address_format: super::super::super::immutable::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
    /// TODO(tim): temporarily removing this since it's always "false"
    /// bool exported = 12 [
    ///   (google.api.field_behavior) = REQUIRED,
    ///   (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_field) = {description: "True when a given Account is exported, false otherwise."}
    /// ];
    #[serde(default)]
    pub wallet_details: ::core::option::Option<Wallet>,
}
