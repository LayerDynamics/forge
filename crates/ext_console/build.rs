use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_console", "runtime:console")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_console_info",
            "op_console_push",
            "op_console_tail",
            "op_console_clear",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_console extension");
}
