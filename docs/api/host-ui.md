# host:ui API Reference

The `host:ui` module provides window management, dialogs, menus, and tray icon functionality.

## Window Management

### openWindow(options?)

Opens a new window and returns a handle for interaction.

```typescript
import { openWindow } from "host:ui";

const win = await openWindow({
  url: "app://index.html",
  width: 800,
  height: 600,
  title: "My Window",
  resizable: true,
  decorations: true,
  channels: ["*"]  // Allowed IPC channels
});
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | `string` | `"app://index.html"` | URL to load |
| `width` | `number` | `800` | Window width in pixels |
| `height` | `number` | `600` | Window height in pixels |
| `title` | `string` | App name | Window title |
| `resizable` | `boolean` | `true` | Whether window can be resized |
| `decorations` | `boolean` | `true` | Show window decorations |
| `transparent` | `boolean` | `false` | Transparent window background |
| `always_on_top` | `boolean` | `false` | Keep window above others |
| `visible` | `boolean` | `true` | Initial visibility |
| `channels` | `string[]` | From manifest | Allowed IPC channels |

**Returns:** `Promise<WindowHandle>`

### WindowHandle

Handle returned by `openWindow()`:

```typescript
interface WindowHandle {
  readonly id: string;
  send(channel: string, payload?: unknown): Promise<void>;
  emit(channel: string, payload?: unknown): Promise<void>;  // Alias for send
  setTitle(title: string): Promise<void>;
  close(): Promise<boolean>;
  events(): AsyncGenerator<WindowEvent, void, unknown>;
}
```

### windowEvents()

Async generator yielding events from all windows:

```typescript
import { windowEvents } from "host:ui";

for await (const event of windowEvents()) {
  console.log(event.windowId, event.channel, event.payload);
}
```

**Event shape:**

```typescript
interface WindowEvent {
  windowId: string;
  channel: string;
  payload: unknown;
  type?: "close" | "focus" | "blur" | "resize" | "move";
}
```

### closeWindow(windowId)

Close a window by ID:

```typescript
import { closeWindow } from "host:ui";
await closeWindow("window-123");
```

---

## Dialogs

### dialog.open(options?)

Show an open file/folder dialog:

```typescript
import { dialog } from "host:ui";

const paths = await dialog.open({
  title: "Select File",
  multiple: true,
  filters: [
    { name: "Images", extensions: ["png", "jpg", "gif"] },
    { name: "All Files", extensions: ["*"] }
  ]
});
// Returns: string[] | null
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `title` | `string` | `"Open"` | Dialog title |
| `defaultPath` | `string` | - | Default directory |
| `filters` | `FileFilter[]` | - | File type filters |
| `multiple` | `boolean` | `false` | Allow multiple selection |
| `directory` | `boolean` | `false` | Select directories |

### dialog.save(options?)

Show a save file dialog:

```typescript
const path = await dialog.save({
  title: "Save As",
  defaultPath: "~/Documents/untitled.txt",
  filters: [
    { name: "Text Files", extensions: ["txt"] }
  ]
});
// Returns: string | null
```

### dialog.message(options)

Show a message dialog:

```typescript
const result = await dialog.message({
  title: "Confirm",
  message: "Are you sure?",
  kind: "warning",  // "info" | "warning" | "error"
  buttons: ["Cancel", "OK"]
});
// Returns: number (button index)
```

### Convenience Methods

```typescript
await dialog.alert("Message");
const confirmed = await dialog.confirm("Are you sure?");  // Returns boolean
await dialog.error("Something went wrong");
await dialog.warning("Be careful!");
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
      { id: "sep", label: "-", type: "separator" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" }
    ]
  },
  {
    label: "Edit",
    submenu: [
      { id: "undo", label: "Undo", accelerator: "CmdOrCtrl+Z" },
      { id: "redo", label: "Redo", accelerator: "CmdOrCtrl+Shift+Z" }
    ]
  }
]);
```

### MenuItem

```typescript
interface MenuItem {
  id?: string;              // Used in events
  label: string;            // Display text (or "-" for separator)
  accelerator?: string;     // e.g., "CmdOrCtrl+S", "Alt+F4"
  enabled?: boolean;        // Default: true
  checked?: boolean;        // For checkbox items
  submenu?: MenuItem[];     // Nested menu
  type?: "normal" | "checkbox" | "separator";
}
```

### showContextMenu(items, windowId?)

Show a context menu at cursor position:

```typescript
import { showContextMenu } from "host:ui";

const selectedId = await showContextMenu([
  { id: "cut", label: "Cut" },
  { id: "copy", label: "Copy" },
  { id: "paste", label: "Paste" }
]);

if (selectedId === "cut") {
  // Handle cut
}
```

### onMenu(callback)

Register a callback for menu events:

```typescript
import { onMenu } from "host:ui";

const unsubscribe = onMenu((event) => {
  console.log("Menu clicked:", event.itemId, event.label);
});

// Later: unsubscribe();
```

**Event shape:**

```typescript
interface MenuEvent {
  menuId: string;   // "app", "context", or "tray"
  itemId: string;   // The MenuItem's id
  label: string;
}
```

---

## Tray Icons

### createTray(options?)

Create a system tray icon:

```typescript
import { createTray } from "host:ui";

const tray = await createTray({
  tooltip: "My App",
  icon: "app://icon.png",  // Optional
  menu: [
    { id: "show", label: "Show Window" },
    { id: "sep", label: "-", type: "separator" },
    { id: "quit", label: "Quit" }
  ]
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

### Updating the Tray

```typescript
await tray.update({
  tooltip: "New Status",
  menu: [
    { id: "status", label: "Online", enabled: false },
    { id: "quit", label: "Quit" }
  ]
});
```

---

## Renderer API (window.host)

In the renderer (web content), use `window.host`:

```javascript
// Send message to Deno
window.host.send("channel", { data: "value" });

// Emit (alias for send)
window.host.emit("ready");

// Listen for messages
const off = window.host.on("update", (payload) => {
  console.log("Received:", payload);
});

// Stop listening
off();
// or
window.host.off("update");
```
