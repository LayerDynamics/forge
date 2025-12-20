# TODO: Placeholder Implementations

> **Project Status:** Alpha (0.1.0-alpha.1)
> **Last Updated:** 2024-12-18
> **Analysis Coverage:** All 32+ `ext_*` extension crates

---

## Summary Statistics

| Category | Count | Percentage |
|----------|-------|------------|
| Fully Stubbed | 4 | 12% |
| Partial Implementation | 4 locations | - |
| Platform Stub | 1 (Linux codesign) | 3% |
| Fully Implemented | 28+ | 85% |

---

## Fully Stubbed Extensions (4 crates)

These extensions only contain basic `_info()` and `_echo()` introspection ops with no actual functionality:

| Extension | File | Lines | Expected Functionality |
|-----------|------|-------|------------------------|
| `ext_database` | `crates/ext_database/src/lib.rs` | 20-33 | SQLite/database operations |
| `ext_display` | `crates/ext_display/src/lib.rs` | 18-31 | Display/screen management |
| `ext_shortcuts` | `crates/ext_shortcuts/src/lib.rs` | 18-31 | Global keyboard shortcuts |
| `ext_updater` | `crates/ext_updater/src/lib.rs` | 18-31 | App update check/install |

**Stub Pattern:**

```rust
pub fn op_<name>_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "<name>".to_string(),
        version: "0.1.0".to_string(),
        status: "stub".to_string(),  // <-- Indicates placeholder
    }
}

pub fn op_<name>_echo(message: String) -> String {
    message  // Only echoes input
}
```

---

## Partial Implementations (4 locations)

### 1. ext_weld - Source Map Support

| | |
|---|---|
| **File** | `crates/ext_weld/src/lib.rs` |
| **Line** | 294 |
| **Code** | `source_map: None, // TODO: implement source map support` |
| **Impact** | TypeScript transpilation lacks debugging source maps |
| **Priority** | Medium - affects debugging experience |

### 2. ext_etcher - Runtime Rust Parsing

| | |
|---|---|
| **File** | `crates/ext_etcher/src/lib.rs` |
| **Lines** | 307-314 |
| **Function** | `op_etcher_parse_rust()` |
| **Behavior** | Returns error: "Direct Rust parsing not yet implemented" |
| **Workaround** | Use `generateDocs()` with `rust_source` option at build time |
| **Priority** | Low - workaround exists |

### 3. ext_etcher - Merge Nodes (Rust disabled)

| | |
|---|---|
| **File** | `crates/ext_etcher/src/lib.rs` |
| **Lines** | 345-350 |
| **Function** | `op_etcher_merge_nodes()` |
| **Issue** | Rust source parsing disabled; only TypeScript nodes merged |
| **Note** | By design - Rust metadata collected at build time via weld inventory |
| **Priority** | Low - intentional design decision |

### 4. ext_shell - Here-Document Parsing

| | |
|---|---|
| **File** | `crates/ext_shell/src/parser.rs` |
| **Line** | 1019 |
| **Code** | `// TODO: Proper here-doc parsing` |
| **Impact** | `<<EOF` style redirections fall back to simple file word parsing |
| **Priority** | Low - edge case in shell parsing |

---

## Platform-Specific Stubs

### ext_codesign - Linux Implementation

**File:** `crates/ext_codesign/src/os_linux.rs`

| Function | Line | Behavior |
|----------|------|----------|
| `sign()` | 11 | Returns `PlatformUnsupported` error |
| `sign_adhoc()` | 18 | Returns `PlatformUnsupported` error |
| `verify()` | 24-33 | Returns dummy success result |
| `get_entitlements()` | 38 | Returns `PlatformUnsupported` error |
| `list_identities()` | 44-45 | Returns empty `Vec` |
| `get_identity_info()` | 50 | Returns `PlatformUnsupported` error |

> **Note:** Expected behavior - native code signing only exists on macOS/Windows. Linux apps typically use other distribution mechanisms (AppImage, Flatpak, Snap).

---

## Fully Implemented Extensions (28+ crates)

These extensions have complete or substantial implementations:

### Core Extensions

- `ext_app` - App lifecycle, info
- `ext_fs` - File system operations
- `ext_ipc` - Inter-process communication
- `ext_net` - Network operations
- `ext_process` - Child process spawning
- `ext_sys` - System info, clipboard, notifications
- `ext_window` - Window management

### Build & Development

- `ext_bundler` - Asset bundling
- `ext_codesign` (macOS/Windows) - Code signing
- `ext_devtools` - Developer tools
- `ext_etcher` - Documentation generation
- `ext_svelte` - Svelte compilation
- `ext_weld` - TypeScript binding generation

### Debugging & Monitoring (Recently Implemented)

- `ext_debugger` - Runtime debugging with breakpoints, scripts, call frames tracking
  - `DebuggerState` with connection, enabled, paused state
  - Breakpoint management (`HashMap<String, Breakpoint>`)
  - Script tracking (`HashMap<String, ScriptInfo>`)
  - Call frame snapshots for paused state
- `ext_monitor` - System metrics and monitoring
  - Real-time CPU, memory, disk, network metrics via `sysinfo`
  - Event loop latency measurement
  - Subscription-based metric polling
- `ext_protocol` - URL protocol registration (platform-specific handlers)
- `ext_trace` - Span-based tracing/telemetry
  - Active and finished span tracking
  - `SpanRecord` with timing, attributes, results
- `ext_web_inspector` - Chrome DevTools Protocol (CDP) integration
  - Custom CDP domains: Forge.Monitor, Forge.Trace, Forge.Signals, Forge.Runtime
  - Extension bridges for cross-extension data aggregation
  - Platform adapters (WebKit, WebView2, WebKitGTK)
  - Real-time event streaming
  - DevTools panel UI (HTML/JS/CSS)

### Utilities

- `ext_crypto` - Cryptographic operations
- `ext_encoding` - Text encoding/decoding
- `ext_image_tools` - Image processing
- `ext_lock` - File/resource locking
- `ext_log` - Logging infrastructure
- `ext_os_compat` - OS compatibility layer
- `ext_path` - Path manipulation
- `ext_shell` - Shell command execution
- `ext_signals` - OS signal handling (Unix: SIGINT, SIGTERM, etc.)
  - Signal subscription system
  - Async signal event streaming
- `ext_storage` - Key-value storage
- `ext_timers` - Timer operations
- `ext_wasm` - WebAssembly loading
- `ext_webview` - WebView control

---

## TypeScript Layer Status

All `ts/init.ts` files are **fully implemented** with proper type definitions and op wrappers.

Single documentation note:

- **File:** `crates/ext_etcher/ts/init.ts:256`
- **Note:** Documents that `parseRust()` delegates to unimplemented Rust op

---

## Implementation Priority Suggestions

### High Priority (User-Facing Features)

1. **ext_shortcuts** - Global hotkeys are common in desktop apps
2. **ext_updater** - Auto-update is essential for distribution

### Medium Priority (Developer Experience)

1. **ext_weld source maps** - Improves TypeScript debugging
2. **ext_database** - Many apps need local SQLite data persistence

### Low Priority (Specialized Use Cases)

1. **ext_display** - Multi-display/screen management

---

## Recently Completed (2024-12-18)

The following extensions were previously listed as stubs but are now fully implemented:

| Extension | What Was Added |
|-----------|----------------|
| `ext_monitor` | Full system metrics (CPU, memory, disk, network) via sysinfo, event loop latency, subscriptions |
| `ext_debugger` | DebuggerState with breakpoints, scripts, call frames, connection/pause state |
| `ext_protocol` | Platform-specific URL protocol handlers (macOS, Windows, Linux) |
| `ext_trace` | TraceState with active/finished span tracking, SpanRecord with attributes |
| `ext_signals` | SignalsState with subscription tracking, Unix signal support |
| `ext_web_inspector` | **NEW** - CDP integration, extension bridges, platform adapters, DevTools panel UI |
