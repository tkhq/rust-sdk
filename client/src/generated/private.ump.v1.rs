#[derive(Debug)]
#[derive(Debug)]
#[derive(Debug)]
/// TODO: we should move this out of the "private" namespace. It's NOT private because it's now used in a public interface.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PolicyEvaluation {
    pub policy_id: ::prost::alloc::string::String,
    pub outcome: super::super::super::immutable::common::v1::Outcome,
}
