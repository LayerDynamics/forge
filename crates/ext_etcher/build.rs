use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("ext_etcher_runtime", "forge:etcher")
        .ts_path("ts/init.ts")
        .ops(&[
            // Core ops
            "op_etcher_info",
            "op_etcher_generate_docs",
            "op_etcher_parse_ts",
            "op_etcher_parse_rust",
            "op_etcher_merge_nodes",
            "op_etcher_nodes_to_astro",
            "op_etcher_nodes_to_html",
            // WeldModule-based documentation
            "op_etcher_from_weld_module",
            // Site update/regeneration
            "op_etcher_update_site",
            "op_etcher_regenerate_site",
            "op_etcher_generate_site_index",
            "op_etcher_validate_config",
            "op_etcher_validate_output_dir",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build forge:etcher extension");
}
