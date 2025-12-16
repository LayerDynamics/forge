# SDK Gap Analysis Report

This report identifies missing methods, processes, utilities, tooling, and framework logic issues in the Forge SDK compared to the capabilities implemented (or that should be implemented) in the ext_* modules.

## Executive Summary

| Category | Gaps Found | Severity |
|----------|------------|----------|
| Missing Rust Ops | 45+ | HIGH |
| Missing SDK Modules | 3 | HIGH |
| Framework Logic Issues | 12 | HIGH |
| Missing Utilities | 8 | MEDIUM |
| API Inconsistencies | 6 | MEDIUM |
| Documentation Gaps | 5 | LOW |

---

## 1. Missing SDK Module Implementations (HIGH SEVERITY)

### 1.1 runtime:ipc Module - NO IMPLEMENTATION FILE

**Location Gap:** `sdk/generated/host.ipc.d.ts` exists but `sdk/runtime.ipc.ts` does NOT exist.

The type definitions declare:

```typescript
// From sdk/generated/host.ipc.d.ts
export function sendToWindow(windowId: string, channel: string, payload?: unknown): Promise<void>;
export function recvWindowEvent(): Promise<IpcEvent | null>;
export function windowEvents(): AsyncGenerator<IpcEvent, void, unknown>;
export function windowEventsFor(windowId: string): AsyncGenerator<IpcEvent, void, unknown>;
export function channelEvents(channel: string): AsyncGenerator<IpcEvent, void, unknown>;
export function onEvent(callback: IpcEventCallback): () => void;
export function onChannel(channel: string, callback: ChannelCallback): () => void;
export function broadcast(windowIds: string[], channel: string, payload?: unknown): Promise<void>;
```

**Required:** Create `sdk/runtime.ipc.ts` with implementations for all declared functions.

### 1.2 Missing Rust Ops for runtime:ipc

The `ext_ipc/src/lib.rs` module should implement:

| Op Name | Purpose | Status |
|---------|---------|--------|
| `op_ipc_send` | Send to window | Likely exists |
| `op_ipc_recv` | Receive event | Likely exists |
| `op_ipc_broadcast` | Send to multiple windows | **MISSING** |

---

## 2. Missing Filesystem Operations (ext_fs)

### 2.1 Missing Rust Ops

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_fs_symlink` | Create symbolic links | HIGH |
| `op_fs_read_link` | Read symlink target | HIGH |
| `op_fs_chmod` | Change file permissions | MEDIUM |
| `op_fs_chown` | Change file ownership | LOW |
| `op_fs_truncate` | Truncate file to size | MEDIUM |
| `op_fs_append_text` | Append text to file | HIGH |
| `op_fs_append_bytes` | Append bytes to file | HIGH |
| `op_fs_metadata` | Full metadata (timestamps, permissions) | HIGH |
| `op_fs_real_path` | Resolve symlinks/normalize path | HIGH |
| `op_fs_temp_file` | Create temporary file | MEDIUM |
| `op_fs_temp_dir` | Create temporary directory | MEDIUM |

### 2.2 SDK Type Gaps

**FileStat interface is incomplete:**

```typescript
// Current (sdk/runtime.fs.ts:5-10)
export interface FileStat {
  is_file: boolean;
  is_dir: boolean;
  size: number;
  readonly: boolean;
}

// Should include:
export interface FileStat {
  is_file: boolean;
  is_dir: boolean;
  is_symlink: boolean;         // MISSING
  size: number;
  readonly: boolean;
  created_at?: number;         // MISSING (timestamp)
  modified_at?: number;        // MISSING (timestamp)
  accessed_at?: number;        // MISSING (timestamp)
  permissions?: number;        // MISSING (Unix mode)
}
```

---

## 3. Missing Network Operations (ext_net)

### 3.1 Missing Rust Ops (HIGH PRIORITY)

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_net_websocket_connect` | Connect to WebSocket server | HIGH |
| `op_net_websocket_send` | Send WebSocket message | HIGH |
| `op_net_websocket_recv` | Receive WebSocket message | HIGH |
| `op_net_websocket_close` | Close WebSocket connection | HIGH |
| `op_net_tcp_connect` | TCP client connection | MEDIUM |
| `op_net_tcp_listen` | TCP server listener | MEDIUM |
| `op_net_tcp_accept` | Accept TCP connection | MEDIUM |
| `op_net_udp_bind` | Bind UDP socket | LOW |
| `op_net_udp_send` | Send UDP packet | LOW |
| `op_net_udp_recv` | Receive UDP packet | LOW |
| `op_net_dns_resolve` | DNS lookup | MEDIUM |
| `op_net_fetch_stream` | Streaming fetch response | HIGH |

### 3.2 SDK Feature Gaps

**Missing in host.net.ts:**

```typescript
// WebSocket support - MISSING
export interface WebSocket {
  send(data: string | Uint8Array): Promise<void>;
  recv(): Promise<string | Uint8Array | null>;
  close(): Promise<void>;
  readonly readyState: number;
}
export function connectWebSocket(url: string): Promise<WebSocket>;

// Streaming responses - MISSING
export function fetchStream(url: string, opts?: FetchOptions): Promise<ReadableStream<Uint8Array>>;

// Form data / multipart - MISSING
export function postFormData(url: string, data: FormData): Promise<FetchResponse>;
```

---

## 4. Missing System Operations (ext_sys)

### 4.1 Missing Rust Ops

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_sys_all_env` | Get all environment variables | HIGH |
| `op_sys_delete_env` | Unset environment variable | HIGH |
| `op_sys_shell_open` | Open URL/file with default app | HIGH |
| `op_sys_set_cwd` | Change working directory | MEDIUM |
| `op_sys_user_info` | Get username, uid, gid | MEDIUM |
| `op_sys_cpu_usage` | CPU utilization percentage | LOW |
| `op_sys_memory_info` | RAM total/available/used | MEDIUM |
| `op_sys_disk_info` | Disk space info | MEDIUM |
| `op_sys_network_interfaces` | List network interfaces | LOW |
| `op_sys_uptime` | System uptime | LOW |
| `op_sys_locale` | User locale/language | HIGH |
| `op_sys_theme` | Light/dark mode preference | HIGH |
| `op_sys_app_paths` | Standard directories (Documents, Downloads, etc.) | HIGH |

### 4.2 SDK Feature Gaps

**Missing in host.sys.ts:**

```typescript
// Environment - MISSING
export function getAllEnv(): Record<string, string>;
export function deleteEnv(key: string): void;

// Shell integration - MISSING
export function shellOpen(path: string): Promise<void>;  // Open with default app

// App paths - MISSING
export interface AppPaths {
  documents: string;
  downloads: string;
  pictures: string;
  music: string;
  videos: string;
  desktop: string;
  cache: string;
  config: string;
  data: string;
  logs: string;
}
export function appPaths(): AppPaths;

// Theme - MISSING
export type ThemePreference = "light" | "dark" | "system";
export function getTheme(): ThemePreference;
export function onThemeChange(callback: (theme: ThemePreference) => void): () => void;

// Locale - MISSING
export function getLocale(): string;  // e.g., "en-US"
```

---

## 5. Missing Process Operations (ext_process)

### 5.1 Missing Rust Ops

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_process_run` | Simple run with collected output | HIGH |
| `op_process_list` | List running processes | MEDIUM |
| `op_process_info` | Get process info by PID | MEDIUM |
| `op_process_close_stdin` | Close stdin pipe | MEDIUM |
| `op_process_current_pid` | Get current process PID | HIGH |

### 5.2 SDK Feature Gaps

**Missing in host.process.ts:**

```typescript
// Simple command execution - MISSING
export interface RunResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}
export function run(command: string, args?: string[]): Promise<RunResult>;

// Current process info - MISSING
export function currentPid(): number;
export function currentExe(): string;

// Process listing - MISSING
export interface ProcessInfo {
  pid: number;
  name: string;
  memory?: number;
  cpu?: number;
}
export function listProcesses(): Promise<ProcessInfo[]>;
```

---

## 6. Missing Window Operations (ext_window)

### 6.1 Missing Rust Ops

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_window_set_icon` | Set window icon | HIGH |
| `op_window_set_cursor` | Set cursor style | MEDIUM |
| `op_window_start_dragging` | Frameless window dragging | HIGH |
| `op_window_request_attention` | Flash taskbar/dock | MEDIUM |
| `op_window_set_progress_bar` | Taskbar progress (Windows) | LOW |
| `op_window_set_badge_count` | Dock badge (macOS) | LOW |
| `op_window_open_devtools` | Open DevTools | HIGH |
| `op_window_close_devtools` | Close DevTools | HIGH |
| `op_window_is_devtools_open` | Check DevTools state | MEDIUM |
| `op_window_print` | Print page contents | MEDIUM |
| `op_window_capture` | Screenshot/capture | MEDIUM |
| `op_window_set_content_protection` | Prevent screenshots | LOW |
| `op_window_set_ignore_cursor_events` | Click-through window | LOW |
| `op_window_get_monitors` | List available monitors | HIGH |
| `op_window_get_current_monitor` | Get window's monitor | HIGH |
| `op_window_center` | Center window on screen | MEDIUM |
| `op_window_set_min_size` | Set minimum size | MEDIUM |
| `op_window_set_max_size` | Set maximum size | MEDIUM |
| `op_window_eval_js` | Execute JavaScript in WebView | HIGH |
| `op_window_inject_css` | Inject CSS into WebView | MEDIUM |

### 6.2 SDK Interface Gaps

**Missing in Window interface (host.window.ts:160-200):**

```typescript
export interface Window {
  // ... existing methods ...

  // MISSING:
  setIcon(path: string): Promise<void>;
  setCursor(cursor: CursorType): Promise<void>;
  startDragging(): Promise<void>;
  requestAttention(): Promise<void>;
  openDevTools(): Promise<void>;
  closeDevTools(): Promise<void>;
  isDevToolsOpen(): Promise<boolean>;
  print(): Promise<void>;
  capture(): Promise<Uint8Array>;
  setMinSize(width: number, height: number): Promise<void>;
  setMaxSize(width: number, height: number): Promise<void>;
  center(): Promise<void>;
  getMonitor(): Promise<Monitor>;
  evalJS(script: string): Promise<unknown>;
  injectCSS(css: string): Promise<void>;
}

// Monitor info - MISSING
export interface Monitor {
  name: string;
  size: Size;
  position: Position;
  scaleFactor: number;
  isPrimary: boolean;
}
export function getMonitors(): Promise<Monitor[]>;
export function getPrimaryMonitor(): Promise<Monitor>;
```

---

## 7. Missing WASM Operations (ext_wasm)

### 7.1 Missing Rust Ops

| Op Name | Purpose | Priority |
|---------|---------|----------|
| `op_wasm_validate` | Validate WASM bytes | MEDIUM |
| `op_wasm_compile_wat` | Compile from WAT text format | LOW |
| `op_wasm_get_globals` | Get global values | LOW |
| `op_wasm_set_global` | Set global value | LOW |
| `op_wasm_link_host` | Link runtime functions | MEDIUM |

---

## 8. Framework Logic Issues (HIGH SEVERITY)

### 8.1 No App Lifecycle Management

**Current State:** No way to handle app-level events.

**Missing:**

```typescript
// Should exist in sdk/runtime.app.ts (NEW MODULE)
declare module "runtime:app" {
  export interface AppInfo {
    name: string;
    version: string;
    identifier: string;
  }

  export function getInfo(): AppInfo;
  export function quit(): void;
  export function relaunch(): void;

  // Lifecycle events
  export function onReady(callback: () => void): void;
  export function onBeforeQuit(callback: () => boolean): void;  // return false to cancel
  export function onWindowAllClosed(callback: () => void): void;
  export function onActivate(callback: () => void): void;  // macOS dock click
}
```

### 8.2 No Global Shortcuts/Hotkeys

**Missing:**

```typescript
// Should exist in sdk/runtime.shortcuts.ts (NEW MODULE)
declare module "runtime:shortcuts" {
  export function register(accelerator: string, callback: () => void): boolean;
  export function unregister(accelerator: string): void;
  export function unregisterAll(): void;
  export function isRegistered(accelerator: string): boolean;
}
```

### 8.3 No Auto-Updater System

**Missing:**

```typescript
// Should exist in sdk/runtime.updater.ts (NEW MODULE)
declare module "runtime:updater" {
  export interface UpdateInfo {
    version: string;
    releaseDate: string;
    releaseNotes?: string;
  }

  export function checkForUpdates(): Promise<UpdateInfo | null>;
  export function downloadUpdate(): Promise<void>;
  export function installUpdate(): Promise<void>;
  export function onUpdateAvailable(callback: (info: UpdateInfo) => void): void;
  export function onDownloadProgress(callback: (percent: number) => void): void;
}
```

### 8.4 No Protocol Handler Registration

**Missing:**

```typescript
// Should be part of runtime:app
export function setAsDefaultProtocolClient(protocol: string): boolean;
export function removeAsDefaultProtocolClient(protocol: string): boolean;
export function isDefaultProtocolClient(protocol: string): boolean;
export function onOpenUrl(callback: (url: string) => void): void;  // Deep linking
```

### 8.5 No Single Instance Enforcement

**Missing:**

```typescript
// Should be part of runtime:app
export function requestSingleInstanceLock(): boolean;
export function releaseSingleInstanceLock(): void;
export function onSecondInstance(callback: (args: string[], workingDir: string) => void): void;
```

### 8.6 No Persistent Storage API

**Missing:**

```typescript
// Should exist in sdk/runtime.storage.ts (NEW MODULE)
declare module "runtime:storage" {
  export interface Storage {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T): Promise<void>;
    delete(key: string): Promise<void>;
    clear(): Promise<void>;
    keys(): Promise<string[]>;
  }

  export function getStorage(name?: string): Storage;
}
```

### 8.7 No Crash Reporter

**Missing:**

```typescript
// Should exist in sdk/runtime.crash.ts (NEW MODULE)
declare module "runtime:crash" {
  export interface CrashReportOptions {
    submitUrl: string;
    productName: string;
    companyName: string;
    extra?: Record<string, string>;
  }

  export function start(options: CrashReportOptions): void;
  export function addExtraParameter(key: string, value: string): void;
}
```

### 8.8 No Logging System

**Missing:**

```typescript
// Should exist in sdk/runtime.log.ts (NEW MODULE)
declare module "runtime:log" {
  export type LogLevel = "trace" | "debug" | "info" | "warn" | "error";

  export function setLevel(level: LogLevel): void;
  export function trace(...args: unknown[]): void;
  export function debug(...args: unknown[]): void;
  export function info(...args: unknown[]): void;
  export function warn(...args: unknown[]): void;
  export function error(...args: unknown[]): void;

  export function getLogFile(): string;
}
```

---

## 9. Missing SDK Utilities

### 9.1 No Path Utilities

**Missing:**

```typescript
// Should exist in sdk/runtime.path.ts (NEW MODULE)
declare module "runtime:path" {
  export function join(...paths: string[]): string;
  export function dirname(path: string): string;
  export function basename(path: string, ext?: string): string;
  export function extname(path: string): string;
  export function normalize(path: string): string;
  export function isAbsolute(path: string): boolean;
  export function relative(from: string, to: string): string;
  export function resolve(...paths: string[]): string;
  export const sep: string;
  export const delimiter: string;
}
```

### 9.2 No Encoding Utilities

**Missing:**

```typescript
// Should exist in sdk/runtime.encoding.ts (NEW MODULE)
declare module "runtime:encoding" {
  export function base64Encode(data: Uint8Array): string;
  export function base64Decode(str: string): Uint8Array;
  export function hexEncode(data: Uint8Array): string;
  export function hexDecode(str: string): Uint8Array;

  export const TextEncoder: typeof globalThis.TextEncoder;
  export const TextDecoder: typeof globalThis.TextDecoder;
}
```

### 9.3 No Crypto Utilities

**Missing:**

```typescript
// Should exist in sdk/runtime.crypto.ts (NEW MODULE)
declare module "runtime:crypto" {
  export function randomBytes(length: number): Uint8Array;
  export function randomUUID(): string;
  export function hash(algorithm: "sha1" | "sha256" | "sha512" | "md5", data: Uint8Array): Promise<Uint8Array>;
  export function hashString(algorithm: "sha1" | "sha256" | "sha512" | "md5", data: string): Promise<string>;
}
```

---

## 10. API Inconsistencies

### 10.1 Duplicate Modules: runtime:ui vs runtime:window

**Problem:** Both modules exist with overlapping but inconsistent APIs.

| Feature | runtime:ui | runtime:window |
|---------|---------|-------------|
| Create window | `openWindow()` | `createWindow()` |
| Window type | `WindowHandle` | `Window` |
| Close window | `closeWindow()` | `closeWindow()` |
| Dialogs | `dialog.open/save/message` | `dialog.open/save/message` |
| Menus | `menu.setAppMenu` | `menu.setAppMenu` |
| Window props | Limited | Full (position, size, state queries) |

**Recommendation:** Deprecate `runtime:ui` in favor of `runtime:window` (which is more complete).

### 10.2 Inconsistent Property Naming

**Problem:** Mixed camelCase and snake_case in SDK types.

| File | Location | Issue |
|------|----------|-------|
| host.d.ts | Line 11-14 | `is_file`, `is_dir` (snake_case) |
| host.d.ts | Line 134 | `cpu_count` (snake_case) |
| host.window.ts | Line 9-42 | `alwaysOnTop` (camelCase) |
| host.sys.ts | Line 27-31 | `has_battery`, `ac_connected` (snake_case) |

**Recommendation:** Standardize on camelCase for JavaScript/TypeScript SDK, normalize in SDK layer.

### 10.3 Inconsistent Return Types

| Module | Function | Returns |
|--------|----------|---------|
| runtime:ui | `showContextMenu()` | `Promise<string \| null>` |
| runtime:window | `menu.showContextMenu()` | `Promise<string \| null>` |
| Generated d.ts | `showContextMenu()` | `Promise<string>` (missing null) |

---

## 11. Documentation Gaps

### 11.1 Missing Documentation Files

| Document | Purpose | Status |
|----------|---------|--------|
| SDK API Reference | Complete API documentation | **MISSING** |
| Migration Guide (runtime:ui → runtime:window) | Help users migrate | **MISSING** |
| Best Practices | Patterns and anti-patterns | **MISSING** |
| Examples Directory | Code examples for each module | Partial |
| TypeDoc/JSDoc | Inline documentation | Incomplete |

### 11.2 Incomplete JSDoc Comments

Many SDK functions lack:

- `@throws` documentation
- `@example` usage examples
- `@since` version information
- `@see` cross-references

---

## 12. Preload Script Issues

### 12.1 Limited IPC Bridge (sdk/preload.ts)

**Current limitations:**

- No typed channels
- No request/response pattern (invoke with reply)
- No acknowledgment/delivery confirmation
- No message queuing for offline/disconnected state

**Missing patterns:**

```typescript
// Request/response pattern - MISSING
window.host.invoke<T>(channel: string, payload?: unknown): Promise<T>;

// Typed channels - MISSING
window.host.on<T>(channel: string, cb: (payload: T) => void): void;
```

### 12.2 HMR Client Hardcoded Port

**Location:** `sdk/preload.ts:92`

```typescript
const HMR_PORT = 35729;  // Hardcoded, should be configurable
```

---

## 13. Recommendations Summary

### Immediate (P0) - Required for Production

1. Create `sdk/runtime.ipc.ts` implementation file
2. Add `op_sys_shell_open` - critical for desktop app UX
3. Add `op_window_open_devtools` - essential for debugging
4. Deprecate `runtime:ui` module with migration path to `runtime:window`
5. Create `runtime:app` module for lifecycle management
6. Fix snake_case/camelCase inconsistencies

### Short-term (P1) - High Value

1. Add WebSocket support to `ext_net`
2. Add `runtime:storage` module for persistent data
3. Add `runtime:shortcuts` module for global hotkeys
4. Add monitor/display information APIs
5. Add missing filesystem ops (symlink, metadata, append)
6. Create SDK API reference documentation

### Medium-term (P2) - Important

1. Add `runtime:path` utilities module
2. Add `runtime:crypto` utilities module
3. Add `runtime:log` logging module
4. Add process listing and management
5. Add theme/locale detection
6. Add streaming fetch support

### Long-term (P3) - Nice to Have

1. Auto-updater system
2. Crash reporter
3. TCP/UDP socket support
4. Protocol handler registration
5. Single instance lock
6. Content protection APIs

---

## Appendix: File Inventory

### SDK Files Reviewed

| File | Lines | Status |
|------|-------|--------|
| `sdk/runtime.d.ts` | 498 | Reviewed |
| `sdk/runtime.fs.ts` | 232 | Reviewed |
| `sdk/runtime.ui.ts` | 404 | Reviewed |
| `sdk/runtime.net.ts` | 159 | Reviewed |
| `sdk/runtime.process.ts` | 258 | Reviewed |
| `sdk/runtime.sys.ts` | 236 | Reviewed |
| `sdk/runtime.wasm.ts` | 395 | Reviewed |
| `sdk/runtime.window.ts` | 751 | Reviewed |
| `sdk/preload.ts` | 156 | Reviewed |
| `sdk/generated/host.ui.d.ts` | 196 | Reviewed |
| `sdk/generated/host.ipc.d.ts` | 120 | Reviewed |
| `sdk/generated/host.window.d.ts` | 307 | Reviewed |

### Missing SDK Files (Need Creation)

| File | Purpose |
|------|---------|
| `sdk/runtime.ipc.ts` | IPC implementation |
| `sdk/runtime.app.ts` | App lifecycle |
| `sdk/runtime.storage.ts` | Persistent storage |
| `sdk/runtime.shortcuts.ts` | Global shortcuts |
| `sdk/runtime.path.ts` | Path utilities |
| `sdk/runtime.crypto.ts` | Crypto utilities |
| `sdk/runtime.encoding.ts` | Encoding utilities |
| `sdk/runtime.log.ts` | Logging |

---

*Report generated on: 2025-12-15*
