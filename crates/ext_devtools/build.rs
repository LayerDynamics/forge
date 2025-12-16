use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_devtools", "runtime:devtools")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_devtools_open",
            "op_devtools_close",
            "op_devtools_is_open",
        ])
        .generate_sdk_types("sdk")
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_devtools extension");
}
