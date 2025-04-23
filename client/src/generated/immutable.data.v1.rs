#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Timestamp {
    pub seconds: ::prost::alloc::string::String,
    pub nanos: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Credential {
    pub public_key: ::prost::alloc::string::String,
    pub r#type: super::super::common::v1::CredentialType,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Quorum {
    #[serde(default)]
    pub threshold: i32,
    #[serde(default)]
    pub user_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Feature {
    pub name: super::super::common::v1::FeatureName,
    #[serde(default)]
    pub value: ::core::option::Option<::prost::alloc::string::String>,
}
