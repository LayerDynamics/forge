//! Build script: compile the embedded bootstrap shim (`ts/init.ts` -> `init.js`).
//!
//! A *smelted* (ahead-of-time compiled) app boots through a single, stable entry
//! module. That entry is authored in TypeScript (`ts/init.ts`) and transpiled to
//! JavaScript here at build time — mirroring the workspace convention where
//! `forge-runtime` transpiles `preload.ts -> preload.js`. The result is embedded
//! into the crate via `include_str!(concat!(env!("OUT_DIR"), "/init.js"))` and
//! exposed as `forge_smelt::BOOTSTRAP_SHIM`.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=ts/init.ts");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set for build scripts");
    let ts = std::fs::read_to_string("ts/init.ts").expect("read ts/init.ts");

    let js = forge_weld::transpile_ts(&ts, "file:///init.ts")
        .expect("transpile ts/init.ts bootstrap shim");

    let out_path = Path::new(&out_dir).join("init.js");
    std::fs::write(&out_path, js).expect("write OUT_DIR/init.js");
}
