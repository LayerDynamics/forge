---
title: "runtime:window"
description: Window management, dialogs, menus, and system tray.
slug: api/runtime-window
---

The `runtime:window` module provides comprehensive window management including creation, manipulation, dialogs, menus, and system tray icons.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_window](/docs/crates/ext-window) for implementation details.

## Window Management

### createWindow(options?)

Create a new window and return a `Window` handle:

```typescript
import { createWindow } from "runtime:window";

const win = await createWindow({
  title: "My App",
  width: 800,
  height: 600,
  url: "app://index.html",
});

console.log("Window ID:", win.id);
```

### WindowOptions

```typescript
interface WindowOptions {
  /** URL to load (default: "app://index.html") */
  url?: string;
  /** Window width in pixels */
  width?: number;
  /** Window height in pixels */
  height?: number;
  /** Window title */
  title?: string;
  /** Whether window is resizable (default: true) */
  resizable?: boolean;
  /** Whether window has decorations (default: true) */
  decorations?: boolean;
  /** Whether window is visible on creation (default: true) */
  visible?: boolean;
  /** Whether window is transparent */
  transparent?: boolean;
  /** Whether window is always on top */
  alwaysOnTop?: boolean;
  /** Initial X position */
  x?: number;
  /** Initial Y position */
  y?: number;
  /** Minimum window width */
  minWidth?: number;
  /** Minimum window height */
  minHeight?: number;
  /** Maximum window width */
  maxWidth?: number;
  /** Maximum window height */
  maxHeight?: number;
  /** IPC channel allowlist */
  channels?: string[];
}
```

### closeWindow(windowId)

Close a window by its ID:

```typescript
import { closeWindow } from "runtime:window";

await closeWindow("my-window-id");
```

---

## Window Handle

The `Window` object returned by `createWindow()` provides methods to control the window.

### Lifecycle Methods

```typescript
const win = await createWindow({ title: "My App" });

// Close the window
await win.close();

// Minimize to taskbar/dock
await win.minimize();

// Maximize to fill screen
await win.maximize();

// Restore from maximized state
await win.unmaximize();

// Restore from minimized state
await win.restore();

// Bring window to front
await win.focus();
```

### Position & Size

```typescript
// Get current position
const pos = await win.getPosition();
console.log(`x: ${pos.x}, y: ${pos.y}`);

// Set position
await win.setPosition(100, 100);

// Get current size
const size = await win.getSize();
console.log(`width: ${size.width}, height: ${size.height}`);

// Set size
await win.setSize(1024, 768);
```

**Types:**

```typescript
interface Position {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}
```

### Title

```typescript
// Get window title
const title = await win.getTitle();

// Set window title
await win.setTitle("New Title");
```

### State Queries

```typescript
// Check window state
const isFullscreen = await win.isFullscreen();
const isFocused = await win.isFocused();
const isMaximized = await win.isMaximized();
const isMinimized = await win.isMinimized();
const isVisible = await win.isVisible();
const isResizable = await win.isResizable();
const hasDecorations = await win.hasDecorations();
const isAlwaysOnTop = await win.isAlwaysOnTop();
```

### Configuration

```typescript
// Toggle fullscreen mode
await win.setFullscreen(true);

// Enable/disable resizing
await win.setResizable(false);

// Show/hide window decorations (title bar, borders)
await win.setDecorations(false);

// Keep window on top of others
await win.setAlwaysOnTop(true);

// Show/hide window
await win.setVisible(true);
await win.show();  // Alias for setVisible(true)
await win.hide();  // Alias for setVisible(false)
```

### Native Handle

Get the platform-specific native window handle for interop with native libraries:

```typescript
const handle = await win.getNativeHandle();
console.log(`Platform: ${handle.platform}`);
console.log(`Handle: ${handle.handle}`);
```

**NativeHandle type:**

```typescript
interface NativeHandle {
  /** Platform: "windows", "macos", "linux-x11", "linux-wayland" */
  platform: string;
  /** Raw handle (HWND, NSView*, X11 window ID) */
  handle: number;
}
```

### Window Events

Listen for events from a specific window:

```typescript
for await (const event of win.events()) {
  console.log(`Event: ${event.type}`);

  if (event.type === "close") {
    console.log("Window close requested");
    break;
  }
}
```

**WindowSystemEvent type:**

```typescript
interface WindowSystemEvent {
  windowId: string;
  type: "close" | "focus" | "blur" | "resize" | "move" | "minimize" | "maximize" | "restore";
  payload: unknown;
}
```

---

## Global Window Events

### windowEvents()

Listen for events from all windows:

```typescript
import { windowEvents } from "runtime:window";

for await (const event of windowEvents()) {
  console.log(`[${event.windowId}] ${event.type}`);
}
```

---

## Dialogs

The `dialog` namespace provides native file and message dialogs.

### dialog.open(options?)

Show a file open dialog. Returns selected paths or `null` if cancelled:

```typescript
import { dialog } from "runtime:window";

// Simple file selection
const files = await dialog.open();

// With options
const files = await dialog.open({
  title: "Select Images",
  filters: [
    { name: "Images", extensions: ["png", "jpg", "gif"] }
  ],
  multiple: true,
});

if (files) {
  console.log("Selected:", files);
}
```

### dialog.save(options?)

Show a save file dialog. Returns the path or `null` if cancelled:

```typescript
import { dialog } from "runtime:window";

const path = await dialog.save({
  title: "Save Document",
  defaultPath: "document.txt",
  filters: [
    { name: "Text Files", extensions: ["txt"] },
    { name: "All Files", extensions: ["*"] }
  ],
});

if (path) {
  console.log("Save to:", path);
}
```

### FileDialogOptions

```typescript
interface FileDialogOptions {
  /** Dialog title */
  title?: string;
  /** Default starting path */
  defaultPath?: string;
  /** File type filters */
  filters?: FileFilter[];
  /** Allow selecting multiple files */
  multiple?: boolean;
  /** Select directories instead of files */
  directory?: boolean;
}

interface FileFilter {
  /** Display name (e.g., "Images") */
  name: string;
  /** Extensions without dots (e.g., ["png", "jpg"]) */
  extensions: string[];
}
```

### dialog.message(options)

Show a message dialog. Returns the index of the clicked button:

```typescript
import { dialog } from "runtime:window";

const result = await dialog.message({
  title: "Confirm Action",
  message: "Are you sure you want to proceed?",
  kind: "warning",
});
```

### Convenience Dialogs

```typescript
import { dialog } from "runtime:window";

// Alert dialog (info icon)
await dialog.alert("Operation completed successfully!");
await dialog.alert("Custom message", "Custom Title");

// Confirm dialog (returns boolean)
const confirmed = await dialog.confirm("Delete this file?");
if (confirmed) {
  // User clicked OK
}

// Error dialog (error icon)
await dialog.error("Something went wrong!");

// Warning dialog (warning icon)
await dialog.warning("This action cannot be undone!");
```

---

## Menus

The `menu` namespace provides application and context menus.

### menu.setAppMenu(items)

Set the application menu bar:

```typescript
import { menu } from "runtime:window";

await menu.setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "new", label: "New", accelerator: "CmdOrCtrl+N" },
      { id: "open", label: "Open", accelerator: "CmdOrCtrl+O" },
      { type: "separator" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
  {
    label: "Edit",
    submenu: [
      { id: "undo", label: "Undo", accelerator: "CmdOrCtrl+Z" },
      { id: "redo", label: "Redo", accelerator: "CmdOrCtrl+Shift+Z" },
      { type: "separator" },
      { id: "cut", label: "Cut", accelerator: "CmdOrCtrl+X" },
      { id: "copy", label: "Copy", accelerator: "CmdOrCtrl+C" },
      { id: "paste", label: "Paste", accelerator: "CmdOrCtrl+V" },
    ],
  },
]);
```

### menu.showContextMenu(items, windowId?)

Show a context menu at the cursor position:

```typescript
import { menu } from "runtime:window";

const selected = await menu.showContextMenu([
  { id: "cut", label: "Cut" },
  { id: "copy", label: "Copy" },
  { id: "paste", label: "Paste" },
  { type: "separator" },
  { id: "delete", label: "Delete", enabled: false },
]);

if (selected) {
  console.log("Selected:", selected);
}
```

### menu.onMenu(callback)

Register a callback for menu events. Returns an unsubscribe function:

```typescript
import { menu } from "runtime:window";

const unsubscribe = menu.onMenu((event) => {
  console.log(`Menu item clicked: ${event.itemId}`);

  switch (event.itemId) {
    case "new":
      createNewDocument();
      break;
    case "quit":
      Deno.exit(0);
      break;
  }
});

// Later, to stop listening:
unsubscribe();
```

### menu.events()

Async iterator for menu events:

```typescript
import { menu } from "runtime:window";

for await (const event of menu.events()) {
  console.log(`Menu: ${event.menuId}, Item: ${event.itemId}`);
}
```

### MenuItem

```typescript
interface MenuItem {
  /** Unique identifier */
  id?: string;
  /** Display label */
  label: string;
  /** Keyboard shortcut (e.g., "CmdOrCtrl+S") */
  accelerator?: string;
  /** Whether enabled (default: true) */
  enabled?: boolean;
  /** Whether checked (for checkbox items) */
  checked?: boolean;
  /** Submenu items */
  submenu?: MenuItem[];
  /** Item type */
  type?: "normal" | "checkbox" | "separator";
}
```

### MenuEvent

```typescript
interface MenuEvent {
  /** Source menu ID */
  menuId: string;
  /** Clicked item ID */
  itemId: string;
  /** Item label */
  label: string;
}
```

---

## System Tray

The `tray` namespace provides system tray icon functionality.

### tray.create(options?)

Create a system tray icon:

```typescript
import { tray } from "runtime:window";

const trayIcon = await tray.create({
  icon: "./assets/tray-icon.png",
  tooltip: "My App",
  menu: [
    { id: "show", label: "Show Window" },
    { id: "hide", label: "Hide Window" },
    { type: "separator" },
    { id: "quit", label: "Quit" },
  ],
});

console.log("Tray ID:", trayIcon.id);
```

### TrayHandle

The returned `TrayHandle` provides methods to update or destroy the tray:

```typescript
// Update tray properties
await trayIcon.update({
  tooltip: "New Status",
  menu: [
    { id: "status", label: "Status: Online", enabled: false },
    { type: "separator" },
    { id: "quit", label: "Quit" },
  ],
});

// Destroy the tray icon
await trayIcon.destroy();
```

### tray.destroy(trayId)

Destroy a tray by ID:

```typescript
import { tray } from "runtime:window";

await tray.destroy("my-tray-id");
```

### TrayOptions

```typescript
interface TrayOptions {
  /** Path to icon file */
  icon?: string;
  /** Tooltip text shown on hover */
  tooltip?: string;
  /** Context menu for the tray */
  menu?: MenuItem[];
}
```

---

## Complete Example

```typescript
import { createWindow, dialog, menu, tray, windowEvents } from "runtime:window";
import { onChannel } from "runtime:ipc";

// Create main window
const mainWindow = await createWindow({
  title: "My Application",
  width: 1200,
  height: 800,
});

// Set up application menu
await menu.setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "open", label: "Open...", accelerator: "CmdOrCtrl+O" },
      { id: "save", label: "Save", accelerator: "CmdOrCtrl+S" },
      { type: "separator" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
  {
    label: "Help",
    submenu: [
      { id: "about", label: "About" },
    ],
  },
]);

// Handle menu events
menu.onMenu(async (event) => {
  switch (event.itemId) {
    case "open": {
      const files = await dialog.open({
        title: "Open File",
        filters: [{ name: "Documents", extensions: ["txt", "md"] }],
      });
      if (files) {
        console.log("Opening:", files[0]);
      }
      break;
    }
    case "save": {
      const path = await dialog.save({ title: "Save File" });
      if (path) {
        console.log("Saving to:", path);
      }
      break;
    }
    case "quit":
      Deno.exit(0);
      break;
    case "about":
      await dialog.alert("My Application v1.0.0", "About");
      break;
  }
});

// Create system tray
const trayIcon = await tray.create({
  tooltip: "My Application",
  menu: [
    { id: "show", label: "Show Window" },
    { id: "quit", label: "Quit" },
  ],
});

// Handle window events
for await (const event of windowEvents()) {
  if (event.type === "close" && event.windowId === mainWindow.id) {
    const confirmed = await dialog.confirm("Are you sure you want to quit?");
    if (confirmed) {
      await trayIcon.destroy();
      Deno.exit(0);
    }
  }
}
```
