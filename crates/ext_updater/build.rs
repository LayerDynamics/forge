use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_updater", "runtime:updater")
        .ts_path("ts/init.ts")
        .ops(&["op_updater_info", "op_updater_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_updater extension");
}
