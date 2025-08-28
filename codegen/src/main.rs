use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use walkdir::WalkDir;

mod transform;

const PUBLIC_API_PROTO_PATH: &str = "proto/services/coordinator/public/v1/public_api.proto";
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
    r#type: String,
    intent_type: String,
    result_type: String,
}

#[derive(Debug, serde::Deserialize)]
struct ActivitiesFile {
    activities: HashMap<String, ActivityDetails>,
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
        // TODO remove me once PolicyEvaluation moves to external namespace.
        .type_attribute(".private", SERDE_DERIVE)
        .type_attribute(".private", SERDE_CAMEL_CASE)
        .type_attribute(".google.rpc", SERDE_DERIVE)
        .field_attribute("google.rpc.Status.details", "#[serde(skip)]")
        .type_attribute("google.rpc", SERDE_CAMEL_CASE)
        .compile_protos(&[PUBLIC_API_PROTO_PATH], &[INCLUDE_PROTO_PATH])
        .unwrap();

    let proto = fs::read_to_string(PUBLIC_API_PROTO_PATH).expect("Failed to read proto file");

    // Capture the start of "service... {" until a single "}" is encountered on its own line without indentation.
    // That's just a simple alternative to writing a nesting-aware parser...
    // We're trying to match on blocks like this one:
    //   service PublicApiService {
    //     rpc GetWhoami(GetWhoamiRequest) returns (GetWhoamiResponse) {
    //       option (google.api.http) = {
    //         post: "/public/v1/query/whoami"
    //         body: "*"
    //       };
    //       option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
    //         description: "Get basic information about your current API or WebAuthN user and their organization. Affords Sub-Organization look ups via Parent Organization for WebAuthN or API key users."
    //         summary: "Who am I?"
    //         tags: "Sessions"
    //       };
    //     }
    //     ....more rpc blocks
    //   }
    let service_re = Regex::new(r"(?ms)^service\s+(\w+)\s*\{\n(.*?)^\}").unwrap();

    // Now that we have the inside of a "service Foo {...}" block, we're parsing the rpc blocks within that
    // We're capturing the RPC name, input type, output type, and contents. For example:
    //  rpc GetWhoami(GetWhoamiRequest) returns (GetWhoamiResponse) {
    //    option (google.api.http) = {
    //      post: "/public/v1/query/whoami"
    //      body: "*"
    //    };
    //    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
    //      description: "Get basic information about your current API or WebAuthN user and their organization. Affords Sub-Organization look ups via Parent Organization for WebAuthN or API key users."
    //      summary: "Who am I?"
    //      tags: "Sessions"
    //    };
    //  }
    let rpc_re = Regex::new(
        r#"(?ms)^  rpc\s+(\w+)\s*\(\s*([a-zA-Z0-9_.]+)\s*\)\s+returns\s+\(\s*([a-zA-Z0-9_.]+)\s*\)\s*\{\n(.*?)^  \}"#
    ).unwrap();

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

    // We manually have to specify the mapping between activity types, route, intent and result type through a file
    // Unfortunately this information isn't available in proto directly because this mapping is semantic, not structural.
    let activities_mapping_data =
        fs::read_to_string(ACTIVITIES_MAPPING_PATH).expect("cannot read activities.json");
    let parsed_activities: ActivitiesFile =
        serde_json::from_str(&activities_mapping_data).expect("cannot parse activities.json");

    for service_caps in service_re.captures_iter(&proto) {
        // Remember: this capture group has the inside of "service Foo {...}" block,
        // and contains many "rpc Foo(input) returns (output) {...}" blocks
        let service_body = &service_caps[2];

        // Here we iterate over each "rpc Foo(input) returns (output) {...}" block
        for rpc_caps in rpc_re.captures_iter(service_body) {
            // e.g. "DeletePolicy"
            let fn_name = to_snake_case(&rpc_caps[1]);

            // e.g. "external.activity.v1.DeletePolicyRequest"
            let req_type = &rpc_caps[2];

            // e.g.. "ActivityResponse"
            let res_type = &rpc_caps[3];

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
            let http_opts = &rpc_caps[4];
            if http_opts
                .contains("option (google.api.method_visibility).restriction = \"INTERNAL\"")
            {
                // Skip internal-only endpoints
                continue;
            }

            if let Some(http_caps) = http_re.captures(http_opts) {
                // This is our URL (e.g. "/public/v1/submit/delete_policy")
                let route = &http_caps[1];

                // We expect a description and summary for each endpoint.
                let summary = &summary_re
                    .captures(http_opts)
                    .unwrap_or_else(|| panic!("no summary found for {}", route))[1];
                let description = &description_re
                    .captures(http_opts)
                    .unwrap_or_else(|| panic!("no description found for {}", route))[1];

                if req_type.contains("external.activity.v1") {
                    let activities_details = parsed_activities
                        .activities
                        .get(route)
                        .unwrap_or_else(|| panic!("route {} not found in activities.json", route));
                    let activity_type = activities_details.r#type.clone();

                    // If the request type is "external.activity.v1.DeletePolicyRequest" the sanitized
                    // request type will be "DeletePolicyRequest", and we'll need to import it from the external activity namespace
                    let short_req_type = req_type.rsplit(".").next().unwrap();
                    let activity_intent = activities_details.intent_type.clone();
                    let activity_result = activities_details.result_type.clone();

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
                            pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<immutable_activity::{activity_result}, TurnkeyClientError> {{
                                let request = external_activity::{short_req_type} {{
                                    r#type: "{activity_type}".to_string(),
                                    timestamp_ms: timestamp_ms.to_string(),
                                    parameters: Some(params),
                                    organization_id,
                                }};

                                let activity: external_activity::Activity = self.process_activity(&request, "{route}".to_string()).await?;

                                let inner = activity
                                    .result
                                    .ok_or_else(|| TurnkeyClientError::MissingResult)?
                                    .inner
                                    .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
                                match inner {{
                                    immutable_activity::result::Inner::{activity_result}(res) => Ok(res),
                                    other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                                        serde_json::to_string(&other)?,
                                    )),
                                }}
                            }}
                        "#
                        )
                    };

                    println!("Generating {} (activity)", fn_name);
                    generated_methods.push_str(&activity_func);
                } else {
                    let func = format!(
                        r#"
                        /// {summary}
                        ///
                        /// {description}
                        pub async fn {fn_name}(&self, request: coordinator::{req_type}) -> Result<coordinator::{res_type}, TurnkeyClientError> {{
                            self.process_request(&request, "{route}".to_string()).await
                        }}
                    "#
                    );
                    println!("Generating {} (query)", fn_name);
                    generated_methods.push_str(&func);
                }
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
