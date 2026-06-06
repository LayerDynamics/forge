//! Console capture extension for Forge (`runtime:console`).
//!
//! Captures `console.*` output into a bounded, queryable ring buffer so app
//! code can read recent logs (e.g. for an in-app log viewer or the web
//! inspector's console panel). Two ingestion paths feed the same buffer:
//!
//! - **Deno side:** `ts/init.ts` patches `console.*` to forward each call to
//!   [`op_console_push`](console::op_console_push) (while still calling the
//!   original console).
//! - **Renderer side:** the WebView forwards console messages over the
//!   `window.host` IPC bridge; [`web_console::parse_renderer_message`] turns the
//!   payload into a record and pushes it through the same buffer.
//!
//! ## Scope vs. `ext_log`
//!
//! `ext_log` is a stateless forwarder to host `tracing` (fire-and-forget, no
//! retrieval). `ext_console` is complementary: it *retains* recent records and
//! exposes `op_console_tail` / `op_console_clear` for reading them back. The two
//! do not overlap.

mod console;
mod console_listener;
mod console_log;
mod web_console;

use deno_core::{Extension, OpState};

pub use console::{
    op_console_clear, op_console_info, op_console_push, op_console_tail, ExtensionInfo,
};
pub use console_listener::{build_record, now_millis};
pub use console_log::{ConsoleRecord, ConsoleSource, ConsoleState, DEFAULT_CAPACITY};
pub use web_console::parse_renderer_message;

// Include the generated extension code from build.rs.
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Get the console extension for registration with the Deno runtime.
pub fn console_extension() -> Extension {
    runtime_console::ext()
}

/// Initialize the console capture state in the op state (Tier 1: SimpleState).
pub fn init_console_state(state: &mut OpState) {
    state.put::<ConsoleState>(ConsoleState::default());
}
