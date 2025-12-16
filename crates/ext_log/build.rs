use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_log", "runtime:log")
        .ts_path("ts/init.ts")
        .ops(&["op_log_info", "op_log_emit"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_log extension");
}
