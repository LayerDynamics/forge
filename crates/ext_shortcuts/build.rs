use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shortcuts", "runtime:shortcuts")
        .ts_path("ts/init.ts")
        .ops(&["op_shortcuts_info", "op_shortcuts_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_shortcuts extension");
}
