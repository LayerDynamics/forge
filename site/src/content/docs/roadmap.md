---
title: Roadmap
description: Forge SDK development roadmap - planned modules, features, and improvements.
---

This document outlines the Forge SDK development roadmap, including planned extension modules, features, and improvements to bring Forge to feature parity with Electron and Tauri.

## Current State

Forge currently has **8 fully implemented extension modules**:

| Module | Crate | Operations | Status |
|--------|-------|------------|--------|
| `host:fs` | ext_fs | 11 | Complete |
| `host:window` | ext_window | 37 | Complete |
| `host:ipc` | ext_ipc | 2 | Complete |
| `host:net` | ext_net | 2 | Complete |
| `host:process` | ext_process | 7 | Complete |
| `host:sys` | ext_sys | 11 | Complete |
| `host:wasm` | ext_wasm | 10 | Complete |
| `host:ui` | ext_ui | - | Legacy (use host:window) |

---

## Phase 1: High Priority Modules

### host:crypto - Cryptography & Security

Cryptographic operations for secure applications.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `randomBytes` | Cryptographically secure random bytes | `(size: number) => Uint8Array` |
| `randomUUID` | Generate UUID v4 | `() => string` |
| `hash` | Hash data (SHA-256, SHA-512, MD5) | `(algorithm: string, data: Uint8Array \| string) => Uint8Array` |
| `hashHex` | Hash returning hex string | `(algorithm: string, data: Uint8Array \| string) => string` |
| `hmac` | HMAC signature | `(algorithm: string, key: Uint8Array, data: Uint8Array) => Uint8Array` |
| `encrypt` | Symmetric encryption (AES-GCM) | `(algorithm: string, key: Uint8Array, data: Uint8Array, iv?: Uint8Array) => EncryptedData` |
| `decrypt` | Symmetric decryption | `(algorithm: string, key: Uint8Array, encrypted: EncryptedData) => Uint8Array` |
| `generateKey` | Generate encryption key | `(algorithm: string, length?: number) => Uint8Array` |
| `deriveKey` | Derive key from password (PBKDF2) | `(password: string, salt: Uint8Array, iterations: number, keyLength: number) => Uint8Array` |
| `verify` | Verify HMAC/signature | `(algorithm: string, key: Uint8Array, data: Uint8Array, signature: Uint8Array) => boolean` |

**Error codes:** 8000-8009

---

### host:storage - Persistent Key-Value Storage

App state persistence with SQLite-backed storage.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `get` | Get value by key | `<T>(key: string) => Promise<T \| null>` |
| `set` | Set value by key | `<T>(key: string, value: T) => Promise<void>` |
| `delete` | Delete key | `(key: string) => Promise<boolean>` |
| `has` | Check if key exists | `(key: string) => Promise<boolean>` |
| `keys` | List all keys | `() => Promise<string[]>` |
| `clear` | Clear all data | `() => Promise<void>` |
| `size` | Get storage size in bytes | `() => Promise<number>` |
| `getMany` | Batch get | `(keys: string[]) => Promise<Map<string, unknown>>` |
| `setMany` | Batch set | `(entries: Record<string, unknown>) => Promise<void>` |
| `deleteMany` | Batch delete | `(keys: string[]) => Promise<number>` |

**Storage location:** `~/.forge/<app-identifier>/storage.db`
**Error codes:** 8100-8109
**Capability:** `[capabilities.storage]`

---

### host:shell - Shell & OS Integration

Common desktop app integrations for opening files, URLs, and system interactions.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `openExternal` | Open URL in default browser | `(url: string) => Promise<void>` |
| `openPath` | Open file/folder with default app | `(path: string) => Promise<void>` |
| `showItemInFolder` | Reveal file in file manager | `(path: string) => Promise<void>` |
| `moveToTrash` | Move file to recycle bin/trash | `(path: string) => Promise<void>` |
| `beep` | System beep sound | `() => void` |
| `getFileIcon` | Get file type icon | `(path: string, size?: number) => Promise<Uint8Array>` |
| `getDefaultApp` | Get default app for file type | `(extension: string) => Promise<string \| null>` |

**Error codes:** 8200-8209
**Capability:** `[capabilities.shell]`

---

### host:app - Application Lifecycle

Core application management and lifecycle control.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `quit` | Quit application | `(exitCode?: number) => void` |
| `exit` | Force exit (no cleanup) | `(exitCode?: number) => void` |
| `relaunch` | Restart application | `(options?: { args?: string[] }) => void` |
| `getVersion` | Get app version | `() => string` |
| `getName` | Get app name | `() => string` |
| `getIdentifier` | Get app identifier | `() => string` |
| `getPath` | Get special path | `(name: PathName) => string` |
| `isPackaged` | Check if running packaged | `() => boolean` |
| `getLocale` | Get system locale | `() => string` |
| `setAppUserModelId` | Set Windows taskbar ID | `(id: string) => void` |
| `requestSingleInstanceLock` | Ensure single instance | `() => boolean` |
| `releaseSingleInstanceLock` | Release instance lock | `() => void` |
| `focus` | Bring app to foreground | `() => void` |
| `hide` | Hide all app windows | `() => void` |
| `show` | Show all app windows | `() => void` |
| `setBadgeCount` | Set dock/taskbar badge | `(count: number) => void` |

**Path names:** `home`, `appData`, `userData`, `temp`, `exe`, `desktop`, `documents`, `downloads`, `music`, `pictures`, `videos`, `logs`

**Error codes:** 8300-8309

---

## Phase 2: Medium Priority Modules

### host:screen - Display Information

Multi-monitor support and display information.

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

### host:globalShortcut - Global Keyboard Shortcuts

System-wide keyboard shortcut registration.

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

### host:autoUpdater - Application Updates

Automatic application update system.

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

### host:theme - Native Theme Detection

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

### host:database - Embedded SQLite Database

Full SQLite database support for complex data storage.

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

## Phase 3: Low Priority Modules

### host:dock - macOS Dock Integration

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

### host:taskbar - Windows Taskbar Integration

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

### host:protocol - Custom Protocol Handlers

Deep linking and custom URL scheme support.

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

### host:session - WebView Session Management

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

### host:download - Download Manager

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

### host:log - Structured Logging

Production-ready logging with file rotation.

| Operation | Description | Signature |
|-----------|-------------|-----------|
| `debug` | Debug level log | `(message: string, ...args: unknown[]) => void` |
| `info` | Info level log | `(message: string, ...args: unknown[]) => void` |
| `warn` | Warning level log | `(message: string, ...args: unknown[]) => void` |
| `error` | Error level log | `(message: string, ...args: unknown[]) => void` |
| `setLevel` | Set minimum level | `(level: 'debug' \| 'info' \| 'warn' \| 'error') => void` |
| `setFile` | Enable file logging | `(path: string, options?: LogFileOptions) => void` |
| `flush` | Flush log buffer | `() => Promise<void>` |

**Log location:** `~/.forge/<app-identifier>/logs/`
**Error codes:** 9400-9409

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

### Single Instance Lock

Prevent multiple app instances.

**Components:**
- Lock file or socket-based detection
- IPC to existing instance
- Focus existing window on duplicate launch

---

### Error Code Standardization

Consistent error codes across all modules.

**Modules requiring updates:**
- `ext_net` - Add codes 4000-4009
- `ext_process` - Add codes 5000-5009
- `ext_sys` - Add codes 5500-5509

---

### Deep Linking

OS-level URL scheme handling.

**Components:**
- Integrate with `host:protocol`
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
- Locale detection
- RTL support hints
- Number/date formatting

---

## Summary

| Category | Count |
|----------|-------|
| New extension modules | 15 |
| New operations | ~116 |
| New error code ranges | 15 |
| New capabilities | 8 |
| Features/processes | 6 |

### Implementation Order

**Phase 1 (High Priority):**
1. `ext_crypto` (10 ops)
2. `ext_storage` (10 ops)
3. `ext_shell` (7 ops)
4. `ext_app` (16 ops)

**Phase 2 (Medium Priority):**
5. `ext_screen` (6 ops)
6. `ext_globalshortcut` (5 ops)
7. `ext_autoupdater` (6 ops)
8. `ext_theme` (6 ops)
9. `ext_database` (8 ops)

**Phase 3 (Low Priority):**
10. `ext_dock` (9 ops)
11. `ext_taskbar` (5 ops)
12. `ext_protocol` (7 ops)
13. `ext_session` (8 ops)
14. `ext_download` (6 ops)
15. `ext_log` (7 ops)

**Phase 4 (Features):**
16. Error code standardization
17. Hot reload system
18. Single instance lock
19. Deep linking
20. Accessibility APIs
21. Internationalization support
