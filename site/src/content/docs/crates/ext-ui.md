---
title: "ext_ui"
description: Basic UI operations extension providing the host:ui module.
---

The `ext_ui` crate provides basic UI operations for Forge applications through the `host:ui` module. For advanced window management, see [ext_window](/docs/crates/ext-window).

## Overview

ext_ui handles:

- **Basic window operations** - Create and close windows
- **Dialogs** - File dialogs and message dialogs
- **Menus** - Application and context menus
- **System tray** - Tray icon with menu
- **IPC re-exports** - Window events from ext_ipc

## Module: `host:ui`

```typescript
import {
  openWindow,
  closeWindow,
  dialog,
  createTray
} from "host:ui";
```

## Key Types

### Window Types

```rust
struct OpenOpts {
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    title: Option<String>,
    resizable: Option<bool>,
    decorations: Option<bool>,
    channels: Option<Vec<String>>,
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

struct MessageDialogOpts {
    title: Option<String>,
    message: String,
    kind: Option<String>,
    buttons: Option<Vec<String>>,
}

struct MessageDialogResult {
    button_index: usize,
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
    item_type: Option<String>,
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

### Command Types

```rust
enum FromDenoCmd {
    CreateWindow(OpenOpts),
    CloseWindow(String),
    SetWindowTitle { window_id: String, title: String },
    ShowOpenDialog(FileDialogOpts),
    ShowSaveDialog(FileDialogOpts),
    ShowMessageDialog(MessageDialogOpts),
    SetAppMenu(Vec<MenuItem>),
    ShowContextMenu { items: Vec<MenuItem>, window_id: Option<String> },
    CreateTray(TrayOpts),
    UpdateTray { tray_id: String, opts: TrayOpts },
    DestroyTray(String),
}
```

### State Types

```rust
struct UiState {
    cmd_tx: mpsc::Sender<FromDenoCmd>,
    event_rx: mpsc::Receiver<UiEvent>,
}

struct UiCapabilities {
    windows: bool,
    dialogs: bool,
    menus: bool,
    tray: bool,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| (via command) | `openWindow(opts?)` | Create window |
| (via command) | `closeWindow(id)` | Close window |
| (via command) | `setWindowTitle(id, title)` | Set title |
| (via command) | `dialog.open(opts?)` | File open dialog |
| (via command) | `dialog.save(opts?)` | File save dialog |
| (via command) | `dialog.message(opts)` | Message dialog |
| (via command) | `setAppMenu(items)` | Set app menu |
| (via command) | `showContextMenu(items)` | Context menu |
| (via command) | `createTray(opts?)` | Create tray |
| (via command) | `destroyTray(id)` | Destroy tray |

## Relationship to ext_window

`ext_ui` provides a simpler API for common operations:

| Feature | ext_ui (host:ui) | ext_window (host:window) |
|---------|------------------|--------------------------|
| Window creation | `openWindow()` | `createWindow()` |
| Window handle | Basic (id, setTitle, close) | Full (37 methods) |
| Position/size | Not available | Full control |
| State queries | Not available | isFullscreen, isFocused, etc. |
| Dialogs | Full | Full |
| Menus | Full | Full |
| Tray | Full | Full |

Use `host:ui` for simple apps, `host:window` for full control.

## File Structure

```text
crates/ext_ui/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async channels |
| `wry`, `tao` | Window/WebView |
| `rfd` | File dialogs |
| `ext_ipc` | IPC re-exports |
| `forge-weld` | Build-time code generation |

## Related

- [host:ui API](/docs/api/host-ui) - TypeScript API documentation
- [ext_window](/docs/crates/ext-window) - Advanced window management
- [ext_ipc](/docs/crates/ext-ipc) - IPC operations
