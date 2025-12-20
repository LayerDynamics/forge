//! Proc macros for forge-weld
//!
//! Provides attribute macros for annotating Rust ops and structs
//! with type metadata for TypeScript code generation.
//!
//! # Usage
//!
//! ```text
//! use forge_weld_macro::{weld_op, weld_struct};
//!
//! #[weld_struct]
//! #[derive(Serialize, Deserialize)]
//! pub struct FileStat {
//!     pub is_file: bool,
//!     pub size: u64,
//! }
//!
//! #[weld_op(async)]
//! #[op2(async)]
//! pub async fn op_fs_read_text(#[string] path: String) -> Result<String, FsError> {
//!     // ...
//! }
//! ```

use proc_macro::TokenStream;

mod type_parser;
mod weld_op;
mod weld_struct;

/// Attribute macro for annotating deno_core ops with type metadata
///
/// This macro:
/// 1. Leaves the original function unchanged
/// 2. Generates a companion function that returns type metadata
/// 3. Registers the metadata in the forge-weld inventory
///
/// # Attributes
/// - `#[weld_op]` - Sync op
/// - `#[weld_op(async)]` - Async op
/// - `#[weld_op(ts_name = "customName")]` - Custom TypeScript function name
///
/// # IMPORTANT: Macro Ordering
///
/// **`#[weld_op]` MUST be placed BEFORE `#[op2]`:**
/// ```text
/// #[weld_op(async)]  // ✓ Correct - weld_op first
/// #[op2(async)]      // ✓ deno_core macro second
/// pub async fn my_op() { ... }
/// ```
///
/// **Incorrect ordering will cause compilation errors:**
/// ```text
/// #[op2(async)]      // ✗ Wrong order
/// #[weld_op(async)]  // ✗ Will fail to compile
/// pub async fn my_op() { ... }
/// ```
///
/// This requirement exists because `#[weld_op]` must analyze the function
/// signature before `#[op2]` transforms it.
///
/// # Example
/// ```text
/// #[weld_op(async)]
/// #[op2(async)]
/// pub async fn op_fs_read_text(
///     #[string] path: String,
/// ) -> Result<String, FsError> {
///     // implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn weld_op(attr: TokenStream, item: TokenStream) -> TokenStream {
    weld_op::weld_op_impl(attr.into(), item.into()).into()
}

/// Attribute macro for annotating structs with type metadata
///
/// This macro:
/// 1. Leaves the original struct unchanged
/// 2. Generates a companion function that returns struct metadata
/// 3. Registers the metadata in the forge-weld inventory
///
/// # Attributes
/// - `#[weld_struct]` - Basic struct
/// - `#[weld_struct(ts_name = "CustomName")]` - Custom TypeScript interface name
///
/// # Example
/// ```text
/// #[weld_struct]
/// #[derive(Serialize, Deserialize)]
/// pub struct FileStat {
///     pub is_file: bool,
///     pub is_directory: bool,
///     pub size: u64,
/// }
/// ```
#[proc_macro_attribute]
pub fn weld_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    weld_struct::weld_struct_impl(attr.into(), item.into()).into()
}

/// Attribute macro for annotating enums with type metadata
///
/// # Example
/// ```text
/// #[weld_enum]
/// #[derive(Serialize, Deserialize)]
/// pub enum WatchEventKind {
///     Create,
///     Modify,
///     Remove,
/// }
/// ```
#[proc_macro_attribute]
pub fn weld_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    weld_struct::weld_enum_impl(attr.into(), item.into()).into()
}
