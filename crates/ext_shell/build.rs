use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shell", "runtime:shell")
        .ts_path("ts/init.ts")
        .ops(&[
            // System integration ops
            "op_shell_open_external",
            "op_shell_open_path",
            "op_shell_show_item_in_folder",
            "op_shell_move_to_trash",
            "op_shell_beep",
            "op_shell_get_file_icon",
            "op_shell_get_default_app",
            // Shell execution ops
            "op_shell_execute",
            "op_shell_kill",
            "op_shell_cwd",
            "op_shell_set_cwd",
            "op_shell_get_env",
            "op_shell_set_env",
            "op_shell_unset_env",
            "op_shell_get_all_env",
            "op_shell_which",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_shell extension");
}
