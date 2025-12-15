//! Type parser for converting Rust types to WeldType token streams
//!
//! This module provides strict type parsing for the weld macros.
//! It panics on unparseable types to ensure no `unknown` types slip through.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericArgument, PathArguments, Type};

/// Parse a Rust type into a WeldType token stream.
///
/// # Strict Mode
/// This function will panic if a type cannot be parsed. This ensures
/// that all types used with weld macros are properly handled and no
/// `unknown` types appear in the generated TypeScript.
///
/// # Supported Types
/// - Primitives: u8-u64, i8-i64, f32, f64, bool, String, char, ()
/// - Containers: Option<T>, Vec<T>, Result<T, E>, HashMap<K, V>, BTreeMap<K, V>
/// - Sets: HashSet<T>, BTreeSet<T>
/// - Wrappers: Box<T>, Arc<T>, Rc<T>, RefCell<T>, Mutex<T>, RwLock<T>
/// - References: &T, &mut T
/// - Tuples: (A, B, C)
/// - Arrays/Slices: [T; N], [T]
/// - Special: serde_json::Value (JsonValue), OpState
/// - Custom types: Treated as Struct references
pub fn rust_type_to_weld_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(type_path) => parse_path_type(type_path),
        Type::Reference(type_ref) => {
            let inner = rust_type_to_weld_type(&type_ref.elem);
            let mutable = type_ref.mutability.is_some();
            quote! {
                forge_weld::WeldType::Reference {
                    inner: Box::new(#inner),
                    mutable: #mutable,
                }
            }
        }
        Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Unit) }
            } else {
                let elems: Vec<_> = type_tuple
                    .elems
                    .iter()
                    .map(rust_type_to_weld_type)
                    .collect();
                quote! { forge_weld::WeldType::Tuple(vec![#(#elems),*]) }
            }
        }
        Type::Slice(type_slice) => {
            let inner = rust_type_to_weld_type(&type_slice.elem);
            quote! { forge_weld::WeldType::Vec(Box::new(#inner)) }
        }
        Type::Array(type_array) => {
            let inner = rust_type_to_weld_type(&type_array.elem);
            // Could extract size from type_array.len for Array variant,
            // but Vec is more common and works for TypeScript
            quote! { forge_weld::WeldType::Vec(Box::new(#inner)) }
        }
        Type::Never(_) => {
            quote! { forge_weld::WeldType::Never }
        }
        Type::Ptr(type_ptr) => {
            let inner = rust_type_to_weld_type(&type_ptr.elem);
            let mutable = type_ptr.mutability.is_some();
            quote! {
                forge_weld::WeldType::Pointer {
                    inner: Box::new(#inner),
                    mutable: #mutable,
                }
            }
        }
        Type::Paren(type_paren) => {
            // Parenthesized type like (T) - just unwrap
            rust_type_to_weld_type(&type_paren.elem)
        }
        Type::Group(type_group) => {
            // Group type (from macro expansion) - just unwrap
            rust_type_to_weld_type(&type_group.elem)
        }
        Type::BareFn(bare_fn) => {
            // Bare function type like fn(A) -> B
            // Treat as Unknown since TypeScript doesn't have direct equivalent
            // Actually per user request, we should panic on unparseable types
            panic!(
                "forge-weld: Bare function types are not supported: `{}`.\n\
                 Consider wrapping in a struct or using a different type.",
                quote!(#bare_fn)
            );
        }
        Type::ImplTrait(impl_trait) => {
            panic!(
                "forge-weld: `impl Trait` types are not supported: `{}`.\n\
                 Use concrete types instead.",
                quote!(#impl_trait)
            );
        }
        Type::TraitObject(trait_obj) => {
            panic!(
                "forge-weld: Trait object types (`dyn Trait`) are not supported: `{}`.\n\
                 Use concrete types instead.",
                quote!(#trait_obj)
            );
        }
        Type::Infer(_) => {
            panic!(
                "forge-weld: Inferred types (`_`) are not supported.\n\
                 Please specify the concrete type."
            );
        }
        Type::Macro(type_macro) => {
            panic!(
                "forge-weld: Macro types are not supported: `{}`.\n\
                 Expand the macro or use a concrete type.",
                quote!(#type_macro)
            );
        }
        Type::Verbatim(verbatim) => {
            panic!(
                "forge-weld: Verbatim type syntax not supported: `{}`.\n\
                 Please use standard Rust type syntax.",
                verbatim
            );
        }
        _ => {
            // Catch-all for any future syn::Type variants
            let ty_str = quote!(#ty).to_string();
            panic!(
                "forge-weld: Unsupported type: `{}`.\n\
                 This type cannot be mapped to TypeScript. \
                 Please use a supported type or wrap it in a struct.",
                ty_str
            );
        }
    }
}

/// Parse a Type::Path into WeldType tokens
fn parse_path_type(type_path: &syn::TypePath) -> TokenStream {
    let segments: Vec<_> = type_path.path.segments.iter().collect();

    if let Some(last_seg) = segments.last() {
        let ident = last_seg.ident.to_string();

        // Handle primitive types
        if let Some(primitive) = parse_primitive(&ident) {
            return primitive;
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

            if let Some(generic) = parse_generic_type(&ident, &inner_types) {
                return generic;
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

        // Handle OpState (internal Deno type)
        if ident == "OpState" {
            return quote! { forge_weld::WeldType::OpState };
        }

        // Default: treat as struct reference (custom types)
        // This is valid - custom structs/enums become Struct references
        return quote! { forge_weld::WeldType::Struct(#ident.to_string()) };
    }

    // Empty path - shouldn't happen but handle gracefully
    panic!(
        "forge-weld: Empty type path encountered. This is likely a bug.\n\
         Type: `{}`",
        quote!(#type_path)
    );
}

/// Parse a primitive type name into WeldType tokens
fn parse_primitive(ident: &str) -> Option<TokenStream> {
    let tokens = match ident {
        "u8" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U8) },
        "u16" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U16) },
        "u32" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U32) },
        "u64" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::U64) },
        "usize" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Usize) },
        "i8" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I8) },
        "i16" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I16) },
        "i32" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I32) },
        "i64" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::I64) },
        "isize" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Isize) },
        "f32" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::F32) },
        "f64" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::F64) },
        "bool" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Bool) },
        "String" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::String) },
        "str" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Str) },
        "char" => quote! { forge_weld::WeldType::Primitive(forge_weld::WeldPrimitive::Char) },
        _ => return None,
    };
    Some(tokens)
}

/// Parse a generic type with its inner types
fn parse_generic_type(ident: &str, inner_types: &[TokenStream]) -> Option<TokenStream> {
    match ident {
        "Option" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Option(Box::new(#inner)) })
        }
        "Vec" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Vec(Box::new(#inner)) })
        }
        "Result" => {
            if inner_types.len() >= 2 {
                let ok_type = &inner_types[0];
                let err_type = &inner_types[1];
                Some(quote! {
                    forge_weld::WeldType::Result {
                        ok: Box::new(#ok_type),
                        err: Box::new(#err_type),
                    }
                })
            } else if inner_types.len() == 1 {
                // Result<T> with implicit error type - use the error type as Struct
                let ok_type = &inner_types[0];
                Some(quote! {
                    forge_weld::WeldType::Result {
                        ok: Box::new(#ok_type),
                        err: Box::new(forge_weld::WeldType::Struct("Error".to_string())),
                    }
                })
            } else {
                None
            }
        }
        "HashMap" => {
            if inner_types.len() >= 2 {
                let key_type = &inner_types[0];
                let val_type = &inner_types[1];
                Some(quote! {
                    forge_weld::WeldType::HashMap {
                        key: Box::new(#key_type),
                        value: Box::new(#val_type),
                    }
                })
            } else {
                None
            }
        }
        "BTreeMap" => {
            if inner_types.len() >= 2 {
                let key_type = &inner_types[0];
                let val_type = &inner_types[1];
                Some(quote! {
                    forge_weld::WeldType::BTreeMap {
                        key: Box::new(#key_type),
                        value: Box::new(#val_type),
                    }
                })
            } else {
                None
            }
        }
        "HashSet" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::HashSet(Box::new(#inner)) })
        }
        "BTreeSet" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::BTreeSet(Box::new(#inner)) })
        }
        "Box" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Box(Box::new(#inner)) })
        }
        "Arc" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Arc(Box::new(#inner)) })
        }
        "Rc" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Rc(Box::new(#inner)) })
        }
        "RefCell" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::RefCell(Box::new(#inner)) })
        }
        "Mutex" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::Mutex(Box::new(#inner)) })
        }
        "RwLock" => {
            let inner = inner_types.first()?;
            Some(quote! { forge_weld::WeldType::RwLock(Box::new(#inner)) })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_primitive_types() {
        let ty: Type = parse_quote!(String);
        let tokens = rust_type_to_weld_type(&ty);
        let expected = "forge_weld :: WeldType :: Primitive (forge_weld :: WeldPrimitive :: String)";
        assert_eq!(tokens.to_string().replace(" ", ""), expected.replace(" ", ""));

        let ty: Type = parse_quote!(u32);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("U32"));

        let ty: Type = parse_quote!(bool);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Bool"));
    }

    #[test]
    fn test_option_type() {
        let ty: Type = parse_quote!(Option<String>);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Option"));
        assert!(tokens.to_string().contains("String"));
    }

    #[test]
    fn test_vec_type() {
        let ty: Type = parse_quote!(Vec<u8>);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Vec"));
        assert!(tokens.to_string().contains("U8"));
    }

    #[test]
    fn test_result_type() {
        let ty: Type = parse_quote!(Result<String, Error>);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Result"));
    }

    #[test]
    fn test_hashmap_type() {
        let ty: Type = parse_quote!(HashMap<String, u32>);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("HashMap"));
    }

    #[test]
    fn test_tuple_type() {
        let ty: Type = parse_quote!((String, u32, bool));
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Tuple"));
    }

    #[test]
    fn test_unit_type() {
        let ty: Type = parse_quote!(());
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Unit"));
    }

    #[test]
    fn test_reference_type() {
        let ty: Type = parse_quote!(&str);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Reference"));
    }

    #[test]
    fn test_custom_struct() {
        let ty: Type = parse_quote!(FileStat);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Struct"));
        assert!(tokens.to_string().contains("FileStat"));
    }

    #[test]
    fn test_nested_generics() {
        let ty: Type = parse_quote!(Option<Vec<String>>);
        let tokens = rust_type_to_weld_type(&ty);
        assert!(tokens.to_string().contains("Option"));
        assert!(tokens.to_string().contains("Vec"));
        assert!(tokens.to_string().contains("String"));
    }

    #[test]
    #[should_panic(expected = "impl Trait")]
    fn test_impl_trait_panics() {
        let ty: Type = parse_quote!(impl Iterator<Item = u32>);
        rust_type_to_weld_type(&ty);
    }
}
