use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_svelte", "runtime:svelte")
        .ts_path("ts/init.ts")
        .ops(&[
            // Detection ops
            "op_svelte_detect",
            // Build phase ops
            "op_svelte_walk",
            "op_svelte_generate_deploy_config",
            "op_svelte_generate_svelte_data",
            // Runtime phase ops
            "op_svelte_prepare_server",
            "op_svelte_get_isr_config",
            "op_svelte_to_cache_key",
            "op_svelte_is_static",
            "op_svelte_get_redirect",
            "op_svelte_get_headers",
            "op_svelte_close_server",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime:svelte extension");
}
