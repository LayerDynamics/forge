use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_debugger", "runtime:debugger")
        .ts_path("ts/init.ts")
        .ops(&["op_debugger_info", "op_debugger_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_debugger extension");
}
