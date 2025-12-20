use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_monitor", "runtime:monitor")
        .ts_path("ts/init.ts")
        .ops(&[
            // Legacy operations (backward compatibility)
            "op_monitor_info",
            "op_monitor_echo",
            // System metrics
            "op_monitor_cpu",
            "op_monitor_memory",
            "op_monitor_disk",
            "op_monitor_network",
            "op_monitor_process_self",
            "op_monitor_processes",
            // Runtime metrics
            "op_monitor_runtime",
            "op_monitor_heap",
            // WebView metrics
            "op_monitor_webview",
            // Subscription API
            "op_monitor_subscribe",
            "op_monitor_next",
            "op_monitor_unsubscribe",
            "op_monitor_subscriptions",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime_monitor extension");
}
