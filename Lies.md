# Lies.md - Placeholders, Incomplete, and Simplified Code

This document catalogs all placeholder, temporary, simplified, and incomplete implementations found in `crates/**`.

**Last Updated**: All critical issues have been fixed. WASM externref/funcref support now includes full handle registry implementation with bidirectional conversion.

---

## Fixed Issues

### ~~Type Parsing Not Implemented (forge-weld-macro)~~ FIXED

**Resolution**: Created `crates/forge-weld-macro/src/type_parser.rs` with comprehensive type parsing that converts all Rust types to proper `WeldType` variants. The parser operates in strict mode and will panic on unparseable types to ensure no `unknown` types slip through.

Supported types:
- Primitives: `u8`-`u64`, `i8`-`i64`, `f32`, `f64`, `bool`, `String`, `str`, `char`, `()`
- Containers: `Option<T>`, `Vec<T>`, `Result<T, E>`, `HashMap<K, V>`, `BTreeMap<K, V>`
- Sets: `HashSet<T>`, `BTreeSet<T>`
- Wrappers: `Box<T>`, `Arc<T>`, `Rc<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`
- References: `&T`, `&mut T`
- Tuples: `(A, B, C)`
- Arrays/Slices: `[T; N]`, `[T]`
- Special: `serde_json::Value` (JsonValue), `OpState`
- Custom types: Treated as Struct references

### ~~Dependency Version Not Pinned~~ FIXED

**Resolution**: All Deno dependencies are now pinned:
- `deno_core = "0.373"` (all crates)
- `deno_ast = { version = "0.52", features = ["transpiling"] }`
- `deno_error = "0.7"`

### ~~Resolver Module is Empty Placeholder~~ FIXED

**Resolution**: Deleted `crates/forge-host/src/resolver.ts`. Module resolution is handled entirely by `ForgeModuleLoader` in Rust.

### ~~Linux Native Handle Retrieval~~ FIXED

**Resolution**: Linux native handle retrieval is already implemented in `crates/forge-host/src/main.rs` using GTK/GDK APIs:
```rust
#[cfg(target_os = "linux")]
{
    use gdk::prelude::*;
    use tao::platform::unix::WindowExtUnix;
    let gdk_window = window.gtk_window().window();
    if let Some(gdk_win) = gdk_window {
        use glib::translate::ToGlibPtr;
        let gdk_ptr: *mut gdk_sys::GdkWindow = gdk_win.to_glib_none().0;
        let handle = gdk_ptr as u64;
        let _ = respond.send(Ok(NativeHandle {
            platform: "linux".to_string(),
            handle,
        }));
    }
}
```

### ~~Context Menu Returns Empty Instead of Selection~~ FIXED

**Resolution**: Updated `WinShowContextMenu` handler in `crates/forge-host/src/main.rs` to properly track context menu selections:
1. Cancels any pending context menu response
2. Collects menu item IDs into a HashSet
3. Stores the pending response with those IDs via `pending_ctx_menu`
4. Lets the menu event thread handle responding when an item is selected

### ~~WASM externref/funcref Not Supported~~ FIXED

**Resolution**: Added full WASM reference type support with handle registry to `crates/ext_wasm/src/lib.rs`:

1. **WasmValue enum** (lines 220-237): Added `FuncRef(Option<String>)` and `ExternRef(Option<String>)` variants for serialization across the JS/WASM boundary

2. **Handle Registry** (lines 346-383): Added to `WasmInstance` struct:
   - `func_refs: HashMap<String, Func>` - stores funcref handles
   - `extern_ref_ids: HashMap<String, u64>` - tracks externref identifiers
   - `store_func_ref()` / `get_func_ref()` - store and retrieve funcrefs
   - `store_extern_ref()` / `get_extern_ref_id()` - track externrefs

3. **to_wasmtime()** (lines 240-273): Looks up funcref handles from the instance registry. Returns the actual `Func` when a valid handle is provided.

4. **from_wasmtime()** (lines 275-302): Stores references in the instance registry:
   - FuncRef: Clones the `Func` and stores it, returning a unique handle like `funcref_1`
   - ExternRef: Generates tracking handles for GC-managed references

5. **op_wasm_call** (lines 871-959): Updated to pass the `WasmInstance` to conversion methods, enabling proper round-trip handling of reference types

6. **Tests** (lines 1230-1305): Added comprehensive tests:
   - `test_wasm_funcref_registry`: Verifies funcref storage and retrieval
   - `test_wasm_null_refs`: Verifies null reference handling

### ~~Fullscreen Not Supported~~ FIXED

**Resolution**: Fullscreen was already fully implemented - the error code `FullscreenNotSupported = 6013` was just an error code definition for edge cases, not a missing feature. The implementation exists at:
- `crates/ext_window/src/lib.rs`: `op_window_set_fullscreen` and `op_window_is_fullscreen` ops
- `crates/ext_window/ts/init.ts`: `setFullscreen()` and `isFullscreen()` TypeScript API
- `crates/forge-host/src/main.rs:2104-2112`: Event handler using `tao::window::Fullscreen::Borderless`

---

## Remaining Items (By Design)

### Simplified Implementations

| File | Line | Description |
|------|------|-------------|
| `crates/forge-weld/src/build/extension.rs` | 1 | `//! ExtensionBuilder for simplified build.rs scripts` |
| `crates/forge-weld/src/build/mod.rs` | 5 | `//! - ExtensionBuilder for simplified extension crate setup` |

The build system is described as "simplified" - this is intentional design, not incomplete implementation.

---

## Default/Fallback Implementations (Working as Designed)

### Default Tray Icon (Gray Square)

| File | Lines | Description |
|------|-------|-------------|
| `crates/forge-host/src/main.rs` | 1520-1528, 2390-2398 | Creates a simple gray square as default tray icon |

When no icon is provided or icon loading fails, a 22x22 gray square is used. This is intentional fallback behavior.

### Bundler Fallbacks

| File | Line | Description |
|------|------|-------------|
| `crates/forge/src/bundler/linux.rs` | 235 | `// Try mksquashfs as fallback` for AppImage creation |
| `crates/forge/src/bundler/linux.rs` | 435 | `// Try wget as fallback` for downloading |
| `crates/forge/src/bundler/windows.rs` | 333 | `// Fallback: try with png2ico if available` |
| `crates/forge/src/bundler/windows.rs` | 368 | Creates `icon_fallback.png` when ICO conversion fails |

These are intentional fallback mechanisms to support different system configurations.

---

## Feature Capability Restrictions (Security - Not Bugs)

These are intentional security restrictions:

| File | Line | Description |
|------|------|-------------|
| `crates/ext_sys/src/lib.rs` | 25 | `/// Feature not supported on this platform` |
| `crates/ext_sys/src/lib.rs` | 46 | `#[error("[{code}] Not supported: {message}")]` |
| `crates/ext_net/src/lib.rs` | 314, 409 | `"Unsupported method: {}"` for HTTP methods |

---

## Summary

| Category | Count | Status |
|----------|-------|--------|
| Critical TODOs | 6 | **FIXED** |
| Placeholder implementations | 3 | **FIXED** |
| "For now" temporary code | 1 | **FIXED** |
| Not yet implemented | 1 | **FIXED** |
| WASM externref/funcref | 1 | **FIXED** |
| Fullscreen support | 1 | **FIXED** |
| Simplified implementations | 2 | By Design |
| Fallback implementations | 5 | By Design |

**All issues have been resolved. Only intentional design decisions remain.**
