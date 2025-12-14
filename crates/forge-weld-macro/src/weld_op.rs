//! Implementation of the #[weld_op] macro

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse2, FnArg, ItemFn, Pat, ReturnType,
    punctuated::Punctuated, Token,
};

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
fn extract_params(inputs: &Punctuated<FnArg, Token![,]>) -> Vec<(String, String, bool)> {
    let mut params = Vec::new();

    for arg in inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Get parameter name
            let name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.to_string()
            } else {
                continue;
            };

            // Get type as string
            let ty = quote!(#(pat_type.ty)).to_string();

            // Check for #[state] or other attributes
            let is_state = pat_type.attrs.iter().any(|attr| {
                attr.path().is_ident("state")
            });

            // Skip OpState parameters
            if ty.contains("OpState") || is_state {
                continue;
            }

            params.push((name, ty, false));
        }
    }

    params
}

/// Extract return type
fn extract_return_type(output: &ReturnType) -> String {
    match output {
        ReturnType::Default => "()".to_string(),
        ReturnType::Type(_, ty) => quote!(#ty).to_string(),
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

    // Extract return type (currently unused but kept for future type inference)
    let _return_type = extract_return_type(&input.sig.output);

    // Generate metadata function name
    let metadata_fn_name = format_ident!("__{}_weld_metadata", fn_name);

    // Generate parameter metadata
    let param_tokens: Vec<_> = params
        .iter()
        .map(|(name, _ty, optional)| {
            let ts_param_name = to_camel_case(name);
            quote! {
                forge_weld::OpParam {
                    rust_name: #name.to_string(),
                    ts_name: #ts_param_name.to_string(),
                    ty: forge_weld::WeldType::Unknown, // TODO: Parse type properly
                    attr: None,
                    optional: #optional,
                    doc: None,
                }
            }
        })
        .collect();

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
                return_type: forge_weld::WeldType::Unknown, // TODO: Parse type properly
                doc: None,
                op2_attrs: None,
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
