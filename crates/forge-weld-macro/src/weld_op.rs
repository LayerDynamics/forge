//! Implementation of the #[weld_op] macro

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse2, punctuated::Punctuated, FnArg, GenericArgument, ItemFn, Pat, PathArguments, ReturnType,
    Token, Type,
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

/// Parse a Rust type into a WeldType token stream
fn rust_type_to_weld_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<_> = type_path.path.segments.iter().collect();
            if let Some(last_seg) = segments.last() {
                let ident = last_seg.ident.to_string();

                // Handle primitive types
                match ident.as_str() {
                    "u8" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U8) }
                    }
                    "u16" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U16) }
                    }
                    "u32" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U32) }
                    }
                    "u64" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U64) }
                    }
                    "usize" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Usize) }
                    }
                    "i8" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I8) }
                    }
                    "i16" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I16) }
                    }
                    "i32" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I32) }
                    }
                    "i64" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I64) }
                    }
                    "isize" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Isize) }
                    }
                    "f32" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::F32) }
                    }
                    "f64" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::F64) }
                    }
                    "bool" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Bool) }
                    }
                    "String" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::String) }
                    }
                    "char" => {
                        return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Char) }
                    }
                    _ => {}
                }

                // Handle generic types
                if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                    let inner_types: Vec<_> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let GenericArgument::Type(inner_ty) = arg {
                                Some(rust_type_to_weld_type(inner_ty))
                            } else {
                                None
                            }
                        })
                        .collect();

                    match ident.as_str() {
                        "Option" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::Option(Box::new(#inner)) };
                            }
                        }
                        "Vec" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::Vec(Box::new(#inner)) };
                            }
                        }
                        "Result" => {
                            if inner_types.len() >= 2 {
                                let ok_type = &inner_types[0];
                                let err_type = &inner_types[1];
                                return quote! {
                                    forge_weld::WeldType::Result {
                                        ok: Box::new(#ok_type),
                                        err: Box::new(#err_type),
                                    }
                                };
                            } else if inner_types.len() == 1 {
                                let ok_type = &inner_types[0];
                                return quote! {
                                    forge_weld::WeldType::Result {
                                        ok: Box::new(#ok_type),
                                        err: Box::new(forge_weld::WeldType::Unknown),
                                    }
                                };
                            }
                        }
                        "HashMap" | "BTreeMap" => {
                            if inner_types.len() >= 2 {
                                let key_type = &inner_types[0];
                                let val_type = &inner_types[1];
                                return quote! {
                                    forge_weld::WeldType::HashMap {
                                        key: Box::new(#key_type),
                                        value: Box::new(#val_type),
                                    }
                                };
                            }
                        }
                        "HashSet" | "BTreeSet" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::HashSet(Box::new(#inner)) };
                            }
                        }
                        "Box" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::Box(Box::new(#inner)) };
                            }
                        }
                        "Arc" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::Arc(Box::new(#inner)) };
                            }
                        }
                        "Rc" => {
                            if let Some(inner) = inner_types.first() {
                                return quote! { forge_weld::WeldType::Rc(Box::new(#inner)) };
                            }
                        }
                        _ => {}
                    }
                }

                // Handle serde_json::Value
                if ident == "Value" {
                    // Check if this is serde_json::Value
                    let path_str = segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    if path_str.contains("serde_json") || path_str == "Value" {
                        return quote! { forge_weld::WeldType::JsonValue };
                    }
                }

                // Handle OpState
                if ident == "OpState" {
                    return quote! { forge_weld::WeldType::OpState };
                }

                // Default: treat as struct reference
                return quote! { forge_weld::WeldType::Struct(#ident.to_string()) };
            }
        }
        Type::Reference(type_ref) => {
            let inner = rust_type_to_weld_type(&type_ref.elem);
            let mutable = type_ref.mutability.is_some();
            return quote! {
                forge_weld::WeldType::Reference {
                    inner: Box::new(#inner),
                    mutable: #mutable,
                }
            };
        }
        Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                return quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Unit) };
            }
            let elems: Vec<_> = type_tuple
                .elems
                .iter()
                .map(rust_type_to_weld_type)
                .collect();
            return quote! { forge_weld::WeldType::Tuple(vec![#(#elems),*]) };
        }
        Type::Slice(type_slice) => {
            let inner = rust_type_to_weld_type(&type_slice.elem);
            return quote! { forge_weld::WeldType::Vec(Box::new(#inner)) };
        }
        Type::Array(type_array) => {
            let inner = rust_type_to_weld_type(&type_array.elem);
            return quote! { forge_weld::WeldType::Vec(Box::new(#inner)) };
        }
        Type::Never(_) => {
            return quote! { forge_weld::WeldType::Never };
        }
        _ => {}
    }

    // Fallback
    quote! { forge_weld::WeldType::Unknown }
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
        Some(ref ty) => rust_type_to_weld_type(ty),
        None => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Unit) },
    };

    // Generate metadata function name
    let metadata_fn_name = format_ident!("__{}_weld_metadata", fn_name);

    // Generate parameter metadata with proper type inference
    let param_tokens: Vec<_> = params
        .iter()
        .map(|(name, ty, optional)| {
            let ts_param_name = to_camel_case(name);
            let type_tokens = rust_type_to_weld_type(ty);
            quote! {
                forge_weld::OpParam {
                    rust_name: #name.to_string(),
                    ts_name: #ts_param_name.to_string(),
                    ty: #type_tokens,
                    attr: forge_weld::ParamAttr::default(),
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
