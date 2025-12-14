//! Implementation of the #[weld_struct] and #[weld_enum] macros

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Fields, GenericArgument, ItemEnum, ItemStruct, PathArguments, Type};

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
                    let path_str = segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    if path_str.contains("serde_json") || path_str == "Value" {
                        return quote! { forge_weld::WeldType::JsonValue };
                    }
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
    let field_tokens: Vec<_> = match &input.fields {
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
                    let type_tokens = rust_type_to_weld_type(ty);

                    Some(quote! {
                        forge_weld::StructField {
                            rust_name: #name_str.to_string(),
                            ts_name: #ts_field_name.to_string(),
                            ty: #type_tokens,
                            optional: #is_optional,
                            readonly: false,
                            doc: None,
                        }
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    };

    // Generate metadata function name
    let metadata_fn_name = format_ident!("__{}_weld_metadata", struct_name);

    let expanded = quote! {
        #input

        #[doc(hidden)]
        fn #metadata_fn_name() -> forge_weld::WeldStruct {
            forge_weld::WeldStruct {
                rust_name: #struct_name_str.to_string(),
                ts_name: #ts_name.to_string(),
                fields: vec![#(#field_tokens),*],
                doc: None,
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
    let variant_tokens: Vec<_> = input
        .variants
        .iter()
        .map(|v| {
            let name = v.ident.to_string();

            // Extract variant fields
            let field_tokens: Vec<_> = match &v.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .filter_map(|f| {
                        let field_name = f.ident.as_ref()?;
                        let field_name_str = field_name.to_string();
                        let ts_field_name = to_camel_case(&field_name_str);
                        let type_tokens = rust_type_to_weld_type(&f.ty);

                        Some(quote! {
                            forge_weld::StructField {
                                rust_name: #field_name_str.to_string(),
                                ts_name: #ts_field_name.to_string(),
                                ty: #type_tokens,
                                optional: false,
                                readonly: false,
                                doc: None,
                            }
                        })
                    })
                    .collect(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let field_name = format!("field{}", i);
                        let type_tokens = rust_type_to_weld_type(&f.ty);
                        quote! {
                            forge_weld::StructField {
                                rust_name: #field_name.to_string(),
                                ts_name: #field_name.to_string(),
                                ty: #type_tokens,
                                optional: false,
                                readonly: false,
                                doc: None,
                            }
                        }
                    })
                    .collect(),
                Fields::Unit => Vec::new(),
            };

            quote! {
                forge_weld::EnumVariant {
                    name: #name.to_string(),
                    fields: vec![#(#field_tokens),*],
                    doc: None,
                }
            }
        })
        .collect();

    // Generate metadata function name
    let metadata_fn_name = format_ident!("__{}_weld_metadata", enum_name);

    let expanded = quote! {
        #input

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
