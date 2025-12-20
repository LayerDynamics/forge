use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shortcuts", "runtime:shortcuts")
        .ts_path("ts/init.ts")
        .ops(&[
            // Legacy operations (backward compatibility)
            "op_shortcuts_info",
            "op_shortcuts_echo",
            // Registration operations
            "op_shortcuts_register",
            "op_shortcuts_unregister",
            "op_shortcuts_unregister_all",
            "op_shortcuts_list",
            "op_shortcuts_enable",
            // Event operations
            "op_shortcuts_next_event",
            // Persistence operations
            "op_shortcuts_save",
            "op_shortcuts_load",
            "op_shortcuts_set_auto_persist",
            "op_shortcuts_get_auto_persist",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_shortcuts extension");
}
