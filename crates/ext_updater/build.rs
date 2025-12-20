use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_updater", "runtime:updater")
        .ts_path("ts/init.ts")
        .ops(&[
            // Legacy operations (backward compatibility)
            "op_updater_info",
            "op_updater_echo",
            // Configuration operations
            "op_updater_configure_github",
            "op_updater_configure_custom",
            // Check operations
            "op_updater_check",
            // Download operations
            "op_updater_download",
            "op_updater_download_progress",
            "op_updater_cancel",
            // Verification operations
            "op_updater_verify",
            // Install operations
            "op_updater_install",
            // Status operations
            "op_updater_status",
            "op_updater_get_current_version",
            "op_updater_get_pending_update",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_updater extension");
}
