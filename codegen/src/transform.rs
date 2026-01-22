use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parser, punctuated::Punctuated, Field, GenericArgument, PathArguments, Token, Type,
    TypePath,
};

/// Top-level function to apply transformations to generated code.
/// This function simply parses the content of a single file, applies transformations to each parsed item,
/// and returns the result. File mutation operations happen in `main.rs`. We only deal with strings here.
pub(crate) fn transform(content: &str) -> String {
    let mut syntax_tree = syn::parse_file(content).expect("Unable to parse file");

    for item in syntax_tree.items.iter_mut() {
        transform_item(item);
    }

    prettyplease::unparse(&syntax_tree)
}

/// Recursive function to transform structs and enums.
/// It needs to be recursive because of nested modules, which contain more enums and structs.
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

/// Function to mutate `enum`s.
/// The main things to fix here are:
/// - rename variants to UPPER_SNAKE_CASE so that serde knows how to parse incoming JSON
///   (for example: `"activityType": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD"`)
/// - remove prost annotations, traits and attributes given we don't need them for JSON serialization
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
    // We also derive "Debug", it's very useful to be able to serialize activities in a pinch.
    if ident == "Inner" {
        attrs.push(quote! {
            #[serde(rename_all = "camelCase")]
            #[derive(Debug)]
        });
    }

    // Add debug derive to enums that don't already have it to ensure compatibility with structs that derive Debug
    let has_debug = attrs
        .iter()
        .any(|attr| attr.to_string().contains("derive") && attr.to_string().contains("Debug"));

    if !has_debug {
        attrs.push(quote! {
            #[derive(Debug)]
        });
    }

    quote! {
        #(#attrs)*
        #vis enum #ident #generics {
            #(#variants),*
        }
    }
}

/// Function to mutate structs. The main things to do here:
/// - fix `enum` fields to have proper enum types instead of bare `i32`. We do this using the prost annotations, which contain this mapping
/// - use `#[serde(flatten)]` to ignore `Inner` structs at parsing time
/// - add `#[serde(default)]` to Options and Vecs to allow for deserialization of missing fields
/// - clean up prost derives and attributes
/// - use `serde_with` to  deserialize big ints from JSON string fields. This requires a field-level macro & struct-level attribute (`serde_as`).
fn mutate_struct(struct_value: &syn::ItemStruct) -> TokenStream {
    let struct_ident = &struct_value.ident;
    let vis = &struct_value.vis;
    let generics = &struct_value.generics;
    let mut struct_attrs: Vec<TokenStream> = struct_value
        .attrs
        .iter()
        .filter_map(strip_prost_derive_from_attr)
        .collect();

    let (fields, serde_as_added_bools): (Vec<TokenStream>, Vec<bool>) = struct_value
        .fields
        .iter()
        .map(|field| {
            let mut field = field.clone(); // Make a copy so we can mutate

            if let Some(enum_type) = extract_field_enum_type(&field) {
                let enum_ident = &enum_type.name;

                let rewritten_ty: syn::Type = if enum_type.repeated {
                    syn::parse_str(&format!("Vec<{enum_ident}>")).unwrap()
                } else if enum_type.optional {
                    syn::parse_str(&format!("Option<{enum_ident}>")).unwrap()
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

            // Add `serde_with::serde_as` attribute if needed
            let serde_as_added = add_serde_as_for_large_int(&mut field);

            (quote! { #field }, serde_as_added)
        })
        .collect();

    // If the struct fields has a `serde_as` added to it, add the struct-level `serde_as`
    if serde_as_added_bools.iter().any(|&b| b) {
        struct_attrs.insert(0, quote! {#[serde_with::serde_as]});
    }
    // Debug should always be present on struct, no excuse not to have it!
    struct_attrs.insert(0, quote! {#[derive(Debug)]});

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

// This modifies fields which are u64, i64, u128, i128 (or their option variants: Option<u64>, etc)
// and uses `serde_with::serde_as` so these big ints can be parsed from JSON strings.
// Returns a boolean to indicate whether the field was mutated or not.
pub fn add_serde_as_for_large_int(field: &mut Field) -> bool {
    // convenience function to match big ints
    fn is_large_int(ident: &syn::Ident) -> bool {
        matches!(ident.to_string().as_str(), "u64" | "i64" | "u128" | "i128")
    }

    match &field.ty {
        Type::Path(TypePath { qself: None, path }) => {
            let seg = match path.segments.last() {
                Some(s) => s,
                None => return false,
            };

            // Plain large int: path must be a single segment like `u64`
            if path.segments.len() == 1 && is_large_int(&seg.ident) {
                field
                    .attrs
                    .push(syn::parse_quote!(#[serde_as(as = "serde_with::DisplayFromStr")]));
                return true;
            }

            // Option<LargeInt>: last segment must be `Option<...>`
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(GenericArgument::Type(Type::Path(TypePath {
                        qself: None,
                        path: inner,
                    }))) = ab.args.first()
                    {
                        if inner.segments.len() == 1 && is_large_int(&inner.segments[0].ident) {
                            field.attrs.push(
                                syn::parse_quote!(#[serde_as(as = "Option<serde_with::DisplayFromStr>")])
                            );
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
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
