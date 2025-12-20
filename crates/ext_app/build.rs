use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_app", "runtime:app")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_app_quit",
            "op_app_exit",
            "op_app_relaunch",
            "op_app_get_version",
            "op_app_get_name",
            "op_app_get_identifier",
            "op_app_get_path",
            "op_app_is_packaged",
            "op_app_get_locale",
            "op_app_request_single_instance_lock",
            "op_app_release_single_instance_lock",
            "op_app_focus",
            "op_app_hide",
            "op_app_show",
            "op_app_set_badge_count",
            "op_app_set_user_model_id",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_app extension");
}
