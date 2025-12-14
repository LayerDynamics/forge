---
title: "host:ui"
description: Basic window operations, dialogs, menus, and system tray.
---

The `host:ui` module provides basic window operations, dialogs, menus, and system tray functionality. For advanced window management (position, size, state control), see [`host:window`](/api/host-window). For inter-process communication, see [`host:ipc`](/api/host-ipc).

## Basic Window Operations

### openWindow(options?)

Create a new window with basic options:

```typescript
import { openWindow } from "host:ui";

const win = await openWindow({
  url: "app://index.html",
  width: 800,
  height: 600,
  title: "My Window",
});

console.log("Window ID:", win.id);
```

**Options:**

```typescript
interface OpenWindowOptions {
  /** URL to load (default: "app://index.html") */
  url?: string;
  /** Window width in pixels */
  width?: number;
  /** Window height in pixels */
  height?: number;
  /** Window title */
  title?: string;
  /** Whether window can be resized */
  resizable?: boolean;
  /** Show window decorations */
  decorations?: boolean;
  /** IPC channel allowlist */
  channels?: string[];
}
```

**Returns:** `Promise<WindowHandle>`

```typescript
interface WindowHandle {
  readonly id: string;
  setTitle(title: string): Promise<void>;
  close(): Promise<boolean>;
}
```

### closeWindow(windowId)

Close a window by ID:

```typescript
import { closeWindow } from "host:ui";

await closeWindow("window-123");
```

### setWindowTitle(windowId, title)

Set a window's title:

```typescript
import { setWindowTitle } from "host:ui";

await setWindowTitle("window-123", "New Title");
```

---

## Dialogs

### dialog.open(options?)

Show a file open dialog. Returns selected paths or `null` if cancelled:

```typescript
import { dialog } from "host:ui";

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
import { dialog } from "host:ui";

const path = await dialog.save({
  title: "Save Document",
  defaultPath: "document.txt",
  filters: [
    { name: "Text Files", extensions: ["txt"] }
  ],
});

if (path) {
  console.log("Save to:", path);
}
```

### FileDialogOptions

```typescript
interface FileDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  multiple?: boolean;
  directory?: boolean;
}

interface FileFilter {
  name: string;
  extensions: string[];
}
```

### dialog.message(options)

Show a message dialog. Returns the index of the clicked button:

```typescript
import { dialog } from "host:ui";

const result = await dialog.message({
  title: "Confirm",
  message: "Are you sure?",
  kind: "warning",
});
```

### Convenience Dialogs

```typescript
import { dialog } from "host:ui";

// Alert dialog
await dialog.alert("Operation completed!");

// Confirm dialog (returns boolean)
const confirmed = await dialog.confirm("Delete this file?");

// Error dialog
await dialog.error("Something went wrong!");

// Warning dialog
await dialog.warning("This action cannot be undone!");
```

---

## Menus

### setAppMenu(items)

Set the application menu bar:

```typescript
import { setAppMenu } from "host:ui";

await setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "new", label: "New", accelerator: "CmdOrCtrl+N" },
      { id: "open", label: "Open...", accelerator: "CmdOrCtrl+O" },
      { type: "separator" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
  {
    label: "Edit",
    submenu: [
      { id: "undo", label: "Undo", accelerator: "CmdOrCtrl+Z" },
      { id: "redo", label: "Redo", accelerator: "CmdOrCtrl+Shift+Z" },
    ],
  },
]);
```

### showContextMenu(items, windowId?)

Show a context menu at the cursor position:

```typescript
import { showContextMenu } from "host:ui";

const selected = await showContextMenu([
  { id: "cut", label: "Cut" },
  { id: "copy", label: "Copy" },
  { id: "paste", label: "Paste" },
]);

if (selected) {
  console.log("Selected:", selected);
}
```

### onMenu(callback)

Register a callback for menu events. Returns an unsubscribe function:

```typescript
import { onMenu } from "host:ui";

const unsubscribe = onMenu((event) => {
  console.log(`Menu clicked: ${event.itemId}`);

  switch (event.itemId) {
    case "new":
      createNewDocument();
      break;
    case "quit":
      Deno.exit(0);
      break;
  }
});

// Later: unsubscribe();
```

### menuEvents()

Async iterator for menu events:

```typescript
import { menuEvents } from "host:ui";

for await (const event of menuEvents()) {
  console.log(`Menu: ${event.menuId}, Item: ${event.itemId}`);
}
```

### MenuItem

```typescript
interface MenuItem {
  id?: string;
  label: string;
  accelerator?: string;
  enabled?: boolean;
  checked?: boolean;
  submenu?: MenuItem[];
  type?: "normal" | "checkbox" | "separator";
}
```

### MenuEvent

```typescript
interface MenuEvent {
  menuId: string;
  itemId: string;
  label: string;
}
```

---

## System Tray

### createTray(options?)

Create a system tray icon:

```typescript
import { createTray } from "host:ui";

const trayIcon = await createTray({
  icon: "./assets/tray-icon.png",
  tooltip: "My App",
  menu: [
    { id: "show", label: "Show Window" },
    { type: "separator" },
    { id: "quit", label: "Quit" },
  ],
});
```

### TrayHandle

```typescript
interface TrayHandle {
  readonly id: string;
  update(options: TrayOptions): Promise<boolean>;
  destroy(): Promise<boolean>;
}
```

Update tray properties:

```typescript
await trayIcon.update({
  tooltip: "New Status",
  menu: [
    { id: "status", label: "Status: Online", enabled: false },
    { type: "separator" },
    { id: "quit", label: "Quit" },
  ],
});
```

Destroy the tray:

```typescript
await trayIcon.destroy();
```

### destroyTray(trayId)

Destroy a tray by ID:

```typescript
import { destroyTray } from "host:ui";

await destroyTray("my-tray-id");
```

### TrayOptions

```typescript
interface TrayOptions {
  icon?: string;
  tooltip?: string;
  menu?: MenuItem[];
}
```

---

## Complete Example

```typescript
import { openWindow, dialog, onMenu, createTray } from "host:ui";
import { sendToWindow, onChannel } from "host:ipc";

// Create main window
const win = await openWindow({
  title: "My Application",
  width: 1024,
  height: 768,
});

// Set up application menu
await setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "open", label: "Open...", accelerator: "CmdOrCtrl+O" },
      { type: "separator" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
]);

// Handle menu events
onMenu(async (event) => {
  if (event.itemId === "open") {
    const files = await dialog.open({
      filters: [{ name: "Documents", extensions: ["txt", "md"] }],
    });
    if (files) {
      await sendToWindow(win.id, "file-opened", { path: files[0] });
    }
  } else if (event.itemId === "quit") {
    Deno.exit(0);
  }
});

// Create system tray
const trayIcon = await createTray({
  tooltip: "My Application",
  menu: [
    { id: "show", label: "Show Window" },
    { id: "quit", label: "Quit" },
  ],
});

// Handle IPC from renderer
onChannel("ready", (payload, windowId) => {
  console.log(`Window ${windowId} is ready`);
});
```
