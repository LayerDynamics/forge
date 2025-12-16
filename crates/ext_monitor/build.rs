use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_monitor", "runtime:monitor")
        .ts_path("ts/init.ts")
        .ops(&["op_monitor_info", "op_monitor_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_monitor extension");
}
