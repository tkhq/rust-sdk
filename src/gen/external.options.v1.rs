// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pagination {
    #[prost(string, tag="1")]
    pub limit: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub before: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub after: ::prost::alloc::string::String,
}
