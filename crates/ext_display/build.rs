use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_display", "runtime:display")
        .ts_path("ts/init.ts")
        .ops(&["op_display_info", "op_display_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_display extension");
}
