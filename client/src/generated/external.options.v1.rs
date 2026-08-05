#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Pagination {
    pub limit: ::prost::alloc::string::String,
    pub before: ::prost::alloc::string::String,
    pub after: ::prost::alloc::string::String,
}
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PageInfo {
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default)]
    pub has_previous_page: bool,
    #[serde(default)]
    pub start_cursor: ::core::option::Option<::prost::alloc::string::String>,
    #[serde(default)]
    pub end_cursor: ::core::option::Option<::prost::alloc::string::String>,
}
