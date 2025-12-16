---
title: "ext_shortcuts"
description: Global keyboard shortcuts extension providing the runtime:shortcuts module.
slug: crates/ext-shortcuts
---

The `ext_shortcuts` crate provides global keyboard shortcut registration for Forge applications through the `runtime:shortcuts` module.

## Overview

ext_shortcuts handles:

- **Global hotkeys** - System-wide keyboard shortcuts
- **Shortcut registration** - Register/unregister shortcuts
- **Modifier keys** - Ctrl, Alt, Shift, Meta/Cmd
- **Key combinations** - Multi-key shortcuts
- **Conflict detection** - Handle existing shortcuts

## Module: `runtime:shortcuts`

```typescript
import {
  register,
  unregister,
  unregisterAll,
  isRegistered
} from "runtime:shortcuts";
```

## Key Types

### Error Types

```rust
enum ShortcutsErrorCode {
    Generic = 10000,
    RegisterFailed = 10001,
    UnregisterFailed = 10002,
    AlreadyRegistered = 10003,
    InvalidShortcut = 10004,
    ConflictingShortcut = 10005,
}

struct ShortcutsError {
    code: ShortcutsErrorCode,
    message: String,
}
```

### Shortcut Types

```rust
struct Shortcut {
    id: u32,
    accelerator: String,
    callback: ShortcutCallback,
}

struct ShortcutEvent {
    id: u32,
    accelerator: String,
    timestamp: u64,
}

enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,  // Cmd on macOS, Win on Windows
    Super, // Win key
}

struct ShortcutState {
    shortcuts: HashMap<u32, Shortcut>,
    next_id: u32,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_shortcuts_register` | `register(accelerator, callback)` | Register shortcut |
| `op_shortcuts_unregister` | `unregister(id)` | Remove shortcut |
| `op_shortcuts_unregister_all` | `unregisterAll()` | Remove all shortcuts |
| `op_shortcuts_is_registered` | `isRegistered(accelerator)` | Check if registered |

## Usage Examples

### Basic Registration

```typescript
import { register, unregister } from "runtime:shortcuts";

// Register Ctrl+Shift+P (Cmd+Shift+P on macOS)
const shortcut = await register("CmdOrCtrl+Shift+P", () => {
  console.log("Command palette triggered!");
  showCommandPalette();
});

// Later: unregister
await unregister(shortcut.id);
```

### Multiple Shortcuts

```typescript
import { register, unregisterAll } from "runtime:shortcuts";

// Register multiple shortcuts
await register("CmdOrCtrl+N", () => createNewDocument());
await register("CmdOrCtrl+O", () => openDocument());
await register("CmdOrCtrl+S", () => saveDocument());
await register("CmdOrCtrl+Shift+S", () => saveDocumentAs());
await register("CmdOrCtrl+W", () => closeDocument());

// Cleanup on exit
await unregisterAll();
```

### Media Keys

```typescript
import { register } from "runtime:shortcuts";

await register("MediaPlayPause", () => togglePlayback());
await register("MediaNextTrack", () => nextTrack());
await register("MediaPreviousTrack", () => previousTrack());
await register("MediaStop", () => stopPlayback());
```

### Function Keys

```typescript
import { register } from "runtime:shortcuts";

await register("F5", () => refresh());
await register("Ctrl+F5", () => hardRefresh());
await register("F11", () => toggleFullscreen());
await register("F12", () => openDevTools());
```

## Accelerator Format

| Format | Example | Description |
|--------|---------|-------------|
| Single key | `F1`, `A`, `Space` | Single key press |
| With modifier | `Ctrl+A`, `Alt+F4` | Modifier + key |
| Multiple modifiers | `Ctrl+Shift+S` | Multiple modifiers |
| Cross-platform | `CmdOrCtrl+C` | Cmd on macOS, Ctrl elsewhere |
| Special keys | `MediaPlayPause`, `VolumeUp` | Media/system keys |

## Modifier Keys

| Modifier | macOS | Windows/Linux |
|----------|-------|---------------|
| `Cmd` | ⌘ Command | - |
| `Ctrl` | ⌃ Control | Ctrl |
| `Alt` | ⌥ Option | Alt |
| `Shift` | ⇧ Shift | Shift |
| `Meta` | ⌘ Command | Win |
| `CmdOrCtrl` | ⌘ Command | Ctrl |

## File Structure

```text
crates/ext_shortcuts/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct ShortcutHandle {
    pub id: u32,
    pub accelerator: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shortcuts_register(
    state: Rc<RefCell<OpState>>,
    #[string] accelerator: String,
) -> Result<ShortcutHandle, ShortcutsError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shortcuts", "runtime:shortcuts")
        .ts_path("ts/init.ts")
        .ops(&["op_shortcuts_register", "op_shortcuts_unregister", "op_shortcuts_is_registered"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_shortcuts extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `global-hotkey` | Cross-platform hotkeys |
| `serde` | Serialization |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_window](/docs/crates/ext-window) - Window management
- [Architecture](/docs/architecture) - Full system architecture
