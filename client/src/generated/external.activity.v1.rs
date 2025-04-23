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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitUserEmailRecoveryRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitUserEmailRecoveryIntent,
    >,
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::InitOtpAuthIntentV2,
    >,
}
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
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthRequest {
    pub r#type: ::prost::alloc::string::String,
    pub timestamp_ms: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub parameters: ::core::option::Option<
        super::super::super::immutable::activity::v1::EmailAuthIntentV2,
    >,
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
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
}
