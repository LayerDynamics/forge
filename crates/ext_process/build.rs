use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_process", "runtime:process")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_process_spawn",
            "op_process_kill",
            "op_process_wait",
            "op_process_status",
            "op_process_write_stdin",
            "op_process_read_stdout",
            "op_process_read_stderr",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_process extension");
}
