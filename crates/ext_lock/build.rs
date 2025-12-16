use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_lock", "runtime:lock")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_lock_acquire",
            "op_lock_try",
            "op_lock_release",
            "op_lock_list",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_lock extension");
}
