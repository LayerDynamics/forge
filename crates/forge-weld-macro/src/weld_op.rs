//! Implementation of the #[weld_op] macro

use crate::type_parser::rust_type_to_weld_type;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, punctuated::Punctuated, FnArg, ItemFn, Pat, ReturnType, Token, Type};

/// Parse weld_op attributes
struct WeldOpAttrs {
    is_async: bool,
    ts_name: Option<String>,
}

impl WeldOpAttrs {
    fn parse(attr: TokenStream) -> Self {
        let mut attrs = WeldOpAttrs {
            is_async: false,
            ts_name: None,
        };

        if attr.is_empty() {
            return attrs;
        }

        // Parse attributes like: async, ts_name = "foo"
        let attr_str = attr.to_string();
        for part in attr_str.split(',') {
            let part = part.trim();
            if part == "async" {
                attrs.is_async = true;
            } else if part.starts_with("ts_name") {
                if let Some(eq_pos) = part.find('=') {
                    let name = part[eq_pos + 1..].trim().trim_matches('"');
                    attrs.ts_name = Some(name.to_string());
                }
            }
        }

        attrs
    }
}

/// Extract parameter info from function arguments
fn extract_params(inputs: &Punctuated<FnArg, Token![,]>) -> Vec<(String, Type, bool)> {
    let mut params = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Get parameter name
            let name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.to_string()
            } else {
                continue;
            };

            // Get type
            let ty = (*pat_type.ty).clone();
            let ty_str = quote!(#ty).to_string();

            // Check for #[state] or other attributes
            let is_state = pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("state"));

            // Skip OpState parameters
            if ty_str.contains("OpState") || is_state {
                continue;
            }

            params.push((name, ty, false));
        }
    }

    params
}

/// Extract return type as a Type
fn extract_return_type(output: &ReturnType) -> Option<Type> {
    match output {
        ReturnType::Default => None, // Unit type
        ReturnType::Type(_, ty) => Some((**ty).clone()),
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

/// Convert op name to TypeScript function name
fn op_name_to_ts(rust_name: &str) -> String {
    // Remove "op_" prefix and convert to camelCase
    let without_prefix = rust_name.strip_prefix("op_").unwrap_or(rust_name);

    // Handle module prefixes like "op_fs_read_text" -> "readText"
    let parts: Vec<&str> = without_prefix.splitn(2, '_').collect();
    if parts.len() == 2 {
        // Has module prefix, use the part after it
        to_camel_case(parts[1])
    } else {
        to_camel_case(without_prefix)
    }
}

pub fn weld_op_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = match parse2(item.clone()) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };

    let attrs = WeldOpAttrs::parse(attr);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Determine if async from the function signature or attribute
    let is_async = attrs.is_async || input.sig.asyncness.is_some();

    // Determine TypeScript name
    let ts_name = attrs.ts_name.unwrap_or_else(|| op_name_to_ts(&fn_name_str));

    // Extract parameters
    let params = extract_params(&input.sig.inputs);

    // Extract return type
    let return_type = extract_return_type(&input.sig.output);
    let return_type_tokens = match return_type {
        Some(ref ty) => match rust_type_to_weld_type(ty) {
            Ok(tokens) => tokens,
            Err(e) => return e.to_compile_error(),
        },
        None => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Unit) },
    };

    // Generate metadata function name
    let metadata_fn_name = format_ident!("__{}_weld_metadata", fn_name);

    // Generate parameter metadata with proper type inference
    let param_tokens: Result<Vec<_>, syn::Error> = params
        .iter()
        .map(|(name, ty, optional)| {
            let ts_param_name = to_camel_case(name);
            let type_tokens = rust_type_to_weld_type(ty)?;
            Ok(quote! {
                forge_weld::OpParam {
                    rust_name: #name.to_string(),
                    ts_name: #ts_param_name.to_string(),
                    ty: #type_tokens,
                    attr: forge_weld::ParamAttr::default(),
                    optional: #optional,
                    doc: None,
                }
            })
        })
        .collect();

    let param_tokens = match param_tokens {
        Ok(tokens) => tokens,
        Err(e) => return e.to_compile_error(),
    };

    // Generate the metadata function and registration
    let expanded = quote! {
        #input

        #[doc(hidden)]
        fn #metadata_fn_name() -> forge_weld::OpSymbol {
            forge_weld::OpSymbol {
                rust_name: #fn_name_str.to_string(),
                ts_name: #ts_name.to_string(),
                is_async: #is_async,
                params: vec![#(#param_tokens),*],
                return_type: #return_type_tokens,
                doc: None,
                op2_attrs: forge_weld::Op2Attrs::default(),
                module: None,
            }
        }

        forge_weld::register_op!(#metadata_fn_name());
    };

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_name_to_ts() {
        assert_eq!(op_name_to_ts("op_fs_read_text"), "readText");
        assert_eq!(op_name_to_ts("op_net_fetch"), "fetch");
        assert_eq!(op_name_to_ts("op_simple"), "simple");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("read_text"), "readText");
        assert_eq!(to_camel_case("simple"), "simple");
        assert_eq!(to_camel_case("read_text_file"), "readTextFile");
    }
}
