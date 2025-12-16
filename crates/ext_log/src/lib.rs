//! Structured logging extension bridging to host tracing.

use deno_core::{op2, Extension};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, error, info, trace, warn, Level};

#[weld_struct]
#[derive(Serialize)]
struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

#[derive(Debug, Error, JsError)]
pub enum LogError {
    #[error("Invalid log level: {0}")]
    #[class(generic)]
    InvalidLevel(String),
}

#[weld_op]
#[op2]
#[serde]
fn op_log_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_log",
        version: env!("CARGO_PKG_VERSION"),
        status: "ready",
    }
}

#[weld_op]
#[op2]
fn op_log_emit(
    #[string] level: String,
    #[string] message: String,
    #[serde] fields: Option<Value>,
) -> Result<(), LogError> {
    let lvl = parse_level(&level)?;

    match lvl {
        Level::TRACE => trace!(fields = ?fields, "{message}"),
        Level::DEBUG => debug!(fields = ?fields, "{message}"),
        Level::INFO => info!(fields = ?fields, "{message}"),
        Level::WARN => warn!(fields = ?fields, "{message}"),
        Level::ERROR => error!(fields = ?fields, "{message}"),
    }

    Ok(())
}

fn parse_level(level: &str) -> Result<Level, LogError> {
    match level.to_ascii_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        other => Err(LogError::InvalidLevel(other.to_string())),
    }
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn log_extension() -> Extension {
    runtime_log::ext()
}
