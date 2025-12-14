use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_net", "host:net")
        .ts_path("ts/init.ts")
        .ops(&["op_net_fetch", "op_net_fetch_bytes"])
        .build()
        .expect("Failed to build host_net extension");
}
