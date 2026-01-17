#[derive(Debug)]
/// An action that can that can be taken within the Turnkey infrastructure.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Activity {
    pub id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub status: super::super::super::immutable::activity::v1::ActivityStatus,
    pub r#type: super::super::super::immutable::activity::v1::ActivityType,
    #[serde(default)]
    pub intent: ::core::option::Option<
        super::super::super::immutable::activity::v1::Intent,
    >,
    #[serde(default)]
    pub result: ::core::option::Option<
        super::super::super::immutable::activity::v1::Result,
    >,
    #[serde(default)]
    pub votes: ::prost::alloc::vec::Vec<Vote>,
    #[serde(default)]
    pub app_proofs: ::prost::alloc::vec::Vec<super::super::data::v1::AppProof>,
    pub fingerprint: ::prost::alloc::string::String,
    #[serde(default)]
    pub can_approve: bool,
    #[serde(default)]
    pub can_reject: bool,
    #[serde(default)]
    pub created_at: ::core::option::Option<super::super::data::v1::Timestamp>,
    #[serde(default)]
    pub updated_at: ::core::option::Option<super::super::data::v1::Timestamp>,
    #[serde(default)]
    pub failure: ::core::option::Option<super::super::super::google::rpc::Status>,
}
#[derive(Debug)]
/// Object representing a particular User's approval or rejection of a Consensus request, including all relevant metadata.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Vote {
    pub id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub user: ::core::option::Option<super::super::data::v1::User>,
    pub activity_id: ::prost::alloc::string::String,
    pub selection: ::prost::alloc::string::String,
    pub message: ::prost::alloc::string::String,
    pub public_key: ::prost::alloc::string::String,
    pub signature: ::prost::alloc::string::String,
    pub scheme: ::prost::alloc::string::String,
    #[serde(default)]
    pub created_at: ::core::option::Option<super::super::data::v1::Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePaymentMethodRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::billing::DeletePaymentMethodIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ActivateBillingTierRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::billing::ActivateBillingTierIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetPaymentMethodRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::billing::SetPaymentMethodIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EnableAuthProxyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::EnableAuthProxyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DisableAuthProxyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DisableAuthProxyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePolicyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreatePolicyIntentV3,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePoliciesRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreatePoliciesIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePolicyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdatePolicyIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePolicyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeletePolicyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadOnlySessionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateReadOnlySessionIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadWriteSessionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateReadWriteSessionIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateInvitationsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateInvitationsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteInvitationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteInvitationIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AcceptInvitationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::AcceptInvitationIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiOnlyUsersRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateApiOnlyUsersIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeysRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreatePrivateKeysIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::SignRawPayloadIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::SignRawPayloadsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUsersRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateUsersIntentV3,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateUserIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserNameRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateUserNameIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserEmailRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateUserEmailIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserPhoneNumberRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateUserPhoneNumberIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUsersRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteUsersIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateAuthenticatorsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateAuthenticatorsIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteAuthenticatorsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteAuthenticatorsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiKeysRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateApiKeysIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteApiKeysRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteApiKeysIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUserTagRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateUserTagIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserTagRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateUserTagIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUserTagsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteUserTagsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeyTagRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreatePrivateKeyTagIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePrivateKeyTagRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdatePrivateKeyTagIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateAuthProxyConfigRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateAuthProxyConfigIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeyTagsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeletePrivateKeyTagsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOrganizationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateOrganizationIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOrganizationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteOrganizationIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignTransactionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::SignTransactionIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSmartContractInterfaceRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateSmartContractInterfaceIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteSmartContractInterfaceRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteSmartContractInterfaceIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApproveActivityRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ApproveActivityIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RejectActivityRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::RejectActivityIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateRootQuorumRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateRootQuorumIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateAllowedOriginsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateAllowedOriginsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateSubOrganizationIntentV7,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitUserEmailRecoveryRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitUserEmailRecoveryIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RecoverUserRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::RecoverUserIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportPrivateKeyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ExportPrivateKeyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ExportWalletIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletAccountRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ExportWalletAccountIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetOrganizationFeatureRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::SetOrganizationFeatureIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RemoveOrganizationFeatureRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::RemoveOrganizationFeatureIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateWalletIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletAccountsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateWalletAccountsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::OauthIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthLoginRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::OauthLoginIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct StampLoginRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::StampLoginIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpLoginRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::OtpLoginIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitOtpIntentV2,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpsertGasUsageConfigRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpsertGasUsageConfigIntent,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitFiatOnRampRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitFiatOnRampIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VerifyOtpRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::VerifyOtpIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitOtpAuthIntentV3,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpAuthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::OtpAuthIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::EmailAuthIntentV3,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportWalletRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitImportWalletIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportWalletRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ImportWalletIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportPrivateKeyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitImportPrivateKeyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportPrivateKeyRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::ImportPrivateKeyIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauthProvidersRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateOauthProvidersIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauthProvidersRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteOauthProvidersIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeysRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeletePrivateKeysIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteWalletsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteSubOrganizationRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteSubOrganizationIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateWalletRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateWalletIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauth2CredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateOauth2CredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateOauth2CredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateOauth2CredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauth2CredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteOauth2CredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateFiatOnRampCredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateFiatOnRampCredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateFiatOnRampCredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::UpdateFiatOnRampCredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteFiatOnRampCredentialRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteFiatOnRampCredentialIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
/// Represents a PolicyEvaluation which contains a set of policy evaluations for a given activity.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PolicyEvaluation {
    pub id: ::prost::alloc::string::String,
    /// The Activity this evaluation belongs to.
    pub activity_id: ::prost::alloc::string::String,
    /// The Organization this evaluation belongs to.
    pub organization_id: ::prost::alloc::string::String,
    /// The Vote associated with this evaluation.
    pub vote_id: ::prost::alloc::string::String,
    /// Individual policy evaluations (one entry per policy).
    #[serde(default)]
    pub policy_evaluations: ::prost::alloc::vec::Vec<
        super::super::super::immutable::common::v1::PolicyEvaluation,
    >,
    /// Time at which this evaluation was recorded.
    #[serde(default)]
    pub created_at: ::core::option::Option<super::super::data::v1::Timestamp>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Oauth2AuthenticateRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::Oauth2AuthenticateIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletAccountsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeleteWalletAccountsIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePoliciesRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::DeletePoliciesIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendRawTransactionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::EthSendRawTransactionIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendTransactionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::EthSendTransactionIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SolSendTransactionRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::SolSendTransactionIntent,
    >,
    #[serde(default)]
    pub generate_app_proofs: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcAppRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateTvcAppIntent,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcDeploymentRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateTvcDeploymentIntent,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcManifestApprovalsRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::CreateTvcManifestApprovalsIntent,
    >,
}
