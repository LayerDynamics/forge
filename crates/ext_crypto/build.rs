use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_crypto", "runtime:crypto")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_crypto_random_bytes",
            "op_crypto_random_uuid",
            "op_crypto_hash",
            "op_crypto_hash_hex",
            "op_crypto_hmac",
            "op_crypto_encrypt",
            "op_crypto_decrypt",
            "op_crypto_generate_key",
            "op_crypto_derive_key",
            "op_crypto_verify",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build host_crypto extension");
}
