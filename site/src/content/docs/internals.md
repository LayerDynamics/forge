---
title: Implementation Reference
description: Detailed implementation reference with file paths and line numbers for Forge internals.
slug: internals
---

This document provides a detailed implementation reference for Forge contributors, with specific file paths and line numbers for all major components.

> **Note**: Line numbers are approximate and may shift as the codebase evolves. Use them as starting points for exploration.

---

## 1. Runtime System

### 1.1 Entry Point & Event Loop

**File:** `crates/forge-runtime/src/main.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `main()` | 413-422 | Entry point, creates tokio runtime |
| `sync_main()` | 424-962 | Main execution with event loop |

**sync_main() Execution Flow:**

| Stage | Lines | Purpose |
|-------|-------|---------|
| Tracing init | 425-432 | Logging setup (FORGE_LOG env) |
| Argument parsing | 434-450 | `--app-dir`, `--dev` flags |
| Bundle detection | 452-494 | macOS .app structure detection |
| Manifest loading | 496-500 | Parse `manifest.app.toml` |
| Crash reporting | 508-528 | Initialize crash reporting |
| Capabilities | 530-536 | Create from manifest permissions |
| Capability adapters | 538-539 | Create adapters for extensions |
| IPC channels | 541-549 | to_deno_tx, to_renderer_tx, window_cmd channels |
| JsRuntime creation | 551-564 | Deno runtime with all extensions |
| App info | 566-576 | Build app metadata |
| Extension states | 578-614 | Tiered initialization |
| Module loading | 616-632 | Load `src/main.ts` |
| Event loop | 643-960 | tao event loop |

**Event Loop Handlers (809-961):**

| Event | Lines | Purpose |
|-------|-------|---------|
| `MainEventsCleared` | 817-855 | JS runtime polling |
| `UserEvent::ToRenderer` | 857-863 | IPC to renderer |
| `WindowEvent::CloseRequested` | 869-882 | Window close |
| `WindowEvent::Focused` | 885-900 | Focus/blur events |
| `WindowEvent::Resized` | 902-924 | Resize events |
| `WindowEvent::Moved` | 926-948 | Move events |
| `WindowCmd` | 953-956 | WindowManager delegation |

### 1.2 Module Loader

**File:** `crates/forge-runtime/src/main.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `ForgeModuleLoader` struct | 187-196 | Custom ES module loader |
| `resolve()` | 199-215 | Resolves `runtime:*` → `ext:runtime_*/init.js` |
| `load()` | 217-300 | Loads and transpiles TS/JS modules |

**Module Resolution Pattern:**
```
runtime:fs → ext:runtime_fs/init.js
```

### 1.3 HMR Server

**File:** `crates/forge-runtime/src/main.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `run_hmr_server()` | 304-411 | Dev mode WebSocket server |

- Listens on `ws://127.0.0.1:35729`
- Watches `web/` directory
- Broadcasts `css:` or `reload:` messages

### 1.4 Asset Provider & Helpers

**File:** `crates/forge-runtime/src/main.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `ForgeAssetProvider` | 76-96 | Provides assets to WindowManager |
| `ForgeChannelChecker` | 99-109 | Channel permission enforcement |
| `preload_js()` | 63-66 | Returns transpiled preload script |
| `warm_up_window_helpers()` | 116-182 | Validates window setup |

---

## 2. Capabilities & Permissions

**File:** `crates/forge-runtime/src/capabilities.rs`

### 2.1 Permission Structures

| Struct | Lines | Purpose |
|--------|-------|---------|
| `Permissions` | 11-19 | Root permissions from manifest |
| `FsPermissions` | 21-25 | Read/write path globs |
| `NetPermissions` | 27-33 | Allow/deny hosts, listen ports |
| `UiPermissions` | 35-43 | Windows, menus, dialogs, tray, channels |
| `SysPermissions` | 45-51 | Clipboard, notify, power, env |
| `EnvPermissions` | 54-60 | Environment variable patterns |
| `ProcessPermissions` | 62-71 | Binary allowlist, max processes |
| `WasmPermissions` | 74-82 | WASM load/preopen, max instances |

### 2.2 Capabilities Struct

| Method | Lines | Purpose |
|--------|-------|---------|
| `from_permissions()` | 125-186 | Creates capabilities from manifest |
| `compile_patterns()` | 188-211 | Compiles filesystem glob patterns |
| `compile_host_patterns()` | 213-236 | Compiles hostname patterns |
| `compile_simple_patterns()` | 238-259 | Compiles env/process patterns |

### 2.3 Permission Check Methods

| Method | Lines | Purpose |
|--------|-------|---------|
| `check_fs_read()` | 261-284 | Filesystem read permission |
| `check_fs_write()` | 286-309 | Filesystem write permission |
| `check_net()` | 311-344 | Network access (deny precedence) |
| `check_net_listen()` | 346-369 | Network listening on ports |
| `check_ui_windows()` | 371-383 | Window creation permission |
| `check_ui_menus()` | 385-397 | Menu permission |
| `check_ui_dialogs()` | 399-411 | Dialog permission |
| `check_ui_tray()` | 413-417 | Tray permission |
| `check_sys_clipboard()` | 419-431 | Clipboard permission |
| `check_sys_notify()` | 433-445 | Notification permission |
| `check_sys_power()` | 447-453 | Power management permission |
| `check_env_read()` | 455-478 | Environment variable read |
| `check_env_write()` | 480-500 | Environment variable write |
| `check_process_spawn()` | 502-525 | Process spawning permission |
| `check_process_env()` | 527-549 | Process env inheritance |
| `check_channel()` | 551-591 | IPC channel permission |
| `check_wasm_load()` | 603-627 | WASM loading permission |
| `check_wasm_preopen()` | 629-651 | WASM preopen permission |

### 2.4 Capability Adapters

| Adapter | Lines | Purpose |
|---------|-------|---------|
| `CapabilityAdapters` struct | 665-674 | Container for all adapters |
| `FsCapabilityAdapter` | 676-699 | ext_fs checker |
| `NetCapabilityAdapter` | 701-722 | ext_net checker |
| `SysCapabilityAdapter` | 724-771 | ext_sys checker |
| `WindowCapabilityAdapter` | 773-813 | ext_window checker |
| `IpcCapabilityAdapter` | 815-836 | ext_ipc checker |
| `ProcessCapabilityAdapter` | 838-861 | ext_process checker |
| `WasmCapabilityAdapter` | 863-886 | ext_wasm checker |
| `create_capability_adapters()` | 888-900 | Factory function |

---

## 3. Extension Registry

**File:** `crates/forge-runtime/src/ext_registry.rs`

### 3.1 Core Types

| Type | Lines | Purpose |
|------|-------|---------|
| `ExtensionTier` enum | 26-36 | Initialization complexity levels |
| `ExtensionDescriptor` | 38-51 | Extension metadata + factory |
| `IpcChannels` | 53-59 | Channel containers for IPC |
| `WindowChannels` | 61-68 | Channel containers for window ops |
| `ExtensionInitContext` | 70-93 | Context for initialization |
| `ExtensionRegistry` | 113-116 | Central registry |

### 3.2 Tier System

**Tier 0 (ExtensionOnly):** No state required
- database, debugger, devtools, display, lock, log, monitor, os_compat, path, protocol, shortcuts, signals, updater, webview

**Tier 1 (SimpleState):** No dependencies
- timers, trace, weld, bundler, etcher

**Tier 2 (CapabilityBased):** Needs capability adapters
- fs (required), net, sys, crypto, storage

**Tier 3 (ComplexContext):** Needs channels/app_info
- ipc (required), window (required), process, wasm, app (required), shell

### 3.3 Registry Methods

| Method | Lines | Purpose |
|--------|-------|---------|
| `ExtensionRegistry::new()` | 119-124 | Creates registry with all descriptors |
| `all()` | 127-130 | Returns all descriptors |
| `by_tier()` | 132-135 | Filters by tier |
| `required()` | 138-141 | Returns required extensions |
| `count()` | 143-146 | Returns count |
| `build_extensions()` | 148-160 | Builds Extension vector for JsRuntime |
| `init_all_states()` | 162-189 | Initializes states in tier order |
| `create_all_descriptors()` | 202-428 | Defines all 28 extensions |

### 3.4 State Initialization Dispatchers

| Function | Lines | Purpose |
|----------|-------|---------|
| `init_simple_state()` | 434-460 | Routes Tier 1 initialization |
| `init_capability_state()` | 462-500 | Routes Tier 2 initialization |
| `init_complex_state()` | 502-566 | Routes Tier 3 initialization |
| `init_ipc_manually()` | 572-583 | Manual IPC initialization |
| `init_window_manually()` | 585-597 | Manual window initialization |

---

## 4. CLI System

**File:** `crates/forge_cli/src/main.rs`

### 4.1 Entry Point

| Function | Lines | Purpose |
|----------|-------|---------|
| `main()` | 1024-1095 | CLI dispatcher |
| `usage()` | 11-45 | Help text |
| `find_forge_host()` | 54-96 | Locates forge-runtime binary |

### 4.2 Commands

| Command | Function | Lines | Purpose |
|---------|----------|-------|---------|
| `dev` | `cmd_dev()` | 98-132 | Development mode |
| `build` | `cmd_build()` | 572-701 | Production build |
| `bundle` | `cmd_bundle()` | 719-752 | Platform packaging |
| `sign` | `cmd_sign()` | 754-814 | Code signing |
| `icon` | `cmd_icon()` | 992-1022 | Icon management |

### 4.3 Build Pipeline

| Function | Lines | Purpose |
|----------|-------|---------|
| `detect_framework()` | 142-179 | React/Vue/Svelte/Minimal detection |
| `bundle_with_esbuild()` | 202-329 | esbuild via Deno |
| `transform_vue_files()` | 331-424 | Vue SFC compilation |
| `transform_svelte_files()` | 426-545 | Svelte compilation |

### 4.4 Icon Commands

| Function | Lines | Purpose |
|----------|-------|---------|
| `cmd_icon()` | 992-1022 | Icon subcommand dispatcher |
| `cmd_icon_create()` | 833-866 | Creates placeholder icon |
| `cmd_icon_validate()` | 868-990 | Validates icon requirements |

---

## 5. Bundler System

### 5.1 Bundler Module

**File:** `crates/forge_cli/src/bundler/mod.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `build_embedded_binary()` | 55-91 | Builds with FORGE_EMBED_DIR |
| `bundle()` | 111-139 | Platform dispatcher |
| `parse_manifest()` | 141-144 | Convenience wrapper |

### 5.2 Manifest Configuration

**File:** `crates/forge_cli/src/bundler/manifest.rs`

| Struct | Lines | Purpose |
|--------|-------|---------|
| `AppManifest` | 12-22 | Full manifest with bundle config |
| `AppConfig` | 25-30 | App name, identifier, version |
| `BundleConfig` | 41-54 | Platform bundle settings |
| `WindowsBundleConfig` | 56-74 | MSIX settings, signing |
| `MacosBundleConfig` | 76-96 | DMG/PKG settings, notarization |
| `LinuxBundleConfig` | 98-114 | AppImage settings |

### 5.3 macOS Bundler

**File:** `crates/forge_cli/src/bundler/macos.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `MacosBundler` struct | 28-49 | macOS bundling orchestrator |
| `bundle()` | 52-107 | Main pipeline |
| `create_app_bundle()` | 110-150 | Creates .app structure |

### 5.4 Windows Bundler

**File:** `crates/forge_cli/src/bundler/windows.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `WindowsBundler` struct | 24-45 | Windows bundling orchestrator |
| `bundle()` | 48-67 | Main pipeline |
| `bundle_portable()` | 76-160 | Standalone exe creation |

### 5.5 Linux Bundler

**File:** `crates/forge_cli/src/bundler/linux.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `LinuxBundler` struct | 18-39 | Linux bundling orchestrator |
| `bundle()` | 42-64 | Main pipeline |
| `create_appdir()` | 67-146 | AppDir per freedesktop.org |

### 5.6 Code Signing

**File:** `crates/forge_cli/src/bundler/codesign.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `SigningConfig` | 15-30 | Configuration struct |
| `sign()` | 72-91 | Unified dispatcher |
| `sign_macos_bundle()` | 98-141 | macOS signing |
| `sign_single_macos()` | 144-150 | Single target signing |

### 5.7 Icon Processing

**File:** `crates/forge_cli/src/bundler/icons.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `MIN_ICON_SIZE` | const | 512px minimum |
| `RECOMMENDED_ICON_SIZE` | const | 1024px recommended |
| `IconValidation` | 31-44 | Validation result |
| `IconProcessor` | 46-49 | Icon processing utility |
| `MsixIconSet` | 52-98 | Windows icon set |

---

## 6. Forge-Weld Binding System

### 6.1 ExtensionBuilder

**File:** `crates/forge-weld/src/build/extension.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `ExtensionBuilder` struct | 91-101 | Builder for extensions |
| `new()` | 109-120 | Create builder |
| `host()` | 126-137 | Shorthand for host modules |
| `ops()` | 146-153 | Register op names |
| `use_inventory_types()` | 175-178 | Enable linkme merging |
| `generate_sdk_types()` | 162-165 | Enable .d.ts generation |
| `generate_sdk_module()` | 168-171 | Enable .ts SDK |
| `dts_generator()` | 187-193 | Custom DTS generator |
| `build()` | 262-421 | Main build process |
| `export_types()` | 493-514 | Add exports to declarations |

**build() Process:**
1. Get OUT_DIR and CARGO_MANIFEST_DIR (263-269)
2. Merge inventory types if enabled (271-290)
3. Transpile TypeScript if ts_path set (292-307)
4. Generate Rust extension.rs (309-312)
5. Generate .d.ts if SDK path requested (314-339)
6. Generate .ts SDK module if requested (341-372)
7. Save module metadata for documentation (374-410)

### 6.2 TypeScript Transpilation

**File:** `crates/forge-weld/src/build/transpile.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `transpile_ts()` | 43-64 | Transpiles TS string to JS |
| `transpile_file()` | 78-91 | Transpiles TS file |

### 6.3 Type System

**File:** `crates/forge-weld/src/ir/types.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `WeldPrimitive` enum | 11-33 | Rust → TS primitive mapping |
| `to_typescript()` | 36-52 | TS type string generation |
| `WeldType` enum | 85-174 | Comprehensive type system |

**WeldType Variants:**
- `Primitive(WeldPrimitive)` → number/bigint/string/boolean
- `Option(Box<WeldType>)` → `T | null`
- `Vec(Box<WeldType>)` → `T[]`
- `Result { ok, err }` → `Promise<T>`
- `HashMap { key, value }` → `Record<K, V>`
- `Tuple(Vec<WeldType>)` → `[T, U, V]`
- `Struct(String)` → Interface reference
- `Enum(String)` → Enum type reference
- Plus: JsonValue, OpState, Box, Arc, Rc, RefCell, Mutex, RwLock, Reference, Pointer, Never

### 6.4 Symbol Metadata

**File:** `crates/forge-weld/src/ir/symbol.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `ParamAttr` enum | 12-26 | #[string], #[serde], etc. |
| `OpParam` struct | 60-73 | Parameter metadata |
| `to_typescript_param()` | 120-123 | Generates TS param |
| `OpSymbol` struct | 128-145 | Op metadata |

### 6.5 Symbol Registry (Linkme)

**File:** `crates/forge-weld/src/ir/inventory.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `WELD_OPS` | 10 | Distributed slice for ops |
| `WELD_STRUCTS` | 14 | Distributed slice for structs |
| `WELD_ENUMS` | 18 | Distributed slice for enums |
| `SymbolRegistry` | 36-102 | Registry struct |
| `from_inventory()` | 50-56 | Loads from linkme slices |
| `collect_ops()` | 21-23 | Iterates WELD_OPS |
| `register_op!()` macro | 106-112 | Op registration |
| `register_struct!()` macro | 116-124 | Struct registration |
| `register_enum!()` macro | 128-135 | Enum registration |

### 6.6 Module Metadata

**File:** `crates/forge-weld/src/ir/module.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `WeldModule` struct | 11-38 | Module metadata |
| `host()` | 60-64 | Factory for host modules |
| `op_names()` | 127-129 | Returns op names |

---

## 7. Code Generators

### 7.1 Extension Generator

**File:** `crates/forge-weld/src/codegen/extension.rs`

| Method | Lines | Purpose |
|--------|-------|---------|
| `ExtensionGenerator` struct | 9-11 | Generator wrapper |
| `generate()` | 30-53 | Generates `deno_core::extension!()` |
| `generate_with_state()` | 56-81 | With state init function |
| `generate_with_deps()` | 84-114 | With dependencies |
| `generate_with_esm_files()` | 117-149 | Multiple ESM files |

### 7.2 TypeScript Generator

**File:** `crates/forge-weld/src/codegen/typescript.rs`

| Method | Lines | Purpose |
|--------|-------|---------|
| `TypeScriptGenerator` struct | 8-10 | Generator wrapper |
| `generate()` | 19-54 | Full TS module generation |
| `generate_interface()` | 62-108 | Interface from WeldStruct |
| `generate_enum()` | 111-167 | Enum generation |
| `generate_export_function()` | 170+ | Function wrappers |

### 7.3 DTS Generator

**File:** `crates/forge-weld/src/codegen/dts.rs`

| Method | Lines | Purpose |
|--------|-------|---------|
| `DtsGenerator` struct | 9-11 | Generator wrapper |
| `generate()` | 20-65 | Declaration module |
| `generate_interface()` | 68-115 | Interface declarations |
| `generate_function_declaration()` | 160+ | Function signatures |

---

## 8. Proc Macros

### 8.1 #[weld_op] Macro

**File:** `crates/forge-weld-macro/src/weld_op.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `weld_op` entry | 149-241 | Main macro implementation |
| `extract_params()` | 44-76 | Extracts parameters |
| `to_camel_case()` | 87-103 | Snake to camelCase |
| `op_name_to_ts()` | 106-118 | Converts op_* to camelCase |
| `extract_doc_comments()` | 124-147 | Extracts /// comments |

**Macro Expansion Process:**
1. Parse `ItemFn` (150-152)
2. Clone and strip weld_op attribute (154-157)
3. Parse attributes (async, ts_name) (159)
4. Determine async from signature/attribute (165)
5. Generate TS name via `op_name_to_ts()` (168)
6. Extract doc comments (171)
7. Extract parameters (skip OpState) (174)
8. Extract return type (177-184)
9. Generate metadata function name (187)
10. Generate expanded code (220-237)
11. Register via `register_op!()` (237)

### 8.2 #[weld_struct] / #[weld_enum]

**File:** `crates/forge-weld-macro/src/weld_struct.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `to_camel_case()` | 56-62 | Snake to camelCase |
| `weld_struct_impl()` | 80-171 | Struct metadata generation |
| `weld_enum_impl()` | 173-248 | Enum metadata generation |

### 8.3 Type Parser

**File:** `crates/forge-weld-macro/src/type_parser.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `rust_type_to_weld_type()` | 27-121 | Main conversion |
| `parse_path_type()` | 124-181 | Path type parsing |

**Supported Types:**
- **Primitives** (185-204): u8-u64, i8-i64, f32, f64, bool, String, char, ()
- **Containers** (210-239): Option, Vec, Result
- **Maps** (241-267): HashMap, BTreeMap
- **Sets** (269-275): HashSet, BTreeSet
- **Wrappers** (277-299): Box, Arc, Rc, RefCell, Mutex, RwLock

---

## 9. Extension Implementation Pattern

### 9.1 Example: ext_fs

**Files:**
- `crates/ext_fs/src/lib.rs` - Op definitions
- `crates/ext_fs/build.rs` - Extension builder config
- `crates/ext_fs/ts/init.ts` - TypeScript shim

**Error System (lib.rs):**

| Component | Lines | Purpose |
|-----------|-------|---------|
| `FsErrorCode` enum | 20-44 | Error code enum |
| `FsError` types | 46-88 | Error types with `#[deno_error::JsError]` |
| Error constructors | 90-172 | Helpers: `io()`, `permission_denied()` |

**State Management:**

| Component | Lines | Purpose |
|-----------|-------|---------|
| `FileEvent` struct | 179-184 | File change event |
| `WatchEntry` | 187-189 | Watcher with receiver |
| `FsWatchState` | 192-196 | Active watchers by ID |

### 9.2 Example: ext_window

**File:** `crates/ext_window/src/lib.rs`

| Function | Lines | Purpose |
|----------|-------|---------|
| `op_window_create` | 652-700 | Creates new window |
| `op_window_close` | 703-727 | Closes window |
| `op_window_minimize` | 730-746 | Minimizes window |

**WindowManager (src/manager.rs):**

| Component | Lines | Purpose |
|-----------|-------|---------|
| `WindowManager<U>` struct | 50-83 | Core manager |
| `new()` | 90-119 | Initialization |
| `menu_id_map()` | 122-124 | Menu tracking |
| `pending_ctx_menu()` | 127-129 | Context menu state |
| `is_empty()` | 132-134 | Check all windows closed |

### 9.3 Example: ext_ipc

**File:** `crates/ext_ipc/src/lib.rs`

| Component | Lines | Purpose |
|-----------|-------|---------|
| `IpcEvent` struct | 82-90 | Message container |
| `ToRendererCmd` enum | 92-100 | Outbound commands |
| `IpcState` | 107-110 | Channel state |
| `IpcCapabilityChecker` trait | 117-142 | Permission checking |
| `op_ipc_send` | 174-210 | Send to renderer |
| `op_ipc_recv` | 213-258 | Receive from renderer |
| `init_ipc_state()` | 272-282 | State registration |
| `init_ipc_capabilities()` | 284-292 | Capability registration |

---

## 10. IPC Communication Flow

### 10.1 Channel Setup

**File:** `crates/forge-runtime/src/main.rs`

```
Lines 541-549: Channel creation
- to_deno_tx/rx: IpcEvent channel (256 capacity)
- to_renderer_tx/rx: ToRendererCmd channel (256 capacity)
- window_cmd_tx/rx: WindowCmd channel (256 capacity)
```

### 10.2 Renderer → Deno Flow

```
1. Renderer: window.host.send(channel, payload)
   Location: sdk/preload.ts:141-146

2. WebView IPC → native bridge

3. IpcEvent created, sent to to_deno_tx
   Location: crates/forge-runtime/src/main.rs:719

4. op_ipc_recv() awaits on receiver
   Location: crates/ext_ipc/src/lib.rs:213-258

5. App code: for await (const event of windowEvents())
```

### 10.3 Deno → Renderer Flow

```
1. App: sendToWindow(windowId, channel, payload)
   Location: sdk/runtime.ipc.ts:64-70

2. op_ipc_send() sends ToRendererCmd::Send
   Location: crates/ext_ipc/src/lib.rs:174-210

3. Main.rs forwards to event loop
   Location: crates/forge-runtime/src/main.rs:661-668

4. Event handler calls window_manager.send_to_renderer()
   Location: crates/forge-runtime/src/main.rs:857-864

5. WebView evaluates: window.__host_dispatch({channel, payload})

6. Preload dispatches to listeners
   Location: sdk/preload.ts:95-105
```

---

## 11. SDK Structure

### 11.1 Generated Files

**Location:** `sdk/`

| File | Source | Purpose |
|------|--------|---------|
| `runtime.fs.ts` | ext_fs | Filesystem ops |
| `runtime.window.ts` | ext_window | Window management |
| `runtime.ipc.ts` | ext_ipc | IPC communication |
| `runtime.net.ts` | ext_net | Networking |
| `runtime.sys.ts` | ext_sys | System info |
| `runtime.*.ts` | ext_* | 30 total modules |

### 11.2 Preload System

**File:** `sdk/preload.ts` (191 lines)

| Component | Lines | Purpose |
|-----------|-------|---------|
| IIFE wrapper | 88-190 | Main bridge implementation |
| `__host_dispatch` | 95-105 | Native → JS dispatch |
| `globalThis.host` | 108-147 | Public bridge API |
| `__console__` listener | 152-177 | Console forwarding |
| HMR client | 179-189 | Dev mode hot reload |

**Public API:**
- `window.host.on(channel, cb)` - Register listener
- `window.host.off(channel, cb)` - Unregister listener
- `window.host.emit(channel, payload)` - Local emit + send to Deno
- `window.host.send(channel, payload)` - Send to Deno only

---

## 12. Quick Reference

### Where to Look

| Task | Location |
|------|----------|
| Add a new op | `crates/ext_*/src/lib.rs` - use `#[weld_op]` |
| Add new extension | Create `crates/ext_<name>/`, follow ext_fs pattern |
| Modify window behavior | `crates/ext_window/src/manager.rs` |
| Change IPC handling | `crates/ext_ipc/src/lib.rs` |
| Modify CLI commands | `crates/forge_cli/src/main.rs` |
| Change permissions | `crates/forge-runtime/src/capabilities.rs` |
| Modify module loading | `crates/forge-runtime/src/main.rs:187-300` |
| Add bundler format | `crates/forge_cli/src/bundler/{platform}.rs` |
| Change type generation | `crates/forge-weld/src/codegen/*.rs` |
| Modify proc macros | `crates/forge-weld-macro/src/*.rs` |

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    forge dev my-app                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  forge_cli/src/main.rs:98 - cmd_dev()                       │
│  - Locates forge-runtime binary                             │
│  - Launches with --app-dir and --dev flags                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  forge-runtime/src/main.rs:424 - sync_main()                │
│  - Parse manifest.app.toml                                  │
│  - Create Capabilities from permissions                     │
│  - Create IPC channels                                      │
│  - Build Deno JsRuntime with 28 extensions                  │
│  - Load src/main.ts                                         │
│  - Start tao event loop                                     │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   Deno JsRuntime        │     │   tao/wry Event Loop    │
│   - Executes TS/JS      │◄───►│   - Window management   │
│   - runtime:* modules   │     │   - WebView rendering   │
│   - IPC send/receive    │     │   - System events       │
└─────────────────────────┘     └─────────────────────────┘
              │                               │
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   App Code (main.ts)    │     │   WebView (index.html)  │
│   import "runtime:*"    │◄───►│   window.host.send()    │
│   createWindow()        │ IPC │   window.host.on()      │
│   sendToWindow()        │     │   preload.ts bridge     │
└─────────────────────────┘     └─────────────────────────┘
```
