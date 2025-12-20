use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_debugger", "runtime:debugger")
        .ts_path("ts/init.ts")
        .ops(&[
            // Extension info
            "op_debugger_info",
            // Connection management
            "op_debugger_connect",
            "op_debugger_disconnect",
            "op_debugger_is_connected",
            // Breakpoint management
            "op_debugger_set_breakpoint",
            "op_debugger_remove_breakpoint",
            "op_debugger_remove_all_breakpoints",
            "op_debugger_list_breakpoints",
            "op_debugger_enable_breakpoint",
            "op_debugger_disable_breakpoint",
            // Execution control
            "op_debugger_pause",
            "op_debugger_resume",
            "op_debugger_step_over",
            "op_debugger_step_into",
            "op_debugger_step_out",
            "op_debugger_continue_to_location",
            // Stack & scope inspection
            "op_debugger_get_call_frames",
            "op_debugger_get_scope_chain",
            "op_debugger_get_properties",
            // Expression evaluation
            "op_debugger_evaluate",
            "op_debugger_set_variable_value",
            // Source management
            "op_debugger_get_script_source",
            "op_debugger_list_scripts",
            // Exception handling
            "op_debugger_set_pause_on_exceptions",
            // Event receivers
            "op_debugger_create_pause_receiver",
            "op_debugger_receive_pause_event",
            "op_debugger_create_script_receiver",
            "op_debugger_receive_script_event",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build runtime_debugger extension");
}
