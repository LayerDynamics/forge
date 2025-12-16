use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_os_compat", "runtime:os_compat")
        .ts_path("ts/init.ts")
        .ops(&["op_os_compat_info", "op_os_compat_path_sep"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_os_compat extension");
}
