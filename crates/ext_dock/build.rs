use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_dock", "runtime:dock")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_dock_info",
            "op_dock_bounce",
            "op_dock_cancel_bounce",
            "op_dock_set_badge",
            "op_dock_get_badge",
            "op_dock_hide",
            "op_dock_show",
            "op_dock_is_visible",
            "op_dock_set_icon",
            "op_dock_set_menu",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_dock");
}
