use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_sys", "host:sys")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_sys_info",
            "op_sys_env_get",
            "op_sys_env_set",
            "op_sys_cwd",
            "op_sys_home_dir",
            "op_sys_temp_dir",
            "op_sys_clipboard_read",
            "op_sys_clipboard_write",
            "op_sys_notify",
            "op_sys_notify_ext",
            "op_sys_power_info",
        ])
        .build()
        .expect("Failed to build host_sys extension");
}
