use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use syn::{parse::Parser, punctuated::Punctuated, Token};
use walkdir::WalkDir;

const PUBLIC_API_PROTO_PATH: &str = "proto/services/coordinator/public/v1/public_api.proto";
const INCLUDE_PROTO_PATH: &str = "proto";
const GENERATED_CLIENT_DIR: &str = "client/src/generated";
const CLIENT_TEMPLATE_PATH: &str = "codegen/src/client.template.rs";
const ACTIVITIES_MAPPING_PATH: &str = "codegen/src/activities.json";

// Necessary derive to parse from and serialize to JSON
const SERDE_DERIVE: &str = "#[derive(::serde::Serialize, ::serde::Deserialize)]";
const DEBUG_TRAIT: &str = "#[derive(Debug)]";
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
        .type_attribute(".google.rpc", DEBUG_TRAIT)
        .field_attribute("google.rpc.Status.details", "#[serde(skip)]")
        .type_attribute("google.rpc", SERDE_CAMEL_CASE)
        .compile_protos(&[PUBLIC_API_PROTO_PATH], &[INCLUDE_PROTO_PATH])
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
    let activities_mapping_data =
        fs::read_to_string(ACTIVITIES_MAPPING_PATH).expect("cannot read activities.json");
    let parsed_activities: ActivitiesFile =
        serde_json::from_str(&activities_mapping_data).expect("cannot parse activities.json");

    for service_caps in service_re.captures_iter(&proto) {
        let service_body = &service_caps[2];

        for rpc_caps in rpc_re.captures_iter(service_body) {
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
            if http_opts
                .contains("option (google.api.method_visibility).restriction = \"INTERNAL\"")
            {
                // Skip internal-only endpoints
                continue;
            }

            let http_re = Regex::new(r#"post\s*:\s*\"([^\"]+)\""#).unwrap();
            if let Some(http_caps) = http_re.captures(http_opts) {
                let route = &http_caps[1];

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
                            pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<external_activity::Activity, TurnkeyClientError> {{
                                let url = format!("{{}}{{}}", self.base_url, "{route}");

                                let request = external_activity::{short_req_type} {{
                                    r#type: "{activity_type}".to_string(),
                                    timestamp_ms: timestamp_ms.to_string(),
                                    parameters: Some(params),
                                    organization_id,
                                }};

                                let post_body = serde_json::to_string(&request).unwrap();

                                let stamp = stamp(post_body.clone(), &self.api_key).unwrap();

                                self.process_activity(url, stamp, post_body).await
                            }}
                        "#
                        )
                    } else {
                        format!(
                            r#"
                            pub async fn {fn_name}(&self, organization_id: String, timestamp_ms: u128, params: immutable_activity::{activity_intent}) -> Result<immutable_activity::{activity_result}, TurnkeyClientError> {{
                                let url = format!("{{}}{{}}", self.base_url, "{route}");

                                let request = external_activity::{short_req_type} {{
                                    r#type: "{activity_type}".to_string(),
                                    timestamp_ms: timestamp_ms.to_string(),
                                    parameters: Some(params),
                                    organization_id,
                                }};

                                let post_body = serde_json::to_string(&request).unwrap();

                                let stamp = stamp(post_body.clone(), &self.api_key).unwrap();

                                let activity = self.process_activity(url, stamp, post_body).await?;

                                let inner = activity
                                    .result.ok_or_else(|| TurnkeyClientError::MissingResult)?
                                    .inner.ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;

                                match inner {{
                                    immutable_activity::result::Inner::{activity_result}(res) => Ok(res),
                                    other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(serde_json::to_string(&other)?))
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

    // This will append a new line to the end of the file
    writeln!(mod_rs, "\n// Added by tkhq_codegen").unwrap();
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
            e.path().extension().map_or(false, |ext| ext == "rs")
                && !e.file_name().to_string_lossy().contains("google.api")
                && !e
                    .file_name()
                    .to_string_lossy()
                    .contains("grpc.gateway.protoc_gen_openapiv2")
        })
    {
        let path = entry.path();
        println!("Post-Processing {}", path.display());

        let content = fs::read_to_string(path).expect("cannot read file");
        let new_content = transform(&content);
        if new_content != content {
            fs::write(path, new_content).expect("cannot write to file");
        }
    }
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

fn transform(content: &str) -> String {
    let mut syntax_tree = syn::parse_file(content).expect("Unable to parse file");

    for item in syntax_tree.items.iter_mut() {
        transform_item(item);
    }

    prettyplease::unparse(&syntax_tree)
}

fn transform_item(item: &mut syn::Item) {
    match item {
        syn::Item::Struct(s) => {
            let replacement = mutate_struct(s);
            *item = syn::parse2(replacement).expect("failed to reparse struct");
        }
        syn::Item::Enum(e) => {
            let replacement = mutate_enum(e);
            *item = syn::parse2(replacement).expect("failed to reparse enum");
        }
        syn::Item::Mod(m) => {
            // Recurse into inline modules
            if let Some((_, ref mut items)) = &mut m.content {
                for item in items.iter_mut() {
                    transform_item(item);
                }
            }
        }
        _ => {}
    }
}

fn mutate_enum(enum_item: &syn::ItemEnum) -> TokenStream {
    let ident = &enum_item.ident;
    let vis = &enum_item.vis;
    let generics = &enum_item.generics;

    // Filter attributes: remove #[derive(...)] prost traits and all #[prost(...)] attrs
    let mut attrs: Vec<_> = enum_item
        .attrs
        .iter()
        .filter_map(|attr| {
            // Remove #[repr(i32)]
            if attr.path().is_ident("repr") {
                if let syn::Meta::List(meta_list) = &attr.meta {
                    if meta_list.tokens.to_string().trim() == "i32" {
                        return None;
                    }
                }
            }

            // Also strip prost derive traits like Message, Oneof, Enumeration
            strip_prost_derive_from_attr(attr)
        })
        .collect();

    // Remove any existing #[serde(rename_all = "...")]
    attrs.retain(|attr| {
        !attr.to_string().contains("serde") || !attr.to_string().contains("rename_all")
    });

    let enum_name_upper = ident.to_string().to_shouty_snake_case();

    let variants = enum_item.variants.iter().map(|v| {
        let mut v = v.clone();
        v.attrs.retain(|attr| !attr.path().is_ident("prost"));

        // Add #[serde(rename = "ENUM_NAME_VARIANT")]
        let variant_name = &v.ident;
        let full_name = format!(
            "{}_{}",
            enum_name_upper,
            variant_name.to_string().to_shouty_snake_case()
        );

        // A bit of a special case, but it doesn't make sense to have these renames for our result::Inner and intent::Inner
        // enums, which are complex enums anyway.
        // TODO: would be nice to filter this in a more generic way: basically if the enum isn't a "simple" enum, we shouldn't
        // have to individually rename the variants
        if ident != "Inner" {
            let rename_attr: syn::Attribute = syn::parse_quote!(
                #[serde(rename = #full_name)]
            );
            v.attrs.push(rename_attr);
        }

        quote! { #v }
    });

    // Another quick special case: Result::Inner and Intent::Inner require #[serde(rename_all = "camelCase")]
    if ident == "Inner" {
        attrs.push(quote! {
            #[serde(rename_all = "camelCase")]
        });
    }

    quote! {
        #(#attrs)*
        #vis enum #ident #generics {
            #(#variants),*
        }
    }
}

fn mutate_struct(struct_value: &syn::ItemStruct) -> TokenStream {
    let struct_ident = &struct_value.ident;
    let vis = &struct_value.vis;
    let generics = &struct_value.generics;
    let struct_attrs: Vec<TokenStream> = struct_value
        .attrs
        .iter()
        .filter_map(strip_prost_derive_from_attr)
        .collect();

    let fields = struct_value.fields.iter().map(|field| {
        let mut field = field.clone(); // Make a copy so we can mutate

        if let Some(enum_type) = extract_field_enum_type(&field) {
            let enum_ident = &enum_type.name;

            let rewritten_ty: syn::Type = if enum_type.repeated {
                syn::parse_str(&format!("Vec<{}>", enum_ident)).unwrap()
            } else if enum_type.optional {
                syn::parse_str(&format!("Option<{}>", enum_ident)).unwrap()
            } else {
                syn::parse_str(enum_ident).unwrap()
            };

            field.ty = rewritten_ty;
        }

        // Remove all #[prost(...)] attributes from the field
        // This is needed because Prost messages cannot have proper enums (types) and be compatible with prost::Message
        field.attrs.retain(|attr| !attr.path().is_ident("prost"));

        // Add "default" to fields which would benefit from it
        if should_add_default(&field.ty) {
            field.attrs.push(syn::parse_quote!(
                #[serde(default)]
            ));
        }

        // Flatten out Result::inner and Intent::inner since we have no "inner" key in the JSON responses we return!
        if field.ident.as_ref().map(|i| i == "inner").unwrap_or(false) {
            field.attrs.push(syn::parse_quote!(
                #[serde(flatten)]
            ));
        }

        quote! { #field }
    });

    quote! {
        #(#struct_attrs)*
        #vis struct #struct_ident #generics {
            #(#fields,)*
        }
    }
}
// This removes the struct's ::prost::Message derive:
// (in e.g. #[derive(Clone, PartialEq, ::prost::Message)])
fn strip_prost_derive_from_attr(attr: &syn::Attribute) -> Option<proc_macro2::TokenStream> {
    if !attr.path().is_ident("derive") {
        return Some(attr.to_token_stream());
    }

    let parsed = attr.parse_args_with(
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
    );
    let Ok(derives) = parsed else {
        return Some(attr.to_token_stream());
    };

    // Keep everything that is not Message or ::prost::Message
    let new_derives = derives
        .into_iter()
        .filter(|path| {
            !path.is_ident("Message")
                && !path.is_ident("Oneof")
                && !path.is_ident("Enumeration")
                && !path
                    .segments
                    .iter()
                    .any(|s| s.ident == "Message" || s.ident == "Oneof" || s.ident == "Enumeration")
        })
        .collect::<Punctuated<syn::Path, Token![,]>>();

    if new_derives.is_empty() {
        return None;
    }

    Some(quote! {
        #[derive(#new_derives)]
    })
}

struct EnumField {
    /// Fully qualified type name, e.g., "super::my::proto::EnumName"
    name: String,
    /// Is this enum a Vec<...>
    repeated: bool,
    /// Is this enum an Option<...>
    optional: bool,
}

fn extract_field_enum_type(field: &syn::Field) -> Option<EnumField> {
    let mut name = None;
    let mut repeated = false;
    let mut optional = false;

    for attr in &field.attrs {
        let meta = &attr.meta;
        let syn::Meta::List(meta_list) = meta else {
            continue;
        };
        if !meta_list.path.is_ident("prost") {
            continue;
        }

        // Extract the token stream inside the parentheses of #[prost(...)]
        let parser = Punctuated::<syn::Meta, Token![,]>::parse_terminated;
        let metas = match parser.parse2(meta_list.tokens.clone()) {
            Ok(m) => m,
            Err(_) => continue,
        };

        for meta in metas {
            match meta {
                syn::Meta::NameValue(nv) => {
                    if nv.path.is_ident("enumeration") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &nv.value
                        {
                            name = Some(lit_str.value());
                        }
                    }
                }
                syn::Meta::Path(p) => {
                    if p.is_ident("repeated") {
                        repeated = true;
                    } else if p.is_ident("optional") {
                        optional = true;
                    }
                }
                _ => {}
            }
        }
    }

    name.map(|name| EnumField {
        name,
        repeated,
        optional,
    })
}

fn should_add_default(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            let ident = type_path.path.segments.last().map(|s| s.ident.to_string());

            matches!(
                ident.as_deref(),
                Some("Option")
                    | Some("Vec")
                    | Some("bool")
                    | Some("u32")
                    | Some("i32")
                    | Some("u64")
                    | Some("i64")
                    | Some("HashMap")
                    | Some("BTreeMap")
            )
        }
        _ => false,
    }
}
