use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_net", "runtime:net")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_net_fetch",
            "op_net_fetch_bytes",
            // WebSocket operations
            "op_net_ws_connect",
            "op_net_ws_send",
            "op_net_ws_recv",
            "op_net_ws_close",
            // Streaming fetch
            "op_net_fetch_stream",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_net extension");
}
