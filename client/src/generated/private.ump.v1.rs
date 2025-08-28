#[derive(Debug)]
#[serde_with::serde_as]
#[derive(Clone, PartialEq)]
pub struct RulingPayload {
    /// Organization data version used for serialization
    pub organization_data_version: ::prost::alloc::string::String,
    /// The intent of the update
    #[serde(default)]
    pub intent: ::core::option::Option<
        super::super::super::immutable::activity::v1::Intent,
    >,
    /// Decision from Ump
    pub outcome: super::super::super::immutable::common::v1::Outcome,
    /// Used to verifiy recency requirements
    /// This timestamp is in ms, in UTC. It comes directly from the NSM.
    #[serde(default)]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub timestamp: u64,
    /// Hash of the organization data
    pub organization_digest: ::prost::alloc::string::String,
    /// The UUID of the organization
    pub organization_id: ::prost::alloc::string::String,
    /// UUID for new suborg if created
    #[serde(default)]
    pub new_suborg_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Debug)]
#[derive(Clone, PartialEq)]
pub struct Ruling {
    #[serde(default)]
    pub payload: ::core::option::Option<RulingPayload>,
    /// Ump signature over hash(payload)
    #[serde(default)]
    pub signature: ::core::option::Option<
        super::super::super::immutable::models::v1::Signature,
    >,
}
#[derive(Debug)]
#[derive(Clone, PartialEq)]
pub struct PolicyEvaluation {
    pub policy_id: ::prost::alloc::string::String,
    pub outcome: super::super::super::immutable::common::v1::Outcome,
}
