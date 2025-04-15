use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;

const PUBLIC_API_PROTO_PATH: &str = "proto/services/coordinator/public/v1/public_api.proto";
const INCLUDE_PROTO_PATH: &str = "proto";
const GENERATED_CLIENT_DIR: &str = "client/src/generated";
const CLIENT_TEMPLATE_PATH: &str = "codegen/src/client.template.rs";

fn main() {
    let out_dir = PathBuf::from(GENERATED_CLIENT_DIR);

    tonic_build::configure()
    .build_server(false)
    .build_client(false)
    .include_file("mod.rs")
    .out_dir(out_dir.clone())
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

    // Structures to track imports
    let mut coordinator_imports = HashSet::new();
    let mut activity_imports = HashSet::new();

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

                let sanitized_req_type = if req_type.contains("external.activity.v1") {
                    // If the request type is "external.activity.v1.DeletePolicyRequest" the sanitized
                    // request type will be "DeletePolicyRequest", and we'll need to import it from the external activity namespace
                     let short_req_type = req_type.rsplit(".").next().unwrap();
                     activity_imports.insert(short_req_type.into());
                     short_req_type
                } else {
                    // Otherwise we're fine, it'll be defined in the coordinator public API namespace.
                    coordinator_imports.insert(req_type.into());
                    req_type
                };

                let func = format!(
                    r#"pub async fn {fn_name}(client: &reqwest::Client, base_url: &str, request: {sanitized_req_type}, stamp: &str) -> Result<{res_type}, reqwest::Error> {{
    let url = format!("{{}}{{}}", base_url, "{route}");
    let res = client
        .post(url)
        .header("X-Stamp", stamp)
        .json(&request)
        .send()
        .await?;
    let parsed = res.json::<{res_type}>().await?;
    Ok(parsed)
}}
"#);
                generated_methods.push_str(&func);
                coordinator_imports.insert(res_type.into());
            }
        }
    }

    // Insert the generated methods into the right spot inside of our template
    let method_placeholder = "// <GENERATED_METHODS_PLACEHOLDER>";
    let client_template = fs::read_to_string(CLIENT_TEMPLATE_PATH).unwrap();
    let output = client_template.replace(method_placeholder, &generated_methods);

    // Populate imports for generic request/response types
    let import_placeholder = "COORDINATOR_SERVICE_IMPORT_PLACEHOLDER";
    let output = output.replace(import_placeholder, &coordinator_imports.into_iter().collect::<Vec<String>>().join(",\n  "));

    // Do the same for activity request types (which are in a different file)
    let import_placeholder = "ACTIVITY_IMPORT_PLACEHOLDER";
    let output = output.replace(import_placeholder, &activity_imports.into_iter().collect::<Vec<String>>().join(",\n  "));

    // Write the generated client to the `client` crate
    let generated_client_path = PathBuf::from(GENERATED_CLIENT_DIR).join("client.rs");
    fs::create_dir_all("../client/src/generated").unwrap();
    fs::write(generated_client_path, output).expect("Failed to write generated file");

    // Now add client.rs to the generated/mod.rs file
    let mut mod_rs = fs::OpenOptions::new()
    .append(true)
    .open(out_dir.join("mod.rs")).unwrap();

    // This will append a new line to the end of the file
    writeln!(mod_rs, "\n// This line was added by the codegen script").unwrap();
    writeln!(mod_rs, "pub mod client;").unwrap();
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
