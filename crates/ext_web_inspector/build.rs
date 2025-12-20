use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_web_inspector", "runtime:web_inspector")
        .ts_path("ts/init.ts")
        .ops(&[
            // Extension info
            "op_web_inspector_info",
            // Session management
            "op_web_inspector_connect",
            "op_web_inspector_disconnect",
            "op_web_inspector_is_connected",
            "op_web_inspector_sessions",
            // CDP communication
            "op_web_inspector_send_cdp",
            "op_web_inspector_enable_domain",
            "op_web_inspector_disable_domain",
            // Panel injection
            "op_web_inspector_inject_panel",
            "op_web_inspector_is_panel_injected",
            // Aggregated metrics
            "op_web_inspector_get_metrics",
            // Event subscription
            "op_web_inspector_subscribe_events",
            "op_web_inspector_next_event",
            "op_web_inspector_unsubscribe_events",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime_web_inspector extension");
}
