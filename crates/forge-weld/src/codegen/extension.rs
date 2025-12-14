//! Rust extension.rs code generator
//!
//! Generates the deno_core::extension! macro invocations that are
//! included in each extension crate's lib.rs file.

use crate::ir::WeldModule;

/// Generator for Rust extension.rs files
pub struct ExtensionGenerator<'a> {
    module: &'a WeldModule,
}

impl<'a> ExtensionGenerator<'a> {
    /// Create a new extension generator
    pub fn new(module: &'a WeldModule) -> Self {
        Self { module }
    }

    /// Generate the extension! macro invocation
    ///
    /// This produces code like:
    /// ```ignore
    /// deno_core::extension!(
    ///     host_fs,
    ///     ops = [op_fs_read_text, op_fs_write_text, ...],
    ///     esm_entry_point = "ext:host_fs/init.js",
    ///     esm = ["ext:host_fs/init.js" = { source = "..." }]
    /// );
    /// ```
    pub fn generate(&self, js_source: &str) -> String {
        let op_names = self.module.op_names();
        let ops_list = if op_names.is_empty() {
            String::new()
        } else {
            format!(
                "ops = [\n        {},\n    ],\n    ",
                op_names.join(",\n        ")
            )
        };

        format!(
            r#"deno_core::extension!(
    {},
    {}esm_entry_point = "{}",
    esm = ["{}" = {{ source = {:?} }}]
);"#,
            self.module.name,
            ops_list,
            self.module.esm_entry_point,
            self.module.esm_entry_point,
            js_source
        )
    }

    /// Generate extension! macro with custom state initialization
    pub fn generate_with_state(
        &self,
        js_source: &str,
        state_fn: &str,
    ) -> String {
        let op_names = self.module.op_names();
        let ops_list = if op_names.is_empty() {
            String::new()
        } else {
            format!(
                "ops = [\n        {},\n    ],\n    ",
                op_names.join(",\n        ")
            )
        };

        format!(
            r#"deno_core::extension!(
    {},
    {}state = {},
    esm_entry_point = "{}",
    esm = ["{}" = {{ source = {:?} }}]
);"#,
            self.module.name,
            ops_list,
            state_fn,
            self.module.esm_entry_point,
            self.module.esm_entry_point,
            js_source
        )
    }

    /// Generate extension! macro for extensions with dependencies
    pub fn generate_with_deps(
        &self,
        js_source: &str,
        deps: &[&str],
    ) -> String {
        let op_names = self.module.op_names();
        let ops_list = if op_names.is_empty() {
            String::new()
        } else {
            format!(
                "ops = [\n        {},\n    ],\n    ",
                op_names.join(",\n        ")
            )
        };

        let deps_list = if deps.is_empty() {
            String::new()
        } else {
            format!("deps = [{}],\n    ", deps.join(", "))
        };

        format!(
            r#"deno_core::extension!(
    {},
    {}{}esm_entry_point = "{}",
    esm = ["{}" = {{ source = {:?} }}]
);"#,
            self.module.name,
            deps_list,
            ops_list,
            self.module.esm_entry_point,
            self.module.esm_entry_point,
            js_source
        )
    }

    /// Generate extension! macro with multiple ESM files
    pub fn generate_with_esm_files(
        &self,
        esm_files: &[(&str, &str)], // (specifier, source)
    ) -> String {
        let op_names = self.module.op_names();
        let ops_list = if op_names.is_empty() {
            String::new()
        } else {
            format!(
                "ops = [\n        {},\n    ],\n    ",
                op_names.join(",\n        ")
            )
        };

        let esm_entries: Vec<String> = esm_files
            .iter()
            .map(|(specifier, source)| {
                format!("\"{}\" = {{ source = {:?} }}", specifier, source)
            })
            .collect();

        format!(
            r#"deno_core::extension!(
    {},
    {}esm_entry_point = "{}",
    esm = [
        {}
    ]
);"#,
            self.module.name,
            ops_list,
            self.module.esm_entry_point,
            esm_entries.join(",\n        ")
        )
    }

    /// Generate a minimal extension! macro (ops only, no ESM)
    pub fn generate_ops_only(&self) -> String {
        let op_names = self.module.op_names();

        if op_names.is_empty() {
            format!(
                r#"deno_core::extension!(
    {}
);"#,
                self.module.name
            )
        } else {
            format!(
                r#"deno_core::extension!(
    {},
    ops = [
        {},
    ]
);"#,
                self.module.name,
                op_names.join(",\n        ")
            )
        }
    }
}

/// Helper to generate the full extension.rs file content with header
pub fn generate_extension_file(module: &WeldModule, js_source: &str) -> String {
    let gen = ExtensionGenerator::new(module);
    let extension_macro = gen.generate(js_source);

    format!(
        r#"// Auto-generated extension definition for {}
// Generated by forge-weld - do not edit manually

{}
"#,
        module.specifier, extension_macro
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OpSymbol, WeldType, OpParam};

    #[test]
    fn test_generate_extension() {
        let module = WeldModule::host("fs")
            .op(OpSymbol::from_rust_name("op_fs_read_text")
                .param(OpParam::new("path", WeldType::string()))
                .returns(WeldType::string()))
            .op(OpSymbol::from_rust_name("op_fs_write_text")
                .param(OpParam::new("path", WeldType::string()))
                .param(OpParam::new("content", WeldType::string()))
                .returns(WeldType::void()));

        let gen = ExtensionGenerator::new(&module);
        let output = gen.generate("const x = 1;");

        assert!(output.contains("deno_core::extension!"));
        assert!(output.contains("host_fs"));
        assert!(output.contains("op_fs_read_text"));
        assert!(output.contains("op_fs_write_text"));
        assert!(output.contains("ext:host_fs/init.js"));
    }

    #[test]
    fn test_generate_with_deps() {
        let module = WeldModule::host("ui")
            .op(OpSymbol::from_rust_name("op_ui_open"));

        let gen = ExtensionGenerator::new(&module);
        let output = gen.generate_with_deps("const x = 1;", &["host_ipc"]);

        assert!(output.contains("deps = [host_ipc]"));
    }
}
