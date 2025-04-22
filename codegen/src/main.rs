use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;

const PUBLIC_API_PROTO_PATH: &str = "proto/services/coordinator/public/v1/public_api.proto";
const INCLUDE_PROTO_PATH: &str = "proto";
const GENERATED_CLIENT_DIR: &str = "client/src/generated";
const CLIENT_TEMPLATE_PATH: &str = "codegen/src/client.template.rs";
const ACTIVITIES_MAPPING_PATH: &str = "codegen/src/activities.json";

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
    .type_attribute(".google.rpc", SERDE_DERIVE)
    .type_attribute("google.rpc", SERDE_CAMEL_CASE)
    .field_attribute("google.rpc.Status.details", "#[serde(skip)]")
    .compile_protos(
        &[PUBLIC_API_PROTO_PATH],
        &[INCLUDE_PROTO_PATH],
    )
    .unwrap();

    let proto = fs::read_to_string(PUBLIC_API_PROTO_PATH).expect("Failed to read proto file");

    // Capture the start of "service... {" until a single "}" is encountered on its own line without indentation.
    // That's just a simple alternative to writing a nesting-aware parser...
    let service_re = Regex::new(r"(?ms)^service\s+(\w+)\s*\{\n(.*?)^\}").unwrap();

    // let rpc_re = Regex::new(
    //     r"rpc\s+(\w+)\((\w+)\)\s+returns\s+\((\w+)\)\s*\{[^}]*?option\s+\(google.api.http\)\s*=\s*\{([^}]*)\}"
    // ).unwrap();
    // let rpc_re = Regex::new(r"(?ms)^  rpc\s+(\w+)\s*\((\w+)\)\s+returns\s+\((\w+)\)\s*\{\n(.*?)^  \}").unwrap();

    let rpc_re = Regex::new(
        r#"(?ms)^  rpc\s+(\w+)\s*\(\s*([a-zA-Z0-9_.]+)\s*\)\s+returns\s+\(\s*([a-zA-Z0-9_.]+)\s*\)\s*\{\n(.*?)^  \}"#
    ).unwrap();

    let mut generated_methods = String::new();

    // We manually have to specify the mapping between activity types, route, intent and result type through a file
    // Unfortunately this information isn't available in proto directly because this mapping is semantic, not structural.
    let activities_mapping_data = fs::read_to_string(ACTIVITIES_MAPPING_PATH).expect("cannot read activities.json");
    let parsed_activities: ActivitiesFile = serde_json::from_str(&activities_mapping_data).expect("cannot parse activities.json");

    for service_caps in service_re.captures_iter(&proto) {
        let service_body = &service_caps[2];
        
        for rpc_caps in rpc_re.captures_iter(service_body) {
            println!("{}", &rpc_caps[1]);
            // e.g. "DeletePolicy"
            let fn_name = to_snake_case(&rpc_caps[1]);
            
            // e.g. "external.activity.v1.DeletePolicyRequest"
            let req_type = &rpc_caps[2];
            
            let res_type = &rpc_caps[3];
            // Sample captured block:
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
            if http_opts.contains("option (google.api.method_visibility).restriction = \"INTERNAL\"") {
                // Skip internal-only endpoints
                continue;
            }

            let http_re = Regex::new(r#"post\s*:\s*\"([^\"]+)\""#).unwrap();
            if let Some(http_caps) = http_re.captures(http_opts) {
                let route = &http_caps[1];

                if req_type.contains("external.activity.v1") {
                    let activities_details = parsed_activities.activities.get(route).expect(&format!("route {} not found in activities.json", route));
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
    pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<external_activity::Activity, TurnkeyClientError> {{
        let url = format!("{{}}{{}}", self.base_url, "{route}");

        let request = external_activity::{short_req_type} {{
            r#type: "{activity_type}".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            organization_id: organization_id,
            parameters: Some(params)
        }};

        let post_body = serde_json::to_string(&request).unwrap();

        let stamp = stamp(post_body.clone(), &self.api_key).unwrap();

        self.process_activity(url, stamp, post_body).await
    }}
"#)
                    } else {
                        format!(
r#"
    pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<immutable_activity::{activity_result}, TurnkeyClientError> {{
        let url = format!("{{}}{{}}", self.base_url, "{route}");

        let request = external_activity::{short_req_type} {{
            r#type: "{activity_type}".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            organization_id: organization_id,
            parameters: Some(params)
        }};

        let post_body = serde_json::to_string(&request).unwrap();

        let stamp = stamp(post_body.clone(), &self.api_key).unwrap();

        let activity = self.process_activity(url, stamp, post_body).await?;

        let inner = activity
            .result.ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner.ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;

        match inner {{
            immutable_activity::result::Inner::{activity_result}(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(format!("{{other:?}}"))),
        }}
            
    }}
"#)
                    };
                    generated_methods.push_str(&activity_func);
                } else {
                    let func = format!(
                    r#"
    pub async fn {fn_name}(&self, request: coordinator::{req_type}) -> Result<coordinator::{res_type}, TurnkeyClientError> {{
        let url = format!("{{}}{{}}", self.base_url, "{route}");
        let post_body = serde_json::to_string(&request).unwrap();
        let stamp = stamp(post_body.clone(), &self.api_key).unwrap();

        let res = self.http
            .post(url)
            .header("X-Stamp", stamp)
            .body(post_body)
            .send()
            .await?;
        let parsed = res.json::<coordinator::{res_type}>().await?;
        Ok(parsed)
    }}
"#);
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
    .open(out_dir.join("mod.rs")).unwrap();

    // This will append a new line to the end of the file
    writeln!(mod_rs, "\n// Added by tkhq_codegen").unwrap();
    writeln!(mod_rs, "mod client;").unwrap();
    writeln!(mod_rs, "pub use client::*;").unwrap();
}

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
