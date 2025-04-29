# Codegen

This crate contains the code necessary to generate the `generated` portion of the client code.

[`main.rs`](./src/main.rs) contains the code for this. At a high-level, the process to generate the Rust SDK client functions has 3 steps:
* Parse the protos and produce a set of Rust files from them with [`tonic_build`](https://docs.rs/tonic-build/latest/tonic_build/) (which uses [`prost`](https://crates.io/crates/prost) under the hood).
* Manually parse the grpc annotations from the top-level service definition. These annotations are crucial because they define REST methods, URLs, etc.
* For each parsed endpoint, produce a generated client method with the right input and output types. This is easy for queries, but tricky for Turnkey **activities** because the mapping of URL to inner intent and result type isn't defined structurally. Hence the need to bring in [`activities.json`](../proto/activities.json), a mapping which defines exactly this.
* Finally, cleanup our generated code with a final `transform` step which parses the generated Rust code and applies a few transformations to it
  * replaces `i32` enum types with proper types (e.g. `pub effect: i32` -> `pub effect: super::super::super::immutable::common::v1::Effect,`)
  * transform enums: remove `#[derive(...)]` prost traits, remove `#[prost(...)]` attributes, remove `#[repr(i32)]`, and add `#[serde(rename = "ENUM_NAME_VARIANT")]` (necesary for JSON serialization and deserialization to work correctly)
  * transform structs: remove `#[prost(...)]` attributes, add `#[serde(default)]` to help with deserialization of empty lists or optional params, add `#[serde(flatten)]` to ignore the `Inner` struct which is a result of the `oneof` structure of activity intent and results. The serialized JSON doesn't have "inner".
