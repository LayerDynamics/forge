use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_signals", "runtime:signals")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_signals_supported",
            "op_signals_subscribe",
            "op_signals_next",
            "op_signals_unsubscribe",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_signals extension");
}
