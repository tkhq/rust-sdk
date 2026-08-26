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
    #[serde(default)]
    pub mfa_policies: ::prost::alloc::vec::Vec<MfaPolicy>,
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
    /// The session profile associated with this credential, if any (only CREDENTIAL_TYPE_LOGIN credentials)
    #[serde(default)]
    pub session_profile_id: ::core::option::Option<::prost::alloc::string::String>,
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
    #[serde(default)]
    pub time: ::core::option::Option<::prost::alloc::string::String>,
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
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MfaPolicy {
    pub mfa_policy_id: ::prost::alloc::string::String,
    pub mfa_policy_name: ::prost::alloc::string::String,
    pub condition: ::prost::alloc::string::String,
    #[serde(default)]
    pub required_authentication_methods: ::prost::alloc::vec::Vec<
        RequiredAuthenticationMethod,
    >,
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub notes: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RequiredAuthenticationMethod {
    #[serde(default)]
    pub any: ::prost::alloc::vec::Vec<AuthenticationMethod>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AuthenticationMethod {
    pub r#type: super::super::super::immutable::common::v1::AuthenticationType,
    #[serde(default)]
    pub id: ::core::option::Option<::prost::alloc::string::String>,
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
    #[serde(default)]
    pub qos_manifest_version: ::core::option::Option<::prost::alloc::string::String>,
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
/// A Velocity Control aggregates data and compares the result to a threshold. Policies use the boolean result.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControl {
    /// Unique identifier for the Velocity Control.
    pub velocity_control_id: ::prost::alloc::string::String,
    /// Identifier of the Organization that owns the Velocity Control.
    pub organization_id: ::prost::alloc::string::String,
    /// Human-readable name for the Velocity Control.
    pub name: ::prost::alloc::string::String,
    /// Data source for the Velocity Control.
    #[serde(default)]
    pub data_source: ::core::option::Option<VelocityControlDataSource>,
    /// Aggregation expression that the Velocity Control evaluates.
    #[serde(default)]
    pub aggregation: ::core::option::Option<VelocityControlAggregation>,
    /// Time when the Velocity Control was created.
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    /// Time when the Velocity Control was last updated.
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    /// Identifier for the Velocity Control. Policies reference it as `controls.<identifier>`. It must be unique within the Organization.
    pub identifier: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Data source for a Velocity Control. Set exactly one definition.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSource {
    #[serde(default)]
    pub definition: ::core::option::Option<velocity_control_data_source::Definition>,
}
/// Nested message and enum types in `VelocityControlDataSource`.
pub mod velocity_control_data_source {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum Definition {
        /// Uses transfers of the listed on-chain assets as input data.
        #[serde(rename = "DEFINITION_CHAIN_ASSET_TRANSFER")]
        ChainAssetTransfer(super::VelocityControlDataSourceChainAssetTransfer),
        /// Uses executed Turnkey activities as input data.
        #[serde(rename = "DEFINITION_ACTIVITY_EXECUTION")]
        ActivityExecution(super::VelocityControlDataSourceActivityExecution),
    }
}
#[derive(Debug)]
/// Uses transfers of the listed on-chain assets as input data.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceChainAssetTransfer {
    /// Assets whose transfers are included in the data source.
    #[serde(default)]
    pub definition: ::prost::alloc::vec::Vec<
        VelocityControlDataSourceChainAssetTransferDefinition,
    >,
    /// Filters asset transfers by activity type.
    #[serde(default)]
    pub filter: ::core::option::Option<
        VelocityControlDataSourceChainAssetTransferFilter,
    >,
    /// Selects when to measure an asset transfer.
    #[serde(default)]
    pub phase: ::core::option::Option<VelocityControlDataSourcePhase>,
}
#[derive(Debug)]
/// A chain asset that provides transfer data.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceChainAssetTransferDefinition {
    /// CAIP-19 identifier for the asset.
    pub caip19: ::prost::alloc::string::String,
    /// Base-10 integer string from 0 through 255 that specifies the number of decimal places for the asset.
    pub decimals: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Filters chain asset transfers by activity type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceChainAssetTransferFilter {
    /// Activity types whose asset transfers are included.
    #[serde(default)]
    pub activity: ::prost::alloc::vec::Vec<VelocityControlDataSourceFilterActivity>,
}
#[derive(Debug)]
/// Uses executed Turnkey activities as input data.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceActivityExecution {
    /// Filters activity executions by type.
    #[serde(default)]
    pub filter: ::core::option::Option<VelocityControlDataSourceActivityExecutionFilter>,
}
#[derive(Debug)]
/// Filters activity executions by type.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceActivityExecutionFilter {
    /// Activity types whose executions are included.
    #[serde(default)]
    pub activity: ::prost::alloc::vec::Vec<VelocityControlDataSourceFilterActivity>,
}
#[derive(Debug)]
/// An activity type filter.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlDataSourceFilterActivity {
    /// Name of an activity type to include, such as `ACTIVITY_TYPE_SOL_SEND_TRANSACTION`.
    pub activity_type: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Selects when to measure an asset transfer. Set exactly one definition.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlDataSourcePhase {
    #[serde(default)]
    pub definition: ::core::option::Option<
        velocity_control_data_source_phase::Definition,
    >,
}
/// Nested message and enum types in `VelocityControlDataSourcePhase`.
pub mod velocity_control_data_source_phase {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, Copy, PartialEq)]
    #[derive(Debug)]
    pub enum Definition {
        /// Measures the transfer when Turnkey signs the transaction and returns it to the user.
        #[serde(rename = "DEFINITION_SIGNATURE")]
        Signature(super::VelocityControlDataSourcePhaseSignature),
        /// Reserved for future use. Measures the transfer after the transaction lands on chain.
        #[serde(rename = "DEFINITION_SUBMISSION")]
        Submission(super::VelocityControlDataSourcePhaseSubmission),
    }
}
#[derive(Debug)]
/// Measures a transfer when Turnkey signs the transaction.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlDataSourcePhaseSignature {}
#[derive(Debug)]
/// Reserved for future measurement after a transaction lands on chain.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlDataSourcePhaseSubmission {}
#[derive(Debug)]
/// Defines how the control aggregates data and compares the result to a threshold.
/// The expression has the form `<method>(<scoped data>) <operator> <threshold>`.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlAggregation {
    /// Method that aggregates matching data points.
    pub method: VelocityControlAggregationMethod,
    /// Comparison between the aggregate and the threshold.
    pub operator: VelocityControlAggregationOperator,
    /// Non-negative base-10 decimal string with at most 38 total digits and 18 fractional digits.
    pub threshold: ::prost::alloc::string::String,
    /// Time window for the aggregation.
    #[serde(default)]
    pub window: ::core::option::Option<VelocityControlAggregationWindow>,
    /// Scope that partitions matching data before aggregation.
    #[serde(default)]
    pub group_by: ::core::option::Option<VelocityControlAggregationGroupBy>,
}
#[derive(Debug)]
/// Time window for aggregation. Set exactly one definition.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlAggregationWindow {
    #[serde(default)]
    pub definition: ::core::option::Option<
        velocity_control_aggregation_window::Definition,
    >,
}
/// Nested message and enum types in `VelocityControlAggregationWindow`.
pub mod velocity_control_aggregation_window {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum Definition {
        /// Uses a rolling time window.
        #[serde(rename = "DEFINITION_ROLLING")]
        Rolling(super::VelocityControlAggregationWindowRolling),
        /// Uses all matching data without a time limit.
        #[serde(rename = "DEFINITION_INFINITE")]
        Infinite(super::VelocityControlAggregationWindowInfinite),
    }
}
#[derive(Debug)]
/// A rolling time window.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VelocityControlAggregationWindowRolling {
    /// Duration of the rolling window, in seconds, as a base-10 integer string.
    pub duration: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// An aggregation window without a time limit.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlAggregationWindowInfinite {}
#[derive(Debug)]
/// Scope that partitions matching data before aggregation. Set exactly one definition.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlAggregationGroupBy {
    #[serde(default)]
    pub definition: ::core::option::Option<
        velocity_control_aggregation_group_by::Definition,
    >,
}
/// Nested message and enum types in `VelocityControlAggregationGroupBy`.
pub mod velocity_control_aggregation_group_by {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, Copy, PartialEq)]
    #[derive(Debug)]
    pub enum Definition {
        /// Uses one shared bucket for the Organization.
        #[serde(rename = "DEFINITION_ORGANIZATION")]
        Organization(super::VelocityControlAggregationGroupByOrganization),
        /// Uses one bucket for each User.
        #[serde(rename = "DEFINITION_USER")]
        User(super::VelocityControlAggregationGroupByUser),
        /// Uses one bucket for each Wallet.
        #[serde(rename = "DEFINITION_WALLET")]
        Wallet(super::VelocityControlAggregationGroupByWallet),
    }
}
#[derive(Debug)]
/// Uses one shared bucket for the Organization.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlAggregationGroupByOrganization {}
#[derive(Debug)]
/// Uses one bucket for each User.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlAggregationGroupByUser {}
#[derive(Debug)]
/// Uses one bucket for each Wallet.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct VelocityControlAggregationGroupByWallet {}
/// Method used to aggregate control data.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VelocityControlAggregationMethod {
    /// The aggregation method is not specified.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_METHOD_UNSPECIFIED")]
    Unspecified = 0,
    /// Sums matching values.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_METHOD_SUM")]
    Sum = 1,
    /// Counts matching data points.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_METHOD_COUNT")]
    Count = 2,
}
impl VelocityControlAggregationMethod {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "VELOCITY_CONTROL_AGGREGATION_METHOD_UNSPECIFIED",
            Self::Sum => "VELOCITY_CONTROL_AGGREGATION_METHOD_SUM",
            Self::Count => "VELOCITY_CONTROL_AGGREGATION_METHOD_COUNT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "VELOCITY_CONTROL_AGGREGATION_METHOD_UNSPECIFIED" => Some(Self::Unspecified),
            "VELOCITY_CONTROL_AGGREGATION_METHOD_SUM" => Some(Self::Sum),
            "VELOCITY_CONTROL_AGGREGATION_METHOD_COUNT" => Some(Self::Count),
            _ => None,
        }
    }
}
/// Comparison used to evaluate aggregated data against a threshold.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VelocityControlAggregationOperator {
    /// The comparison is not specified.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_UNSPECIFIED")]
    Unspecified = 0,
    /// Checks whether the aggregate is less than the threshold.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN")]
    LessThan = 1,
    /// Checks whether the aggregate is less than or equal to the threshold.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN_OR_EQUAL")]
    LessThanOrEqual = 2,
    /// Checks whether the aggregate equals the threshold.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_EQUAL")]
    Equal = 3,
    /// Checks whether the aggregate is greater than or equal to the threshold.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN_OR_EQUAL")]
    GreaterThanOrEqual = 4,
    /// Checks whether the aggregate is greater than the threshold.
    #[serde(rename = "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN")]
    GreaterThan = 5,
}
impl VelocityControlAggregationOperator {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "VELOCITY_CONTROL_AGGREGATION_OPERATOR_UNSPECIFIED",
            Self::LessThan => "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN",
            Self::LessThanOrEqual => {
                "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN_OR_EQUAL"
            }
            Self::Equal => "VELOCITY_CONTROL_AGGREGATION_OPERATOR_EQUAL",
            Self::GreaterThanOrEqual => {
                "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN_OR_EQUAL"
            }
            Self::GreaterThan => "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_UNSPECIFIED" => {
                Some(Self::Unspecified)
            }
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN" => Some(Self::LessThan),
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_LESS_THAN_OR_EQUAL" => {
                Some(Self::LessThanOrEqual)
            }
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_EQUAL" => Some(Self::Equal),
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN_OR_EQUAL" => {
                Some(Self::GreaterThanOrEqual)
            }
            "VELOCITY_CONTROL_AGGREGATION_OPERATOR_GREATER_THAN" => {
                Some(Self::GreaterThan)
            }
            _ => None,
        }
    }
}
#[derive(Debug)]
/// EarnValueDisplay holds normalized, display-only representations of an on-chain
/// amount. Values are for presentation only — do not do arithmetic with them;
/// use the corresponding raw atomic field instead.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnValueDisplay {
    pub usd: ::prost::alloc::string::String,
    pub crypto: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnVault {
    pub vault_address: ::prost::alloc::string::String,
    pub provider: super::super::super::immutable::data::v1::EarnProvider,
    pub caip19: ::prost::alloc::string::String,
    pub tvl: ::prost::alloc::string::String,
    pub apy_pct: ::prost::alloc::string::String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub display: ::core::option::Option<EarnValueDisplay>,
    pub name: ::prost::alloc::string::String,
    pub curator: ::prost::alloc::string::String,
    pub liquidity: ::prost::alloc::string::String,
    #[serde(default)]
    pub liquidity_display: ::core::option::Option<EarnValueDisplay>,
}
#[derive(Debug)]
/// EarnVaultExposure is one underlying market a vault allocates into, and the
/// share of the vault's assets held there.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnVaultExposure {
    pub market_id: ::prost::alloc::string::String,
    pub collateral_symbol: ::prost::alloc::string::String,
    pub collateral_caip19: ::prost::alloc::string::String,
    pub lltv_pct: ::prost::alloc::string::String,
    pub supplied: ::prost::alloc::string::String,
    #[serde(default)]
    pub display: ::core::option::Option<EarnValueDisplay>,
    pub share_pct: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnEnabledVault {
    pub vault_address: ::prost::alloc::string::String,
    pub wrapper_address: ::prost::alloc::string::String,
    pub provider: super::super::super::immutable::data::v1::EarnProvider,
    pub caip19: ::prost::alloc::string::String,
    pub apy_pct: ::prost::alloc::string::String,
    pub total_deposited: ::prost::alloc::string::String,
    #[serde(default)]
    pub display: ::core::option::Option<EarnValueDisplay>,
    pub net_apy_pct: ::prost::alloc::string::String,
    pub client_fee_bps: ::prost::alloc::string::String,
    #[serde(default)]
    pub deposits_disabled: bool,
    pub name: ::prost::alloc::string::String,
    pub curator: ::prost::alloc::string::String,
    #[serde(default)]
    pub claimable_client_fee: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub claimable_client_fee_display: ::core::option::Option<EarnValueDisplay>,
    #[serde(default)]
    pub client_fee_wallet: ::core::option::Option<::prost::alloc::string::String>,
    pub liquidity: ::prost::alloc::string::String,
    #[serde(default)]
    pub liquidity_display: ::core::option::Option<EarnValueDisplay>,
    #[serde(default)]
    pub exposures: ::prost::alloc::vec::Vec<EarnVaultExposure>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AssetMetadata {
    pub caip19: ::prost::alloc::string::String,
    pub symbol: ::prost::alloc::string::String,
    #[serde(default)]
    pub decimals: i32,
    pub logo_url: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    #[serde(default)]
    pub stable: bool,
    #[serde(default)]
    pub earn_providers: Vec<super::super::super::immutable::data::v1::EarnProvider>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct MfaStatus {
    pub mfa_policy_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub satisfied: bool,
    #[serde(default)]
    pub satisfied_methods: ::prost::alloc::vec::Vec<AuthenticationMethod>,
    #[serde(default)]
    pub required_methods: ::prost::alloc::vec::Vec<RequiredAuthenticationMethod>,
}
#[derive(Debug)]
/// SessionProfile defines the constraints and capabilities of a session that can be created by a user.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SessionProfile {
    pub session_profile_id: ::prost::alloc::string::String,
    pub session_profile_name: ::prost::alloc::string::String,
    pub scope: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub notes: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
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
    #[serde(default)]
    pub manifest_set: ::core::option::Option<TvcOperatorSet>,
    #[serde(default)]
    pub share_set: ::core::option::Option<TvcOperatorSet>,
    #[serde(default)]
    pub enable_egress: bool,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub live_deployment_id: ::core::option::Option<::prost::alloc::string::String>,
    pub public_domain: ::prost::alloc::string::String,
    #[serde(default)]
    pub enable_debug_mode_deployments: bool,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcDeployment {
    pub id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub app_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub manifest_set: ::core::option::Option<TvcOperatorSet>,
    #[serde(default)]
    pub share_set: ::core::option::Option<TvcOperatorSet>,
    #[serde(default)]
    pub manifest: ::core::option::Option<TvcManifest>,
    #[serde(default)]
    pub manifest_approvals: ::prost::alloc::vec::Vec<TvcOperatorApproval>,
    pub qos_version: ::prost::alloc::string::String,
    #[serde(default)]
    pub pivot_container: ::core::option::Option<TvcContainerSpec>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub delete: bool,
    #[serde(default)]
    pub debug_mode: bool,
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
    pub health_check_type: super::super::super::immutable::common::v1::TvcHealthCheckType,
    #[serde(default)]
    pub health_check_port: u32,
    #[serde(default)]
    pub public_ingress_port: u32,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcOperatorApproval {
    pub id: ::prost::alloc::string::String,
    pub manifest_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub operator: ::core::option::Option<TvcOperator>,
    #[serde(default)]
    #[serde_as(as = "serde_with::base64::Base64")]
    pub approval: ::prost::alloc::vec::Vec<u8>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcOperatorSet {
    pub id: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub operators: ::prost::alloc::vec::Vec<TvcOperator>,
    #[serde(default)]
    pub threshold: u32,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcOperator {
    pub id: ::prost::alloc::string::String,
    pub name: ::prost::alloc::string::String,
    pub public_key: ::prost::alloc::string::String,
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
pub struct TvcManifest {
    pub id: ::prost::alloc::string::String,
    #[serde(default)]
    #[serde_as(as = "serde_with::base64::Base64")]
    pub manifest: ::prost::alloc::vec::Vec<u8>,
    #[serde(default)]
    pub created_at: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeploymentStatus {
    pub deployment_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub ready_replicas: i32,
    #[serde(default)]
    pub desired_replicas: i32,
    #[serde(default)]
    pub last_updated_time: ::core::option::Option<Timestamp>,
    #[serde(default)]
    pub provisioning_state: Option<ProvisioningState>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AppStatus {
    pub app_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub deployments: ::prost::alloc::vec::Vec<DeploymentStatus>,
    pub targeted_deployment_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct LogLine {
    pub content: ::prost::alloc::string::String,
    #[serde(default)]
    pub ts: ::core::option::Option<Timestamp>,
}
/// ProvisioningState describes the observed quorum-key provisioning progress of
/// a TVC deployment.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProvisioningState {
    /// The provisioning state was not reported. This value supports responses
    /// produced before provisioning state reporting was introduced.
    #[serde(rename = "PROVISIONING_STATE_UNSPECIFIED")]
    Unspecified = 0,
    /// The deployment is starting up and provisioning details are not yet
    /// available.
    #[serde(rename = "PROVISIONING_STATE_PENDING")]
    Pending = 1,
    /// A deployment replica is ready to accept quorum key provisioning.
    #[serde(rename = "PROVISIONING_STATE_AWAITING_PROVISION")]
    AwaitingProvision = 2,
    /// At least one deployment replica has a provisioned quorum key.
    #[serde(rename = "PROVISIONING_STATE_PROVISIONED")]
    Provisioned = 3,
}
impl ProvisioningState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "PROVISIONING_STATE_UNSPECIFIED",
            Self::Pending => "PROVISIONING_STATE_PENDING",
            Self::AwaitingProvision => "PROVISIONING_STATE_AWAITING_PROVISION",
            Self::Provisioned => "PROVISIONING_STATE_PROVISIONED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PROVISIONING_STATE_UNSPECIFIED" => Some(Self::Unspecified),
            "PROVISIONING_STATE_PENDING" => Some(Self::Pending),
            "PROVISIONING_STATE_AWAITING_PROVISION" => Some(Self::AwaitingProvision),
            "PROVISIONING_STATE_PROVISIONED" => Some(Self::Provisioned),
            _ => None,
        }
    }
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
    #[serde(default)]
    pub wallet_details: ::core::option::Option<Wallet>,
    #[serde(default)]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// TODO(tim): temporarily removing this since it's always "false"
    /// bool exported = 12 [
    ///   (google.api.field_behavior) = REQUIRED,
    ///   (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_field) = {description: "True when a given Account is exported, false otherwise."}
    /// ];
    #[serde(default)]
    pub caip2_prefix: ::core::option::Option<::prost::alloc::string::String>,
}
