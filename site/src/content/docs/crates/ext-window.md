---
title: "ext_window"
description: Advanced window management extension providing the runtime:window module.
slug: crates/ext-window
---

The `ext_window` crate provides comprehensive window management for Forge applications through the `runtime:window` module. It offers full control over windows, dialogs, menus, and system tray.

## Overview

ext_window handles:

- **Window lifecycle** - Create, close, minimize, maximize, restore
- **Window properties** - Position, size, title, visibility, decorations
- **Window state** - Fullscreen, focus, always-on-top
- **Dialogs** - File open/save, message dialogs
- **Menus** - Application menu, context menus
- **System tray** - Tray icons with menus
- **Native handles** - Platform-specific window handles
- **Window events** - Close, resize, move, focus

## Module: `runtime:window`

```typescript
import {
  createWindow,
  closeWindow,
  dialog,
  menu,
  tray
} from "runtime:window";
```

## Key Types

### Error Types

```rust
enum WindowErrorCode {
    Generic = 6000,
    PermissionDenied = 6001,
    WindowNotFound = 6002,
    TrayNotFound = 6003,
    DialogCancelled = 6004,
    MenuError = 6005,
    InvalidOperation = 6006,
}

struct WindowError {
    code: WindowErrorCode,
    message: String,
}
```

### Window Types

```rust
struct WindowOpts {
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    title: Option<String>,
    resizable: Option<bool>,
    decorations: Option<bool>,
    visible: Option<bool>,
    transparent: Option<bool>,
    always_on_top: Option<bool>,
    x: Option<i32>,
    y: Option<i32>,
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    channels: Option<Vec<String>>,
}

struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: u32,
    height: u32,
}

struct NativeHandle {
    platform: String,  // "windows", "macos", "linux-x11", "linux-wayland"
    handle: u64,
}

struct WindowSystemEvent {
    window_id: String,
    event_type: String,  // "close", "focus", "blur", "resize", "move"
    payload: serde_json::Value,
}
```

### Dialog Types

```rust
struct FileDialogOpts {
    title: Option<String>,
    default_path: Option<String>,
    filters: Option<Vec<FileFilter>>,
    multiple: Option<bool>,
    directory: Option<bool>,
}

struct FileFilter {
    name: String,
    extensions: Vec<String>,
}

struct MessageDialogOpts {
    title: Option<String>,
    message: String,
    kind: Option<String>,  // "info", "warning", "error"
    buttons: Option<Vec<String>>,
}
```

### Menu Types

```rust
struct MenuItem {
    id: Option<String>,
    label: String,
    accelerator: Option<String>,
    enabled: Option<bool>,
    checked: Option<bool>,
    submenu: Option<Vec<MenuItem>>,
    item_type: Option<String>,  // "normal", "checkbox", "separator"
}

struct MenuEvent {
    menu_id: String,
    item_id: String,
    label: String,
}
```

### Tray Types

```rust
struct TrayOpts {
    icon: Option<String>,
    tooltip: Option<String>,
    menu: Option<Vec<MenuItem>>,
}
```

## Operations

### Window Lifecycle (10 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_create` | `createWindow(opts?)` | Create window |
| `op_window_close` | `win.close()` | Close window |
| `op_window_minimize` | `win.minimize()` | Minimize |
| `op_window_maximize` | `win.maximize()` | Maximize |
| `op_window_unmaximize` | `win.unmaximize()` | Restore from maximize |
| `op_window_restore` | `win.restore()` | Restore from minimize |
| `op_window_set_fullscreen` | `win.setFullscreen(bool)` | Set fullscreen |
| `op_window_is_fullscreen` | `win.isFullscreen()` | Check fullscreen |
| `op_window_focus` | `win.focus()` | Focus window |
| `op_window_is_focused` | `win.isFocused()` | Check focused |

### Window Properties (16 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_get_position` | `win.getPosition()` | Get position |
| `op_window_set_position` | `win.setPosition(x, y)` | Set position |
| `op_window_get_size` | `win.getSize()` | Get size |
| `op_window_set_size` | `win.setSize(w, h)` | Set size |
| `op_window_get_title` | `win.getTitle()` | Get title |
| `op_window_set_title` | `win.setTitle(title)` | Set title |
| `op_window_set_resizable` | `win.setResizable(bool)` | Set resizable |
| `op_window_is_resizable` | `win.isResizable()` | Check resizable |
| `op_window_set_decorations` | `win.setDecorations(bool)` | Set decorations |
| `op_window_has_decorations` | `win.hasDecorations()` | Check decorations |
| `op_window_set_always_on_top` | `win.setAlwaysOnTop(bool)` | Set always on top |
| `op_window_is_always_on_top` | `win.isAlwaysOnTop()` | Check always on top |
| `op_window_set_visible` | `win.setVisible(bool)` | Set visibility |
| `op_window_is_visible` | `win.isVisible()` | Check visible |
| `op_window_is_maximized` | `win.isMaximized()` | Check maximized |
| `op_window_is_minimized` | `win.isMinimized()` | Check minimized |

### Dialogs (3 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_dialog_open` | `dialog.open(opts?)` | File open dialog |
| `op_window_dialog_save` | `dialog.save(opts?)` | File save dialog |
| `op_window_dialog_message` | `dialog.message(opts)` | Message dialog |

### Menus (3 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_set_app_menu` | `menu.setAppMenu(items)` | Set app menu |
| `op_window_show_context_menu` | `menu.showContextMenu(items)` | Show context menu |
| `op_window_menu_recv` | `menu.events()` | Receive menu events |

### Tray (3 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_create_tray` | `tray.create(opts?)` | Create tray |
| `op_window_update_tray` | `trayHandle.update(opts)` | Update tray |
| `op_window_destroy_tray` | `tray.destroy(id)` | Destroy tray |

### Events & Native (2 ops)

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_window_events_recv` | `windowEvents()` | Receive window events |
| `op_window_get_native_handle` | `win.getNativeHandle()` | Get native handle |

## File Structure

```text
crates/ext_window/
├── src/
│   └── lib.rs        # Extension implementation (37 ops)
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tao` | Window management |
| `wry` | WebView |
| `rfd` | File dialogs |
| `muda` | Menu system |
| `tray-icon` | System tray |
| `image` | Icon processing |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowOpts {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
    // ...
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_window_create(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: WindowOpts,
) -> Result<String, WindowError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_window", "runtime:window")
        .ts_path("ts/init.ts")
        .ops(&["op_window_create", "op_window_close", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_window extension");
}
```

## Related

- [runtime:window API](/docs/api/runtime-window) - TypeScript API documentation
- [ext_ipc](/docs/crates/ext-ipc) - IPC extension
- [ext_webview](/docs/crates/ext-webview) - WebView operations
- [forge-runtime](/docs/crates/forge-runtime) - Runtime that manages windows
- [forge-weld](/docs/crates/forge-weld) - Code generation library
