// This file is @generated by prost-build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FieldBehavior {
    Unspecified = 0,
    Optional = 1,
    Required = 2,
    OutputOnly = 3,
    InputOnly = 4,
    Immutable = 5,
    UnorderedList = 6,
    NonEmptyDefault = 7,
}
impl FieldBehavior {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIELD_BEHAVIOR_UNSPECIFIED",
            Self::Optional => "OPTIONAL",
            Self::Required => "REQUIRED",
            Self::OutputOnly => "OUTPUT_ONLY",
            Self::InputOnly => "INPUT_ONLY",
            Self::Immutable => "IMMUTABLE",
            Self::UnorderedList => "UNORDERED_LIST",
            Self::NonEmptyDefault => "NON_EMPTY_DEFAULT",
        }
    }
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIELD_BEHAVIOR_UNSPECIFIED" => Some(Self::Unspecified),
            "OPTIONAL" => Some(Self::Optional),
            "REQUIRED" => Some(Self::Required),
            "OUTPUT_ONLY" => Some(Self::OutputOnly),
            "INPUT_ONLY" => Some(Self::InputOnly),
            "IMMUTABLE" => Some(Self::Immutable),
            "UNORDERED_LIST" => Some(Self::UnorderedList),
            "NON_EMPTY_DEFAULT" => Some(Self::NonEmptyDefault),
            _ => None,
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Http {
    #[prost(message, repeated, tag = "1")]
    pub rules: ::prost::alloc::vec::Vec<HttpRule>,
    #[prost(bool, tag = "2")]
    pub fully_decode_reserved_expansion: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpRule {
    #[prost(string, tag = "1")]
    pub selector: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub body: ::prost::alloc::string::String,
    #[prost(string, tag = "12")]
    pub response_body: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "11")]
    pub additional_bindings: ::prost::alloc::vec::Vec<HttpRule>,
    #[prost(oneof = "http_rule::Pattern", tags = "2, 3, 4, 5, 6, 8")]
    pub pattern: ::core::option::Option<http_rule::Pattern>,
}
pub mod http_rule {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Pattern {
        #[prost(string, tag = "2")]
        Get(::prost::alloc::string::String),
        #[prost(string, tag = "3")]
        Put(::prost::alloc::string::String),
        #[prost(string, tag = "4")]
        Post(::prost::alloc::string::String),
        #[prost(string, tag = "5")]
        Delete(::prost::alloc::string::String),
        #[prost(string, tag = "6")]
        Patch(::prost::alloc::string::String),
        #[prost(message, tag = "8")]
        Custom(super::CustomHttpPattern),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CustomHttpPattern {
    #[prost(string, tag = "1")]
    pub kind: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Visibility {
    #[prost(message, repeated, tag = "1")]
    pub rules: ::prost::alloc::vec::Vec<VisibilityRule>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VisibilityRule {
    #[prost(string, tag = "1")]
    pub selector: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub restriction: ::prost::alloc::string::String,
}
