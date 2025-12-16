//! OS compatibility helpers: platform info and paths.

use deno_core::{op2, Extension};
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;

#[weld_struct]
#[derive(Serialize)]
struct OsInfo {
    os: &'static str,
    arch: &'static str,
    family: &'static str,
    path_sep: &'static str,
    env_sep: &'static str,
    tmp_dir: String,
    home_dir: Option<String>,
}

#[weld_op]
#[op2]
#[serde]
fn op_os_compat_info() -> OsInfo {
    OsInfo {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        family: std::env::consts::FAMILY,
        path_sep: if cfg!(windows) { "\\" } else { "/" },
        env_sep: if cfg!(windows) { ";" } else { ":" },
        tmp_dir: std::env::temp_dir().to_string_lossy().to_string(),
        home_dir: dirs::home_dir().map(|p| p.to_string_lossy().to_string()),
    }
}

#[weld_op]
#[op2]
#[string]
fn op_os_compat_path_sep() -> String {
    if cfg!(windows) {
        "\\".to_string()
    } else {
        "/".to_string()
    }
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn os_compat_extension() -> Extension {
    runtime_os_compat::ext()
}
