//! TypeScript init.ts module generator
//!
//! Generates the TypeScript source files that wrap Deno.core.ops calls.

use crate::ir::{OpSymbol, WeldEnum, WeldModule, WeldStruct, WeldType};

/// Generator for TypeScript init.ts modules
pub struct TypeScriptGenerator<'a> {
    module: &'a WeldModule,
}

impl<'a> TypeScriptGenerator<'a> {
    /// Create a new TypeScript generator for a module
    pub fn new(module: &'a WeldModule) -> Self {
        Self { module }
    }

    /// Generate the complete init.ts source
    pub fn generate(&self) -> String {
        let mut output = String::new();

        // Module header comment
        output.push_str(&format!(
            "// {} module - TypeScript wrapper for Deno core ops\n\n",
            self.module.specifier
        ));

        // Generate Deno.core.ops declaration
        output.push_str(&self.generate_deno_core_declaration());
        output.push('\n');

        // Generate interface declarations for structs
        for s in &self.module.structs {
            output.push_str(&self.generate_interface(s));
            output.push('\n');
        }

        // Generate enum declarations
        for e in &self.module.enums {
            output.push_str(&self.generate_enum(e));
            output.push('\n');
        }

        // Generate const core reference
        output.push_str("const core = Deno.core;\n\n");

        // Generate export functions for each op
        for op in &self.module.ops {
            output.push_str(&self.generate_export_function(op));
            output.push('\n');
        }

        output
    }

    /// Generate the Deno.core.ops type declaration block
    fn generate_deno_core_declaration(&self) -> String {
        self.module.deno_core_ops_declaration()
    }

    /// Generate a TypeScript interface from a WeldStruct
    fn generate_interface(&self, s: &WeldStruct) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(ref doc) = s.doc {
            output.push_str("/**\n");
            for line in doc.lines() {
                output.push_str(&format!(" * {}\n", line));
            }
            output.push_str(" */\n");
        }

        // Interface declaration with optional type parameters
        if s.type_params.is_empty() {
            output.push_str(&format!("interface {} {{\n", s.ts_name));
        } else {
            output.push_str(&format!(
                "interface {}<{}> {{\n",
                s.ts_name,
                s.type_params.join(", ")
            ));
        }

        // Fields
        for field in &s.fields {
            // Field doc comment
            if let Some(ref doc) = field.doc {
                output.push_str(&format!("  /** {} */\n", doc));
            }

            let readonly_prefix = if field.readonly { "readonly " } else { "" };
            let optional_suffix = if field.optional { "?" } else { "" };

            output.push_str(&format!(
                "  {}{}{}: {};\n",
                readonly_prefix,
                field.ts_name,
                optional_suffix,
                field.ty.to_typescript()
            ));
        }

        output.push_str("}\n");
        output
    }

    /// Generate a TypeScript enum or union type from a WeldEnum
    fn generate_enum(&self, e: &WeldEnum) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(ref doc) = e.doc {
            output.push_str("/**\n");
            for line in doc.lines() {
                output.push_str(&format!(" * {}\n", line));
            }
            output.push_str(" */\n");
        }

        // Check if it's a simple string enum (all unit variants - no data)
        let all_unit = e.variants.iter().all(|v| v.data.is_none());

        if all_unit {
            // Generate as string literal union type
            let variants: Vec<String> = e
                .variants
                .iter()
                .map(|v| {
                    let value = v.value.as_ref().unwrap_or(&v.name);
                    format!("\"{}\"", value)
                })
                .collect();
            output.push_str(&format!("type {} = {};\n", e.ts_name, variants.join(" | ")));
        } else {
            // Generate as discriminated union with interfaces
            let mut variant_types = Vec::new();

            for variant in &e.variants {
                let variant_type_name = format!("{}_{}", e.ts_name, variant.name);

                output.push_str(&format!("interface {} {{\n", variant_type_name));
                output.push_str(&format!("  type: \"{}\";\n", variant.name));

                if let Some(ref data) = variant.data {
                    output.push_str(&format!("  data: {};\n", data.to_typescript()));
                }

                output.push_str("}\n\n");
                variant_types.push(variant_type_name);
            }

            output.push_str(&format!(
                "type {} = {};\n",
                e.ts_name,
                variant_types.join(" | ")
            ));
        }

        output
    }

    /// Generate an export function that wraps a Deno op
    fn generate_export_function(&self, op: &OpSymbol) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(ref doc) = op.doc {
            output.push_str("/**\n");
            for line in doc.lines() {
                output.push_str(&format!(" * {}\n", line));
            }
            output.push_str(" */\n");
        }

        // Build parameter list for function signature
        let visible_params: Vec<_> = op.visible_params().collect();
        let params: Vec<String> = visible_params
            .iter()
            .map(|p| {
                let optional_mark = if p.optional { "?" } else { "" };
                format!("{}{}: {}", p.ts_name, optional_mark, p.ty.to_typescript())
            })
            .collect();

        // Build parameter names for the op call
        let param_names: Vec<String> = visible_params.iter().map(|p| p.ts_name.clone()).collect();

        // Determine return type
        let return_type = op.ts_return_type();

        // Generate function
        let ts_name = &op.ts_name;
        let async_keyword = if op.is_async { "async " } else { "" };
        let await_keyword = if op.is_async { "await " } else { "" };

        output.push_str(&format!(
            "export {}function {}({}): {} {{\n",
            async_keyword,
            ts_name,
            params.join(", "),
            return_type
        ));

        output.push_str(&format!(
            "  return {}core.ops.{}({});\n",
            await_keyword,
            op.rust_name,
            param_names.join(", ")
        ));

        output.push_str("}\n");
        output
    }

    /// Generate TypeScript type alias declarations
    pub fn generate_type_aliases(&self, aliases: &[(String, WeldType)]) -> String {
        let mut output = String::new();

        for (name, ty) in aliases {
            output.push_str(&format!("type {} = {};\n", name, ty.to_typescript()));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OpParam, StructField, WeldPrimitive};

    #[test]
    fn test_generate_interface() {
        let module = WeldModule::host("fs").struct_def(
            WeldStruct::new("FileStat")
                .field(StructField::new("isFile", WeldType::bool()))
                .field(StructField::new(
                    "size",
                    WeldType::Primitive(WeldPrimitive::U64),
                ))
                .with_doc("File status information"),
        );

        let gen = TypeScriptGenerator::new(&module);
        let output = gen.generate();

        assert!(output.contains("interface FileStat"));
        assert!(output.contains("isFile: boolean"));
        assert!(output.contains("size: bigint"));
    }

    #[test]
    fn test_generate_export_function() {
        let module = WeldModule::host("fs").op(OpSymbol::from_rust_name("op_fs_read_text")
            .async_op()
            .ts_name("readTextFile")
            .param(OpParam::new("path", WeldType::string()))
            .returns(WeldType::result(
                WeldType::string(),
                WeldType::struct_ref("FsError"),
            ))
            .with_doc("Read a file as UTF-8 text"));

        let gen = TypeScriptGenerator::new(&module);
        let output = gen.generate();

        assert!(output.contains("export async function readTextFile"));
        assert!(output.contains("path: string"));
        assert!(output.contains("Promise<string>"));
        assert!(output.contains("core.ops.op_fs_read_text(path)"));
    }
}
