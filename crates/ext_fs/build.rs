use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_fs", "host:fs")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_fs_read_text",
            "op_fs_write_text",
            "op_fs_read_bytes",
            "op_fs_write_bytes",
            "op_fs_stat",
            "op_fs_read_dir",
            "op_fs_mkdir",
            "op_fs_remove",
            "op_fs_rename",
            "op_fs_copy",
            "op_fs_exists",
            "op_fs_watch",
            "op_fs_watch_next",
            "op_fs_watch_close",
        ])
        .build()
        .expect("Failed to build host_fs extension");
}
