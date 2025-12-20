use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_encoding", "runtime:encoding")
        .ts_path("ts/init.ts")
        .ops(&[]) // No custom ops - pure JavaScript implementation
        .enable_extensibility()
        .build()
        .expect("Failed to build ext_encoding");
}
