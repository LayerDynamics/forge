use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_wasm", "runtime:wasm")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_wasm_compile",
            "op_wasm_compile_file",
            "op_wasm_drop_module",
            "op_wasm_instantiate",
            "op_wasm_drop_instance",
            "op_wasm_get_exports",
            "op_wasm_call",
            "op_wasm_memory_read",
            "op_wasm_memory_write",
            "op_wasm_memory_size",
            "op_wasm_memory_grow",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_wasm extension");
}
