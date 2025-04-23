#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct Pagination {
    pub limit: ::prost::alloc::string::String,
    pub before: ::prost::alloc::string::String,
    pub after: ::prost::alloc::string::String,
}
