//! Prelude: the common forge-smelt surface for glob import.
//!
//! ```
//! use forge_smelt::prelude::*;
//! ```
//!
//! Wired into the crate from `lib.rs` via `#[path = "mod.rs"] pub mod prelude;`.

pub use crate::binary::SmeltOutput;
pub use crate::build::{embed, EmbedManifest};
pub use crate::compile::{smelt, smelt_entry};
pub use crate::parse::{ModuleGraph, ModuleNode};
pub use crate::{SmeltError, SmeltErrorCode, SmeltResult, BOOTSTRAP_SHIM};
