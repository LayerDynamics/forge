use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shell", "runtime:shell")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_shell_open_external",
            "op_shell_open_path",
            "op_shell_show_item_in_folder",
            "op_shell_move_to_trash",
            "op_shell_beep",
            "op_shell_get_file_icon",
            "op_shell_get_default_app",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_shell extension");
}
