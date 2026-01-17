#[derive(Debug)]
/// Intent object crafted by Turnkey based on the user request, used to assess the permissibility of an action.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Intent {
    #[serde(default)]
    #[serde(flatten)]
    pub inner: ::core::option::Option<intent::Inner>,
}
/// Nested message and enum types in `Intent`.
pub mod intent {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    #[derive(Debug)]
    pub enum Inner {
        CreateOrganizationIntent(super::CreateOrganizationIntent),
        CreateAuthenticatorsIntent(super::CreateAuthenticatorsIntent),
        CreateUsersIntent(super::CreateUsersIntent),
        CreatePrivateKeysIntent(super::CreatePrivateKeysIntent),
        SignRawPayloadIntent(super::SignRawPayloadIntent),
        CreateInvitationsIntent(super::CreateInvitationsIntent),
        AcceptInvitationIntent(super::AcceptInvitationIntent),
        CreatePolicyIntent(super::CreatePolicyIntent),
        DisablePrivateKeyIntent(super::DisablePrivateKeyIntent),
        DeleteUsersIntent(super::DeleteUsersIntent),
        DeleteAuthenticatorsIntent(super::DeleteAuthenticatorsIntent),
        DeleteInvitationIntent(super::DeleteInvitationIntent),
        DeleteOrganizationIntent(super::DeleteOrganizationIntent),
        DeletePolicyIntent(super::DeletePolicyIntent),
        CreateUserTagIntent(super::CreateUserTagIntent),
        DeleteUserTagsIntent(super::DeleteUserTagsIntent),
        SignTransactionIntent(super::SignTransactionIntent),
        CreateApiKeysIntent(super::CreateApiKeysIntent),
        DeleteApiKeysIntent(super::DeleteApiKeysIntent),
        ApproveActivityIntent(super::ApproveActivityIntent),
        RejectActivityIntent(super::RejectActivityIntent),
        CreatePrivateKeyTagIntent(super::CreatePrivateKeyTagIntent),
        DeletePrivateKeyTagsIntent(super::DeletePrivateKeyTagsIntent),
        CreatePolicyIntentV2(super::CreatePolicyIntentV2),
        SetPaymentMethodIntent(super::super::billing::SetPaymentMethodIntent),
        ActivateBillingTierIntent(super::super::billing::ActivateBillingTierIntent),
        DeletePaymentMethodIntent(super::super::billing::DeletePaymentMethodIntent),
        CreatePolicyIntentV3(super::CreatePolicyIntentV3),
        CreateApiOnlyUsersIntent(super::CreateApiOnlyUsersIntent),
        UpdateRootQuorumIntent(super::UpdateRootQuorumIntent),
        UpdateUserTagIntent(super::UpdateUserTagIntent),
        UpdatePrivateKeyTagIntent(super::UpdatePrivateKeyTagIntent),
        CreateAuthenticatorsIntentV2(super::CreateAuthenticatorsIntentV2),
        AcceptInvitationIntentV2(super::AcceptInvitationIntentV2),
        CreateOrganizationIntentV2(super::CreateOrganizationIntentV2),
        CreateUsersIntentV2(super::CreateUsersIntentV2),
        CreateSubOrganizationIntent(super::CreateSubOrganizationIntent),
        CreateSubOrganizationIntentV2(super::CreateSubOrganizationIntentV2),
        UpdateAllowedOriginsIntent(super::UpdateAllowedOriginsIntent),
        CreatePrivateKeysIntentV2(super::CreatePrivateKeysIntentV2),
        UpdateUserIntent(super::UpdateUserIntent),
        UpdatePolicyIntent(super::UpdatePolicyIntent),
        SetPaymentMethodIntentV2(super::super::billing::SetPaymentMethodIntentV2),
        CreateSubOrganizationIntentV3(super::CreateSubOrganizationIntentV3),
        CreateWalletIntent(super::CreateWalletIntent),
        CreateWalletAccountsIntent(super::CreateWalletAccountsIntent),
        InitUserEmailRecoveryIntent(super::InitUserEmailRecoveryIntent),
        RecoverUserIntent(super::RecoverUserIntent),
        SetOrganizationFeatureIntent(super::SetOrganizationFeatureIntent),
        RemoveOrganizationFeatureIntent(super::RemoveOrganizationFeatureIntent),
        SignRawPayloadIntentV2(super::SignRawPayloadIntentV2),
        SignTransactionIntentV2(super::SignTransactionIntentV2),
        ExportPrivateKeyIntent(super::ExportPrivateKeyIntent),
        ExportWalletIntent(super::ExportWalletIntent),
        CreateSubOrganizationIntentV4(super::CreateSubOrganizationIntentV4),
        EmailAuthIntent(super::EmailAuthIntent),
        ExportWalletAccountIntent(super::ExportWalletAccountIntent),
        InitImportWalletIntent(super::InitImportWalletIntent),
        ImportWalletIntent(super::ImportWalletIntent),
        InitImportPrivateKeyIntent(super::InitImportPrivateKeyIntent),
        ImportPrivateKeyIntent(super::ImportPrivateKeyIntent),
        CreatePoliciesIntent(super::CreatePoliciesIntent),
        SignRawPayloadsIntent(super::SignRawPayloadsIntent),
        CreateReadOnlySessionIntent(super::CreateReadOnlySessionIntent),
        CreateOauthProvidersIntent(super::CreateOauthProvidersIntent),
        DeleteOauthProvidersIntent(super::DeleteOauthProvidersIntent),
        CreateSubOrganizationIntentV5(super::CreateSubOrganizationIntentV5),
        OauthIntent(super::OauthIntent),
        CreateApiKeysIntentV2(super::CreateApiKeysIntentV2),
        CreateReadWriteSessionIntent(super::CreateReadWriteSessionIntent),
        EmailAuthIntentV2(super::EmailAuthIntentV2),
        CreateSubOrganizationIntentV6(super::CreateSubOrganizationIntentV6),
        DeletePrivateKeysIntent(super::DeletePrivateKeysIntent),
        DeleteWalletsIntent(super::DeleteWalletsIntent),
        CreateReadWriteSessionIntentV2(super::CreateReadWriteSessionIntentV2),
        DeleteSubOrganizationIntent(super::DeleteSubOrganizationIntent),
        InitOtpAuthIntent(super::InitOtpAuthIntent),
        OtpAuthIntent(super::OtpAuthIntent),
        CreateSubOrganizationIntentV7(super::CreateSubOrganizationIntentV7),
        UpdateWalletIntent(super::UpdateWalletIntent),
        UpdatePolicyIntentV2(super::UpdatePolicyIntentV2),
        CreateUsersIntentV3(super::CreateUsersIntentV3),
        InitOtpAuthIntentV2(super::InitOtpAuthIntentV2),
        InitOtpIntent(super::InitOtpIntent),
        VerifyOtpIntent(super::VerifyOtpIntent),
        OtpLoginIntent(super::OtpLoginIntent),
        StampLoginIntent(super::StampLoginIntent),
        OauthLoginIntent(super::OauthLoginIntent),
        UpdateUserNameIntent(super::UpdateUserNameIntent),
        UpdateUserEmailIntent(super::UpdateUserEmailIntent),
        UpdateUserPhoneNumberIntent(super::UpdateUserPhoneNumberIntent),
        InitFiatOnRampIntent(super::InitFiatOnRampIntent),
        CreateSmartContractInterfaceIntent(super::CreateSmartContractInterfaceIntent),
        DeleteSmartContractInterfaceIntent(super::DeleteSmartContractInterfaceIntent),
        EnableAuthProxyIntent(super::EnableAuthProxyIntent),
        DisableAuthProxyIntent(super::DisableAuthProxyIntent),
        UpdateAuthProxyConfigIntent(super::UpdateAuthProxyConfigIntent),
        CreateOauth2CredentialIntent(super::CreateOauth2CredentialIntent),
        UpdateOauth2CredentialIntent(super::UpdateOauth2CredentialIntent),
        DeleteOauth2CredentialIntent(super::DeleteOauth2CredentialIntent),
        Oauth2AuthenticateIntent(super::Oauth2AuthenticateIntent),
        DeleteWalletAccountsIntent(super::DeleteWalletAccountsIntent),
        DeletePoliciesIntent(super::DeletePoliciesIntent),
        EthSendRawTransactionIntent(super::EthSendRawTransactionIntent),
        EthSendTransactionIntent(super::EthSendTransactionIntent),
        CreateFiatOnRampCredentialIntent(super::CreateFiatOnRampCredentialIntent),
        UpdateFiatOnRampCredentialIntent(super::UpdateFiatOnRampCredentialIntent),
        DeleteFiatOnRampCredentialIntent(super::DeleteFiatOnRampCredentialIntent),
        EmailAuthIntentV3(super::EmailAuthIntentV3),
        InitUserEmailRecoveryIntentV2(super::InitUserEmailRecoveryIntentV2),
        InitOtpIntentV2(super::InitOtpIntentV2),
        InitOtpAuthIntentV3(super::InitOtpAuthIntentV3),
        UpsertGasUsageConfigIntent(super::UpsertGasUsageConfigIntent),
        CreateTvcAppIntent(super::CreateTvcAppIntent),
        CreateTvcDeploymentIntent(super::CreateTvcDeploymentIntent),
        CreateTvcManifestApprovalsIntent(super::CreateTvcManifestApprovalsIntent),
        SolSendTransactionIntent(super::SolSendTransactionIntent),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateAuthProxyConfigIntent {
    /// @inject_tag: validate:"omitempty,dive"
    #[serde(default)]
    pub allowed_origins: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,dive"
    #[serde(default)]
    pub allowed_auth_methods: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub email_auth_template_id: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,uuid"
    #[serde(default)]
    pub otp_template_id: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization_params: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub sms_customization_params: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub wallet_kit_settings: ::core::option::Option<WalletKitSettingsParams>,
    /// @inject_tag: validate:"omitempty,numeric"
    #[serde(default)]
    pub otp_expiration_seconds: ::core::option::Option<i32>,
    /// @inject_tag: validate:"omitempty,numeric"
    #[serde(default)]
    pub verification_token_expiration_seconds: ::core::option::Option<i32>,
    /// @inject_tag: validate:"omitempty,numeric"
    #[serde(default)]
    pub session_expiration_seconds: ::core::option::Option<i32>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub otp_alphanumeric: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,numeric,min=6,max=9"
    #[serde(default)]
    pub otp_length: ::core::option::Option<i32>,
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub verification_token_required_for_get_account_pii: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateAuthProxyConfigResult {
    /// @inject_tag: validate:"required,uuid"
    pub config_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct EnableAuthProxyIntent {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct DisableAuthProxyIntent {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOrganizationIntent {
    /// @inject_tag: validate:"required,tk_label_length"
    pub organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,email,tk_email"
    pub root_email: ::prost::alloc::string::String,
    #[serde(default)]
    pub root_authenticator: ::core::option::Option<AuthenticatorParams>,
    /// @inject_tag: validate:"uuid"
    #[serde(default)]
    pub root_user_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOrganizationIntentV2 {
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,email,tk_email"
    pub root_email: ::prost::alloc::string::String,
    #[serde(default)]
    pub root_authenticator: ::core::option::Option<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"uuid"
    #[serde(default)]
    pub root_user_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateAuthenticatorsIntent {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParams>,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateAuthenticatorsIntentV2 {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiKeysIntent {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiKeysIntentV2 {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<ApiKeyParamsV2>,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUsersIntent {
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub users: ::prost::alloc::vec::Vec<UserParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUsersIntentV2 {
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub users: ::prost::alloc::vec::Vec<UserParamsV2>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUsersIntentV3 {
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub users: ::prost::alloc::vec::Vec<UserParamsV3>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserIntent {
    /// @inject_tag: validate:"uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    #[serde(default)]
    pub user_name: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,dive,uuid"
    #[serde(default)]
    pub user_tag_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,e164"
    #[serde(default)]
    pub user_phone_number: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserNameIntent {
    /// @inject_tag: validate:"uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub user_name: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserEmailIntent {
    /// @inject_tag: validate:"uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    pub user_email: ::prost::alloc::string::String,
    #[serde(default)]
    pub verification_token: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserPhoneNumberIntent {
    /// @inject_tag: validate:"uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,e164"
    pub user_phone_number: ::prost::alloc::string::String,
    #[serde(default)]
    pub verification_token: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateWalletIntent {
    /// @inject_tag: validate:"uuid"
    pub wallet_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub wallet_name: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateInvitationsIntent {
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub invitations: ::prost::alloc::vec::Vec<InvitationParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AcceptInvitationIntent {
    /// @inject_tag: validate:"required,uuid"
    pub invitation_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub authenticator: ::core::option::Option<AuthenticatorParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AcceptInvitationIntentV2 {
    /// @inject_tag: validate:"required,uuid"
    pub invitation_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub authenticator: ::core::option::Option<AuthenticatorParamsV2>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiOnlyUsersIntent {
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub api_only_users: ::prost::alloc::vec::Vec<ApiOnlyUserParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletIntent {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub wallet_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub accounts: ::prost::alloc::vec::Vec<WalletAccountParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub mnemonic_length: ::core::option::Option<i32>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletAccountsIntent {
    /// @inject_tag: validate:"required,uuid"
    pub wallet_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub accounts: ::prost::alloc::vec::Vec<WalletAccountParams>,
    #[serde(default)]
    pub persist: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeysIntent {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKeyParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeysIntentV2 {
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKeyParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadIntent {
    /// @inject_tag: validate:"required,uuid"
    pub private_key_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub payload: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encoding: super::super::common::v1::PayloadEncoding,
    pub hash_function: super::super::common::v1::HashFunction,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadIntentV2 {
    /// @inject_tag: validate:"required"
    pub sign_with: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub payload: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encoding: super::super::common::v1::PayloadEncoding,
    pub hash_function: super::super::common::v1::HashFunction,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadsIntent {
    /// @inject_tag: validate:"required"
    pub sign_with: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub payloads: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    pub encoding: super::super::common::v1::PayloadEncoding,
    pub hash_function: super::super::common::v1::HashFunction,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePolicyIntent {
    /// @inject_tag: validate:"required,tk_label_length"
    pub policy_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub selectors: ::prost::alloc::vec::Vec<Selector>,
    pub effect: super::super::common::v1::Effect,
    pub notes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePolicyIntentV2 {
    /// @inject_tag: validate:"required,tk_label_length"
    pub policy_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub selectors: ::prost::alloc::vec::Vec<SelectorV2>,
    pub effect: super::super::common::v1::Effect,
    pub notes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePolicyIntentV3 {
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub policy_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub effect: super::super::common::v1::Effect,
    #[serde(default)]
    pub condition: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub consensus: ::core::option::Option<::prost::alloc::string::String>,
    pub notes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePoliciesIntent {
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub policies: ::prost::alloc::vec::Vec<CreatePolicyIntentV3>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct CreateReadOnlySessionIntent {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadWriteSessionIntent {
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadWriteSessionIntentV2 {
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,uuid"
    #[serde(default)]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Selector {
    pub subject: ::prost::alloc::string::String,
    pub operator: super::super::common::v1::Operator,
    pub target: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SelectorV2 {
    pub subject: ::prost::alloc::string::String,
    pub operator: super::super::common::v1::Operator,
    #[serde(default)]
    pub targets: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DisablePrivateKeyIntent {
    /// @inject_tag: validate:"required,uuid"
    pub private_key_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUsersIntent {
    /// @inject_tag: validate:"required,dive,required,uuid"
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteInvitationIntent {
    /// @inject_tag: validate:"required,uuid"
    pub invitation_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteApiKeysIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive,required,uuid"
    #[serde(default)]
    pub api_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteAuthenticatorsIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive,required,uuid"
    #[serde(default)]
    pub authenticator_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOrganizationIntent {
    /// @inject_tag: validate:"required,uuid"
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePolicyIntent {
    /// @inject_tag: validate:"required,uuid"
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUserTagIntent {
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub user_tag_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserTagIntent {
    /// @inject_tag: validate:"uuid"
    pub user_tag_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    #[serde(default)]
    pub new_user_tag_name: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub add_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub remove_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUserTagsIntent {
    /// @inject_tag: validate:"required,dive,required,uuid"
    #[serde(default)]
    pub user_tag_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeyTagIntent {
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub private_key_tag_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePrivateKeyTagIntent {
    /// @inject_tag: validate:"uuid"
    pub private_key_tag_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    #[serde(default)]
    pub new_private_key_tag_name: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub add_private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub remove_private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeyTagsIntent {
    /// @inject_tag: validate:"required,dive,required,uuid"
    #[serde(default)]
    pub private_key_tag_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignTransactionIntent {
    /// @inject_tag: validate:"required,uuid"
    pub private_key_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub unsigned_transaction: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub r#type: super::super::common::v1::TransactionType,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignTransactionIntentV2 {
    /// @inject_tag: validate:"required"
    pub sign_with: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub unsigned_transaction: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub r#type: super::super::common::v1::TransactionType,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SolSendTransactionIntent {
    /// @inject_tag: validate:"required"
    pub unsigned_transaction: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub sign_with: ::prost::alloc::string::String,
    /// If true, Turnkey acts as fee payer and may inject a fresh blockhash
    #[serde(default)]
    pub sponsor: ::core::option::Option<bool>,
    /// @inject_tag: validate:"required"
    pub caip2: ::prost::alloc::string::String,
    #[serde(default)]
    pub recent_blockhash: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendTransactionIntent {
    /// @inject_tag: validate:"required"
    pub from: ::prost::alloc::string::String,
    /// If false or unset, constructs a standard EIP-1559 transaction. If true, constructs an EIP-712 meta-transaction for Gas Station.
    #[serde(default)]
    pub sponsor: ::core::option::Option<bool>,
    /// @inject_tag: validate:"required"
    pub caip2: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub to: ::prost::alloc::string::String,
    #[serde(default)]
    pub value: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub data: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub nonce: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub gas_limit: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub max_fee_per_gas: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub max_priority_fee_per_gas: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub gas_station_nonce: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApproveActivityIntent {
    /// @inject_tag: validate:"required"
    pub fingerprint: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RejectActivityIntent {
    /// @inject_tag: validate:"required"
    pub fingerprint: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateRootQuorumIntent {
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub threshold: i32,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateAllowedOriginsIntent {
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub allowed_origins: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSmartContractInterfaceIntent {
    /// @inject_tag: validate:"required"
    pub smart_contract_address: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_max_length=400000"
    pub smart_contract_interface: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub r#type: super::super::common::v1::SmartContractInterfaceType,
    /// @inject_tag: validate:"required,tk_label,tk_label_length"
    pub label: ::prost::alloc::string::String,
    pub notes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteSmartContractInterfaceIntent {
    /// @inject_tag: validate:"required"
    pub smart_contract_interface_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntent {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub name: ::prost::alloc::string::String,
    #[serde(default)]
    pub root_authenticator: ::core::option::Option<AuthenticatorParamsV2>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV2 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParams>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV3 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParams>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKeyParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV4 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParams>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_recovery: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_auth: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV5 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParamsV2>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_recovery: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_auth: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV6 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParamsV3>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_recovery: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_auth: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationIntentV7 {
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    pub sub_organization_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive"
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<RootUserParamsV4>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub root_quorum_threshold: i32,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_recovery: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_email_auth: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_sms_auth: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub disable_otp_email_auth: ::core::option::Option<bool>,
    #[serde(default)]
    pub verification_token: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub client_signature: ::core::option::Option<ClientSignature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePolicyIntent {
    /// @inject_tag: validate:"uuid"
    pub policy_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    #[serde(default)]
    pub policy_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_effect: Option<super::super::common::v1::Effect>,
    #[serde(default)]
    pub policy_condition: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_consensus: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_notes: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePolicyIntentV2 {
    /// @inject_tag: validate:"uuid"
    pub policy_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label,tk_label_length"
    #[serde(default)]
    pub policy_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_effect: Option<super::super::common::v1::Effect>,
    #[serde(default)]
    pub policy_condition: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_consensus: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub policy_notes: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RecoverUserIntent {
    #[serde(default)]
    pub authenticator: ::core::option::Option<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetOrganizationFeatureIntent {
    pub name: super::super::common::v1::FeatureName,
    #[serde(default)]
    pub value: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct RemoveOrganizationFeatureIntent {
    pub name: super::super::common::v1::FeatureName,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportPrivateKeyIntent {
    /// @inject_tag: validate:"required,uuid"
    pub private_key_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletIntent {
    /// @inject_tag: validate:"required,uuid"
    pub wallet_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub language: Option<super::super::common::v1::MnemonicLanguage>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletAccountIntent {
    /// @inject_tag: validate:"required"
    pub address: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportWalletIntent {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportPrivateKeyIntent {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RootUserParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RootUserParamsV2 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProviderParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RootUserParamsV3 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<ApiKeyParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProviderParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RootUserParamsV4 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,e164"
    #[serde(default)]
    pub user_phone_number: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<ApiKeyParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProviderParams>,
}
#[derive(Debug)]
/// Each of these customization parameters are optional; resort to defaults if any are not provided.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailCustomizationParams {
    #[serde(default)]
    pub app_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub logo_url: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub magic_link_template: ::core::option::Option<::prost::alloc::string::String>,
    ///
    /// We're electing to support user-provided dynamic template variables via JSON string.
    /// This is for a subset of customers who want to have custom email templates with Turnkey
    /// and the ability to update them on the fly.
    /// The procedure: provide a Turnkey eng the new template, and pass their desired variables through this field.
    /// These variables will get injected into their template. Since we have no control over the defined variables,
    /// we'll opt use a key-value map (JSON string) to set them. Note that we can't use protobuf maps due to serialization issues,
    /// which may produce issues with user request signature verification.
    #[serde(default)]
    pub template_variables: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub template_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// This proto message is to be used for newer email-related activities (OTP).
/// Note that app_name is no longer a parameter here, as it is required in the top-level intent for these activities.
/// All other fields remain optional and will fall back to defaults.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailCustomizationParamsV2 {
    #[serde(default)]
    pub logo_url: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub magic_link_template: ::core::option::Option<::prost::alloc::string::String>,
    ///
    /// We're electing to support user-provided dynamic template variables via JSON string.
    /// This is for a subset of customers who want to have custom email templates with Turnkey
    /// and the ability to update them on the fly.
    /// The procedure: provide a Turnkey eng the new template, and pass their desired variables through this field.
    /// These variables will get injected into their template. Since we have no control over the defined variables,
    /// we'll opt use a key-value map (JSON string) to set them. Note that we can't use protobuf maps due to serialization issues,
    /// which may produce issues with user request signature verification.
    #[serde(default)]
    pub template_variables: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub template_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// A new proto message specifically for "legacy" endpoints: Email Auth and Email Recovery.
/// Note that app_name is now a required parameter for newer versions of these activities.
/// All other fields remain optional and will fall back to defaults.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthCustomizationParams {
    /// @inject_tag: validate:"tk_label_length,tk_label"
    pub app_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub logo_url: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub magic_link_template: ::core::option::Option<::prost::alloc::string::String>,
    ///
    /// We're electing to support user-provided dynamic template variables via JSON string.
    /// This is for a subset of customers who want to have custom email templates with Turnkey
    /// and the ability to update them on the fly.
    /// The procedure: provide a Turnkey eng the new template, and pass their desired variables through this field.
    /// These variables will get injected into their template. Since we have no control over the defined variables,
    /// we'll opt use a key-value map (JSON string) to set them. Note that we can't use protobuf maps due to serialization issues,
    /// which may produce issues with user request signature verification.
    #[serde(default)]
    pub template_variables: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub template_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// Each of these customization parameters are optional; resort to defaults if any are not provided.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SmsCustomizationParams {
    #[serde(default)]
    pub template: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// The Wallet Kit pulls from these settings automatically. They can be overwritten locally by passing them into the TurnkeyProvider
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WalletKitSettingsParams {
    #[serde(default)]
    pub enabled_social_providers: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    /// Map of social login providers to their OAuth client IDs.
    /// Example: { "google": "123.apps.googleusercontent.com", "apple": "com.example.app" }
    #[serde(default)]
    pub oauth_client_ids: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    /// Global OAuth redirect URL used for social logins.
    pub oauth_redirect_url: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitUserEmailRecoveryIntent {
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitUserEmailRecoveryIntentV2 {
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailAuthCustomizationParams>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthLoginIntent {
    /// @inject_tag: validate:"required"
    pub oidc_token: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,hexadecimal"
    pub public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct StampLoginIntent {
    /// @inject_tag: validate:"omitempty,hexadecimal"
    pub public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpLoginIntent {
    pub verification_token: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,hexadecimal"
    pub public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
    #[serde(default)]
    pub client_signature: ::core::option::Option<ClientSignature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthIntent {
    /// @inject_tag: validate:"required,oneof=OTP_TYPE_SMS OTP_TYPE_EMAIL"
    pub otp_type: ::prost::alloc::string::String,
    pub contact: ::prost::alloc::string::String,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub sms_customization: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub user_identifier: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpIntent {
    /// @inject_tag: validate:"required,oneof=OTP_TYPE_SMS OTP_TYPE_EMAIL"
    pub otp_type: ::prost::alloc::string::String,
    pub contact: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,min=6,max=9"
    #[serde(default)]
    pub otp_length: ::core::option::Option<i32>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub sms_customization: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub user_identifier: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub alphanumeric: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,numeric,max=600"
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpIntentV2 {
    /// @inject_tag: validate:"required,oneof=OTP_TYPE_SMS OTP_TYPE_EMAIL"
    pub otp_type: ::prost::alloc::string::String,
    pub contact: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,min=6,max=9"
    #[serde(default)]
    pub otp_length: ::core::option::Option<i32>,
    /// @inject_tag: validate:"tk_label_length,tk_label"
    pub app_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParamsV2>,
    #[serde(default)]
    pub sms_customization: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub user_identifier: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub alphanumeric: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,numeric,max=600"
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthIntentV2 {
    /// @inject_tag: validate:"required,oneof=OTP_TYPE_SMS OTP_TYPE_EMAIL"
    pub otp_type: ::prost::alloc::string::String,
    pub contact: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,min=6,max=9"
    #[serde(default)]
    pub otp_length: ::core::option::Option<i32>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub sms_customization: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub user_identifier: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub alphanumeric: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthIntentV3 {
    /// @inject_tag: validate:"required,oneof=OTP_TYPE_SMS OTP_TYPE_EMAIL"
    pub otp_type: ::prost::alloc::string::String,
    pub contact: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,min=6,max=9"
    #[serde(default)]
    pub otp_length: ::core::option::Option<i32>,
    /// @inject_tag: validate:"tk_label_length,tk_label"
    pub app_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParamsV2>,
    #[serde(default)]
    pub sms_customization: ::core::option::Option<SmsCustomizationParams>,
    #[serde(default)]
    pub user_identifier: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub alphanumeric: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,numeric,max=600"
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpsertGasUsageConfigIntent {
    /// @inject_tag: validate:"required,numeric"
    pub org_window_limit_usd: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,numeric"
    pub sub_org_window_limit_usd: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,numeric"
    pub window_duration_minutes: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VerifyOtpIntent {
    /// @inject_tag: validate:"required"
    pub otp_id: ::prost::alloc::string::String,
    pub otp_code: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,numeric,max=86400"
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,hexadecimal"
    #[serde(default)]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpAuthIntent {
    /// @inject_tag: validate:"required"
    pub otp_id: ::prost::alloc::string::String,
    pub otp_code: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthIntent {
    /// @inject_tag: validate:"required"
    pub oidc_token: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthIntent {
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthIntentV2 {
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailCustomizationParams>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthIntentV3 {
    /// @inject_tag: validate:"email,tk_email"
    pub email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal"
    pub target_public_key: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub api_key_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub email_customization: ::core::option::Option<EmailAuthCustomizationParams>,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub send_from_email_address: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,tk_label_length,tk_label"
    #[serde(default)]
    pub send_from_email_sender_name: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub reply_to_email_address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitFiatOnRampIntent {
    /// @inject_tag: validate:"required"
    pub onramp_provider: super::super::common::v1::FiatOnRampProvider,
    /// @inject_tag: validate:"required"
    pub wallet_address: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub network: super::super::common::v1::FiatOnRampBlockchainNetwork,
    /// @inject_tag: validate:"required"
    pub crypto_currency_code: super::super::common::v1::FiatOnRampCryptoCurrency,
    #[serde(default)]
    pub fiat_currency_code: Option<super::super::common::v1::FiatOnRampCurrency>,
    #[serde(default)]
    pub fiat_currency_amount: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub payment_method: Option<super::super::common::v1::FiatOnRampPaymentMethod>,
    #[serde(default)]
    pub country_code: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub country_subdivision_code: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub sandbox_mode: ::core::option::Option<bool>,
    #[serde(default)]
    pub url_for_signature: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportWalletIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub wallet_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encrypted_bundle: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub accounts: ::prost::alloc::vec::Vec<WalletAccountParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportPrivateKeyIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub private_key_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encrypted_bundle: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub curve: super::super::common::v1::Curve,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub address_formats: Vec<super::super::common::v1::AddressFormat>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauthProvidersIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,dive,required"
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProviderParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauthProvidersIntent {
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,required,uuid"
    #[serde(default)]
    pub provider_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeysIntent {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletsIntent {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub wallet_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct DeleteSubOrganizationIntent {
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauth2CredentialIntent {
    /// @inject_tag: validate:"required"
    pub provider: super::super::common::v1::Oauth2Provider,
    /// @inject_tag: validate:"required"
    pub client_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encrypted_client_secret: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateOauth2CredentialIntent {
    /// @inject_tag: validate:"required"
    pub oauth2_credential_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub provider: super::super::common::v1::Oauth2Provider,
    /// @inject_tag: validate:"required"
    pub client_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub encrypted_client_secret: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauth2CredentialIntent {
    /// @inject_tag: validate:"required"
    pub oauth2_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Oauth2AuthenticateIntent {
    /// @inject_tag: validate:"required"
    pub oauth2_credential_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub auth_code: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub redirect_uri: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub code_verifier: ::prost::alloc::string::String,
    #[serde(default)]
    pub nonce: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub bearer_token_target_public_key: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletAccountsIntent {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub wallet_account_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePoliciesIntent {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub policy_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendRawTransactionIntent {
    /// @inject_tag: validate:"required"
    pub signed_transaction: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub caip2: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateFiatOnRampCredentialIntent {
    /// @inject_tag: validate:"required"
    pub onramp_provider: super::super::common::v1::FiatOnRampProvider,
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
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateFiatOnRampCredentialIntent {
    /// @inject_tag: validate:"required"
    pub fiat_onramp_credential_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub onramp_provider: super::super::common::v1::FiatOnRampProvider,
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
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteFiatOnRampCredentialIntent {
    /// @inject_tag: validate:"required"
    pub fiat_onramp_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcAppIntent {
    /// @inject_tag: validate:"required"
    pub name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub quorum_public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub manifest_set_id: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub manifest_set_params: ::core::option::Option<TvcOperatorSetParams>,
    #[serde(default)]
    pub share_set_id: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub share_set_params: ::core::option::Option<TvcOperatorSetParams>,
    #[serde(default)]
    pub external_connectivity: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcOperatorSetParams {
    /// @inject_tag: validate:"required"
    pub name: ::prost::alloc::string::String,
    #[serde(default)]
    pub new_operators: ::prost::alloc::vec::Vec<TvcOperatorParams>,
    #[serde(default)]
    pub existing_operator_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub threshold: u32,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcOperatorParams {
    /// @inject_tag: validate:"required"
    pub name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub public_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcDeploymentIntent {
    /// @inject_tag: validate:"required"
    pub app_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub qos_version: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub pivot_container_image_url: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub pivot_path: ::prost::alloc::string::String,
    #[serde(default)]
    pub pivot_args: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    pub expected_pivot_digest: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub host_container_image_url: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub host_path: ::prost::alloc::string::String,
    #[serde(default)]
    pub host_args: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub nonce: ::core::option::Option<u32>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcManifestApprovalsIntent {
    /// @inject_tag: validate:"required,uuid"
    pub manifest_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    #[serde(default)]
    pub approvals: ::prost::alloc::vec::Vec<TvcManifestApproval>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcManifestApproval {
    /// @inject_tag: validate:"required,uuid"
    pub operator_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub signature: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// Result of the intended action.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Result {
    #[serde(default)]
    #[serde(flatten)]
    pub inner: ::core::option::Option<result::Inner>,
}
/// Nested message and enum types in `Result`.
pub mod result {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    #[derive(Debug)]
    pub enum Inner {
        CreateOrganizationResult(super::CreateOrganizationResult),
        CreateAuthenticatorsResult(super::CreateAuthenticatorsResult),
        CreateUsersResult(super::CreateUsersResult),
        CreatePrivateKeysResult(super::CreatePrivateKeysResult),
        CreateInvitationsResult(super::CreateInvitationsResult),
        AcceptInvitationResult(super::AcceptInvitationResult),
        SignRawPayloadResult(super::SignRawPayloadResult),
        CreatePolicyResult(super::CreatePolicyResult),
        DisablePrivateKeyResult(super::DisablePrivateKeyResult),
        DeleteUsersResult(super::DeleteUsersResult),
        DeleteAuthenticatorsResult(super::DeleteAuthenticatorsResult),
        DeleteInvitationResult(super::DeleteInvitationResult),
        DeleteOrganizationResult(super::DeleteOrganizationResult),
        DeletePolicyResult(super::DeletePolicyResult),
        CreateUserTagResult(super::CreateUserTagResult),
        DeleteUserTagsResult(super::DeleteUserTagsResult),
        SignTransactionResult(super::SignTransactionResult),
        DeleteApiKeysResult(super::DeleteApiKeysResult),
        CreateApiKeysResult(super::CreateApiKeysResult),
        CreatePrivateKeyTagResult(super::CreatePrivateKeyTagResult),
        DeletePrivateKeyTagsResult(super::DeletePrivateKeyTagsResult),
        SetPaymentMethodResult(super::super::billing::SetPaymentMethodResult),
        ActivateBillingTierResult(super::super::billing::ActivateBillingTierResult),
        DeletePaymentMethodResult(super::super::billing::DeletePaymentMethodResult),
        CreateApiOnlyUsersResult(super::CreateApiOnlyUsersResult),
        UpdateRootQuorumResult(super::UpdateRootQuorumResult),
        UpdateUserTagResult(super::UpdateUserTagResult),
        UpdatePrivateKeyTagResult(super::UpdatePrivateKeyTagResult),
        CreateSubOrganizationResult(super::CreateSubOrganizationResult),
        UpdateAllowedOriginsResult(super::UpdateAllowedOriginsResult),
        CreatePrivateKeysResultV2(super::CreatePrivateKeysResultV2),
        UpdateUserResult(super::UpdateUserResult),
        UpdatePolicyResult(super::UpdatePolicyResult),
        CreateSubOrganizationResultV3(super::CreateSubOrganizationResultV3),
        CreateWalletResult(super::CreateWalletResult),
        CreateWalletAccountsResult(super::CreateWalletAccountsResult),
        InitUserEmailRecoveryResult(super::InitUserEmailRecoveryResult),
        RecoverUserResult(super::RecoverUserResult),
        SetOrganizationFeatureResult(super::SetOrganizationFeatureResult),
        RemoveOrganizationFeatureResult(super::RemoveOrganizationFeatureResult),
        ExportPrivateKeyResult(super::ExportPrivateKeyResult),
        ExportWalletResult(super::ExportWalletResult),
        CreateSubOrganizationResultV4(super::CreateSubOrganizationResultV4),
        EmailAuthResult(super::EmailAuthResult),
        ExportWalletAccountResult(super::ExportWalletAccountResult),
        InitImportWalletResult(super::InitImportWalletResult),
        ImportWalletResult(super::ImportWalletResult),
        InitImportPrivateKeyResult(super::InitImportPrivateKeyResult),
        ImportPrivateKeyResult(super::ImportPrivateKeyResult),
        CreatePoliciesResult(super::CreatePoliciesResult),
        SignRawPayloadsResult(super::SignRawPayloadsResult),
        CreateReadOnlySessionResult(super::CreateReadOnlySessionResult),
        CreateOauthProvidersResult(super::CreateOauthProvidersResult),
        DeleteOauthProvidersResult(super::DeleteOauthProvidersResult),
        CreateSubOrganizationResultV5(super::CreateSubOrganizationResultV5),
        OauthResult(super::OauthResult),
        CreateReadWriteSessionResult(super::CreateReadWriteSessionResult),
        CreateSubOrganizationResultV6(super::CreateSubOrganizationResultV6),
        DeletePrivateKeysResult(super::DeletePrivateKeysResult),
        DeleteWalletsResult(super::DeleteWalletsResult),
        CreateReadWriteSessionResultV2(super::CreateReadWriteSessionResultV2),
        DeleteSubOrganizationResult(super::DeleteSubOrganizationResult),
        InitOtpAuthResult(super::InitOtpAuthResult),
        OtpAuthResult(super::OtpAuthResult),
        CreateSubOrganizationResultV7(super::CreateSubOrganizationResultV7),
        UpdateWalletResult(super::UpdateWalletResult),
        UpdatePolicyResultV2(super::UpdatePolicyResultV2),
        InitOtpAuthResultV2(super::InitOtpAuthResultV2),
        InitOtpResult(super::InitOtpResult),
        VerifyOtpResult(super::VerifyOtpResult),
        OtpLoginResult(super::OtpLoginResult),
        StampLoginResult(super::StampLoginResult),
        OauthLoginResult(super::OauthLoginResult),
        UpdateUserNameResult(super::UpdateUserNameResult),
        UpdateUserEmailResult(super::UpdateUserEmailResult),
        UpdateUserPhoneNumberResult(super::UpdateUserPhoneNumberResult),
        InitFiatOnRampResult(super::InitFiatOnRampResult),
        CreateSmartContractInterfaceResult(super::CreateSmartContractInterfaceResult),
        DeleteSmartContractInterfaceResult(super::DeleteSmartContractInterfaceResult),
        EnableAuthProxyResult(super::EnableAuthProxyResult),
        DisableAuthProxyResult(super::DisableAuthProxyResult),
        UpdateAuthProxyConfigResult(super::UpdateAuthProxyConfigResult),
        CreateOauth2CredentialResult(super::CreateOauth2CredentialResult),
        UpdateOauth2CredentialResult(super::UpdateOauth2CredentialResult),
        DeleteOauth2CredentialResult(super::DeleteOauth2CredentialResult),
        Oauth2AuthenticateResult(super::Oauth2AuthenticateResult),
        DeleteWalletAccountsResult(super::DeleteWalletAccountsResult),
        DeletePoliciesResult(super::DeletePoliciesResult),
        EthSendRawTransactionResult(super::EthSendRawTransactionResult),
        CreateFiatOnRampCredentialResult(super::CreateFiatOnRampCredentialResult),
        UpdateFiatOnRampCredentialResult(super::UpdateFiatOnRampCredentialResult),
        DeleteFiatOnRampCredentialResult(super::DeleteFiatOnRampCredentialResult),
        EthSendTransactionResult(super::EthSendTransactionResult),
        UpsertGasUsageConfigResult(super::UpsertGasUsageConfigResult),
        CreateTvcAppResult(super::CreateTvcAppResult),
        CreateTvcDeploymentResult(super::CreateTvcDeploymentResult),
        CreateTvcManifestApprovalsResult(super::CreateTvcManifestApprovalsResult),
        SolSendTransactionResult(super::SolSendTransactionResult),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpsertGasUsageConfigResult {
    /// @inject_tag: validate:"required,uuid4"
    pub gas_usage_config_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EnableAuthProxyResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct DisableAuthProxyResult {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOrganizationResult {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateAuthenticatorsResult {
    #[serde(default)]
    pub authenticator_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiKeysResult {
    #[serde(default)]
    pub api_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUsersResult {
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserNameResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserEmailResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserPhoneNumberResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateWalletResult {
    pub wallet_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateApiOnlyUsersResult {
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateInvitationsResult {
    #[serde(default)]
    pub invitation_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AcceptInvitationResult {
    pub invitation_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeysResult {
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeysResultV2 {
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKeyResult>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PrivateKeyResult {
    pub private_key_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<Address>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Address {
    pub format: super::super::common::v1::AddressFormat,
    pub address: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadResult {
    pub r: ::prost::alloc::string::String,
    pub s: ::prost::alloc::string::String,
    pub v: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignRawPayloadsResult {
    #[serde(default)]
    pub signatures: ::prost::alloc::vec::Vec<SignRawPayloadResult>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletResult {
    pub wallet_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateWalletAccountsResult {
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitUserEmailRecoveryResult {
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthLoginResult {
    pub session: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitFiatOnRampResult {
    pub on_ramp_url: ::prost::alloc::string::String,
    pub on_ramp_transaction_id: ::prost::alloc::string::String,
    pub on_ramp_url_signature: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct StampLoginResult {
    pub session: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpLoginResult {
    pub session: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpResult {
    pub otp_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthResult {
    pub otp_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitOtpAuthResultV2 {
    pub otp_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OtpAuthResult {
    pub user_id: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
    pub credential_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct VerifyOtpResult {
    pub verification_token: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthResult {
    pub user_id: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
    pub credential_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailAuthResult {
    pub user_id: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePolicyResult {
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePoliciesResult {
    #[serde(default)]
    pub policy_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePolicyResult {
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePolicyResultV2 {
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadOnlySessionResult {
    pub organization_id: ::prost::alloc::string::String,
    pub organization_name: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    pub username: ::prost::alloc::string::String,
    pub session: ::prost::alloc::string::String,
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub session_expiry: u64,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadWriteSessionResult {
    pub organization_id: ::prost::alloc::string::String,
    pub organization_name: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    pub username: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
    pub credential_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateReadWriteSessionResultV2 {
    pub organization_id: ::prost::alloc::string::String,
    pub organization_name: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    pub username: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
    pub credential_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DisablePrivateKeyResult {
    pub private_key_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUsersResult {
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteInvitationResult {
    pub invitation_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteAuthenticatorsResult {
    #[serde(default)]
    pub authenticator_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteApiKeysResult {
    #[serde(default)]
    pub api_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOrganizationResult {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePolicyResult {
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateUserTagResult {
    pub user_tag_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateUserTagResult {
    pub user_tag_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteUserTagsResult {
    #[serde(default)]
    pub user_tag_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreatePrivateKeyTagResult {
    pub private_key_tag_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdatePrivateKeyTagResult {
    pub private_key_tag_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeyTagsResult {
    #[serde(default)]
    pub private_key_tag_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignTransactionResult {
    pub signed_transaction: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSmartContractInterfaceResult {
    pub smart_contract_interface_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteSmartContractInterfaceResult {
    pub smart_contract_interface_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
/// TODO: this should include the new root quorum
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct UpdateRootQuorumResult {}
#[derive(Debug)]
/// TODO: this should include the new origins
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct UpdateAllowedOriginsResult {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResult {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// Going directly to V3 to have it in parity with intent versioning
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResultV3 {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<PrivateKeyResult>,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WalletResult {
    pub wallet_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
/// Going directly to V4 to have it in parity with intent versioning
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResultV4 {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletResult>,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResultV5 {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletResult>,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResultV6 {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletResult>,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateSubOrganizationResultV7 {
    pub sub_organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet: ::core::option::Option<WalletResult>,
    #[serde(default)]
    pub root_user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RecoverUserResult {
    #[serde(default)]
    pub authenticator_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetOrganizationFeatureResult {
    #[serde(default)]
    pub features: ::prost::alloc::vec::Vec<super::super::data::v1::Feature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RemoveOrganizationFeatureResult {
    #[serde(default)]
    pub features: ::prost::alloc::vec::Vec<super::super::data::v1::Feature>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportPrivateKeyResult {
    pub private_key_id: ::prost::alloc::string::String,
    pub export_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletResult {
    pub wallet_id: ::prost::alloc::string::String,
    pub export_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ExportWalletAccountResult {
    pub address: ::prost::alloc::string::String,
    pub export_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportWalletResult {
    pub import_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportWalletResult {
    pub wallet_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InitImportPrivateKeyResult {
    pub import_bundle: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ImportPrivateKeyResult {
    pub private_key_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub addresses: ::prost::alloc::vec::Vec<Address>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauthProvidersResult {
    #[serde(default)]
    pub provider_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauthProvidersResult {
    #[serde(default)]
    pub provider_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeysResult {
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletsResult {
    #[serde(default)]
    pub wallet_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteSubOrganizationResult {
    pub sub_organization_uuid: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateOauth2CredentialResult {
    pub oauth2_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateOauth2CredentialResult {
    pub oauth2_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteOauth2CredentialResult {
    pub oauth2_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Oauth2AuthenticateResult {
    /// @inject_tag: validate:"required"
    pub oidc_token: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletAccountsResult {
    #[serde(default)]
    pub wallet_account_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePoliciesResult {
    #[serde(default)]
    pub policy_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcAppResult {
    pub app_id: ::prost::alloc::string::String,
    pub manifest_set_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub manifest_set_operator_ids: ::prost::alloc::vec::Vec<
        ::prost::alloc::string::String,
    >,
    #[serde(default)]
    pub manifest_set_threshold: u32,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcDeploymentResult {
    pub deployment_id: ::prost::alloc::string::String,
    pub manifest_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateTvcManifestApprovalsResult {
    #[serde(default)]
    pub approval_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendRawTransactionResult {
    pub transaction_hash: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CreateFiatOnRampCredentialResult {
    pub fiat_on_ramp_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UpdateFiatOnRampCredentialResult {
    pub fiat_on_ramp_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteFiatOnRampCredentialResult {
    pub fiat_on_ramp_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendTransactionResult {
    pub send_transaction_status_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SolSendTransactionResult {
    pub send_transaction_status_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct OauthProviderParams {
    pub provider_name: ::prost::alloc::string::String,
    pub oidc_token: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApiKeyParamsV2 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub api_key_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal,tk_api_key"
    pub public_key: ::prost::alloc::string::String,
    pub curve_type: super::super::common::v1::ApiKeyCurve,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UserParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    pub access_type: super::super::common::v1::AccessType,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParams>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UserParamsV2 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UserParamsV3 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"omitempty,e164"
    #[serde(default)]
    pub user_phone_number: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<ApiKeyParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<AuthenticatorParamsV2>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<OauthProviderParams>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AuthenticatorParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub authenticator_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,uuid"
    pub user_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub attestation: ::core::option::Option<
        super::super::webauthn::v1::PublicKeyCredentialWithAttestation,
    >,
    /// @inject_tag: validate:"required,max=256"
    pub challenge: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AuthenticatorParamsV2 {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub authenticator_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,max=256"
    pub challenge: ::prost::alloc::string::String,
    #[serde(default)]
    pub attestation: ::core::option::Option<Attestation>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Attestation {
    /// @inject_tag: validate:"required,max=256"
    pub credential_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub client_data_json: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub attestation_object: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub transports: Vec<super::super::webauthn::v1::AuthenticatorTransport>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct InvitationParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub receiver_user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,email,tk_email"
    pub receiver_user_email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub receiver_user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"required"
    pub access_type: super::super::common::v1::AccessType,
    /// @inject_tag: validate:"required,uuid"
    pub sender_user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApiOnlyUserParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub user_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"omitempty,email,tk_email"
    #[serde(default)]
    pub user_email: ::core::option::Option<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::api::ApiKeyParams>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PrivateKeyParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub private_key_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub curve: super::super::common::v1::Curve,
    /// @inject_tag: validate:"dive,uuid"
    #[serde(default)]
    pub private_key_tags: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// @inject_tag: validate:"dive"
    #[serde(default)]
    pub address_formats: Vec<super::super::common::v1::AddressFormat>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WalletParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub wallet_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"dive,required"
    #[serde(default)]
    pub accounts: ::prost::alloc::vec::Vec<WalletAccountParams>,
    /// @inject_tag: validate:"omitempty"
    #[serde(default)]
    pub mnemonic_length: ::core::option::Option<i32>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WalletAccountParams {
    /// @inject_tag: validate:"required"
    pub curve: super::super::common::v1::Curve,
    /// @inject_tag: validate:"required"
    pub path_format: super::super::common::v1::PathFormat,
    /// @inject_tag: validate:"required"
    pub path: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required"
    pub address_format: super::super::common::v1::AddressFormat,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePrivateKeysParams {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub private_key_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletsParams {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub wallet_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeleteWalletAccountsParams {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub wallet_account_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    pub delete_without_export: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePoliciesParams {
    /// @inject_tag: validate:"required,dive,uuid"
    #[serde(default)]
    pub policy_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ClientSignature {
    pub public_key: ::prost::alloc::string::String,
    pub scheme: super::super::common::v1::ClientSignatureScheme,
    pub message: ::prost::alloc::string::String,
    pub signature: ::prost::alloc::string::String,
}
/// Type of Activity, such as Add User, or Sign Transaction.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActivityType {
    #[serde(rename = "ACTIVITY_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_API_KEYS")]
    CreateApiKeys = 1,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_USERS")]
    CreateUsers = 2,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS")]
    CreatePrivateKeys = 3,
    #[serde(rename = "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD")]
    SignRawPayload = 4,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_INVITATIONS")]
    CreateInvitations = 5,
    #[serde(rename = "ACTIVITY_TYPE_ACCEPT_INVITATION")]
    AcceptInvitation = 6,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_POLICY")]
    CreatePolicy = 7,
    #[serde(rename = "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY")]
    DisablePrivateKey = 8,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_USERS")]
    DeleteUsers = 9,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_API_KEYS")]
    DeleteApiKeys = 10,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_INVITATION")]
    DeleteInvitation = 11,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_ORGANIZATION")]
    DeleteOrganization = 12,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_POLICY")]
    DeletePolicy = 13,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_USER_TAG")]
    CreateUserTag = 14,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_USER_TAGS")]
    DeleteUserTags = 15,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_ORGANIZATION")]
    CreateOrganization = 16,
    #[serde(rename = "ACTIVITY_TYPE_SIGN_TRANSACTION")]
    SignTransaction = 17,
    #[serde(rename = "ACTIVITY_TYPE_APPROVE_ACTIVITY")]
    ApproveActivity = 18,
    #[serde(rename = "ACTIVITY_TYPE_REJECT_ACTIVITY")]
    RejectActivity = 19,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_AUTHENTICATORS")]
    DeleteAuthenticators = 20,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_AUTHENTICATORS")]
    CreateAuthenticators = 21,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG")]
    CreatePrivateKeyTag = 22,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS")]
    DeletePrivateKeyTags = 23,
    #[serde(rename = "ACTIVITY_TYPE_SET_PAYMENT_METHOD")]
    SetPaymentMethod = 24,
    #[serde(rename = "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER")]
    ActivateBillingTier = 25,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD")]
    DeletePaymentMethod = 26,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_POLICY_V2")]
    CreatePolicyV2 = 27,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_POLICY_V3")]
    CreatePolicyV3 = 28,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_API_ONLY_USERS")]
    CreateApiOnlyUsers = 29,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM")]
    UpdateRootQuorum = 30,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_USER_TAG")]
    UpdateUserTag = 31,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG")]
    UpdatePrivateKeyTag = 32,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2")]
    CreateAuthenticatorsV2 = 33,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2")]
    CreateOrganizationV2 = 34,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_USERS_V2")]
    CreateUsersV2 = 35,
    #[serde(rename = "ACTIVITY_TYPE_ACCEPT_INVITATION_V2")]
    AcceptInvitationV2 = 36,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION")]
    CreateSubOrganization = 37,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2")]
    CreateSubOrganizationV2 = 38,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS")]
    UpdateAllowedOrigins = 39,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2")]
    CreatePrivateKeysV2 = 40,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_USER")]
    UpdateUser = 41,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_POLICY")]
    UpdatePolicy = 42,
    #[serde(rename = "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2")]
    SetPaymentMethodV2 = 43,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3")]
    CreateSubOrganizationV3 = 44,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_WALLET")]
    CreateWallet = 45,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS")]
    CreateWalletAccounts = 46,
    #[serde(rename = "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY")]
    InitUserEmailRecovery = 47,
    #[serde(rename = "ACTIVITY_TYPE_RECOVER_USER")]
    RecoverUser = 48,
    #[serde(rename = "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE")]
    SetOrganizationFeature = 49,
    #[serde(rename = "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE")]
    RemoveOrganizationFeature = 50,
    #[serde(rename = "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2")]
    SignRawPayloadV2 = 51,
    #[serde(rename = "ACTIVITY_TYPE_SIGN_TRANSACTION_V2")]
    SignTransactionV2 = 52,
    #[serde(rename = "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY")]
    ExportPrivateKey = 53,
    #[serde(rename = "ACTIVITY_TYPE_EXPORT_WALLET")]
    ExportWallet = 54,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4")]
    CreateSubOrganizationV4 = 55,
    #[serde(rename = "ACTIVITY_TYPE_EMAIL_AUTH")]
    EmailAuth = 56,
    #[serde(rename = "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT")]
    ExportWalletAccount = 57,
    #[serde(rename = "ACTIVITY_TYPE_INIT_IMPORT_WALLET")]
    InitImportWallet = 58,
    #[serde(rename = "ACTIVITY_TYPE_IMPORT_WALLET")]
    ImportWallet = 59,
    #[serde(rename = "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY")]
    InitImportPrivateKey = 60,
    #[serde(rename = "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY")]
    ImportPrivateKey = 61,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_POLICIES")]
    CreatePolicies = 62,
    #[serde(rename = "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS")]
    SignRawPayloads = 63,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION")]
    CreateReadOnlySession = 64,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_OAUTH_PROVIDERS")]
    CreateOauthProviders = 65,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_OAUTH_PROVIDERS")]
    DeleteOauthProviders = 66,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V5")]
    CreateSubOrganizationV5 = 67,
    #[serde(rename = "ACTIVITY_TYPE_OAUTH")]
    Oauth = 68,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_API_KEYS_V2")]
    CreateApiKeysV2 = 69,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION")]
    CreateReadWriteSession = 70,
    #[serde(rename = "ACTIVITY_TYPE_EMAIL_AUTH_V2")]
    EmailAuthV2 = 71,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V6")]
    CreateSubOrganizationV6 = 72,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_PRIVATE_KEYS")]
    DeletePrivateKeys = 73,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_WALLETS")]
    DeleteWallets = 74,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION_V2")]
    CreateReadWriteSessionV2 = 75,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION")]
    DeleteSubOrganization = 76,
    #[serde(rename = "ACTIVITY_TYPE_INIT_OTP_AUTH")]
    InitOtpAuth = 77,
    #[serde(rename = "ACTIVITY_TYPE_OTP_AUTH")]
    OtpAuth = 78,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7")]
    CreateSubOrganizationV7 = 79,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_WALLET")]
    UpdateWallet = 80,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_POLICY_V2")]
    UpdatePolicyV2 = 81,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_USERS_V3")]
    CreateUsersV3 = 82,
    #[serde(rename = "ACTIVITY_TYPE_INIT_OTP_AUTH_V2")]
    InitOtpAuthV2 = 83,
    #[serde(rename = "ACTIVITY_TYPE_INIT_OTP")]
    InitOtp = 84,
    #[serde(rename = "ACTIVITY_TYPE_VERIFY_OTP")]
    VerifyOtp = 85,
    #[serde(rename = "ACTIVITY_TYPE_OTP_LOGIN")]
    OtpLogin = 86,
    #[serde(rename = "ACTIVITY_TYPE_STAMP_LOGIN")]
    StampLogin = 87,
    #[serde(rename = "ACTIVITY_TYPE_OAUTH_LOGIN")]
    OauthLogin = 88,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_USER_NAME")]
    UpdateUserName = 89,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_USER_EMAIL")]
    UpdateUserEmail = 90,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_USER_PHONE_NUMBER")]
    UpdateUserPhoneNumber = 91,
    #[serde(rename = "ACTIVITY_TYPE_INIT_FIAT_ON_RAMP")]
    InitFiatOnRamp = 92,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_SMART_CONTRACT_INTERFACE")]
    CreateSmartContractInterface = 93,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_SMART_CONTRACT_INTERFACE")]
    DeleteSmartContractInterface = 94,
    #[serde(rename = "ACTIVITY_TYPE_ENABLE_AUTH_PROXY")]
    EnableAuthProxy = 95,
    #[serde(rename = "ACTIVITY_TYPE_DISABLE_AUTH_PROXY")]
    DisableAuthProxy = 96,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_AUTH_PROXY_CONFIG")]
    UpdateAuthProxyConfig = 97,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_OAUTH2_CREDENTIAL")]
    CreateOauth2Credential = 98,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_OAUTH2_CREDENTIAL")]
    UpdateOauth2Credential = 99,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_OAUTH2_CREDENTIAL")]
    DeleteOauth2Credential = 100,
    #[serde(rename = "ACTIVITY_TYPE_OAUTH2_AUTHENTICATE")]
    Oauth2Authenticate = 101,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_WALLET_ACCOUNTS")]
    DeleteWalletAccounts = 102,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_POLICIES")]
    DeletePolicies = 103,
    #[serde(rename = "ACTIVITY_TYPE_ETH_SEND_RAW_TRANSACTION")]
    EthSendRawTransaction = 104,
    #[serde(rename = "ACTIVITY_TYPE_ETH_SEND_TRANSACTION")]
    EthSendTransaction = 105,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_FIAT_ON_RAMP_CREDENTIAL")]
    CreateFiatOnRampCredential = 106,
    #[serde(rename = "ACTIVITY_TYPE_UPDATE_FIAT_ON_RAMP_CREDENTIAL")]
    UpdateFiatOnRampCredential = 107,
    #[serde(rename = "ACTIVITY_TYPE_DELETE_FIAT_ON_RAMP_CREDENTIAL")]
    DeleteFiatOnRampCredential = 108,
    #[serde(rename = "ACTIVITY_TYPE_EMAIL_AUTH_V3")]
    EmailAuthV3 = 109,
    #[serde(rename = "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY_V2")]
    InitUserEmailRecoveryV2 = 110,
    #[serde(rename = "ACTIVITY_TYPE_INIT_OTP_AUTH_V3")]
    InitOtpAuthV3 = 111,
    #[serde(rename = "ACTIVITY_TYPE_INIT_OTP_V2")]
    InitOtpV2 = 112,
    #[serde(rename = "ACTIVITY_TYPE_UPSERT_GAS_USAGE_CONFIG")]
    UpsertGasUsageConfig = 113,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_TVC_APP")]
    CreateTvcApp = 114,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT")]
    CreateTvcDeployment = 115,
    #[serde(rename = "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS")]
    CreateTvcManifestApprovals = 116,
    #[serde(rename = "ACTIVITY_TYPE_SOL_SEND_TRANSACTION")]
    SolSendTransaction = 117,
}
impl ActivityType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ACTIVITY_TYPE_UNSPECIFIED",
            Self::CreateApiKeys => "ACTIVITY_TYPE_CREATE_API_KEYS",
            Self::CreateUsers => "ACTIVITY_TYPE_CREATE_USERS",
            Self::CreatePrivateKeys => "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS",
            Self::SignRawPayload => "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD",
            Self::CreateInvitations => "ACTIVITY_TYPE_CREATE_INVITATIONS",
            Self::AcceptInvitation => "ACTIVITY_TYPE_ACCEPT_INVITATION",
            Self::CreatePolicy => "ACTIVITY_TYPE_CREATE_POLICY",
            Self::DisablePrivateKey => "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY",
            Self::DeleteUsers => "ACTIVITY_TYPE_DELETE_USERS",
            Self::DeleteApiKeys => "ACTIVITY_TYPE_DELETE_API_KEYS",
            Self::DeleteInvitation => "ACTIVITY_TYPE_DELETE_INVITATION",
            Self::DeleteOrganization => "ACTIVITY_TYPE_DELETE_ORGANIZATION",
            Self::DeletePolicy => "ACTIVITY_TYPE_DELETE_POLICY",
            Self::CreateUserTag => "ACTIVITY_TYPE_CREATE_USER_TAG",
            Self::DeleteUserTags => "ACTIVITY_TYPE_DELETE_USER_TAGS",
            Self::CreateOrganization => "ACTIVITY_TYPE_CREATE_ORGANIZATION",
            Self::SignTransaction => "ACTIVITY_TYPE_SIGN_TRANSACTION",
            Self::ApproveActivity => "ACTIVITY_TYPE_APPROVE_ACTIVITY",
            Self::RejectActivity => "ACTIVITY_TYPE_REJECT_ACTIVITY",
            Self::DeleteAuthenticators => "ACTIVITY_TYPE_DELETE_AUTHENTICATORS",
            Self::CreateAuthenticators => "ACTIVITY_TYPE_CREATE_AUTHENTICATORS",
            Self::CreatePrivateKeyTag => "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG",
            Self::DeletePrivateKeyTags => "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS",
            Self::SetPaymentMethod => "ACTIVITY_TYPE_SET_PAYMENT_METHOD",
            Self::ActivateBillingTier => "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER",
            Self::DeletePaymentMethod => "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD",
            Self::CreatePolicyV2 => "ACTIVITY_TYPE_CREATE_POLICY_V2",
            Self::CreatePolicyV3 => "ACTIVITY_TYPE_CREATE_POLICY_V3",
            Self::CreateApiOnlyUsers => "ACTIVITY_TYPE_CREATE_API_ONLY_USERS",
            Self::UpdateRootQuorum => "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM",
            Self::UpdateUserTag => "ACTIVITY_TYPE_UPDATE_USER_TAG",
            Self::UpdatePrivateKeyTag => "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG",
            Self::CreateAuthenticatorsV2 => "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2",
            Self::CreateOrganizationV2 => "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2",
            Self::CreateUsersV2 => "ACTIVITY_TYPE_CREATE_USERS_V2",
            Self::AcceptInvitationV2 => "ACTIVITY_TYPE_ACCEPT_INVITATION_V2",
            Self::CreateSubOrganization => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION",
            Self::CreateSubOrganizationV2 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2",
            Self::UpdateAllowedOrigins => "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS",
            Self::CreatePrivateKeysV2 => "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2",
            Self::UpdateUser => "ACTIVITY_TYPE_UPDATE_USER",
            Self::UpdatePolicy => "ACTIVITY_TYPE_UPDATE_POLICY",
            Self::SetPaymentMethodV2 => "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2",
            Self::CreateSubOrganizationV3 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3",
            Self::CreateWallet => "ACTIVITY_TYPE_CREATE_WALLET",
            Self::CreateWalletAccounts => "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS",
            Self::InitUserEmailRecovery => "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY",
            Self::RecoverUser => "ACTIVITY_TYPE_RECOVER_USER",
            Self::SetOrganizationFeature => "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE",
            Self::RemoveOrganizationFeature => {
                "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE"
            }
            Self::SignRawPayloadV2 => "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
            Self::SignTransactionV2 => "ACTIVITY_TYPE_SIGN_TRANSACTION_V2",
            Self::ExportPrivateKey => "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY",
            Self::ExportWallet => "ACTIVITY_TYPE_EXPORT_WALLET",
            Self::CreateSubOrganizationV4 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4",
            Self::EmailAuth => "ACTIVITY_TYPE_EMAIL_AUTH",
            Self::ExportWalletAccount => "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT",
            Self::InitImportWallet => "ACTIVITY_TYPE_INIT_IMPORT_WALLET",
            Self::ImportWallet => "ACTIVITY_TYPE_IMPORT_WALLET",
            Self::InitImportPrivateKey => "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY",
            Self::ImportPrivateKey => "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY",
            Self::CreatePolicies => "ACTIVITY_TYPE_CREATE_POLICIES",
            Self::SignRawPayloads => "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS",
            Self::CreateReadOnlySession => "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION",
            Self::CreateOauthProviders => "ACTIVITY_TYPE_CREATE_OAUTH_PROVIDERS",
            Self::DeleteOauthProviders => "ACTIVITY_TYPE_DELETE_OAUTH_PROVIDERS",
            Self::CreateSubOrganizationV5 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V5",
            Self::Oauth => "ACTIVITY_TYPE_OAUTH",
            Self::CreateApiKeysV2 => "ACTIVITY_TYPE_CREATE_API_KEYS_V2",
            Self::CreateReadWriteSession => "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION",
            Self::EmailAuthV2 => "ACTIVITY_TYPE_EMAIL_AUTH_V2",
            Self::CreateSubOrganizationV6 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V6",
            Self::DeletePrivateKeys => "ACTIVITY_TYPE_DELETE_PRIVATE_KEYS",
            Self::DeleteWallets => "ACTIVITY_TYPE_DELETE_WALLETS",
            Self::CreateReadWriteSessionV2 => {
                "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION_V2"
            }
            Self::DeleteSubOrganization => "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION",
            Self::InitOtpAuth => "ACTIVITY_TYPE_INIT_OTP_AUTH",
            Self::OtpAuth => "ACTIVITY_TYPE_OTP_AUTH",
            Self::CreateSubOrganizationV7 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7",
            Self::UpdateWallet => "ACTIVITY_TYPE_UPDATE_WALLET",
            Self::UpdatePolicyV2 => "ACTIVITY_TYPE_UPDATE_POLICY_V2",
            Self::CreateUsersV3 => "ACTIVITY_TYPE_CREATE_USERS_V3",
            Self::InitOtpAuthV2 => "ACTIVITY_TYPE_INIT_OTP_AUTH_V2",
            Self::InitOtp => "ACTIVITY_TYPE_INIT_OTP",
            Self::VerifyOtp => "ACTIVITY_TYPE_VERIFY_OTP",
            Self::OtpLogin => "ACTIVITY_TYPE_OTP_LOGIN",
            Self::StampLogin => "ACTIVITY_TYPE_STAMP_LOGIN",
            Self::OauthLogin => "ACTIVITY_TYPE_OAUTH_LOGIN",
            Self::UpdateUserName => "ACTIVITY_TYPE_UPDATE_USER_NAME",
            Self::UpdateUserEmail => "ACTIVITY_TYPE_UPDATE_USER_EMAIL",
            Self::UpdateUserPhoneNumber => "ACTIVITY_TYPE_UPDATE_USER_PHONE_NUMBER",
            Self::InitFiatOnRamp => "ACTIVITY_TYPE_INIT_FIAT_ON_RAMP",
            Self::CreateSmartContractInterface => {
                "ACTIVITY_TYPE_CREATE_SMART_CONTRACT_INTERFACE"
            }
            Self::DeleteSmartContractInterface => {
                "ACTIVITY_TYPE_DELETE_SMART_CONTRACT_INTERFACE"
            }
            Self::EnableAuthProxy => "ACTIVITY_TYPE_ENABLE_AUTH_PROXY",
            Self::DisableAuthProxy => "ACTIVITY_TYPE_DISABLE_AUTH_PROXY",
            Self::UpdateAuthProxyConfig => "ACTIVITY_TYPE_UPDATE_AUTH_PROXY_CONFIG",
            Self::CreateOauth2Credential => "ACTIVITY_TYPE_CREATE_OAUTH2_CREDENTIAL",
            Self::UpdateOauth2Credential => "ACTIVITY_TYPE_UPDATE_OAUTH2_CREDENTIAL",
            Self::DeleteOauth2Credential => "ACTIVITY_TYPE_DELETE_OAUTH2_CREDENTIAL",
            Self::Oauth2Authenticate => "ACTIVITY_TYPE_OAUTH2_AUTHENTICATE",
            Self::DeleteWalletAccounts => "ACTIVITY_TYPE_DELETE_WALLET_ACCOUNTS",
            Self::DeletePolicies => "ACTIVITY_TYPE_DELETE_POLICIES",
            Self::EthSendRawTransaction => "ACTIVITY_TYPE_ETH_SEND_RAW_TRANSACTION",
            Self::EthSendTransaction => "ACTIVITY_TYPE_ETH_SEND_TRANSACTION",
            Self::CreateFiatOnRampCredential => {
                "ACTIVITY_TYPE_CREATE_FIAT_ON_RAMP_CREDENTIAL"
            }
            Self::UpdateFiatOnRampCredential => {
                "ACTIVITY_TYPE_UPDATE_FIAT_ON_RAMP_CREDENTIAL"
            }
            Self::DeleteFiatOnRampCredential => {
                "ACTIVITY_TYPE_DELETE_FIAT_ON_RAMP_CREDENTIAL"
            }
            Self::EmailAuthV3 => "ACTIVITY_TYPE_EMAIL_AUTH_V3",
            Self::InitUserEmailRecoveryV2 => "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY_V2",
            Self::InitOtpAuthV3 => "ACTIVITY_TYPE_INIT_OTP_AUTH_V3",
            Self::InitOtpV2 => "ACTIVITY_TYPE_INIT_OTP_V2",
            Self::UpsertGasUsageConfig => "ACTIVITY_TYPE_UPSERT_GAS_USAGE_CONFIG",
            Self::CreateTvcApp => "ACTIVITY_TYPE_CREATE_TVC_APP",
            Self::CreateTvcDeployment => "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT",
            Self::CreateTvcManifestApprovals => {
                "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS"
            }
            Self::SolSendTransaction => "ACTIVITY_TYPE_SOL_SEND_TRANSACTION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACTIVITY_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "ACTIVITY_TYPE_CREATE_API_KEYS" => Some(Self::CreateApiKeys),
            "ACTIVITY_TYPE_CREATE_USERS" => Some(Self::CreateUsers),
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS" => Some(Self::CreatePrivateKeys),
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD" => Some(Self::SignRawPayload),
            "ACTIVITY_TYPE_CREATE_INVITATIONS" => Some(Self::CreateInvitations),
            "ACTIVITY_TYPE_ACCEPT_INVITATION" => Some(Self::AcceptInvitation),
            "ACTIVITY_TYPE_CREATE_POLICY" => Some(Self::CreatePolicy),
            "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY" => Some(Self::DisablePrivateKey),
            "ACTIVITY_TYPE_DELETE_USERS" => Some(Self::DeleteUsers),
            "ACTIVITY_TYPE_DELETE_API_KEYS" => Some(Self::DeleteApiKeys),
            "ACTIVITY_TYPE_DELETE_INVITATION" => Some(Self::DeleteInvitation),
            "ACTIVITY_TYPE_DELETE_ORGANIZATION" => Some(Self::DeleteOrganization),
            "ACTIVITY_TYPE_DELETE_POLICY" => Some(Self::DeletePolicy),
            "ACTIVITY_TYPE_CREATE_USER_TAG" => Some(Self::CreateUserTag),
            "ACTIVITY_TYPE_DELETE_USER_TAGS" => Some(Self::DeleteUserTags),
            "ACTIVITY_TYPE_CREATE_ORGANIZATION" => Some(Self::CreateOrganization),
            "ACTIVITY_TYPE_SIGN_TRANSACTION" => Some(Self::SignTransaction),
            "ACTIVITY_TYPE_APPROVE_ACTIVITY" => Some(Self::ApproveActivity),
            "ACTIVITY_TYPE_REJECT_ACTIVITY" => Some(Self::RejectActivity),
            "ACTIVITY_TYPE_DELETE_AUTHENTICATORS" => Some(Self::DeleteAuthenticators),
            "ACTIVITY_TYPE_CREATE_AUTHENTICATORS" => Some(Self::CreateAuthenticators),
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG" => Some(Self::CreatePrivateKeyTag),
            "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS" => Some(Self::DeletePrivateKeyTags),
            "ACTIVITY_TYPE_SET_PAYMENT_METHOD" => Some(Self::SetPaymentMethod),
            "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER" => Some(Self::ActivateBillingTier),
            "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD" => Some(Self::DeletePaymentMethod),
            "ACTIVITY_TYPE_CREATE_POLICY_V2" => Some(Self::CreatePolicyV2),
            "ACTIVITY_TYPE_CREATE_POLICY_V3" => Some(Self::CreatePolicyV3),
            "ACTIVITY_TYPE_CREATE_API_ONLY_USERS" => Some(Self::CreateApiOnlyUsers),
            "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM" => Some(Self::UpdateRootQuorum),
            "ACTIVITY_TYPE_UPDATE_USER_TAG" => Some(Self::UpdateUserTag),
            "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG" => Some(Self::UpdatePrivateKeyTag),
            "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2" => {
                Some(Self::CreateAuthenticatorsV2)
            }
            "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2" => Some(Self::CreateOrganizationV2),
            "ACTIVITY_TYPE_CREATE_USERS_V2" => Some(Self::CreateUsersV2),
            "ACTIVITY_TYPE_ACCEPT_INVITATION_V2" => Some(Self::AcceptInvitationV2),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION" => Some(Self::CreateSubOrganization),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2" => {
                Some(Self::CreateSubOrganizationV2)
            }
            "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS" => Some(Self::UpdateAllowedOrigins),
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2" => Some(Self::CreatePrivateKeysV2),
            "ACTIVITY_TYPE_UPDATE_USER" => Some(Self::UpdateUser),
            "ACTIVITY_TYPE_UPDATE_POLICY" => Some(Self::UpdatePolicy),
            "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2" => Some(Self::SetPaymentMethodV2),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3" => {
                Some(Self::CreateSubOrganizationV3)
            }
            "ACTIVITY_TYPE_CREATE_WALLET" => Some(Self::CreateWallet),
            "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS" => Some(Self::CreateWalletAccounts),
            "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY" => Some(Self::InitUserEmailRecovery),
            "ACTIVITY_TYPE_RECOVER_USER" => Some(Self::RecoverUser),
            "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE" => {
                Some(Self::SetOrganizationFeature)
            }
            "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE" => {
                Some(Self::RemoveOrganizationFeature)
            }
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2" => Some(Self::SignRawPayloadV2),
            "ACTIVITY_TYPE_SIGN_TRANSACTION_V2" => Some(Self::SignTransactionV2),
            "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY" => Some(Self::ExportPrivateKey),
            "ACTIVITY_TYPE_EXPORT_WALLET" => Some(Self::ExportWallet),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4" => {
                Some(Self::CreateSubOrganizationV4)
            }
            "ACTIVITY_TYPE_EMAIL_AUTH" => Some(Self::EmailAuth),
            "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT" => Some(Self::ExportWalletAccount),
            "ACTIVITY_TYPE_INIT_IMPORT_WALLET" => Some(Self::InitImportWallet),
            "ACTIVITY_TYPE_IMPORT_WALLET" => Some(Self::ImportWallet),
            "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY" => Some(Self::InitImportPrivateKey),
            "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY" => Some(Self::ImportPrivateKey),
            "ACTIVITY_TYPE_CREATE_POLICIES" => Some(Self::CreatePolicies),
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS" => Some(Self::SignRawPayloads),
            "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION" => Some(Self::CreateReadOnlySession),
            "ACTIVITY_TYPE_CREATE_OAUTH_PROVIDERS" => Some(Self::CreateOauthProviders),
            "ACTIVITY_TYPE_DELETE_OAUTH_PROVIDERS" => Some(Self::DeleteOauthProviders),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V5" => {
                Some(Self::CreateSubOrganizationV5)
            }
            "ACTIVITY_TYPE_OAUTH" => Some(Self::Oauth),
            "ACTIVITY_TYPE_CREATE_API_KEYS_V2" => Some(Self::CreateApiKeysV2),
            "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION" => {
                Some(Self::CreateReadWriteSession)
            }
            "ACTIVITY_TYPE_EMAIL_AUTH_V2" => Some(Self::EmailAuthV2),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V6" => {
                Some(Self::CreateSubOrganizationV6)
            }
            "ACTIVITY_TYPE_DELETE_PRIVATE_KEYS" => Some(Self::DeletePrivateKeys),
            "ACTIVITY_TYPE_DELETE_WALLETS" => Some(Self::DeleteWallets),
            "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION_V2" => {
                Some(Self::CreateReadWriteSessionV2)
            }
            "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION" => Some(Self::DeleteSubOrganization),
            "ACTIVITY_TYPE_INIT_OTP_AUTH" => Some(Self::InitOtpAuth),
            "ACTIVITY_TYPE_OTP_AUTH" => Some(Self::OtpAuth),
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7" => {
                Some(Self::CreateSubOrganizationV7)
            }
            "ACTIVITY_TYPE_UPDATE_WALLET" => Some(Self::UpdateWallet),
            "ACTIVITY_TYPE_UPDATE_POLICY_V2" => Some(Self::UpdatePolicyV2),
            "ACTIVITY_TYPE_CREATE_USERS_V3" => Some(Self::CreateUsersV3),
            "ACTIVITY_TYPE_INIT_OTP_AUTH_V2" => Some(Self::InitOtpAuthV2),
            "ACTIVITY_TYPE_INIT_OTP" => Some(Self::InitOtp),
            "ACTIVITY_TYPE_VERIFY_OTP" => Some(Self::VerifyOtp),
            "ACTIVITY_TYPE_OTP_LOGIN" => Some(Self::OtpLogin),
            "ACTIVITY_TYPE_STAMP_LOGIN" => Some(Self::StampLogin),
            "ACTIVITY_TYPE_OAUTH_LOGIN" => Some(Self::OauthLogin),
            "ACTIVITY_TYPE_UPDATE_USER_NAME" => Some(Self::UpdateUserName),
            "ACTIVITY_TYPE_UPDATE_USER_EMAIL" => Some(Self::UpdateUserEmail),
            "ACTIVITY_TYPE_UPDATE_USER_PHONE_NUMBER" => Some(Self::UpdateUserPhoneNumber),
            "ACTIVITY_TYPE_INIT_FIAT_ON_RAMP" => Some(Self::InitFiatOnRamp),
            "ACTIVITY_TYPE_CREATE_SMART_CONTRACT_INTERFACE" => {
                Some(Self::CreateSmartContractInterface)
            }
            "ACTIVITY_TYPE_DELETE_SMART_CONTRACT_INTERFACE" => {
                Some(Self::DeleteSmartContractInterface)
            }
            "ACTIVITY_TYPE_ENABLE_AUTH_PROXY" => Some(Self::EnableAuthProxy),
            "ACTIVITY_TYPE_DISABLE_AUTH_PROXY" => Some(Self::DisableAuthProxy),
            "ACTIVITY_TYPE_UPDATE_AUTH_PROXY_CONFIG" => Some(Self::UpdateAuthProxyConfig),
            "ACTIVITY_TYPE_CREATE_OAUTH2_CREDENTIAL" => {
                Some(Self::CreateOauth2Credential)
            }
            "ACTIVITY_TYPE_UPDATE_OAUTH2_CREDENTIAL" => {
                Some(Self::UpdateOauth2Credential)
            }
            "ACTIVITY_TYPE_DELETE_OAUTH2_CREDENTIAL" => {
                Some(Self::DeleteOauth2Credential)
            }
            "ACTIVITY_TYPE_OAUTH2_AUTHENTICATE" => Some(Self::Oauth2Authenticate),
            "ACTIVITY_TYPE_DELETE_WALLET_ACCOUNTS" => Some(Self::DeleteWalletAccounts),
            "ACTIVITY_TYPE_DELETE_POLICIES" => Some(Self::DeletePolicies),
            "ACTIVITY_TYPE_ETH_SEND_RAW_TRANSACTION" => Some(Self::EthSendRawTransaction),
            "ACTIVITY_TYPE_ETH_SEND_TRANSACTION" => Some(Self::EthSendTransaction),
            "ACTIVITY_TYPE_CREATE_FIAT_ON_RAMP_CREDENTIAL" => {
                Some(Self::CreateFiatOnRampCredential)
            }
            "ACTIVITY_TYPE_UPDATE_FIAT_ON_RAMP_CREDENTIAL" => {
                Some(Self::UpdateFiatOnRampCredential)
            }
            "ACTIVITY_TYPE_DELETE_FIAT_ON_RAMP_CREDENTIAL" => {
                Some(Self::DeleteFiatOnRampCredential)
            }
            "ACTIVITY_TYPE_EMAIL_AUTH_V3" => Some(Self::EmailAuthV3),
            "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY_V2" => {
                Some(Self::InitUserEmailRecoveryV2)
            }
            "ACTIVITY_TYPE_INIT_OTP_AUTH_V3" => Some(Self::InitOtpAuthV3),
            "ACTIVITY_TYPE_INIT_OTP_V2" => Some(Self::InitOtpV2),
            "ACTIVITY_TYPE_UPSERT_GAS_USAGE_CONFIG" => Some(Self::UpsertGasUsageConfig),
            "ACTIVITY_TYPE_CREATE_TVC_APP" => Some(Self::CreateTvcApp),
            "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT" => Some(Self::CreateTvcDeployment),
            "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS" => {
                Some(Self::CreateTvcManifestApprovals)
            }
            "ACTIVITY_TYPE_SOL_SEND_TRANSACTION" => Some(Self::SolSendTransaction),
            _ => None,
        }
    }
}
/// The current processing status of an Activity.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActivityStatus {
    #[serde(rename = "ACTIVITY_STATUS_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "ACTIVITY_STATUS_CREATED")]
    Created = 1,
    #[serde(rename = "ACTIVITY_STATUS_PENDING")]
    Pending = 2,
    #[serde(rename = "ACTIVITY_STATUS_COMPLETED")]
    Completed = 3,
    #[serde(rename = "ACTIVITY_STATUS_FAILED")]
    Failed = 4,
    #[serde(rename = "ACTIVITY_STATUS_CONSENSUS_NEEDED")]
    ConsensusNeeded = 5,
    #[serde(rename = "ACTIVITY_STATUS_REJECTED")]
    Rejected = 6,
}
impl ActivityStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ACTIVITY_STATUS_UNSPECIFIED",
            Self::Created => "ACTIVITY_STATUS_CREATED",
            Self::Pending => "ACTIVITY_STATUS_PENDING",
            Self::Completed => "ACTIVITY_STATUS_COMPLETED",
            Self::Failed => "ACTIVITY_STATUS_FAILED",
            Self::ConsensusNeeded => "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            Self::Rejected => "ACTIVITY_STATUS_REJECTED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACTIVITY_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "ACTIVITY_STATUS_CREATED" => Some(Self::Created),
            "ACTIVITY_STATUS_PENDING" => Some(Self::Pending),
            "ACTIVITY_STATUS_COMPLETED" => Some(Self::Completed),
            "ACTIVITY_STATUS_FAILED" => Some(Self::Failed),
            "ACTIVITY_STATUS_CONSENSUS_NEEDED" => Some(Self::ConsensusNeeded),
            "ACTIVITY_STATUS_REJECTED" => Some(Self::Rejected),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UsageVariant {
    #[serde(rename = "USAGE_VARIANT_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "USAGE_VARIANT_ON_RAMP_COINBASE")]
    OnRampCoinbase = 1,
    #[serde(rename = "USAGE_VARIANT_ON_RAMP_MOONPAY")]
    OnRampMoonpay = 2,
    #[serde(rename = "USAGE_VARIANT_PAYMASTER")]
    Paymaster = 3,
}
impl UsageVariant {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "USAGE_VARIANT_UNSPECIFIED",
            Self::OnRampCoinbase => "USAGE_VARIANT_ON_RAMP_COINBASE",
            Self::OnRampMoonpay => "USAGE_VARIANT_ON_RAMP_MOONPAY",
            Self::Paymaster => "USAGE_VARIANT_PAYMASTER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "USAGE_VARIANT_UNSPECIFIED" => Some(Self::Unspecified),
            "USAGE_VARIANT_ON_RAMP_COINBASE" => Some(Self::OnRampCoinbase),
            "USAGE_VARIANT_ON_RAMP_MOONPAY" => Some(Self::OnRampMoonpay),
            "USAGE_VARIANT_PAYMASTER" => Some(Self::Paymaster),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ActivityProtectedCategory {
    #[serde(rename = "ACTIVITY_PROTECTED_CATEGORY_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "ACTIVITY_PROTECTED_CATEGORY_SIGN")]
    Sign = 2,
    #[serde(rename = "ACTIVITY_PROTECTED_CATEGORY_SMS")]
    Sms = 3,
    #[serde(rename = "ACTIVITY_PROTECTED_CATEGORY_FIAT_ON_RAMP")]
    FiatOnRamp = 4,
}
impl ActivityProtectedCategory {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ACTIVITY_PROTECTED_CATEGORY_UNSPECIFIED",
            Self::Sign => "ACTIVITY_PROTECTED_CATEGORY_SIGN",
            Self::Sms => "ACTIVITY_PROTECTED_CATEGORY_SMS",
            Self::FiatOnRamp => "ACTIVITY_PROTECTED_CATEGORY_FIAT_ON_RAMP",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACTIVITY_PROTECTED_CATEGORY_UNSPECIFIED" => Some(Self::Unspecified),
            "ACTIVITY_PROTECTED_CATEGORY_SIGN" => Some(Self::Sign),
            "ACTIVITY_PROTECTED_CATEGORY_SMS" => Some(Self::Sms),
            "ACTIVITY_PROTECTED_CATEGORY_FIAT_ON_RAMP" => Some(Self::FiatOnRamp),
            _ => None,
        }
    }
}
