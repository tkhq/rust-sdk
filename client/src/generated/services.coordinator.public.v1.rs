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
pub struct GetAttestationDocumentRequest {
    pub organization_id: ::prost::alloc::string::String,
    pub enclave_type: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct GetAttestationDocumentResponse {
    #[serde(default)]
    pub attestation_document: ::prost::alloc::vec::Vec<u8>,
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
    pub wallet_id: ::prost::alloc::string::String,
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
