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
| `crates/forge_cli/src/bundler/linux.rs` | 235 | `// Try mksquashfs as fallback` for AppImage creation |
| `crates/forge_cli/src/bundler/linux.rs` | 435 | `// Try wget as fallback` for downloading |
| `crates/forge_cli/src/bundler/windows.rs` | 333 | `// Fallback: try with png2ico if available` |
| `crates/forge_cli/src/bundler/windows.rs` | 368 | Creates `icon_fallback.png` when ICO conversion fails |

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









Fetch(url: "<https://github.com/denoland/deno_bindgen>", prompt: "Extract information about how deno_bindgen automatically
       generates TypeScript bindings from Rust code. What is the pattern/approach? How does it generate TypeScript declarations?")
  ⎿  Received 295.4KB (200 OK)

# deno_bindgen's TypeScript Generation Approach

## Pattern Overview

deno_bindgen automatically generates TypeScript bindings from Rust code through a macro-based code generation system. The
tool uses Rust procedural macros to analyze exported functions and structures, then generates corresponding TypeScript
declarations.

## Key Generation Method

The approach leverages the `#[deno_bindgen]` attribute macro applied to Rust items:

> "TypeScript types are generated and supported OOTB."

When developers annotate Rust functions or structs with this macro, the tool's intermediate representation (IR) layer
processes the type information and generates matching TypeScript interfaces and function signatures.

## Generated Output

The system produces:

- **Type-safe function declarations** with matching parameter and return types
- **Class bindings** for Rust structs, including constructor and method signatures
- **Automatic disposal support** for memory management using TypeScript's `using` statement

For example, a simple Rust function becomes immediately callable in TypeScript with proper typing, eliminating manual glue
code creation that developers would otherwise need to write.

## Technical Architecture

The generation happens through multiple specialized crates:

- `deno_bindgen_ir` - analyzes Rust type information
- `deno_bindgen_macro` - provides the procedural macro system
- `deno_bindgen_cli` - generates the final TypeScript output files

This design ensures type safety across the FFI boundary automatically.

⏺ Read(crates/forge-weld/src/codegen/typescript.rs)
  ⎿  Read 274 lines

⏺ Read(crates/forge-weld/src/codegen/dts.rs)
  ⎿  Read 385 lines

∴ Thinking…

  This is very interesting! forge-weld already has:

  1. TypeScriptGenerator - Generates complete init.ts files with:
    - Deno.core.ops declarations
    - Interface declarations for structs
    - Enum/union declarations
    - Export functions wrapping each op
  2. DtsGenerator - Generates .d.ts declaration files with:
    - Ambient module declarations (declare module "runtime:*")
    - Exported interfaces
    - Exported types/enums
    - Function declarations with JSDoc
  3. ExtensionBuilder - Has a generate_sdk_types(path) method that can output to a folder

  The key insight is that the infrastructure already exists, but:

  1. The SDK files (sdk/runtime.*.ts) are being manually maintained rather than generated
  2. The generate_sdk_types() functionality writes to sdk/generated/ but produces .d.ts files, not .ts implementation files

  What we need to do:

  1. Add a method to ExtensionBuilder to generate the full runtime.*.ts SDK files (not just .d.ts)
  2. Have it use TypeScriptGenerator to produce the implementation
  3. Make it part of the build process

  Let me read the ExtensionBuilder to see how it currently works.

⏺ Excellent! I found the key insight - forge-weld already has the code generation infrastructure! Let me read the ExtensionBuilder to see how SDK generation currently works:

⏺ Read(crates/forge-weld/src/build/extension.rs)
  ⎿  Read 297 lines

⏺ Read(crates/forge-weld/src/ir/module.rs)
  ⎿  Read 367 lines

∴ Thinking…

  Now I have a very clear picture! I can see that:1. forge-weld already has all the infrastructure needed:   - TypeScriptGenerator

- Generates complete init.ts files with Deno.core.ops wrappers
- DtsGenerator - Generates .d.ts declaration files
- ExtensionBuilder - Build script integration2. Current behavior:
- generate_sdk_types("../../sdk") writes .d.ts files to sdk/generated/
- The TypeScript implementation files (sdk/runtime.*.ts) are manually maintained3. What we need to add:
- A method like generate_sdk_module("../../sdk") that writes the full .ts implementation file to sdk/runtime.*.ts
- Use TypeScriptGenerator instead of DtsGenerator for this
- This would auto-generate the SDK files during build

  The key insight from deno_bindgen is that it uses a similar approach - procedural macros + IR layer + code generation.
  forge-weld already has this pattern!

  Now I need to write the plan. The approach is clear:

  1. Add generate_sdk_module() method to ExtensionBuilder
  2. Have it use TypeScriptGenerator::generate() to produce full .ts files
  3. Write to sdk/runtime.{module_name}.ts instead of sdk/generated/*.d.ts
  4. Update each ext_*/build.rs to call this new method

  Let me write the plan file now.

⏺ Now I have a complete understanding. forge-weld already has all the code generation infrastructure - it just needs a small enhancement to generate SDK .ts files instead of just .d.ts files.

  Let me write the plan:

⏺ Updated plan
  ⎿  Wrote 191 lines to /Users/ryanoboyle/.claude/plans/rosy-finding-giraffe.md
     # Plan: Auto-Generate SDK TypeScript Files via forge-weld

     ## Overview

     Add SDK `.ts` module generation to forge-weld, so each extension's `build.rs` can automatically generate
     `sdk/runtime.*.ts` files during the Rust build process. This eliminates manual SDK maintenance.

     ---

     ## Current State

     ### What forge-weld Already Has ✅

     1. **`TypeScriptGenerator`** (`codegen/typescript.rs:18-226`)
        - Generates complete `init.ts` files with:
          - `Deno.core.ops` type declarations
          - Interface declarations for structs
          - Enum/union type declarations
          - Export functions wrapping each op
        - **This is exactly what we need for SDK files!**

     2. **`DtsGenerator`** (`codegen/dts.rs:9-271`)
        - Generates `.d.ts` declaration files
        - Currently used by `generate_sdk_types()`

     3. **`ExtensionBuilder`** (`build/extension.rs:62-296`)
        - Has `generate_sdk_types(path)` which writes `.d.ts` to `sdk/generated/`
        - **Missing**: Method to generate full `.ts` implementation files

     ### What's Missing ❌

     - `ExtensionBuilder` doesn't have a method to generate `sdk/runtime.*.ts` files
     - Only generates `.d.ts` (declarations) not `.ts` (implementations)

     ---

     ## Solution Design

     ### Approach: Add `generate_sdk_module()` to ExtensionBuilder

     The fix is simple - add one new method that uses the existing `TypeScriptGenerator`:

     ```rust
     /// Enable SDK module generation (full .ts files, not just .d.ts)
     pub fn generate_sdk_module(mut self, sdk_relative_path: impl AsRef<Path>) -> Self {
         self.sdk_module_path = Some(sdk_relative_path.as_ref().to_path_buf());
         self
     }
     ```

     Then in `build()`, generate using `TypeScriptGenerator`:

     ```rust
     // Generate SDK module .ts file if requested
     if let Some(ref sdk_path) = self.sdk_module_path {
         let workspace_root = manifest_path.parent().unwrap().parent().unwrap();
         let sdk_dir = workspace_root.join(sdk_path);

         // Generate full .ts module using TypeScriptGenerator
         let ts_gen = TypeScriptGenerator::new(&self.module);
         let ts_content = ts_gen.generate();

         // Write to sdk/runtime.{name}.ts
         let ts_filename = format!("runtime.{}.ts", self.module.specifier.split(':').last().unwrap());
         let ts_path = sdk_dir.join(&ts_filename);
         fs::write(&ts_path, &ts_content)?;
     }
     ```

     ---

     ## Implementation Plan

     ### Phase 1: Enhance forge-weld (1 file)

     **File: `crates/forge-weld/src/build/extension.rs`**

     1. Add new field to `ExtensionBuilder`:
        ```rust
        sdk_module_path: Option<PathBuf>,
        ```

     2. Add builder method:
        ```rust
        pub fn generate_sdk_module(mut self, sdk_relative_path: impl AsRef<Path>) -> Self
        ```

     3. Update `build()` method to generate `.ts` files when `sdk_module_path` is set
        - Use `TypeScriptGenerator::new(&self.module).generate()`
        - Write to `sdk/runtime.{module_name}.ts`

     4. Add to `BuildOutput`:
        ```rust
        pub sdk_ts_path: Option<PathBuf>,
        ```

     ### Phase 2: Update Extension build.rs Files (27 files)

     Update each `crates/ext_*/build.rs` to add:

     ```rust
     ExtensionBuilder::host("fs")
         .ts_path("ts/init.ts")
         .ops(&[...])
         .generate_sdk_module("../../sdk")  // NEW: Generate sdk/runtime.fs.ts
         .build()
     ```

     **Files to update:**
     - `crates/ext_fs/build.rs`
     - `crates/ext_net/build.rs`
     - `crates/ext_process/build.rs`
     - `crates/ext_window/build.rs`
     - `crates/ext_ipc/build.rs`
     - `crates/ext_wasm/build.rs`
     - `crates/ext_sys/build.rs`
     - `crates/ext_app/build.rs`
     - `crates/ext_crypto/build.rs`
     - `crates/ext_storage/build.rs`
     - `crates/ext_shell/build.rs`
     - `crates/ext_timers/build.rs`
     - `crates/ext_lock/build.rs`
     - `crates/ext_log/build.rs`
     - `crates/ext_trace/build.rs`
     - `crates/ext_os_compat/build.rs`
     - `crates/ext_path/build.rs`
     - `crates/ext_signals/build.rs`
     - `crates/ext_devtools/build.rs`
     - `crates/ext_webview/build.rs`
     - `crates/ext_database/build.rs`
     - `crates/ext_debugger/build.rs`
     - `crates/ext_display/build.rs`
     - `crates/ext_monitor/build.rs`
     - `crates/ext_protocol/build.rs`
     - `crates/ext_shortcuts/build.rs`
     - `crates/ext_updater/build.rs`

     ### Phase 3: Clean Up (2 tasks)

     1. **Delete manual SDK files** - The auto-generated ones will replace them:
        - Remove all `sdk/runtime.*.ts` files (will be regenerated)

     2. **Verify** - Build the project to generate all SDK files:
        ```bash
        cargo build
        ```

     ---

     ## Key Files to Modify

     | File | Change |
     |------|--------|
     | `crates/forge-weld/src/build/extension.rs` | Add `generate_sdk_module()` method |
     | `crates/ext_*/build.rs` (27 files) | Add `.generate_sdk_module("../../sdk")` call |

     ---

     ## Benefits

     1. **Single Source of Truth**: `ts/init.ts` files define the API, SDK is auto-generated
     2. **Always in Sync**: SDK files regenerated on every build
     3. **No Manual Maintenance**: Developers only edit `ts/init.ts`, SDK updates automatically
     4. **Consistent**: All SDK files follow the same pattern

     ---

     ## Alternative Considered: CLI Tool

     Could create a `forge weld generate-sdk` CLI command, but the build.rs approach is better because:
     - Runs automatically during `cargo build`
     - Uses existing forge-weld infrastructure
     - No extra tools to run

     ---

     ## References

     - **deno_bindgen**: Uses similar pattern (proc macros → IR → codegen)
     - **forge-weld TypeScriptGenerator**: Already generates perfect init.ts format
     - **Current build.rs pattern**: Already uses ExtensionBuilder fluently

     ---

     ## Execution Order

     1. Add `generate_sdk_module()` to `crates/forge-weld/src/build/extension.rs`
     2. Update all 27 `ext_*/build.rs` files
     3. Run `cargo build` to generate SDK files
     4. Verify generated files match expected output
     5. Delete old `sdk/generated/` folder (already done)

═══════════════════════════════════════════ Conversation compacted · ctrl+o for history ══════════════════════════════════════════
  ⎿  Read ../.claude/todos/79714e27-f5e4-4368-8376-c6345e6e0f4d-agent-79714e27-f5e4-4368-8376-c6345e6e0f4d.json (22 lines)
  ⎿  Read ../CLAUDE.md (2 lines)
  ⎿  Read crates/ext_webview/ts/init.ts (98 lines)
  ⎿  Read crates/ext_devtools/ts/init.ts (27 lines)
  ⎿  Read crates/ext_signals/ts/init.ts (43 lines)
  ⎿  Todo list read (4 items)
  ⎿  Plan file referenced (~/.claude/plans/rosy-finding-giraffe.md)

∴ Thinking…

  Looking at the context, I was working on implementing automatic SDK TypeScript file generation via forge-weld. The plan has been
   created at /Users/ryanoboyle/.claude/plans/rosy-finding-giraffe.md.

  The user's last instruction was to figure out how to generate TypeScript files using forge-weld and update it if needed. I've
  completed the research and planning phase.

  Now I need to:

  1. Exit plan mode and implement the solution
  2. Add generate_sdk_module() method to ExtensionBuilder in crates/forge-weld/src/build/extension.rs
  3. Update all 27 extension build.rs files
  4. Run build to auto-generate SDK files

  Let me start by reading the current state of the extension.rs file to implement the changes.

⏺ I'll continue implementing the automatic SDK TypeScript generation feature in forge-weld. Let me start by reading the current state of the extension builder and then implementing the changes.
  ⎿  Limit reached · resets 5am (America/Los_Angeles) · contact an admin to increase limits

  Todos
  ☐ Add generate_sdk_module() method to ExtensionBuilder
  ☐ Update all 27 ext_*/build.rs files to use new method
  ☐ Run cargo build to generate SDK files
  ☐ Verify generated SDK files are correct
