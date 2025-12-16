# Code Duplication Report

This report identifies methods, types, and functions implemented in `ext_*` modules that are reimplemented or duplicated in `forge-host` or other modules.

## Summary

| Duplication Category | Severity | Files Involved |
|---------------------|----------|----------------|
| ext_ui vs ext_window types | HIGH | ext_ui/src/lib.rs, ext_window/src/lib.rs |
| ext_ui vs ext_window ops | HIGH | ext_ui/src/lib.rs, ext_window/src/lib.rs |
| forge-host menu helpers | MEDIUM | forge-host/src/main.rs, ext_window/src/manager.rs |
| Capability checker patterns | MEDIUM | ext_ui/src/lib.rs, ext_window/src/lib.rs |

---

## 1. ext_ui vs ext_window - Type Duplications (HIGH SEVERITY)

The `ext_ui` and `ext_window` modules contain nearly identical type definitions. These should be consolidated into a shared types module.

### 1.1 Window Options

| Type | ext_ui Location | ext_window Location |
|------|-----------------|---------------------|
| `OpenOpts` / `WindowOpts` | `crates/ext_ui/src/lib.rs:9-20` | `crates/ext_window/src/lib.rs:167-186` |

**ext_ui/src/lib.rs (OpenOpts):**
```rust
pub struct OpenOpts {
    pub url: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    pub transparent: Option<bool>,
    pub always_on_top: Option<bool>,
    pub skip_taskbar: Option<bool>,
    pub channels: Option<Vec<String>>,
}
```

**ext_window/src/lib.rs (WindowOpts):**
```rust
pub struct WindowOpts {
    pub url: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    pub transparent: Option<bool>,
    pub always_on_top: Option<bool>,
    pub skip_taskbar: Option<bool>,
    pub center: Option<bool>,
    pub channels: Option<Vec<String>>,
}
```

### 1.2 FileDialogOpts (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 23-30 |
| `crates/ext_window/src/lib.rs` | Lines 233-240 |

```rust
pub struct FileDialogOpts {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Option<Vec<FileFilter>>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
}
```

### 1.3 FileFilter (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 32-37 |
| `crates/ext_window/src/lib.rs` | Lines 243-247 |

```rust
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}
```

### 1.4 MessageDialogOpts (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 39-46 |
| `crates/ext_window/src/lib.rs` | Lines 250-256 |

```rust
pub struct MessageDialogOpts {
    pub title: Option<String>,
    pub message: String,
    pub kind: Option<String>,
    pub buttons: Option<Vec<String>>,
}
```

### 1.5 MenuItem (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 54-65 |
| `crates/ext_window/src/lib.rs` | Lines 259-269 |

```rust
pub struct MenuItem {
    pub id: Option<String>,
    pub label: String,
    pub enabled: Option<bool>,
    pub checked: Option<bool>,
    pub accelerator: Option<String>,
    pub item_type: Option<String>,
    pub submenu: Option<Vec<MenuItem>>,
}
```

### 1.6 TrayOpts (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 67-73 |
| `crates/ext_window/src/lib.rs` | Lines 272-277 |

```rust
pub struct TrayOpts {
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub menu: Option<Vec<MenuItem>>,
}
```

### 1.7 MenuEvent (EXACT DUPLICATE)

| Location | Line Numbers |
|----------|--------------|
| `crates/ext_ui/src/lib.rs` | Lines 75-84 |
| `crates/ext_window/src/lib.rs` | Lines 280-288 |

```rust
pub struct MenuEvent {
    pub menu_id: String,
    pub item_id: String,
    pub label: String,
}
```

---

## 2. ext_ui vs ext_window - Capability Checker Duplications (MEDIUM SEVERITY)

### 2.1 Capability Checker Traits

| Trait | ext_ui Location | ext_window Location |
|-------|-----------------|---------------------|
| `UiCapabilityChecker` | `crates/ext_ui/src/lib.rs:171-177` | N/A |
| `WindowCapabilityChecker` | N/A | `crates/ext_window/src/lib.rs:439-445` |

Both traits define the same methods:
- `check_windows_allowed()`
- `check_dialogs_allowed()`
- `check_menus_allowed()`
- `check_tray_allowed()`

### 2.2 Permissive Checker Implementations

| Type | ext_ui Location | ext_window Location |
|------|-----------------|---------------------|
| `PermissiveUiChecker` | `crates/ext_ui/src/lib.rs:179-195` | N/A |
| `PermissiveWindowChecker` | N/A | `crates/ext_window/src/lib.rs:448-466` |

Both implementations return `Ok(())` for all capability checks.

### 2.3 Check Helper Functions

| ext_ui Function | ext_window Function | Purpose |
|-----------------|---------------------|---------|
| `check_ui_windows()` (lines 210-223) | `check_window_capability()` (lines 482-496) | Check window creation permission |
| `check_ui_dialogs()` (lines 225-238) | `check_dialog_capability()` (lines 498-512) | Check dialog permission |
| `check_ui_menus()` (lines 240-250) | `check_menu_capability()` (lines 514-528) | Check menu permission |
| `check_ui_tray()` (lines N/A) | `check_tray_capability()` (lines 530-544) | Check tray permission |

---

## 3. ext_ui vs ext_window - Op Duplications (HIGH SEVERITY)

The following operations are duplicated between the two modules:

### 3.1 Window Operations

| Operation | ext_ui Location | ext_window Location |
|-----------|-----------------|---------------------|
| Create Window | `op_ui_open_window` (lines 252-285) | `op_window_create` (lines 536-567) |
| Close Window | `op_ui_close_window` (lines 287-313) | `op_window_close` (lines 570-593) |
| Set Title | `op_ui_set_title` (lines 315-337) | `op_window_set_title` (lines 596-617) |

### 3.2 Dialog Operations

| Operation | ext_ui Location | ext_window Location |
|-----------|-----------------|---------------------|
| Open Dialog | `op_ui_dialog_open` (lines 339-372) | `op_window_dialog_open` (lines 1179-1209) |
| Save Dialog | `op_ui_dialog_save` (lines 374-409) | `op_window_dialog_save` (lines 1211-1245) |
| Message Dialog | `op_ui_dialog_message` (lines 411-443) | `op_window_dialog_message` (lines 1247-1278) |

### 3.3 Menu Operations

| Operation | ext_ui Location | ext_window Location |
|-----------|-----------------|---------------------|
| Set App Menu | `op_ui_set_app_menu` (lines 449-481) | `op_window_set_app_menu` (lines 1284-1313) |
| Show Context Menu | `op_ui_show_context_menu` (lines 483-520) | `op_window_show_context_menu` (lines 1316-1349) |
| Menu Receive | `op_ui_menu_recv` (lines 635-667) | `op_window_menu_recv` (lines 1352-1383) |

### 3.4 Tray Operations

| Operation | ext_ui Location | ext_window Location |
|-----------|-----------------|---------------------|
| Create Tray | `op_ui_create_tray` (lines 526-559) | `op_window_create_tray` (lines 1389-1419) |
| Update Tray | `op_ui_update_tray` (lines 561-595) | `op_window_update_tray` (lines 1422-1452) |
| Destroy Tray | `op_ui_destroy_tray` (lines 597-629) | `op_window_destroy_tray` (lines 1455-1483) |

---

## 4. forge-host/src/main.rs - Helper Function Duplications (MEDIUM SEVERITY)

The `forge-host/src/main.rs` contains helper functions that duplicate functionality already present or that should be moved to `ext_window/src/manager.rs`.

### 4.1 Menu Helper Functions

| Function | forge-host Location | Notes |
|----------|---------------------|-------|
| `add_menu_items_with_tracking()` | Lines 1108-1143 | Duplicate pattern with ext_window |
| `add_submenu_items_with_tracking()` | Lines 1145-1180 | Duplicate pattern with ext_window |
| `add_context_menu_items()` | Lines 1252-1288 | Duplicate pattern with ext_window |
| `add_context_submenu_items()` | Lines 1290-1327 | Duplicate pattern with ext_window |

### 4.2 Tray Helper Functions

| Function | forge-host Location | ext_window Location |
|----------|---------------------|---------------------|
| `create_default_tray_icon()` | Lines 1393-1401 | `crates/ext_window/src/manager.rs` (similar) |
| `add_tray_menu_items()` | Lines 1404-1489 | Similar patterns in manager.rs |

---

## 5. Recommendations

### 5.1 Immediate Actions (High Priority)

1. **Consolidate ext_ui and ext_window**: These two modules have significant overlap. Consider:
   - Deprecating `ext_ui` in favor of `ext_window` (which has more features)
   - OR creating a shared types crate (`ext_ui_common`) for common types
   - OR merging both into a single `ext_window` module

2. **Extract Shared Types**: Create `crates/ext_common/src/types.rs`:
   ```rust
   // Shared types used by multiple ext_* modules
   pub mod dialog;  // FileDialogOpts, FileFilter, MessageDialogOpts
   pub mod menu;    // MenuItem, MenuEvent
   pub mod tray;    // TrayOpts
   pub mod window;  // WindowOpts/OpenOpts
   ```

### 5.2 Medium Priority

3. **Unify Capability Checker Pattern**: Create a generic capability checker trait:
   ```rust
   pub trait CapabilityChecker: Send + Sync {
       fn check(&self, capability: &str) -> Result<(), CapabilityError>;
   }
   ```

4. **Move Helper Functions to WindowManager**: The menu and tray helper functions in `forge-host/src/main.rs` should be moved to `ext_window/src/manager.rs` to avoid duplication.

### 5.3 Lower Priority

5. **Standardize Error Types**: Consider a shared error module pattern for all ext_* modules to reduce boilerplate.

---

## Appendix: File Summary

| File | Lines | Primary Purpose |
|------|-------|-----------------|
| `crates/ext_ui/src/lib.rs` | ~670 | Legacy UI extension (overlaps with ext_window) |
| `crates/ext_window/src/lib.rs` | ~1750 | Window management extension |
| `crates/ext_window/src/manager.rs` | ~1505 | Window manager implementation |
| `crates/forge-host/src/main.rs` | ~1678 | Main runtime with event loop |
| `crates/ext_fs/src/lib.rs` | ~450 | Filesystem extension |
| `crates/ext_ipc/src/lib.rs` | ~350 | IPC extension |
| `crates/ext_net/src/lib.rs` | ~500 | Network extension |
| `crates/ext_process/src/lib.rs` | ~600 | Process management extension |
| `crates/ext_sys/src/lib.rs` | ~400 | System information extension |
| `crates/ext_wasm/src/lib.rs` | ~1200 | WASM runtime extension |

---

*Report generated on: 2025-12-15*
