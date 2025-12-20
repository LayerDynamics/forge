use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("ext_weld_runtime", "forge:weld")
        .ts_path("ts/init.ts")
        .ops(&[
            // Core ops
            "op_weld_info",
            "op_weld_transpile",
            "op_weld_generate_dts",
            "op_weld_json_to_interface",
            "op_weld_validate_ts",
            // Module registration
            "op_weld_register_module",
            "op_weld_list_modules",
            // Code generation
            "op_weld_generate_module_ts",
            "op_weld_generate_module_dts",
            "op_weld_generate_module",
            "op_weld_generate_from_definition",
            // Weld + Etcher integration (documentation generation)
            "op_weld_generate_docs",
            "op_weld_register_and_document",
            "op_weld_generate_sdk_with_docs",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build forge:weld extension");
}
