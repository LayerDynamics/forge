use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_storage", "runtime:storage")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_storage_get",
            "op_storage_set",
            "op_storage_delete",
            "op_storage_has",
            "op_storage_keys",
            "op_storage_clear",
            "op_storage_size",
            "op_storage_get_many",
            "op_storage_set_many",
            "op_storage_delete_many",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_storage extension");
}
