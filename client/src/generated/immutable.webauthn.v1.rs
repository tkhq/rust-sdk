#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PublicKeyCredentialDescriptor {
    /// Must be literal string "public-key"
    pub r#type: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub id: ::prost::alloc::string::String,
    #[serde(default)]
    pub transports: Vec<AuthenticatorTransport>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AuthenticatorAssertionResponse {
    /// ENCODING: base64url
    pub client_data_json: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub authenticator_data: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub signature: ::prost::alloc::string::String,
    /// NOTE(keyan): The (TypeScript) spec says this field is non-optional but nullable, i.e.
    ///    `userHandle: string | null`
    ///
    /// What we have here is optional and nullable:
    ///    `userHandle?: string | null`
    ///
    /// We need it to be optional because the field needs to be nil-able in go.
    /// However, a future version of TypeScript (w/ stricter options) might not like the type.
    #[serde(default)]
    pub user_handle: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct AuthenticatorAttestationResponse {
    /// ENCODING: base64url
    pub client_data_json: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub attestation_object: ::prost::alloc::string::String,
    #[serde(default)]
    pub transports: Vec<AuthenticatorTransport>,
    #[serde(default)]
    pub authenticator_attachment: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PublicKeyCredentialWithAttestation {
    pub id: ::prost::alloc::string::String,
    /// Must be literal string "public-key"
    pub r#type: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub raw_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub authenticator_attachment: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub response: ::core::option::Option<AuthenticatorAttestationResponse>,
    #[serde(default)]
    pub client_extension_results: ::core::option::Option<SimpleClientExtensionResults>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PublicKeyCredentialWithAssertion {
    pub id: ::prost::alloc::string::String,
    /// Must be literal string "public-key"
    pub r#type: ::prost::alloc::string::String,
    /// ENCODING: base64url
    pub raw_id: ::prost::alloc::string::String,
    #[serde(default)]
    pub authenticator_attachment: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub response: ::core::option::Option<AuthenticatorAssertionResponse>,
    #[serde(default)]
    pub client_extension_results: ::core::option::Option<SimpleClientExtensionResults>,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct SimpleClientExtensionResults {
    #[serde(default)]
    pub appid: ::core::option::Option<bool>,
    #[serde(default)]
    pub appid_exclude: ::core::option::Option<bool>,
    #[serde(default)]
    pub cred_props: ::core::option::Option<
        CredPropsAuthenticationExtensionsClientOutputs,
    >,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, PartialEq)]
pub struct CredPropsAuthenticationExtensionsClientOutputs {
    #[serde(default)]
    pub rk: bool,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthenticatorTransport {
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_BLE")]
    Ble = 1,
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_INTERNAL")]
    Internal = 2,
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_NFC")]
    Nfc = 3,
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_USB")]
    Usb = 4,
    #[serde(rename = "AUTHENTICATOR_TRANSPORT_HYBRID")]
    Hybrid = 5,
}
impl AuthenticatorTransport {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "AUTHENTICATOR_TRANSPORT_UNSPECIFIED",
            Self::Ble => "AUTHENTICATOR_TRANSPORT_BLE",
            Self::Internal => "AUTHENTICATOR_TRANSPORT_INTERNAL",
            Self::Nfc => "AUTHENTICATOR_TRANSPORT_NFC",
            Self::Usb => "AUTHENTICATOR_TRANSPORT_USB",
            Self::Hybrid => "AUTHENTICATOR_TRANSPORT_HYBRID",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AUTHENTICATOR_TRANSPORT_UNSPECIFIED" => Some(Self::Unspecified),
            "AUTHENTICATOR_TRANSPORT_BLE" => Some(Self::Ble),
            "AUTHENTICATOR_TRANSPORT_INTERNAL" => Some(Self::Internal),
            "AUTHENTICATOR_TRANSPORT_NFC" => Some(Self::Nfc),
            "AUTHENTICATOR_TRANSPORT_USB" => Some(Self::Usb),
            "AUTHENTICATOR_TRANSPORT_HYBRID" => Some(Self::Hybrid),
            _ => None,
        }
    }
}
