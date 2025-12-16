use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_protocol", "runtime:protocol")
        .ts_path("ts/init.ts")
        .ops(&["op_protocol_info", "op_protocol_echo"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_protocol extension");
}
