use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_database", "runtime:database")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_database_info",
            "op_database_echo",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_database extension");
}
