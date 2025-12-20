---
title: Roadmap
description: Forge SDK development roadmap - planned modules, features, and improvements.
slug: roadmap
---

This document outlines the Forge SDK development roadmap, including planned extension modules, features, and improvements to bring Forge to feature parity with Electron and Tauri.

## Current State

Forge currently has **40+ implemented extension modules** (as of v1.0.0p-steel-donut 🍩):

### Core Extensions (Fully Implemented)

| Module | Crate | Operations | Status |
|--------|-------|------------|--------|
| `runtime:fs` | ext_fs | 11 | Complete |
| `runtime:window` | ext_window | 37 | Complete |
| `runtime:ipc` | ext_ipc | 2 | Complete |
| `runtime:net` | ext_net | 7 | Complete |
| `runtime:process` | ext_process | 7 | Complete |
| `runtime:sys` | ext_sys | 11 | Complete |
| `runtime:wasm` | ext_wasm | 10 | Complete |
| `runtime:crypto` | ext_crypto | 10 | Complete |
| `runtime:storage` | ext_storage | 10 | Complete |
| `runtime:app` | ext_app | 16 | Complete |
| `runtime:shell` | ext_shell | 7 | Complete |

### Support Extensions (Implemented)

| Module | Crate | Operations | Status |
|--------|-------|------------|--------|
| `runtime:log` | ext_log | 2 | Complete |
| `runtime:trace` | ext_trace | 5 | Complete |
| `runtime:timers` | ext_timers | 6 | Complete |
| `runtime:lock` | ext_lock | 4 | Complete |
| `runtime:signals` | ext_signals | 4 | Complete |
| `runtime:path` | ext_path | 5 | Complete |
| `runtime:webview` | ext_webview | 8 | Complete |
| `runtime:devtools` | ext_devtools | 3 | Complete |
| `runtime:os_compat` | ext_os_compat | 2 | Complete |

### New Extensions (Steel Donut Release)

| Module | Crate | Purpose | Status |
|--------|-------|---------|--------|
| `runtime:bundler` | ext_bundler | App bundling operations | Complete |
| `runtime:codesign` | ext_codesign | Code signing (macOS/Windows/Linux) | Complete |
| `runtime:dock` | ext_dock | macOS dock integration | Complete |
| `runtime:encoding` | ext_encoding | Text encoding/decoding | Complete |
| `runtime:etcher` | ext_etcher | Documentation generation | Complete |
| `runtime:image_tools` | ext_image_tools | Image conversion (PNG, SVG, WebP, ICO) | Complete |
| `runtime:svelte` | ext_svelte | SvelteKit integration | Complete |
| `runtime:web_inspector` | ext_web_inspector | Chrome DevTools Protocol bridge | Complete |
| `runtime:weld` | ext_weld | Runtime binding system access | Complete |
| `runtime:database` | ext_database | Database operations | Complete |
| `runtime:debugger` | ext_debugger | Debugger integration | Complete |
| `runtime:monitor` | ext_monitor | System monitoring | Complete |
| `runtime:display` | ext_display | Display information | Complete |
| `runtime:updater` | ext_updater | App update system | Complete |
| `runtime:protocol` | ext_protocol | Custom protocol handlers | Complete |
| `runtime:shortcuts` | ext_shortcuts | Keyboard shortcuts | Complete |

---

## Completed Modules (Previously Phase 1)

The following high-priority modules have been fully implemented:

### runtime:crypto - Cryptography & Security ✅

Cryptographic operations for secure applications.

| Operation | Description | Status |
|-----------|-------------|--------|
| `randomBytes` | Cryptographically secure random bytes | ✅ |
| `randomUUID` | Generate UUID v4 | ✅ |
| `hash` | Hash data (SHA-256, SHA-384, SHA-512) | ✅ |
| `hashHex` | Hash returning hex string | ✅ |
| `hmac` | HMAC signature | ✅ |
| `encrypt` | Symmetric encryption (AES-256-GCM) | ✅ |
| `decrypt` | Symmetric decryption | ✅ |
| `generateKey` | Generate encryption key | ✅ |
| `deriveKey` | Derive key from password (PBKDF2) | ✅ |
| `verify` | Verify HMAC signature | ✅ |

**Error codes:** 8000-8009

---

### runtime:storage - Persistent Key-Value Storage ✅

App state persistence with SQLite-backed storage.

| Operation | Description | Status |
|-----------|-------------|--------|
| `get` | Get value by key | ✅ |
| `set` | Set value by key | ✅ |
| `delete` | Delete key | ✅ |
| `has` | Check if key exists | ✅ |
| `keys` | List all keys | ✅ |
| `clear` | Clear all data | ✅ |
| `size` | Get storage size in bytes | ✅ |
| `getMany` | Batch get | ✅ |
| `setMany` | Batch set | ✅ |
| `deleteMany` | Batch delete | ✅ |

**Storage location:** `~/.forge/<app-identifier>/storage.db`
**Error codes:** 8100-8109
**Capability:** `[capabilities.storage]`

---

### runtime:shell - Shell & OS Integration ✅

Common desktop app integrations for opening files, URLs, and system interactions.

| Operation | Description | Status |
|-----------|-------------|--------|
| `openExternal` | Open URL in default browser | ✅ |
| `openPath` | Open file/folder with default app | ✅ |
| `showItemInFolder` | Reveal file in file manager | ✅ |
| `moveToTrash` | Move file to recycle bin/trash | ✅ |
| `beep` | System beep sound | ✅ |
| `getFileIcon` | Get file type icon | ⚠️ Partial |
| `getDefaultApp` | Get default app for file type | ✅ |

**Error codes:** 8200-8209
**Capability:** `[capabilities.shell]`

---

### runtime:app - Application Lifecycle ✅

Core application management and lifecycle control.

| Operation | Description | Status |
|-----------|-------------|--------|
| `quit` | Quit application | ✅ |
| `exit` | Force exit (no cleanup) | ✅ |
| `relaunch` | Restart application | ✅ |
| `getVersion` | Get app version | ✅ |
| `getName` | Get app name | ✅ |
| `getIdentifier` | Get app identifier | ✅ |
| `getPath` | Get special path | ✅ |
| `isPackaged` | Check if running packaged | ✅ |
| `getLocale` | Get system locale | ✅ |
| `setAppUserModelId` | Set Windows taskbar ID | ✅ |
| `requestSingleInstanceLock` | Ensure single instance | ✅ |
| `releaseSingleInstanceLock` | Release instance lock | ✅ |
| `focus` | Bring app to foreground | ✅ |
| `hide` | Hide all app windows | ✅ |
| `show` | Show all app windows | ✅ |
| `setBadgeCount` | Set dock/taskbar badge | ✅ |

**Path names:** `home`, `appData`, `documents`, `downloads`, `desktop`, `music`, `pictures`, `videos`, `temp`, `exe`, `resources`, `logs`, `cache`

**Error codes:** 8300-8319

---

## Phase 1: High Priority Modules (In Progress)

### runtime:screen - Display Information

Multi-monitor support and display information. Currently a stub extension (`ext_display`).

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `getPrimaryDisplay` | Get primary monitor | `() => Display` |
| `getAllDisplays` | Get all monitors | `() => Display[]` |
| `getDisplayMatching` | Get display containing rect | `(rect: Rect) => Display` |
| `getDisplayNearestPoint` | Get display nearest to point | `(point: Point) => Display` |
| `getCursorScreenPoint` | Get cursor position | `() => Point` |
| `screenEvents` | Display change events | `() => AsyncGenerator<ScreenEvent>` |

```typescript
interface Display {
  id: number;
  label: string;
  bounds: Rect;
  workArea: Rect;
  scaleFactor: number;
  rotation: 0 | 90 | 180 | 270;
  isPrimary: boolean;
}
```

**Error codes:** 8400-8409

---

### runtime:globalShortcut - Global Keyboard Shortcuts

System-wide keyboard shortcut registration. Currently a stub extension (`ext_shortcuts`).

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `register` | Register global hotkey | `(accelerator: string, callback: () => void) => boolean` |
| `unregister` | Unregister hotkey | `(accelerator: string) => void` |
| `unregisterAll` | Unregister all hotkeys | `() => void` |
| `isRegistered` | Check if registered | `(accelerator: string) => boolean` |
| `shortcutEvents` | Hotkey trigger events | `() => AsyncGenerator<ShortcutEvent>` |

**Accelerator format:** `"CommandOrControl+Shift+Z"`, `"Alt+Space"`, etc.

**Error codes:** 8500-8509
**Capability:** `[capabilities.globalShortcut]`

---

### runtime:autoUpdater - Application Updates

Automatic application update system. Currently a stub extension (`ext_updater`).

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `checkForUpdates` | Check for new version | `() => Promise<UpdateInfo \| null>` |
| `downloadUpdate` | Download available update | `() => Promise<void>` |
| `quitAndInstall` | Apply update and restart | `() => void` |
| `setFeedURL` | Set update server URL | `(url: string) => void` |
| `getFeedURL` | Get current update URL | `() => string` |
| `updateEvents` | Update progress events | `() => AsyncGenerator<UpdateEvent>` |

```typescript
interface UpdateInfo {
  version: string;
  releaseDate: string;
  releaseNotes?: string;
  mandatory?: boolean;
}

type UpdateEvent =
  | { type: 'checking' }
  | { type: 'available'; info: UpdateInfo }
  | { type: 'not-available' }
  | { type: 'downloading'; progress: number }
  | { type: 'downloaded' }
  | { type: 'error'; message: string };
```

**Error codes:** 8600-8609

---

### runtime:database - Embedded SQLite Database

Full SQLite database support for complex data storage. Currently a stub extension.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `open` | Open/create database | `(name: string, options?: DbOptions) => Promise<Database>` |
| `close` | Close database | `(db: Database) => Promise<void>` |
| `execute` | Execute SQL (no return) | `(db: Database, sql: string, params?: unknown[]) => Promise<number>` |
| `query` | Query with results | `<T>(db: Database, sql: string, params?: unknown[]) => Promise<T[]>` |
| `queryOne` | Query single row | `<T>(db: Database, sql: string, params?: unknown[]) => Promise<T \| null>` |
| `batch` | Execute multiple statements | `(db: Database, statements: string[]) => Promise<void>` |
| `transaction` | Run in transaction | `<T>(db: Database, fn: () => Promise<T>) => Promise<T>` |
| `prepare` | Prepare statement | `(db: Database, sql: string) => PreparedStatement` |

**Database location:** `~/.forge/<app-identifier>/databases/<name>.db`

**Error codes:** 9000-9019
**Capability:** `[capabilities.database]`

---

## Phase 2: Medium Priority Modules

### runtime:theme - Native Theme Detection

System theme detection and preference management.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `shouldUseDarkColors` | Check if dark mode active | `() => boolean` |
| `getThemeSource` | Get theme setting | `() => 'system' \| 'light' \| 'dark'` |
| `setThemeSource` | Set theme preference | `(source: 'system' \| 'light' \| 'dark') => void` |
| `getAccentColor` | Get system accent color | `() => string` |
| `getHighContrast` | Check high contrast mode | `() => boolean` |
| `themeEvents` | Theme change events | `() => AsyncGenerator<ThemeEvent>` |

**Error codes:** 8700-8709

---

### runtime:protocol - Custom Protocol Handlers

Deep linking and custom URL scheme support. Currently a stub extension.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `registerScheme` | Register URL scheme | `(scheme: string) => Promise<boolean>` |
| `unregisterScheme` | Unregister scheme | `(scheme: string) => Promise<void>` |
| `isSchemeRegistered` | Check registration | `(scheme: string) => boolean` |
| `setAsDefaultProtocolClient` | Set as OS default | `(scheme: string) => Promise<boolean>` |
| `removeAsDefaultProtocolClient` | Remove as default | `(scheme: string) => Promise<boolean>` |
| `isDefaultProtocolClient` | Check if default | `(scheme: string) => boolean` |
| `protocolEvents` | Incoming URL events | `() => AsyncGenerator<ProtocolEvent>` |

**Error codes:** 9100-9109

---

## Phase 3: Low Priority Modules

### runtime:dock - macOS Dock Integration

macOS-specific dock customization.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `bounce` | Bounce dock icon | `(type?: 'critical' \| 'informational') => number` |
| `cancelBounce` | Stop bounce | `(id: number) => void` |
| `setBadge` | Set badge text | `(text: string) => void` |
| `getBadge` | Get badge text | `() => string` |
| `hide` | Hide dock icon | `() => void` |
| `show` | Show dock icon | `() => void` |
| `isVisible` | Check dock visibility | `() => boolean` |
| `setIcon` | Set custom dock icon | `(image: Uint8Array \| null) => void` |
| `setMenu` | Set dock menu | `(menu: MenuItem[]) => void` |

**Platform:** macOS only (no-op on other platforms)
**Error codes:** 8800-8809

---

### runtime:taskbar - Windows Taskbar Integration

Windows-specific taskbar customization.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `setProgressBar` | Set progress indicator | `(progress: number, options?: ProgressOptions) => void` |
| `setOverlayIcon` | Set overlay icon | `(icon: Uint8Array \| null, description?: string) => void` |
| `setThumbarButtons` | Set thumbnail toolbar | `(buttons: ThumbarButton[]) => void` |
| `setJumpList` | Set jump list | `(categories: JumpListCategory[]) => void` |
| `flashFrame` | Flash taskbar button | `(flash: boolean) => void` |

**Platform:** Windows only (no-op on other platforms)
**Error codes:** 8900-8909

---

### runtime:session - WebView Session Management

WebView session and cookie management.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `getCookies` | Get cookies for URL | `(url: string) => Promise<Cookie[]>` |
| `setCookie` | Set cookie | `(cookie: Cookie) => Promise<void>` |
| `removeCookies` | Remove cookies | `(url: string, name?: string) => Promise<void>` |
| `clearStorageData` | Clear browsing data | `(options?: ClearStorageOptions) => Promise<void>` |
| `setProxy` | Set proxy config | `(config: ProxyConfig) => Promise<void>` |
| `resolveProxy` | Resolve proxy for URL | `(url: string) => Promise<string>` |
| `setUserAgent` | Set user agent | `(userAgent: string) => void` |
| `getUserAgent` | Get user agent | `() => string` |

**Error codes:** 9200-9209

---

### runtime:download - Download Manager

File download tracking and management.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `start` | Start download | `(url: string, options?: DownloadOptions) => Promise<Download>` |
| `pause` | Pause download | `(downloadId: string) => Promise<void>` |
| `resume` | Resume download | `(downloadId: string) => Promise<void>` |
| `cancel` | Cancel download | `(downloadId: string) => Promise<void>` |
| `getState` | Get download state | `(downloadId: string) => DownloadState` |
| `downloadEvents` | Download progress events | `() => AsyncGenerator<DownloadEvent>` |

**Error codes:** 9300-9309

---

## Phase 4: Features & Processes

### Hot Reload System

Live reload for development workflow.

**Components:**

- File watcher for `src/` and `web/` directories
- WebSocket server for reload signals
- Deno module cache invalidation
- HMR protocol for web assets

---

### Single Instance Lock ✅

Prevent multiple app instances. **Implemented in `ext_app`.**

**Components:**

- Lock file-based detection ✅
- IPC to existing instance (partial)
- Focus existing window on duplicate launch ✅

---

### Error Code Standardization ✅

Consistent error codes across all modules. **Implemented across all new extensions.**

All new extensions use standardized error code ranges:
- `ext_crypto`: 8000-8009
- `ext_storage`: 8100-8109
- `ext_shell`: 8200-8209
- `ext_app`: 8300-8319

---

### Deep Linking

OS-level URL scheme handling.

**Components:**

- Integrate with `runtime:protocol`
- Startup argument parsing for URLs
- Event dispatch to running app

---

### Accessibility APIs

Screen reader and accessibility support.

**Components:**

- Screen reader announcements
- Accessibility tree access
- High contrast detection

---

### Internationalization Support

Locale and RTL support.

**Components:**

- Locale detection ✅ (in `ext_app`)
- RTL support hints
- Number/date formatting

---

## Summary

| Category | Count |
|----------|-------|
| Implemented extension modules | 22 |
| Stub extension modules (need implementation) | 5 |
| Implemented operations | ~150+ |
| Planned operations (stubs) | ~32 |
| Features completed | 3 |
| Features in progress | 4 |

### Implementation Status

**Completed:**
1. `ext_crypto` (10 ops) ✅
2. `ext_storage` (10 ops) ✅
3. `ext_shell` (7 ops) ✅
4. `ext_app` (16 ops) ✅
5. `ext_log` (2 ops) ✅
6. `ext_trace` (5 ops) ✅
7. `ext_timers` (6 ops) ✅
8. `ext_lock` (4 ops) ✅
9. `ext_signals` (4 ops) ✅
10. `ext_path` (5 ops) ✅
11. `ext_webview` (8 ops) ✅
12. `ext_devtools` (3 ops) ✅
13. `ext_os_compat` (2 ops) ✅
14. `ext_bundler` (build tooling) ✅
15. `ext_weld` (TypeScript generation) ✅

**Phase 1 - High Priority (Stub → Full Implementation):**
1. `ext_display` (screen) - 6 ops planned
2. `ext_shortcuts` (globalShortcut) - 5 ops planned
3. `ext_updater` (autoUpdater) - 6 ops planned
4. `ext_database` - 8 ops planned

**Phase 2 - Medium Priority (New Modules):**
5. `ext_theme` - 6 ops planned
6. `ext_protocol` - 7 ops planned (stub exists)

**Phase 3 - Low Priority (Platform-Specific):**
7. `ext_dock` (macOS) - 9 ops planned
8. `ext_taskbar` (Windows) - 5 ops planned
9. `ext_session` - 8 ops planned
10. `ext_download` - 6 ops planned

**Phase 4 - Features:**
- Hot reload system
- Deep linking
- Accessibility APIs
- Internationalization support
