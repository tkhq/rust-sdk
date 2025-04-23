/// We expect this to be passed in as a JSON-encoded, then base64-encoded string within a X-Stamp-Webauthn header
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct WebAuthnStamp {
    pub credential_id: ::prost::alloc::string::String,
    pub client_data_json: ::prost::alloc::string::String,
    pub authenticator_data: ::prost::alloc::string::String,
    pub signature: ::prost::alloc::string::String,
}
/// buf:lint:ignore ENUM_VALUE_PREFIX
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthenticatorType {
    /// buf:lint:ignore ENUM_ZERO_VALUE_SUFFIX
    #[serde(rename = "AUTHENTICATOR_TYPE_UNKNOWN")]
    Unknown = 0,
    #[serde(rename = "AUTHENTICATOR_TYPE_CROSS_PLATFORM")]
    CrossPlatform = 1,
    #[serde(rename = "AUTHENTICATOR_TYPE_PLATFORM")]
    Platform = 2,
    #[serde(rename = "AUTHENTICATOR_TYPE_UNSPECIFIED")]
    Unspecified = 3,
}
impl AuthenticatorType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unknown => "AUTHENTICATOR_TYPE_UNKNOWN",
            Self::CrossPlatform => "CROSS_PLATFORM",
            Self::Platform => "PLATFORM",
            Self::Unspecified => "UNSPECIFIED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AUTHENTICATOR_TYPE_UNKNOWN" => Some(Self::Unknown),
            "CROSS_PLATFORM" => Some(Self::CrossPlatform),
            "PLATFORM" => Some(Self::Platform),
            "UNSPECIFIED" => Some(Self::Unspecified),
            _ => None,
        }
    }
}
