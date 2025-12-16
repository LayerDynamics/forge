use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_path", "runtime:path")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_path_join",
            "op_path_dirname",
            "op_path_basename",
            "op_path_extname",
            "op_path_parts",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_path extension");
}
