use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_webview", "runtime:webview")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_host_webview_new",
            "op_host_webview_exit",
            "op_host_webview_eval",
            "op_host_webview_set_color",
            "op_host_webview_set_title",
            "op_host_webview_set_fullscreen",
            "op_host_webview_loop",
            "op_host_webview_run",
        ])
        .generate_sdk_types("sdk")
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build host_webview extension");
}
