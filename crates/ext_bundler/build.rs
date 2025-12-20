use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("forge_bundler", "forge:bundler")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_bundler_info",
            "op_bundler_icon_create",
            "op_bundler_icon_validate",
            "op_bundler_icon_resize",
            "op_bundler_manifest_parse",
            "op_bundler_sanitize_name",
            "op_bundler_platform_info",
            "op_bundler_icon_requirements",
            "op_bundler_set_app_dir",
            "op_bundler_get_app_dir",
            "op_bundler_set_build_config",
            "op_bundler_get_build_config",
            "op_bundler_path_info",
            "op_bundler_path_join",
            "op_bundler_manifest_path",
            "op_bundler_cache_manifest",
            "op_bundler_get_cached_manifest",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build forge:bundler extension");
}
