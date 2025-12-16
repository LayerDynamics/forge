---
title: "ext_display"
description: Display and monitor information extension providing the runtime:display module.
slug: crates/ext-display
---

The `ext_display` crate provides display and monitor information for Forge applications through the `runtime:display` module.

## Overview

ext_display handles:

- **Monitor enumeration** - List connected displays
- **Display properties** - Resolution, scale, bounds
- **Primary display** - Identify main monitor
- **Display events** - Monitor connect/disconnect
- **Cursor position** - Global cursor coordinates

## Module: `runtime:display`

```typescript
import {
  getAll,
  getPrimary,
  getFromPoint,
  getCursorPosition,
  onDisplayChange
} from "runtime:display";
```

## Key Types

### Error Types

```rust
enum DisplayErrorCode {
    Generic = 9700,
    NotFound = 9701,
    QueryFailed = 9702,
}

struct DisplayError {
    code: DisplayErrorCode,
    message: String,
}
```

### Display Types

```rust
struct Display {
    id: u32,
    name: String,
    bounds: Rect,
    work_area: Rect,
    scale_factor: f64,
    is_primary: bool,
    rotation: Rotation,
}

struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

struct CursorPosition {
    x: i32,
    y: i32,
    display_id: u32,
}

struct DisplayChangeEvent {
    kind: DisplayChangeKind,
    display: Display,
}

enum DisplayChangeKind {
    Added,
    Removed,
    Changed,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_display_get_all` | `getAll()` | Get all displays |
| `op_display_get_primary` | `getPrimary()` | Get primary display |
| `op_display_get_from_point` | `getFromPoint(x, y)` | Get display at point |
| `op_display_get_cursor_position` | `getCursorPosition()` | Get cursor position |
| `op_display_on_change` | `onDisplayChange(callback)` | Listen for changes |

## Usage Examples

### Listing Displays

```typescript
import { getAll, getPrimary } from "runtime:display";

const displays = await getAll();
for (const display of displays) {
  console.log(`Display: ${display.name}`);
  console.log(`  Resolution: ${display.bounds.width}x${display.bounds.height}`);
  console.log(`  Scale: ${display.scale_factor}x`);
  console.log(`  Primary: ${display.is_primary}`);
}

const primary = await getPrimary();
console.log(`Primary display: ${primary.name}`);
```

### Display at Point

```typescript
import { getFromPoint, getCursorPosition } from "runtime:display";

// Get display under cursor
const cursor = await getCursorPosition();
const display = await getFromPoint(cursor.x, cursor.y);
console.log(`Cursor is on display: ${display.name}`);
```

### Listening for Changes

```typescript
import { onDisplayChange } from "runtime:display";

const unsubscribe = await onDisplayChange((event) => {
  switch (event.kind) {
    case "Added":
      console.log(`Display connected: ${event.display.name}`);
      break;
    case "Removed":
      console.log(`Display disconnected: ${event.display.name}`);
      break;
    case "Changed":
      console.log(`Display changed: ${event.display.name}`);
      break;
  }
});

// Later: stop listening
unsubscribe();
```

### Window Positioning

```typescript
import { getPrimary } from "runtime:display";
import { createWindow } from "runtime:window";

const primary = await getPrimary();

// Center window on primary display
const windowWidth = 800;
const windowHeight = 600;
const x = primary.bounds.x + (primary.bounds.width - windowWidth) / 2;
const y = primary.bounds.y + (primary.bounds.height - windowHeight) / 2;

await createWindow({
  title: "Centered Window",
  x, y,
  width: windowWidth,
  height: windowHeight
});
```

## Display Properties

| Property | Description |
|----------|-------------|
| `id` | Unique display identifier |
| `name` | Display name/model |
| `bounds` | Full display bounds (x, y, width, height) |
| `work_area` | Usable area (excludes taskbar/dock) |
| `scale_factor` | DPI scaling (1.0 = 100%, 2.0 = 200%) |
| `is_primary` | Whether this is the main display |
| `rotation` | Display rotation (0°, 90°, 180°, 270°) |

## File Structure

```text
crates/ext_display/
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
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct Display {
    pub id: u32,
    pub name: String,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub rotation: Rotation,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_display_get_all(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<Display>, DisplayError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_display", "runtime:display")
        .ts_path("ts/init.ts")
        .ops(&["op_display_get_all", "op_display_get_primary", "op_display_get_from_point", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_display extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tao` | Window/display management |
| `serde` | Serialization |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_window](/docs/crates/ext-window) - Window management
- [Architecture](/docs/architecture) - Full system architecture
