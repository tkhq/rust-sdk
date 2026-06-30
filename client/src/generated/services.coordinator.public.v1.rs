#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct NoopCodegenAnchorRequest {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct NoopCodegenAnchorResponse {
    #[serde(default)]
    pub stamp: ::core::option::Option<
        super::super::super::super::external::webauthn::v1::WebAuthnStamp,
    >,
    #[serde(default)]
    pub token_usage: ::core::option::Option<
        super::super::super::super::immutable::sdk_models::v1::TokenUsage,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TestRateLimitsRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub is_set_limit: bool,
    #[serde(default)]
    pub limit: u32,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct TestRateLimitsResponse {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWhoamiRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWhoamiResponse {
    pub organization_id: ::prost::alloc::string::String,
    pub organization_name: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    pub username: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSubOrgIdsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub filter_type: ::prost::alloc::string::String,
    pub filter_value: ::prost::alloc::string::String,
    #[serde(default)]
    pub pagination_options: ::core::option::Option<
        super::super::super::super::external::options::v1::Pagination,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetVerifiedSubOrgIdsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub filter_type: ::prost::alloc::string::String,
    pub filter_value: ::prost::alloc::string::String,
    #[serde(default)]
    pub pagination_options: ::core::option::Option<
        super::super::super::super::external::options::v1::Pagination,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSubOrgIdsResponse {
    #[serde(default)]
    pub organization_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetVerifiedSubOrgIdsResponse {
    #[serde(default)]
    pub organization_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOrganizationRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOrganizationResponse {
    #[serde(default)]
    pub organization_data: ::core::option::Option<
        super::super::super::super::external::data::v1::OrganizationData,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetActivityRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub activity_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetActivitiesRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub filter_by_status: Vec<
        super::super::super::super::immutable::activity::v1::ActivityStatus,
    >,
    #[serde(default)]
    pub pagination_options: ::core::option::Option<
        super::super::super::super::external::options::v1::Pagination,
    >,
    #[serde(default)]
    pub filter_by_type: Vec<
        super::super::super::super::immutable::activity::v1::ActivityType,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetActivitiesResponse {
    #[serde(default)]
    pub activities: ::prost::alloc::vec::Vec<
        super::super::super::super::external::activity::v1::Activity,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetUserRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetUserResponse {
    #[serde(default)]
    pub user: ::core::option::Option<
        super::super::super::super::external::data::v1::User,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetUsersRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetUsersResponse {
    #[serde(default)]
    pub users: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::User,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ActivityResponse {
    #[serde(default)]
    pub activity: ::core::option::Option<
        super::super::super::super::external::activity::v1::Activity,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPoliciesRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPoliciesResponse {
    #[serde(default)]
    pub policies: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Policy,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPolicyRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPolicyResponse {
    #[serde(default)]
    pub policy: ::core::option::Option<
        super::super::super::super::external::data::v1::Policy,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSmartContractInterfacesRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSmartContractInterfacesResponse {
    #[serde(default)]
    pub smart_contract_interfaces: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::SmartContractInterface,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSmartContractInterfaceRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub smart_contract_interface_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSmartContractInterfaceResponse {
    #[serde(default)]
    pub smart_contract_interface: ::core::option::Option<
        super::super::super::super::external::data::v1::SmartContractInterface,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAuthenticatorRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub authenticator_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAuthenticatorResponse {
    #[serde(default)]
    pub authenticator: ::core::option::Option<
        super::super::super::super::external::data::v1::Authenticator,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAuthenticatorsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAuthenticatorsResponse {
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Authenticator,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOauthProvidersRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOauthProvidersResponse {
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::OauthProvider,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetApiKeyRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub api_key_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetApiKeyResponse {
    #[serde(default)]
    pub api_key: ::core::option::Option<
        super::super::super::super::external::data::v1::ApiKey,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetApiKeysRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetApiKeysResponse {
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::ApiKey,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSendTransactionStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub send_transaction_status_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthSendTransactionStatus {
    #[serde(default)]
    pub tx_hash: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SolanaSendTransactionStatus {
    #[serde(default)]
    pub signature: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSendTransactionStatusResponse {
    pub tx_status: ::prost::alloc::string::String,
    #[serde(default)]
    pub tx_error: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub error: ::core::option::Option<TxError>,
    /// VM-specific transaction details
    #[serde(default)]
    pub details: ::core::option::Option<get_send_transaction_status_response::Details>,
}
/// Nested message and enum types in `GetSendTransactionStatusResponse`.
pub mod get_send_transaction_status_response {
    /// VM-specific transaction details
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum Details {
        #[serde(rename = "DETAILS_ETH")]
        Eth(super::EthSendTransactionStatus),
        #[serde(rename = "DETAILS_SOLANA")]
        Solana(super::SolanaSendTransactionStatus),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOnRampTransactionStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub transaction_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub refresh: ::core::option::Option<bool>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOnRampTransactionStatusResponse {
    pub transaction_status: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPrivateKeysRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPrivateKeysResponse {
    #[serde(default)]
    pub private_keys: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::PrivateKey,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPrivateKeyRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub private_key_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPrivateKeyResponse {
    #[serde(default)]
    pub private_key: ::core::option::Option<
        super::super::super::super::external::data::v1::PrivateKey,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletsResponse {
    #[serde(default)]
    pub wallets: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Wallet,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletResponse {
    #[serde(default)]
    pub wallet: ::core::option::Option<
        super::super::super::super::external::data::v1::Wallet,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAccountsRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub wallet_id: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub include_wallet_details: ::core::option::Option<bool>,
    #[serde(default)]
    pub pagination_options: ::core::option::Option<
        super::super::super::super::external::options::v1::Pagination,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAccountsResponse {
    #[serde(default)]
    pub accounts: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::WalletAccount,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAccountRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub path: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAccountResponse {
    #[serde(default)]
    pub account: ::core::option::Option<
        super::super::super::super::external::data::v1::WalletAccount,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListUserTagsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListUserTagsResponse {
    #[serde(default)]
    pub user_tags: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Tag,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListPrivateKeyTagsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListPrivateKeyTagsResponse {
    #[serde(default)]
    pub private_key_tags: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Tag,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOrganizationConfigsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOrganizationConfigsResponse {
    #[serde(default)]
    pub configs: ::core::option::Option<
        super::super::super::super::external::data::v1::Config,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPolicyEvaluationsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub activity_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetPolicyEvaluationsResponse {
    #[serde(default)]
    pub policy_evaluations: ::prost::alloc::vec::Vec<
        super::super::super::super::external::activity::v1::PolicyEvaluation,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListOauth2CredentialsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListOauth2CredentialsResponse {
    #[serde(default)]
    pub oauth2_credentials: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::Oauth2Credential,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListWebhookEndpointsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListWebhookEndpointsResponse {
    #[serde(default)]
    pub webhook_endpoints: ::prost::alloc::vec::Vec<
        super::super::super::super::immutable::activity::v1::WebhookEndpointData,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOauth2CredentialRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub oauth2_credential_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetOauth2CredentialResponse {
    #[serde(default)]
    pub oauth2_credential: ::core::option::Option<
        super::super::super::super::external::data::v1::Oauth2Credential,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetBootProofRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub ephemeral_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetLatestBootProofRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub app_name: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct BootProofResponse {
    #[serde(default)]
    pub boot_proof: ::core::option::Option<
        super::super::super::super::external::data::v1::BootProof,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAppProofsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub activity_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAppProofsResponse {
    #[serde(default)]
    pub app_proofs: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::AppProof,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListFiatOnRampCredentialsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListFiatOnRampCredentialsResponse {
    #[serde(default)]
    pub fiat_on_ramp_credentials: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::FiatOnRampCredential,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSwapQuoteRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub input_token: ::prost::alloc::string::String,
    pub output_token: ::prost::alloc::string::String,
    pub input_amount: ::prost::alloc::string::String,
    #[serde(default)]
    pub slippage: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSwapQuoteResponse {
    pub input_token: ::prost::alloc::string::String,
    pub output_token: ::prost::alloc::string::String,
    pub input_amount: ::prost::alloc::string::String,
    #[serde(default)]
    pub quotes: ::prost::alloc::vec::Vec<SwapQuoteOption>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SwapQuoteOption {
    pub quote_id: ::prost::alloc::string::String,
    pub provider: ::prost::alloc::string::String,
    pub output_amount: ::prost::alloc::string::String,
    #[serde(default)]
    pub min_output_amount: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetGasUsageRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetGasUsageResponse {
    #[serde(default)]
    pub window_duration_minutes: i32,
    pub window_limit_usd: ::prost::alloc::string::String,
    pub usage_usd: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnVaultsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnVault {
    pub vault_address: ::prost::alloc::string::String,
    pub chain_caip2: ::prost::alloc::string::String,
    pub provider: ::prost::alloc::string::String,
    pub asset: ::prost::alloc::string::String,
    pub asset_address: ::prost::alloc::string::String,
    pub tvl: ::prost::alloc::string::String,
    pub apy_pct: ::prost::alloc::string::String,
    #[serde(default)]
    pub enabled: bool,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnVaultsResponse {
    #[serde(default)]
    pub vaults: ::prost::alloc::vec::Vec<EarnVault>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnEnabledVaultsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnEnabledVault {
    pub vault_address: ::prost::alloc::string::String,
    pub wrapper_address: ::prost::alloc::string::String,
    pub splitter_address: ::prost::alloc::string::String,
    pub chain_caip2: ::prost::alloc::string::String,
    pub provider: ::prost::alloc::string::String,
    pub asset: ::prost::alloc::string::String,
    pub apy_pct: ::prost::alloc::string::String,
    pub total_deposited: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnEnabledVaultsResponse {
    #[serde(default)]
    pub enabled_vaults: ::prost::alloc::vec::Vec<EarnEnabledVault>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnPositionsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub wallet_address: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnPosition {
    pub vault_address: ::prost::alloc::string::String,
    pub wrapper_address: ::prost::alloc::string::String,
    pub chain_caip2: ::prost::alloc::string::String,
    pub provider: ::prost::alloc::string::String,
    pub asset: ::prost::alloc::string::String,
    pub shares: ::prost::alloc::string::String,
    pub current_value: ::prost::alloc::string::String,
    pub cost_basis: ::prost::alloc::string::String,
    pub gross_yield: ::prost::alloc::string::String,
    pub projected_fee: ::prost::alloc::string::String,
    pub yield_less_fees: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnPositionsResponse {
    #[serde(default)]
    pub positions: ::prost::alloc::vec::Vec<EarnPosition>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnWithdrawStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub withdraw_request_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnWithdrawStatusResponse {
    pub status: ::prost::alloc::string::String,
    #[serde(default)]
    pub withdraw_tx_hash: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnDepositStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub deposit_request_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EarnDepositStatusResponse {
    pub status: ::prost::alloc::string::String,
    #[serde(default)]
    pub deposit_tx_hash: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetNoncesRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub address: ::prost::alloc::string::String,
    pub caip2: ::prost::alloc::string::String,
    #[serde(default)]
    pub nonce: bool,
    #[serde(default)]
    pub gas_station_nonce: bool,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct GetNoncesResponse {
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub nonce: ::core::option::Option<u64>,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub gas_station_nonce: ::core::option::Option<u64>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TxError {
    pub message: ::prost::alloc::string::String,
    #[serde(default)]
    pub revert_chain: ::prost::alloc::vec::Vec<RevertChainEntry>,
    #[serde(default)]
    pub solana: ::core::option::Option<SolanaFailureDetails>,
    #[serde(default)]
    pub eth: ::core::option::Option<EthFailureDetails>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EthFailureDetails {
    #[serde(default)]
    pub revert_chain: ::prost::alloc::vec::Vec<RevertChainEntry>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct RevertChainEntry {
    pub address: ::prost::alloc::string::String,
    pub error_type: ::prost::alloc::string::String,
    pub display_message: ::prost::alloc::string::String,
    #[serde(default)]
    pub error_details: ::core::option::Option<revert_chain_entry::ErrorDetails>,
}
/// Nested message and enum types in `RevertChainEntry`.
pub mod revert_chain_entry {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum ErrorDetails {
        #[serde(rename = "ERROR_DETAILS_UNKNOWN")]
        Unknown(super::UnknownRevertError),
        #[serde(rename = "ERROR_DETAILS_NATIVE")]
        Native(super::NativeRevertError),
        #[serde(rename = "ERROR_DETAILS_CUSTOM")]
        Custom(super::CustomRevertError),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct UnknownRevertError {
    #[serde(default)]
    pub selector: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub data: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct NativeRevertError {
    #[serde(default)]
    pub native_type: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub message: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub panic_code: ::core::option::Option<u64>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct CustomRevertError {
    #[serde(default)]
    pub error_name: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub params_json: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SolanaFailureDetails {
    pub source: ::prost::alloc::string::String,
    #[serde(default)]
    pub rpc_code: ::core::option::Option<i32>,
    #[serde(default)]
    pub rpc_message: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub transaction_error_json: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub logs: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub units_consumed: ::core::option::Option<u64>,
    #[serde(default)]
    pub inner_instructions_json: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppsRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppsResponse {
    #[serde(default)]
    pub tvc_apps: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::TvcApp,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub tvc_app_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppResponse {
    #[serde(default)]
    pub tvc_app: ::core::option::Option<
        super::super::super::super::external::data::v1::TvcApp,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppDeploymentsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub app_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcAppDeploymentsResponse {
    #[serde(default)]
    pub tvc_deployments: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::TvcDeployment,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub deployment_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentResponse {
    #[serde(default)]
    pub tvc_deployment: ::core::option::Option<
        super::super::super::super::external::data::v1::TvcDeployment,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ValidateTvcImageRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub pivot_container_image_url: ::prost::alloc::string::String,
    #[serde(default)]
    pub pivot_container_encrypted_pull_secret: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ValidateTvcImageResponse {
    pub resolved_image_digest: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentProvisioningDetailsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub deployment_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentProvisioningDetailsResponse {
    #[serde(default)]
    #[serde_as(as = "serde_with::base64::Base64")]
    pub attestation_document: ::prost::alloc::vec::Vec<u8>,
    #[serde(default)]
    #[serde_as(as = "serde_with::base64::Base64")]
    pub manifest_envelope: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAppStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub app_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAppStatusResponse {
    #[serde(default)]
    pub app_status: ::core::option::Option<
        super::super::super::super::external::data::v1::AppStatus,
    >,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentDebugLogsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub deployment_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub tail_lines: i32,
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub since_seconds: i64,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TvcDeploymentDebugLogEntry {
    #[serde(default)]
    pub line: ::core::option::Option<
        super::super::super::super::external::data::v1::LogLine,
    >,
    pub replica_label: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetTvcDeploymentDebugLogsResponse {
    #[serde(default)]
    pub entries: ::prost::alloc::vec::Vec<TvcDeploymentDebugLogEntry>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct RefreshFeatureFlagsRequest {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct RefreshFeatureFlagsResponse {}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAddressBalancesRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub address: ::prost::alloc::string::String,
    pub caip2: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetWalletAddressBalancesResponse {
    #[serde(default)]
    pub balances: ::prost::alloc::vec::Vec<AssetBalance>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AssetBalance {
    pub caip19: ::prost::alloc::string::String,
    pub symbol: ::prost::alloc::string::String,
    pub balance: ::prost::alloc::string::String,
    #[serde(default)]
    pub decimals: i32,
    #[serde(default)]
    pub display: ::core::option::Option<AssetBalanceDisplay>,
    pub name: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AssetBalanceDisplay {
    pub usd: ::prost::alloc::string::String,
    pub crypto: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListSupportedAssetsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub caip2: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListSupportedAssetsResponse {
    #[serde(default)]
    pub assets: ::prost::alloc::vec::Vec<AssetMetadata>,
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
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetIpAllowlistRequest {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct IpAllowlistRule {
    pub cidr: ::prost::alloc::string::String,
    #[serde(default)]
    pub label: ::core::option::Option<::prost::alloc::string::String>,
    pub created_at: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct IpAllowlist {
    pub organization_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub rules: ::prost::alloc::vec::Vec<IpAllowlistRule>,
    #[serde(default)]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub enabled: ::core::option::Option<bool>,
    /// @inject_tag: validate:"omitempty,oneof=ALLOW DENY"
    #[serde(default)]
    pub on_evaluation_error: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetIpAllowlistResponse {
    #[serde(default)]
    pub allowlist: ::core::option::Option<IpAllowlist>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaPolicyRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
    pub mfa_policy_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaPolicyResponse {
    #[serde(default)]
    pub mfa_policy: ::core::option::Option<
        super::super::super::super::external::data::v1::MfaPolicy,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaPoliciesRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub user_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaPoliciesResponse {
    #[serde(default)]
    pub mfa_policies: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::MfaPolicy,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSessionProfileRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub session_profile_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSessionProfileResponse {
    #[serde(default)]
    pub session_profile: ::core::option::Option<
        super::super::super::super::external::data::v1::SessionProfile,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSessionProfilesRequest {
    pub organization_id: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetSessionProfilesResponse {
    #[serde(default)]
    pub session_profiles: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::SessionProfile,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaStatusRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub activity_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub user_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetMfaStatusResponse {
    #[serde(default)]
    pub mfa_statuses: ::prost::alloc::vec::Vec<
        super::super::super::super::external::data::v1::MfaStatus,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListEmailEventsRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub email: ::prost::alloc::string::String,
    pub event_type: ::prost::alloc::string::String,
    #[serde(default)]
    pub pagination_options: ::core::option::Option<
        super::super::super::super::external::options::v1::Pagination,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ListEmailEventsResponse {
    #[serde(default)]
    pub email_events: ::prost::alloc::vec::Vec<EmailEvent>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailEvent {
    pub id: ::prost::alloc::string::String,
    pub organization_id: ::prost::alloc::string::String,
    pub message_id: ::prost::alloc::string::String,
    pub event_type: ::prost::alloc::string::String,
    pub from_address: ::prost::alloc::string::String,
    pub to_address: ::prost::alloc::string::String,
    pub sender_tenant: ::prost::alloc::string::String,
    pub timestamp: ::prost::alloc::string::String,
    pub created_at: ::prost::alloc::string::String,
    #[serde(default)]
    pub details: ::core::option::Option<EmailEventDetails>,
}
#[derive(Debug)]
#[serde_with::serde_as]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct EmailEventDetails {
    pub bounce_type: ::prost::alloc::string::String,
    pub bounce_sub_type: ::prost::alloc::string::String,
    pub diagnostic_code: ::prost::alloc::string::String,
    pub delivery_smtp_response: ::prost::alloc::string::String,
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub delivery_processing_time_millis: u64,
    pub delivery_delay_type: ::prost::alloc::string::String,
}
