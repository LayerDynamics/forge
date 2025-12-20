use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_protocol", "runtime:protocol")
        .ts_path("ts/init.ts")
        .ops(&[
            // Extension info
            "op_protocol_info",
            // Registration operations
            "op_protocol_register",
            "op_protocol_unregister",
            "op_protocol_is_registered",
            "op_protocol_list_registered",
            "op_protocol_set_as_default",
            // Invocation handling
            "op_protocol_get_launch_url",
            "op_protocol_receive_invocation",
            // URL utilities
            "op_protocol_parse_url",
            "op_protocol_build_url",
            // Platform capabilities
            "op_protocol_check_capabilities",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime_protocol extension");
}
