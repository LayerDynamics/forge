---
title: "ext_dock"
description: macOS Dock customization extension providing the runtime:dock module.
slug: crates/ext-dock
---

The `ext_dock` crate provides macOS Dock customization APIs for Forge applications through the `runtime:dock` module.

## Overview

ext_dock handles:

- **Badge Text** - Set and clear the dock icon badge (notification count)
- **Icon Bounce** - Trigger attention-requesting bounce animations
- **Visibility** - Show/hide the app's dock icon
- **Custom Icon** - Change the dock icon at runtime
- **Dock Menu** - Custom right-click menu (planned)

> **Note:** These are macOS-only features. On other platforms, operations will no-op gracefully.

## Module: `runtime:dock`

```typescript
import {
  setBadge,
  getBadge,
  bounce,
  cancelBounce,
  hide,
  show,
  isVisible,
  setIcon,
  setMenu
} from "runtime:dock";
```

## Key Types

### Error Types

```rust
enum DockErrorCode {
    DockError = 8800,
    IconError = 8801,
    BadgeError = 8802,
    BounceError = 8803,
    MenuError = 8804,
    PlatformNotSupported = 8805,
    InvalidParameter = 8806,
}
```

### Data Types

```rust
enum BounceType {
    Critical,      // Continues until app is activated
    Informational, // Bounces once (default)
}

struct BounceResult {
    id: u64,       // Bounce request ID (for cancellation)
    success: bool, // Whether bounce started successfully
}

struct MenuItem {
    id: Option<String>,
    label: String,
    accelerator: Option<String>,
    enabled: Option<bool>,
    checked: Option<bool>,
    submenu: Option<Vec<MenuItem>>,
    item_type: Option<String>, // "normal", "checkbox", "separator"
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_dock_set_badge` | `setBadge(text)` | Set badge text on dock icon |
| `op_dock_get_badge` | `getBadge()` | Get current badge text |
| `op_dock_bounce` | `bounce(type?)` | Bounce the dock icon |
| `op_dock_cancel_bounce` | `cancelBounce(id)` | Stop a bounce animation |
| `op_dock_hide` | `hide()` | Hide the dock icon |
| `op_dock_show` | `show()` | Show the dock icon |
| `op_dock_is_visible` | `isVisible()` | Check dock icon visibility |
| `op_dock_set_icon` | `setIcon(path)` | Set custom dock icon |
| `op_dock_set_menu` | `setMenu(items)` | Set dock menu (planned) |
| `op_dock_info` | `info()` | Get extension information |

## Usage Example

```typescript
import { setBadge, bounce, hide, show, setIcon } from "runtime:dock";

// Show notification count on dock icon
setBadge("5");

// Clear the badge
setBadge("");

// Bounce to get user attention (informational - bounces once)
const result = bounce("informational");

// Critical bounce - continues until app is focused
const critical = bounce("critical");

// Cancel a bounce
cancelBounce(critical.id);

// Hide app from dock (becomes accessory/background app)
hide();

// Show app in dock again
show();

// Set custom dock icon
setIcon("/path/to/custom-icon.png");

// Reset to default icon
setIcon("");
```

## Platform Behavior

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Badge text | NSApplication dockTile | No-op | No-op |
| Bounce | requestUserAttention | No-op | No-op |
| Hide/Show | setActivationPolicy | No-op | No-op |
| Custom icon | setApplicationIconImage | No-op | No-op |

On non-macOS platforms, all operations return success but perform no action. This allows cross-platform code without conditional checks.

## Bounce Types

### Informational
- Single bounce
- User sees brief notification
- Ideal for: new message, download complete

### Critical
- Continues bouncing until app is activated
- More intrusive, use sparingly
- Ideal for: urgent alerts, errors requiring attention

## File Structure

```text
crates/ext_dock/
├── src/
│   └── lib.rs        # Extension implementation with macOS Cocoa bindings
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `cocoa` | macOS AppKit bindings |
| `objc` | Objective-C runtime |
| `image` | Icon format conversion |

## Related

- [runtime:window](/docs/crates/ext-window) - Window management
- [runtime:sys](/docs/crates/ext-sys) - System tray and notifications
