#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct TokenUsage {
    pub r#type: UsageType,
    pub token_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub usage: ::core::option::Option<token_usage::Usage>,
}
/// Nested message and enum types in `TokenUsage`.
pub mod token_usage {
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[derive(Clone, PartialEq)]
    #[derive(Debug)]
    pub enum Usage {
        #[serde(rename = "USAGE_SIGNUP")]
        Signup(super::SignupUsage),
        #[serde(rename = "USAGE_LOGIN")]
        Login(super::LoginUsage),
        #[serde(rename = "USAGE_SIGNUP_V2")]
        SignupV2(super::SignupUsageV2),
        #[serde(rename = "USAGE_LOGIN_V2")]
        LoginV2(super::LoginUsageV2),
        #[serde(rename = "USAGE_SIGNUP_V3")]
        SignupV3(super::SignupUsageV3),
    }
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignupUsage {
    #[serde(default)]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub phone_number: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::super::activity::v1::ApiKeyParamsV2>,
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<
        super::super::activity::v1::AuthenticatorParamsV2,
    >,
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<
        super::super::activity::v1::OauthProviderParams,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignupUsageV2 {
    #[serde(default)]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub phone_number: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub api_keys: ::prost::alloc::vec::Vec<super::super::activity::v1::ApiKeyParamsV2>,
    #[serde(default)]
    pub authenticators: ::prost::alloc::vec::Vec<
        super::super::activity::v1::AuthenticatorParamsV2,
    >,
    #[serde(default)]
    pub oauth_providers: ::prost::alloc::vec::Vec<
        super::super::activity::v1::OauthProviderParamsV2,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct LoginUsage {
    pub public_key: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct LoginUsageV2 {
    pub organization_id: ::prost::alloc::string::String,
    pub public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub invalidate_existing: ::core::option::Option<bool>,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub session_profile_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SignupUsageV3 {
    pub parent_organization_id: ::prost::alloc::string::String,
    pub sub_organization_name: ::prost::alloc::string::String,
    #[serde(default)]
    pub root_users: ::prost::alloc::vec::Vec<
        super::super::activity::v1::RootUserParamsV5,
    >,
    #[serde(default)]
    pub root_quorum_threshold: i32,
    #[serde(default)]
    pub wallet: ::core::option::Option<super::super::activity::v1::WalletParams>,
    #[serde(default)]
    pub disable_email_recovery: ::core::option::Option<bool>,
    #[serde(default)]
    pub disable_email_auth: ::core::option::Option<bool>,
    #[serde(default)]
    pub disable_sms_auth: ::core::option::Option<bool>,
    #[serde(default)]
    pub disable_otp_email_auth: ::core::option::Option<bool>,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UsageType {
    #[serde(rename = "USAGE_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "USAGE_TYPE_SIGNUP")]
    Signup = 1,
    #[serde(rename = "USAGE_TYPE_LOGIN")]
    Login = 2,
}
impl UsageType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "USAGE_TYPE_UNSPECIFIED",
            Self::Signup => "USAGE_TYPE_SIGNUP",
            Self::Login => "USAGE_TYPE_LOGIN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "USAGE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "USAGE_TYPE_SIGNUP" => Some(Self::Signup),
            "USAGE_TYPE_LOGIN" => Some(Self::Login),
            _ => None,
        }
    }
}
