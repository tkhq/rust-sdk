use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use walkdir::WalkDir;

mod transform;

const PUBLIC_API_PROTO_PATH: &str = "proto/services/coordinator/public/v1/public_api.proto";
const EXTERNAL_ACTIVITY_PROTO_PATH: &str = "proto/external/activity/v1/activity.proto";
const INCLUDE_PROTO_PATH: &str = "proto";
const ACTIVITIES_MAPPING_PATH: &str = "proto/activities.json";
const GENERATED_CLIENT_DIR: &str = "client/src/generated";
const CLIENT_TEMPLATE_PATH: &str = "codegen/src/client.template.rs";

// Necessary derive to parse from and serialize to JSON
const SERDE_DERIVE: &str = "#[derive(::serde::Serialize, ::serde::Deserialize)]";
const SERDE_CAMEL_CASE: &str = "#[serde(rename_all = \"camelCase\")]";

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivityDetails {
    intent_type: String,
    result_type: String,
    #[serde(default)]
    internal: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ActivitiesFile {
    activities: HashMap<String, ActivityDetails>,
}

#[derive(Debug, PartialEq, Eq)]
struct Rpc {
    name: String,
    request_type: String,
    response_type: String,
    response_kind: RpcResponseKind,
    options: String,
}

#[derive(Debug, PartialEq, Eq)]
enum RpcResponseKind {
    Unary,
}

fn main() {
    let out_dir = PathBuf::from(GENERATED_CLIENT_DIR);

    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .include_file("mod.rs")
        .out_dir(out_dir.clone())
        .type_attribute(".services", SERDE_DERIVE)
        .type_attribute(".services", SERDE_CAMEL_CASE)
        .type_attribute(".external", SERDE_DERIVE)
        .type_attribute(".external", SERDE_CAMEL_CASE)
        .type_attribute(".immutable", SERDE_DERIVE)
        .type_attribute(".immutable", SERDE_CAMEL_CASE)
        .type_attribute(".google.rpc", SERDE_DERIVE)
        .field_attribute("google.rpc.Status.details", "#[serde(skip)]")
        .type_attribute("google.rpc", SERDE_CAMEL_CASE)
        .compile_protos(&[PUBLIC_API_PROTO_PATH], &[INCLUDE_PROTO_PATH])
        .unwrap();

    let proto = fs::read_to_string(PUBLIC_API_PROTO_PATH).expect("Failed to read proto file");
    // Now we try to parse the URL out of the option. For example:
    //   option (google.api.http) = {
    //     post: "/public/v1/submit/delete_policy"
    //     body: "*"
    //   };
    // --> we want to extract "/public/v1/submit/delete_policy"
    //
    // Note that we can afford the specificity of "post" because Turnkey's API is entirely made of POST requests.
    let http_re = Regex::new(r#"post\s*:\s*\"([^\"]+)\""#).unwrap();

    // Now we try to parse the description and summary out of the options. For example:
    //    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
    //      description: "Get basic information about your current API or WebAuthN user and their organization. Affords Sub-Organization look ups via Parent Organization for WebAuthN or API key users."
    //      summary: "Who am I?"
    //      tags: "Sessions"
    //    };
    // --> we want to extract "Who Am I?" as the summary
    // --> we want to extract "Get basic information about...." as the description
    let description_re = Regex::new(r#"description\s*:\s*\"([^\"]+)\""#).unwrap();
    let summary_re = Regex::new(r#"summary\s*:\s*\"([^\"]+)\""#).unwrap();

    let mut generated_methods = String::new();

    // The activity type is read directly from the enum annotation on each request message in the external activity proto.
    // Intent and result types are looked up from activities.json using the activity type as the key.
    let activities_mapping_data =
        fs::read_to_string(ACTIVITIES_MAPPING_PATH).expect("cannot read activities.json");
    let parsed_activities: ActivitiesFile =
        serde_json::from_str(&activities_mapping_data).expect("cannot parse activities.json");

    let external_activity_proto = fs::read_to_string(EXTERNAL_ACTIVITY_PROTO_PATH)
        .expect("Failed to read external activity proto file");
    let request_to_activity_type = request_to_activity_types(&external_activity_proto);
    let requests_with_app_proofs = requests_with_generate_app_proofs(&external_activity_proto);

    for rpc in parse_rpcs(&proto) {
        // e.g. "DeletePolicy"
        let fn_name = to_snake_case(&rpc.name);

        // e.g. "external.activity.v1.DeletePolicyRequest"
        let req_type = rpc.request_type.as_str();

        // e.g.. "ActivityResponse"
        let res_type = rpc.response_type.as_str();

        // This is the inside of our "rpc Foo(input) returns (output) { ... }" block.
        // It contains a list of options. For example:
        //     option (google.api.http) = {
        //      post: "/public/v1/submit/delete_policy"
        //      body: "*"
        //    };
        //    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
        //      description: "Delete an existing Policy"
        //      summary: "Delete Policy"
        //      tags: "Policies"
        //    };
        let http_opts = rpc.options.as_str();
        let is_internal =
            http_opts.contains("option (google.api.method_visibility).restriction = \"INTERNAL\"");
        let is_tvc = fn_name.contains("tvc");
        if is_internal && !is_tvc {
            // Skip internal-only endpoints (except TVC endpoints)
            continue;
        }

        if fn_name == "n_o_o_p_codegen_anchor" {
            // Skip the NOOP anchor endpoint — it exists only to anchor extra
            // OpenAPI/TypeScript types and has no summary/description.
            continue;
        }

        if let Some(http_caps) = http_re.captures(http_opts) {
            // This is our URL (e.g. "/public/v1/submit/delete_policy")
            let route = &http_caps[1];

            // We expect a description and summary for each endpoint.
            let summary = &summary_re
                .captures(http_opts)
                .unwrap_or_else(|| panic!("no summary found for {route}"))[1];
            let description = &description_re
                .captures(http_opts)
                .unwrap_or_else(|| panic!("no description found for {route}"))[1];

            if req_type.contains("external.activity.v1") {
                // If the request type is "external.activity.v1.DeletePolicyRequest" the sanitized
                // request type will be "DeletePolicyRequest", and we'll need to import it from the external activity namespace
                let short_req_type = req_type.rsplit(".").next().unwrap();
                let activity_type = request_to_activity_type
                        .get(short_req_type)
                        .unwrap_or_else(|| panic!("no activity type annotation found for {short_req_type} in external activity proto"));
                let activities_details = parsed_activities
                    .activities
                    .get(activity_type.as_str())
                    .unwrap_or_else(|| {
                        panic!("activity type {activity_type} not found in activities.json")
                    });
                if activities_details.internal {
                    continue;
                }
                let activity_intent = activities_details.intent_type.clone();
                let activity_result = activities_details.result_type.clone();
                let app_proofs_field =
                    build_generate_app_proofs_field(short_req_type, &requests_with_app_proofs);

                // Approve and Reject activity functions are a bit different than the rest
                // In the mapping they have a resultType set to "*" (because they can indeed reference ANY activity.
                let activity_func = if activity_result == "*" {
                    format!(
                        r#"
                            /// {summary}
                            ///
                            /// {description}
                            pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<external_activity::Activity, TurnkeyClientError> {{
                                let request = external_activity::{short_req_type} {{
                                    r#type: "{activity_type}".to_string(),
                                    timestamp_ms: timestamp_ms.to_string(),
                                    parameters: Some(params),
                                    organization_id,
                                    {app_proofs_field}
                                }};

                                self.process_activity(&request, "{route}".to_string()).await
                            }}
                        "#
                    )
                } else {
                    format!(
                        r#"
                            /// {summary}
                            ///
                            /// {description}
                            pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<ActivityResult<immutable_activity::{activity_result}>, TurnkeyClientError> {{
                                let request = external_activity::{short_req_type} {{
                                    r#type: "{activity_type}".to_string(),
                                    timestamp_ms: timestamp_ms.to_string(),
                                    parameters: Some(params),
                                    organization_id,
                                    {app_proofs_field}
                                }};

                                let activity: external_activity::Activity = self.process_activity(&request, "{route}".to_string()).await?;

                                let inner = activity
                                    .result
                                    .ok_or_else(|| TurnkeyClientError::MissingResult)?
                                    .inner
                                    .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
                                let result = match inner {{
                                    immutable_activity::result::Inner::{activity_result}(res) => res,
                                    other => return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                                        serde_json::to_string(&other)?,
                                    )),
                                }};

                                Ok(ActivityResult {{
                                    result,
                                    activity_id: activity.id,
                                    status: activity.status,
                                    app_proofs: activity.app_proofs,
                                }})
                            }}
                        "#
                    )
                };

                println!("Generating {fn_name} (activity)");
                generated_methods.push_str(&activity_func);
            } else {
                let func = match rpc.response_kind {
                    RpcResponseKind::Unary => format!(
                        r#"
                            /// {summary}
                            ///
                            /// {description}
                            pub async fn {fn_name}(&self, request: coordinator::{req_type}) -> Result<coordinator::{res_type}, TurnkeyClientError> {{
                                self.process_request(&request, "{route}".to_string()).await
                            }}
                        "#
                    ),
                };
                println!("Generating {fn_name} (query)");
                generated_methods.push_str(&func);
            }
        }
    }

    // Insert the generated methods into the right spot inside of our template
    let method_placeholder = "// <GENERATED_METHODS_PLACEHOLDER>";
    let client_template = fs::read_to_string(CLIENT_TEMPLATE_PATH).unwrap();
    let output = client_template.replace(method_placeholder, &generated_methods);

    // Write the generated client to the `client` crate
    let generated_client_path = PathBuf::from(GENERATED_CLIENT_DIR).join("client.rs");
    fs::create_dir_all("../client/src/generated").unwrap();
    fs::write(generated_client_path, output).expect("Failed to write generated file");

    // Now add client.rs to the generated/mod.rs file
    let mut mod_rs = fs::OpenOptions::new()
        .append(true)
        .open(out_dir.join("mod.rs"))
        .unwrap();

    // This will append new lines to the end of the mod.rs file
    writeln!(mod_rs, "\n// Added by turnkey_codegen").unwrap();
    writeln!(mod_rs, "mod client;").unwrap();
    writeln!(mod_rs, "pub use services::coordinator::public::v1::*;").unwrap();
    writeln!(mod_rs, "pub use external::activity::v1::*;").unwrap();
    writeln!(mod_rs, "pub use immutable::activity::v1::*;").unwrap();

    // Now, are you ready for this? tonic/prost do not support `enums` in ergonomic forms.
    // A field that is an enum is represented as an i32 :(
    // Prost has annotations for this, but serde has no clue and can't do anything about this:
    //   #[prost(enumeration = "super::super::common::v1::ApiKeyCurve", tag = "3")]
    //   pub curve_type: i32,
    // ...and prost doesn't support JSON serialization...
    // So here we are, doing some post-processing on the generated code, to manually switch enum fields from
    // the generic "i32" type to their _actual_ type (e.g. super::super::common::v1::ApiKeyCurve)
    // files that we need to add custom JSON logic to
    for entry in WalkDir::new(GENERATED_CLIENT_DIR)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                // No need to process mod.rs
                && !e.file_name().to_string_lossy().ends_with("mod.rs")
                // No need to process google vendored files
                && !e.file_name().to_string_lossy().ends_with("google.api.rs")
                && !e.file_name().to_string_lossy().ends_with("google.rpc.rs")
                && !e
                    .file_name()
                    .to_string_lossy()
                    .ends_with("grpc.gateway.protoc_gen_openapiv2.options.rs")
        })
    {
        let path = entry.path();
        println!("Post-Processing {}", path.display());

        let content = fs::read_to_string(path).expect("cannot read file");
        let new_content = transform::transform(&content);
        if new_content != content {
            fs::write(path, new_content).expect("cannot write to file");
        }
    }
}

fn parse_rpcs(proto: &str) -> Vec<Rpc> {
    // Capture the start of "service... {" until a single "}" is encountered on its own line without indentation.
    // That's just a simple alternative to writing a nesting-aware parser.
    let service_re = Regex::new(r"(?ms)^service\s+(\w+)\s*\{\n(.*?)^\}").unwrap();

    // Parse unary rpc blocks inside a service body.
    let rpc_re = Regex::new(
        r#"(?ms)^  rpc\s+(\w+)\s*\(\s*([a-zA-Z0-9_.]+)\s*\)\s+returns\s+\(\s*([a-zA-Z0-9_.]+)\s*\)\s*\{\n(.*?)^  \}"#
    ).unwrap();

    service_re
        .captures_iter(proto)
        .flat_map(|service_caps| {
            let service_body = service_caps.get(2).unwrap().as_str();
            rpc_re.captures_iter(service_body).map(|rpc_caps| Rpc {
                name: rpc_caps[1].to_string(),
                request_type: rpc_caps[2].to_string(),
                response_type: rpc_caps[3].to_string(),
                response_kind: RpcResponseKind::Unary,
                options: rpc_caps[4].to_string(),
            })
        })
        .collect()
}

// Simple utility function to convert CamelCase to snake_case
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
fn build_generate_app_proofs_field(
    req_type: &str,
    requests_with_app_proofs: &HashSet<String>,
) -> String {
    if requests_with_app_proofs.contains(req_type) {
        "generate_app_proofs: self.generate_app_proofs(),".to_string()
    } else {
        String::new()
    }
}

fn request_to_activity_types(proto: &str) -> HashMap<String, String> {
    let message_re = Regex::new(r"(?ms)^message\s+(\w+)\s*\{\n(.*?)\n\}").unwrap();
    let enum_annotation_re = Regex::new(r#"enum:\s*\["(ACTIVITY_TYPE_[^"]+)"\]"#).unwrap();
    message_re
        .captures_iter(proto)
        .filter_map(|caps| {
            let name = caps[1].to_string();
            let body = &caps[2];
            enum_annotation_re
                .captures(body)
                .map(|e| (name, e[1].to_string()))
        })
        .collect()
}

fn requests_with_generate_app_proofs(proto: &str) -> HashSet<String> {
    // Find proto `message` definitions and return the message names whose bodies
    // contain `generate_app_proofs`.
    let message_re = Regex::new(r"(?ms)^message\s+(\w+)\s*\{(.*?)^\}").unwrap();
    message_re
        .captures_iter(proto)
        .filter_map(|caps| {
            let name = caps[1].to_string();
            let body = &caps[2];
            if body.contains("generate_app_proofs") {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rpcs_decodes_unary_rpc() {
        let proto = r#"
service PublicApiService {
  rpc GetWhoami(GetWhoamiRequest) returns (GetWhoamiResponse) {
    option (google.api.http) = {
      post: "/public/v1/query/whoami"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      description: "Get basic information about your current API user."
      summary: "Who am I?"
      tags: "Sessions"
    };
  }
}
"#;

        let rpcs = parse_rpcs(proto);

        assert_eq!(rpcs.len(), 1);
        assert_eq!(rpcs[0].name, "GetWhoami");
        assert_eq!(rpcs[0].request_type, "GetWhoamiRequest");
        assert_eq!(rpcs[0].response_type, "GetWhoamiResponse");
        assert_eq!(rpcs[0].response_kind, RpcResponseKind::Unary);
        assert!(rpcs[0]
            .options
            .contains("post: \"/public/v1/query/whoami\""));
    }

    #[test]
    fn parse_rpcs_ignores_server_streaming_rpc() {
        let proto = r#"
service PublicApiService {
  rpc GetEnclaveDebugLogs(GetEnclaveDebugLogsRequest) returns (stream GetEnclaveDebugLogsResponse) {
    option (google.api.method_visibility).restriction = "INTERNAL";
    option (google.api.http) = {
      post: "/public/v1/query/get_enclave_debug_logs"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      description: "Get debug logs for a TVC enclave deployment."
      summary: "Get enclave debug logs"
      tags: "TVC"
    };
  }
}
"#;

        assert!(parse_rpcs(proto).is_empty());
    }

    #[test]
    fn parse_rpcs_decodes_fully_qualified_types() {
        let proto = r#"
service PublicApiService {
  rpc SubmitFoo(external.activity.v1.FooRequest) returns (external.activity.v1.ActivityResponse) {
    option (google.api.http) = {
      post: "/public/v1/submit/foo"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      description: "Create foo."
      summary: "Create foo"
      tags: "Foo"
    };
  }
}
"#;

        let rpcs = parse_rpcs(proto);

        assert_eq!(rpcs.len(), 1);
        assert_eq!(rpcs[0].name, "SubmitFoo");
        assert_eq!(rpcs[0].request_type, "external.activity.v1.FooRequest");
        assert_eq!(
            rpcs[0].response_type,
            "external.activity.v1.ActivityResponse"
        );
        assert_eq!(rpcs[0].response_kind, RpcResponseKind::Unary);
    }

    #[test]
    fn parse_rpcs_ignores_client_streaming_rpc() {
        let proto = r#"
service PublicApiService {
  rpc UploadLogs(stream UploadLogsRequest) returns (UploadLogsResponse) {
    option (google.api.http) = {
      post: "/public/v1/query/upload_logs"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      description: "Upload logs."
      summary: "Upload logs"
      tags: "TVC"
    };
  }
}
"#;

        assert!(parse_rpcs(proto).is_empty());
    }

    #[test]
    fn generate_app_proofs_field_is_emitted_only_when_request_supports_it() {
        let mut requests = HashSet::new();
        requests.insert("CreateUsersRequest".to_string());
        requests.insert("UpdateOrganizationNameRequest".to_string());
        requests.insert("SetIpAllowlistRequest".to_string());
        requests.insert("RemoveIpAllowlistRequest".to_string());
        requests.insert("DeleteTvcAppAndDeploymentsRequest".to_string());

        assert!(
            build_generate_app_proofs_field("CreateUsersRequest", &requests)
                .contains("generate_app_proofs")
        );
        assert!(
            build_generate_app_proofs_field("UpdateOrganizationNameRequest", &requests)
                .contains("generate_app_proofs")
        );
        assert!(
            build_generate_app_proofs_field("SetIpAllowlistRequest", &requests)
                .contains("generate_app_proofs")
        );
        assert!(
            build_generate_app_proofs_field("RemoveIpAllowlistRequest", &requests)
                .contains("generate_app_proofs")
        );
        assert!(
            build_generate_app_proofs_field("DeleteTvcAppAndDeploymentsRequest", &requests)
                .contains("generate_app_proofs")
        );
        assert_eq!(
            build_generate_app_proofs_field("CreateTvcAppRequest", &requests),
            ""
        );
    }

    #[test]
    fn requests_with_generate_app_proofs_parses_proto() {
        let proto = r#"
message FooRequest {
  string type = 1;
  optional bool generate_app_proofs = 5;
}

message BarRequest {
  string type = 1;
}

message BazRequest {
  string type = 1;
  optional bool generate_app_proofs = 5;
}
"#;
        let set = requests_with_generate_app_proofs(proto);
        assert!(set.contains("FooRequest"));
        assert!(!set.contains("BarRequest"));
        assert!(set.contains("BazRequest"));
    }
}
