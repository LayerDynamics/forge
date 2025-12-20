use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_display", "runtime:display")
        .ts_path("ts/init.ts")
        .ops(&[
            // Legacy operations (backward compatibility)
            "op_display_info",
            "op_display_echo",
            // Display query operations
            "op_display_get_all",
            "op_display_get_primary",
            "op_display_get_by_id",
            "op_display_get_at_point",
            "op_display_get_cursor_position",
            "op_display_get_count",
            // Subscription operations
            "op_display_subscribe",
            "op_display_unsubscribe",
            "op_display_next_event",
            "op_display_subscriptions",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_display extension");
}
