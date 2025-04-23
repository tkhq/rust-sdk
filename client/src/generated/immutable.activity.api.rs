#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ApiKeyParams {
    /// @inject_tag: validate:"required,tk_label_length,tk_label"
    pub api_key_name: ::prost::alloc::string::String,
    /// @inject_tag: validate:"hexadecimal,len=66"
    pub public_key: ::prost::alloc::string::String,
    #[serde(default)]
    pub expiration_seconds: ::core::option::Option<::prost::alloc::string::String>,
}
