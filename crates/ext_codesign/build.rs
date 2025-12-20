use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_codesign", "runtime:codesign")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_codesign_sign",
            "op_codesign_sign_adhoc",
            "op_codesign_verify",
            "op_codesign_get_entitlements",
            "op_codesign_list_identities",
            "op_codesign_get_identity_info",
            "op_codesign_check_capabilities",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_codesign extension");
}
