use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_trace", "runtime:trace")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_trace_info",
            "op_trace_start",
            "op_trace_end",
            "op_trace_instant",
            "op_trace_flush",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_trace extension");
}
