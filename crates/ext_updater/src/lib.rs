//! Minimal updater extension placeholder.

use deno_core::{op2, Extension};
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;

#[weld_struct]
#[derive(Serialize)]
struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

#[weld_op]
#[op2]
#[serde]
fn op_updater_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_updater",
        version: env!("CARGO_PKG_VERSION"),
        status: "stub",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_updater_echo(#[string] message: String) -> String {
    message
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn updater_extension() -> Extension {
    runtime_updater::ext()
}
