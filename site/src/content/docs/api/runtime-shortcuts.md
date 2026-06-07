---
title: "runtime:shortcuts"
description: "Global keyboard shortcuts for Forge applications"
slug: api/runtime-shortcuts
---

Global keyboard shortcuts for Forge applications with hotkey registration, event handling, and persistence across restarts.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_shortcuts](/docs/crates/ext-shortcuts) for implementation details.

## Features

- Register system-wide global keyboard shortcuts
- Listen for shortcut events even when app is not focused
- Enable/disable shortcuts dynamically
- Persist shortcuts across app restarts
- Cross-platform accelerator syntax (CmdOrCtrl, etc.)

## Import

```typescript
import {
  register,
  unregister,
  unregisterAll,
  list,
  enable,
  nextEvent,
  listen,
  handleShortcuts,
  save,
  load,
  setAutoPersist,
  getAutoPersist,
  parseAccelerator,
  formatAccelerator,
  registerAll
} from "runtime:shortcuts";
```

## Accelerator Syntax

Shortcuts are defined using an accelerator string that combines modifiers and keys.

### Supported Modifiers

| Modifier | Description |
|----------|-------------|
| `Ctrl` or `Control` | Control key |
| `Alt` or `Option` | Alt/Option key |
| `Shift` | Shift key |
| `Meta`, `Cmd`, or `Command` | Meta/Command key |
| `Super` | Super/Windows key |
| `CmdOrCtrl` | Command on macOS, Ctrl on Windows/Linux |

### Supported Keys

| Category | Keys |
|----------|------|
| Letters | A-Z |
| Numbers | 0-9 |
| Function keys | F1-F24 |
| Special | Space, Enter, Tab, Backspace, Delete, Escape, Home, End, PageUp, PageDown |
| Arrows | Up, Down, Left, Right |
| Punctuation | Minus, Equal, BracketLeft, BracketRight, Backslash, Semicolon, Quote, Comma, Period, Slash |

### Accelerator Examples

```typescript
"CmdOrCtrl+S"        // Save (Cmd+S on macOS, Ctrl+S on Windows/Linux)
"CmdOrCtrl+Shift+S"  // Save As
"Alt+F4"             // Close window
"F12"                // Developer tools
"Ctrl+Alt+Delete"    // Task manager (Windows)
"CmdOrCtrl+Shift+I"  // Inspect element
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.shortcuts.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
info(): ExtensionInfo
echo(message: string): string
register(config: ShortcutConfig): ShortcutInfo
unregister(id: string): void
unregisterAll(): void
list(): ShortcutInfo[]
enable(id: string, enabled: boolean): void
nextEvent(): Promise<ShortcutEvent | null>
save(): Promise<void>
load(): Promise<ShortcutConfig[]>
setAutoPersist(enabled: boolean): void
getAutoPersist(): boolean
registerAll(configs: ShortcutConfig[]): ShortcutInfo[]
listen( callback: (event: ShortcutEvent) => void ): Promise<() => void>
handleShortcuts( handlers: Record<string, () => void> ): Promise<() => void>
parseAccelerator(accelerator: string):
formatAccelerator(accelerator: string): string
add(config: ShortcutConfig): ShortcutInfo
remove(id: string): void
getAll(): ShortcutInfo[]
```
<!-- /forge:api -->

### Registration

#### register(config)

Register a global keyboard shortcut.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `config` | `ShortcutConfig` | Shortcut configuration |

**ShortcutConfig:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `id` | `string` | - | Unique identifier for the shortcut |
| `accelerator` | `string` | - | Keyboard accelerator string |
| `enabled` | `boolean` | `true` | Whether the shortcut is enabled |

**Returns:** `ShortcutInfo` - Information about the registered shortcut

**Throws:** Error if accelerator is invalid or ID already exists

**Example:**

```typescript
import { register } from "runtime:shortcuts";

// Register a save shortcut
const info = register({
  id: "save",
  accelerator: "CmdOrCtrl+S",
});
console.log(`Registered: ${info.id} (${info.accelerator})`);

// Register a custom shortcut
register({
  id: "toggle-dev-tools",
  accelerator: "CmdOrCtrl+Shift+I",
});

// Register as disabled initially
register({
  id: "secret-feature",
  accelerator: "Ctrl+Shift+Alt+S",
  enabled: false,
});
```

---

#### registerAll(configs)

Register multiple shortcuts at once.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `configs` | `ShortcutConfig[]` | Array of shortcut configurations |

**Returns:** `ShortcutInfo[]` - Array of registered shortcut info

**Example:**

```typescript
import { registerAll } from "runtime:shortcuts";

const shortcuts = registerAll([
  { id: "save", accelerator: "CmdOrCtrl+S" },
  { id: "open", accelerator: "CmdOrCtrl+O" },
  { id: "new", accelerator: "CmdOrCtrl+N" },
  { id: "close", accelerator: "CmdOrCtrl+W" },
]);
```

---

#### unregister(id)

Unregister a shortcut by ID.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | ID of the shortcut to unregister |

**Throws:** Error if shortcut with ID does not exist

**Example:**

```typescript
import { unregister } from "runtime:shortcuts";

unregister("save");
```

---

#### unregisterAll()

Unregister all registered shortcuts.

**Example:**

```typescript
import { unregisterAll } from "runtime:shortcuts";

unregisterAll();
```

---

#### list()

List all registered shortcuts.

**Returns:** `ShortcutInfo[]` - Array of shortcut info objects

**Example:**

```typescript
import { list } from "runtime:shortcuts";

const shortcuts = list();
for (const shortcut of shortcuts) {
  console.log(`${shortcut.id}: ${shortcut.accelerator}`);
  console.log(`  Enabled: ${shortcut.enabled}`);
  console.log(`  Triggered: ${shortcut.trigger_count} times`);
}
```

---

#### enable(id, enabled)

Enable or disable a shortcut. Disabled shortcuts will not trigger events.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | ID of the shortcut |
| `enabled` | `boolean` | Whether to enable or disable |

**Throws:** Error if shortcut with ID does not exist

**Example:**

```typescript
import { enable } from "runtime:shortcuts";

// Disable a shortcut temporarily
enable("save", false);

// Re-enable it
enable("save", true);
```

### Event Handling

#### nextEvent()

Wait for the next shortcut event. Async operation that resolves when any registered shortcut is triggered.

**Returns:** `Promise<ShortcutEvent | null>` - Event or null if shutting down

**Example:**

```typescript
import { register, nextEvent } from "runtime:shortcuts";

// Register shortcuts
register({ id: "save", accelerator: "CmdOrCtrl+S" });
register({ id: "quit", accelerator: "CmdOrCtrl+Q" });

// Listen for events
while (true) {
  const event = await nextEvent();
  if (!event) break;

  console.log(`Shortcut triggered: ${event.id}`);
  console.log(`At: ${new Date(event.timestamp_ms)}`);

  switch (event.id) {
    case "save":
      await saveDocument();
      break;
    case "quit":
      await quitApp();
      break;
  }
}
```

---

#### listen(callback)

Listen for shortcut events with a callback function.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callback` | `(event: ShortcutEvent) => void` | Function called on trigger |

**Returns:** `Promise<() => void>` - Stop function to cancel listening

**Example:**

```typescript
import { register, listen } from "runtime:shortcuts";

register({ id: "save", accelerator: "CmdOrCtrl+S" });

const stop = await listen((event) => {
  if (event.id === "save") {
    saveDocument();
  }
});

// Later, stop listening
stop();
```

---

#### handleShortcuts(handlers)

Create a shortcut handler map for cleaner event handling.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `handlers` | `Record<string, () => void>` | Map of shortcut IDs to handlers |

**Returns:** `Promise<() => void>` - Stop function to cancel listening

**Example:**

```typescript
import { register, handleShortcuts } from "runtime:shortcuts";

register({ id: "save", accelerator: "CmdOrCtrl+S" });
register({ id: "open", accelerator: "CmdOrCtrl+O" });
register({ id: "quit", accelerator: "CmdOrCtrl+Q" });

const stop = await handleShortcuts({
  save: () => saveDocument(),
  open: () => openFile(),
  quit: () => quitApp(),
});
```

### Persistence

#### save()

Save all registered shortcuts to persistent storage.

**Example:**

```typescript
import { register, save } from "runtime:shortcuts";

register({ id: "save", accelerator: "CmdOrCtrl+S" });
register({ id: "open", accelerator: "CmdOrCtrl+O" });

// Save for next app launch
await save();
```

---

#### load()

Load shortcuts from persistent storage. Returns configurations without registering them.

**Returns:** `Promise<ShortcutConfig[]>` - Array of saved shortcut configurations

**Example:**

```typescript
import { load, register } from "runtime:shortcuts";

// On app startup, restore saved shortcuts
const savedShortcuts = await load();
for (const config of savedShortcuts) {
  try {
    register(config);
  } catch (e) {
    console.error(`Failed to restore shortcut ${config.id}:`, e);
  }
}
```

---

#### setAutoPersist(enabled)

Enable or disable automatic persistence. When enabled, shortcuts are automatically saved on changes.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `enabled` | `boolean` | Whether to enable auto-persist |

**Example:**

```typescript
import { setAutoPersist, register } from "runtime:shortcuts";

// Enable auto-save
setAutoPersist(true);

// This will automatically be saved
register({ id: "save", accelerator: "CmdOrCtrl+S" });
```

---

#### getAutoPersist()

Check if auto-persist is enabled.

**Returns:** `boolean` - Whether auto-persist is enabled

### Utilities

#### parseAccelerator(accelerator)

Parse an accelerator string into its components.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `accelerator` | `string` | Accelerator string to parse |

**Returns:** `{ modifiers: string[], key: string }` - Parsed components

**Example:**

```typescript
import { parseAccelerator } from "runtime:shortcuts";

const { modifiers, key } = parseAccelerator("CmdOrCtrl+Shift+S");
console.log(modifiers); // ["CmdOrCtrl", "Shift"]
console.log(key);       // "S"
```

---

#### formatAccelerator(accelerator)

Format an accelerator for display (platform-specific).

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `accelerator` | `string` | Accelerator string |

**Returns:** `string` - Human-readable platform-specific string

**Example:**

```typescript
import { formatAccelerator } from "runtime:shortcuts";

const display = formatAccelerator("CmdOrCtrl+Shift+S");
// On macOS: "Cmd+Shift+S"
// On Windows/Linux: "Ctrl+Shift+S"
```

## Type Definitions

```typescript
/**
 * Configuration for registering a keyboard shortcut
 */
interface ShortcutConfig {
  /** Unique identifier for the shortcut */
  id: string;
  /** Keyboard accelerator string */
  accelerator: string;
  /** Whether the shortcut is enabled (default: true) */
  enabled?: boolean;
}

/**
 * Shortcut trigger event
 */
interface ShortcutEvent {
  /** ID of the triggered shortcut */
  id: string;
  /** Timestamp when triggered (Unix milliseconds) */
  timestamp_ms: number;
}

/**
 * Information about a registered shortcut
 */
interface ShortcutInfo {
  /** Unique identifier */
  id: string;
  /** Accelerator string */
  accelerator: string;
  /** Whether currently enabled */
  enabled: boolean;
  /** Number of times this shortcut has been triggered */
  trigger_count: number;
}
```

## Lifecycle Hooks

Shortcut operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:shortcuts";

onBefore("register", (args) => {
  console.log("Registering shortcut");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:shortcuts";

onAfter("register", (result) => {
  console.log("Shortcut registered:", result);
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:shortcuts";

onError("register", (error) => {
  console.error("Failed to register shortcut:", error.message);
});
```

**Available operation names:** `"info"`, `"echo"`, `"register"`, `"unregister"`, `"unregisterAll"`, `"list"`, `"enable"`, `"nextEvent"`, `"save"`, `"load"`, `"setAutoPersist"`, `"getAutoPersist"`

## Aliases

The module provides convenient aliases for common functions:

```typescript
import { add, remove, getAll } from "runtime:shortcuts";

add({ id: "save", accelerator: "CmdOrCtrl+S" });  // alias for register
remove("save");                                   // alias for unregister
const shortcuts = getAll();                       // alias for list
```

## Complete Example

```typescript
import {
  register,
  handleShortcuts,
  save,
  load,
  setAutoPersist,
  formatAccelerator,
  list
} from "runtime:shortcuts";

// Application state
let documentModified = false;
let currentFile: string | null = null;

// Shortcut handlers
async function saveDocument() {
  if (!currentFile) {
    return saveDocumentAs();
  }
  console.log(`Saving to ${currentFile}...`);
  documentModified = false;
}

async function saveDocumentAs() {
  // Would show file dialog
  console.log("Save As dialog...");
}

async function openFile() {
  // Would show file dialog
  console.log("Open dialog...");
}

async function newDocument() {
  if (documentModified) {
    // Would prompt to save
    console.log("Save changes?");
  }
  currentFile = null;
  documentModified = false;
  console.log("New document created");
}

async function quitApp() {
  if (documentModified) {
    console.log("Save before quit?");
  }
  console.log("Quitting...");
}

async function toggleDevTools() {
  console.log("Toggle developer tools");
}

// Initialize shortcuts
async function initShortcuts() {
  // Enable auto-save of shortcuts
  setAutoPersist(true);

  // Try to load saved shortcuts
  const savedShortcuts = await load();

  if (savedShortcuts.length > 0) {
    // Restore saved shortcuts
    console.log("Restoring saved shortcuts...");
    for (const config of savedShortcuts) {
      try {
        register(config);
      } catch (e) {
        console.warn(`Could not restore ${config.id}:`, e);
      }
    }
  } else {
    // Register default shortcuts
    console.log("Registering default shortcuts...");
    register({ id: "new", accelerator: "CmdOrCtrl+N" });
    register({ id: "open", accelerator: "CmdOrCtrl+O" });
    register({ id: "save", accelerator: "CmdOrCtrl+S" });
    register({ id: "save-as", accelerator: "CmdOrCtrl+Shift+S" });
    register({ id: "quit", accelerator: "CmdOrCtrl+Q" });
    register({ id: "dev-tools", accelerator: "F12" });
  }

  // Display registered shortcuts
  console.log("\nRegistered shortcuts:");
  for (const shortcut of list()) {
    console.log(`  ${shortcut.id}: ${formatAccelerator(shortcut.accelerator)}`);
  }

  // Start handling shortcuts
  const stop = await handleShortcuts({
    "new": newDocument,
    "open": openFile,
    "save": saveDocument,
    "save-as": saveDocumentAs,
    "quit": quitApp,
    "dev-tools": toggleDevTools,
  });

  // Return cleanup function
  return stop;
}

// Main
async function main() {
  console.log("Starting application...\n");

  const cleanup = await initShortcuts();

  console.log("\nApplication ready. Press shortcuts to trigger actions.");
  console.log("Press Ctrl+C to exit.\n");

  // Keep running
  await new Promise(() => {});
}

main().catch(console.error);
```

## Best Practices

### Use CmdOrCtrl for Cross-Platform Shortcuts

```typescript
// Good - works on all platforms
register({ id: "save", accelerator: "CmdOrCtrl+S" });

// Avoid - only works on one platform
register({ id: "save", accelerator: "Cmd+S" });     // macOS only
register({ id: "save", accelerator: "Ctrl+S" });    // Windows/Linux
```

### Check for Conflicts

```typescript
function registerSafely(config: ShortcutConfig): boolean {
  try {
    register(config);
    return true;
  } catch (error) {
    console.warn(`Shortcut ${config.id} already exists or invalid`);
    return false;
  }
}
```

### Clean Up on Exit

```typescript
import { unregisterAll } from "runtime:shortcuts";

// On app exit
process.on("beforeExit", () => {
  unregisterAll();
});
```

### Disable During Modal Dialogs

```typescript
import { enable, list } from "runtime:shortcuts";

function showModal() {
  // Disable all shortcuts during modal
  for (const shortcut of list()) {
    enable(shortcut.id, false);
  }

  // ... show modal ...

  // Re-enable after modal closes
  for (const shortcut of list()) {
    enable(shortcut.id, true);
  }
}
```
