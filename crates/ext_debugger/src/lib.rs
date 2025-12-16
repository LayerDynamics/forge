//! Minimal debugger extension placeholder.

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
fn op_debugger_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_debugger",
        version: env!("CARGO_PKG_VERSION"),
        status: "stub",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_debugger_echo(#[string] message: String) -> String {
    message
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn debugger_extension() -> Extension {
    runtime_debugger::ext()
}
