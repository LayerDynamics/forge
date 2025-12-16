use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_timers", "runtime:timers")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_host_timer_create",
            "op_host_timer_cancel",
            "op_host_timer_sleep",
            "op_host_timer_exists",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_timers extension");
}
