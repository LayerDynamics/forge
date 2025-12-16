//! Implementation of the #[weld_struct] and #[weld_enum] macros

use crate::type_parser::rust_type_to_weld_type;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Fields, ItemEnum, ItemStruct};

/// Parse weld_struct attributes
struct WeldStructAttrs {
    ts_name: Option<String>,
}

impl WeldStructAttrs {
    fn parse(attr: TokenStream) -> Self {
        let mut attrs = WeldStructAttrs { ts_name: None };

        if attr.is_empty() {
            return attrs;
        }

        let attr_str = attr.to_string();
        for part in attr_str.split(',') {
            let part = part.trim();
            if part.starts_with("ts_name") {
                if let Some(eq_pos) = part.find('=') {
                    let name = part[eq_pos + 1..].trim().trim_matches('"');
                    attrs.ts_name = Some(name.to_string());
                }
            }
        }

        attrs
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert PascalCase to camelCase (first letter lowercase)
fn to_camel_case_first_lower(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub fn weld_struct_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemStruct = match parse2(item.clone()) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let attrs = WeldStructAttrs::parse(attr);

    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // TypeScript name (same as Rust name by default for structs)
    let ts_name = attrs.ts_name.unwrap_or_else(|| struct_name_str.clone());

    // Extract fields
    let field_results: Vec<_> = match &input.fields {
        Fields::Named(fields) => {
            fields
                .named
                .iter()
                .filter_map(|f| {
                    let name = f.ident.as_ref()?;
                    let name_str = name.to_string();
                    let ts_field_name = to_camel_case(&name_str);
                    let ty = &f.ty;

                    // Check for Option<T> to determine if optional
                    let is_optional = quote!(#ty).to_string().starts_with("Option");

                    // Parse the type
                    let type_tokens = match rust_type_to_weld_type(ty) {
                        Ok(tokens) => tokens,
                        Err(e) => return Some(Err(e)),
                    };

                    Some(Ok(quote! {
                        forge_weld::StructField {
                            rust_name: #name_str.to_string(),
                            ts_name: #ts_field_name.to_string(),
                            ty: #type_tokens,
                            optional: #is_optional,
                            readonly: false,
                            doc: None,
                        }
                    }))
                })
                .collect()
        }
        _ => Vec::new(),
    };

    // Check for errors in field parsing
    let field_tokens: Vec<_> = {
        let mut tokens = Vec::new();
        for result in field_results {
            match result {
                Ok(t) => tokens.push(t),
                Err(e) => return e.to_compile_error(),
            }
        }
        tokens
    };

    // Generate metadata function name (snake_case to avoid warnings)
    let snake_case_name = to_snake_case(&struct_name_str);
    let metadata_fn_name = format_ident!("__{}_weld_metadata", snake_case_name);

    let mut stripped = input.clone();
    stripped
        .attrs
        .retain(|a| !a.path().is_ident("weld_struct") && !a.path().is_ident("weld_enum"));

    let expanded = quote! {
        #stripped

        #[doc(hidden)]
        fn #metadata_fn_name() -> forge_weld::WeldStruct {
            forge_weld::WeldStruct {
                rust_name: #struct_name_str.to_string(),
                ts_name: #ts_name.to_string(),
                fields: vec![#(#field_tokens),*],
                doc: None,
                is_type_alias: false,
                type_params: Vec::new(),
            }
        }

        forge_weld::register_struct!(#metadata_fn_name());
    };

    expanded
}

pub fn weld_enum_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemEnum = match parse2(item.clone()) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let attrs = WeldStructAttrs::parse(attr);

    let enum_name = &input.ident;
    let enum_name_str = enum_name.to_string();

    // TypeScript name (same as Rust name by default)
    let ts_name = attrs.ts_name.unwrap_or_else(|| enum_name_str.clone());

    // Extract variants
    let variant_results: Result<Vec<_>, syn::Error> = input
        .variants
        .iter()
        .map(|v| {
            let name = v.ident.to_string();
            // Convert to camelCase for the value (matches serde rename_all = "camelCase")
            let value = to_camel_case_first_lower(&name);

            // Determine if variant has associated data
            let data_tokens = match &v.fields {
                Fields::Named(_) | Fields::Unnamed(_) => {
                    // For now, treat complex variants as having unknown data
                    // A more complete implementation would parse the field types
                    quote! { None }
                }
                Fields::Unit => quote! { None },
            };

            Ok(quote! {
                forge_weld::EnumVariant {
                    name: #name.to_string(),
                    value: Some(#value.to_string()),
                    data: #data_tokens,
                    doc: None,
                }
            })
        })
        .collect();

    let variant_tokens = match variant_results {
        Ok(tokens) => tokens,
        Err(e) => return e.to_compile_error(),
    };

    // Generate metadata function name (snake_case to avoid warnings)
    let snake_case_name = to_snake_case(&enum_name_str);
    let metadata_fn_name = format_ident!("__{}_weld_metadata", snake_case_name);

    let mut stripped = input.clone();
    stripped
        .attrs
        .retain(|a| !a.path().is_ident("weld_struct") && !a.path().is_ident("weld_enum"));

    let expanded = quote! {
        #stripped

        #[doc(hidden)]
        fn #metadata_fn_name() -> forge_weld::WeldEnum {
            forge_weld::WeldEnum {
                rust_name: #enum_name_str.to_string(),
                ts_name: #ts_name.to_string(),
                variants: vec![#(#variant_tokens),*],
                doc: None,
            }
        }

        forge_weld::register_enum!(#metadata_fn_name());
    };

    expanded
}
