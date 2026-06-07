---
title: "runtime:dock"
description: "macOS dock customization for Forge applications - badges, bounce, icons, and menus"
slug: api/runtime-dock
---

macOS dock icon customization for Forge applications. Control dock icon badges, bounce animations, visibility, custom icons, and right-click menus.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_dock](/docs/crates/ext-dock) for implementation details.

> **Platform**: macOS only. These functions are no-ops on Windows and Linux.

## Features

- Set dock icon badge text (unread counts, notifications)
- Bounce dock icon to get user attention
- Hide/show dock icon (accessory mode)
- Set custom dock icon at runtime
- Configure right-click dock menu

## Import

```typescript
import {
  // Functions
  bounce,
  cancelBounce,
  setBadge,
  getBadge,
  hide,
  show,
  isVisible,
  setIcon,
  setMenu,
  // Types
  type BounceType,
  type BounceResult,
  type MenuItem,
  // Hooks
  onBefore,
  onAfter,
  onError,
} from "runtime:dock";
```

## API Reference

### bounce(type?)

Bounce the dock icon to get user attention.

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `type` | `BounceType` | `"informational"` | Bounce behavior type |

**Bounce Types:**

| Value | Description |
|-------|-------------|
| `"informational"` | Bounces once |
| `"critical"` | Continues bouncing until app is activated |

**Returns:** `BounceResult` - Object with bounce ID and success status

**Example:**

```typescript
import { bounce, cancelBounce } from "runtime:dock";

// Single bounce to notify user
const result = bounce();

// Critical bounce for urgent notifications
const urgentResult = bounce("critical");

// Later, cancel the critical bounce if needed
if (urgentResult.success) {
  cancelBounce(urgentResult.id);
}
```

---

### cancelBounce(bounceId)

Cancel an active dock icon bounce animation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `bounceId` | `number` | Bounce ID returned from `bounce()` |

**Returns:** `void`

**Example:**

```typescript
import { bounce, cancelBounce } from "runtime:dock";

const result = bounce("critical");

// Cancel after user acknowledges
setTimeout(() => {
  cancelBounce(result.id);
}, 5000);
```

---

### setBadge(text)

Set the dock icon badge text.

Displays a badge overlay on the dock icon, typically used for unread counts or notification indicators.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `text` | `string` | Badge text (empty string clears badge) |

**Returns:** `void`

**Example:**

```typescript
import { setBadge } from "runtime:dock";

// Show unread message count
setBadge("5");

// Show indicator
setBadge("!");

// Clear the badge
setBadge("");
```

---

### getBadge()

Get the current dock badge text.

**Returns:** `string` - Current badge text, or empty string if no badge

**Example:**

```typescript
import { getBadge, setBadge } from "runtime:dock";

const current = getBadge();
if (current !== "") {
  console.log(`Current badge: ${current}`);
}
```

---

### hide()

Hide the dock icon.

Switches the app to "accessory" mode where it doesn't appear in the dock or Cmd+Tab app switcher, but can still have windows. Useful for menu bar apps or background utilities.

**Returns:** `void`

**Example:**

```typescript
import { hide, show, isVisible } from "runtime:dock";

// Switch to menu bar app mode
hide();

console.log("Dock visible:", isVisible()); // false

// Later, restore dock presence
show();
```

---

### show()

Show the dock icon.

Restores the app to "regular" mode where it appears in the dock and Cmd+Tab app switcher.

**Returns:** `void`

**Example:**

```typescript
import { show } from "runtime:dock";

// Restore dock icon visibility
show();
```

---

### isVisible()

Check if the dock icon is currently visible.

**Returns:** `boolean` - `true` if dock icon is visible

**Example:**

```typescript
import { isVisible, hide, show } from "runtime:dock";

function toggleDockVisibility() {
  if (isVisible()) {
    hide();
  } else {
    show();
  }
}
```

---

### setIcon(iconPath)

Set a custom dock icon.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `iconPath` | `string` | Path to image file (PNG, JPEG, etc.), or empty string to reset |

**Returns:** `boolean` - `true` if icon was set successfully

**Example:**

```typescript
import { setIcon } from "runtime:dock";

// Set custom dock icon
const success = setIcon("./assets/custom-icon.png");

if (success) {
  console.log("Dock icon updated");
}

// Reset to default app icon
setIcon("");
```

---

### setMenu(menu)

Set the dock menu (right-click menu on dock icon).

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `menu` | `MenuItem[]` | Array of menu items |

**Returns:** `boolean` - `true` if menu was set successfully

**Example:**

```typescript
import { setMenu } from "runtime:dock";

setMenu([
  { id: "new-window", label: "New Window", accelerator: "Cmd+N" },
  { id: "new-tab", label: "New Tab", accelerator: "Cmd+T" },
  { type: "separator" },
  { id: "preferences", label: "Preferences...", accelerator: "Cmd+," },
]);
```

### info()

Get dock extension information.

**Returns:** `ExtensionInfo`

---

### nextMenuEvent()

Wait for the next dock-menu item click (pull-based). Resolves when a menu item that has an `id` is clicked, or `null` if the event channel is unavailable (for example, another caller is already awaiting). Most apps should use [`onMenuItemClick`](#onmenuitemclicklistener) instead of polling directly.

**Returns:** `Promise<MenuClickEvent | null>`

**Platform:** macOS only (always resolves `null` on other platforms)

---

### onMenuItemClick(listener)

Register a listener for dock-menu item clicks. The listener is invoked with the clicked item's `id` (the `id` you set on the corresponding `MenuItem`). Items without an `id` do not emit events.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `listener` | `MenuItemClickListener` | Called with `(id, event)` when a menu item is clicked |

**Returns:** `() => void` - an unsubscribe function that removes the listener

**Platform:** macOS only (no events fire on other platforms)

**Example:**

```typescript
import { setMenu, onMenuItemClick } from "runtime:dock";

setMenu([
  { id: "new-window", label: "New Window" },
  { type: "separator" },
  { id: "preferences", label: "Preferences..." },
]);

const off = onMenuItemClick((id) => {
  if (id === "new-window") openWindow();
  if (id === "preferences") openPreferences();
});
// later, to stop listening:
off();
```

## Type Definitions

### BounceType

```typescript
type BounceType = "critical" | "informational";
```

### BounceResult

```typescript
interface BounceResult {
  /** Bounce request ID (used to cancel) */
  id: number;
  /** Whether the bounce was started successfully */
  success: boolean;
}
```

### MenuItem

```typescript
interface MenuItem {
  /** Unique identifier for the menu item */
  id?: string;
  /** Display label */
  label: string;
  /** Keyboard shortcut (e.g., "Cmd+N") */
  accelerator?: string;
  /** Whether the item is enabled (default: true) */
  enabled?: boolean;
  /** Whether the item is checked (for checkbox items) */
  checked?: boolean;
  /** Submenu items for nested menus */
  submenu?: MenuItem[];
  /** Item type */
  type?: "normal" | "checkbox" | "separator";
}
```

### MenuClickEvent

```typescript
interface MenuClickEvent {
  /** The `id` of the clicked menu item. */
  id: string;
  /** Click time in epoch milliseconds. */
  timestamp_ms: number;
}
```

### MenuItemClickListener

```typescript
type MenuItemClickListener = (id: string, event: MenuClickEvent) => void;
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:dock";

const unsubscribe = onBefore("bounce", () => {
  console.log("Bouncing dock icon...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:dock";

onAfter("setBadge", () => {
  console.log("Badge updated");
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:dock";

onError("setIcon", (error) => {
  console.error("Failed to set icon:", error.message);
});
```

**Available operation names:** `"bounce"`, `"cancelBounce"`, `"setBadge"`, `"getBadge"`, `"hide"`, `"show"`, `"isVisible"`, `"setIcon"`, `"setMenu"`

## Complete Examples

### Notification Badge Manager

```typescript
import { setBadge, getBadge, bounce } from "runtime:dock";

class NotificationBadge {
  private count = 0;

  increment(): void {
    this.count++;
    this.update();
  }

  decrement(): void {
    this.count = Math.max(0, this.count - 1);
    this.update();
  }

  clear(): void {
    this.count = 0;
    this.update();
  }

  private update(): void {
    if (this.count === 0) {
      setBadge("");
    } else if (this.count > 99) {
      setBadge("99+");
    } else {
      setBadge(String(this.count));
    }
  }

  notifyUrgent(): void {
    bounce("critical");
  }
}

// Usage
const badge = new NotificationBadge();
badge.increment(); // Shows "1"
badge.increment(); // Shows "2"
badge.notifyUrgent(); // Bounces until app activated
badge.clear(); // Clears badge
```

### Menu Bar App Mode

```typescript
import { hide, show, isVisible, setMenu } from "runtime:dock";

class MenuBarApp {
  private dockHidden = false;

  enableMenuBarMode(): void {
    hide();
    this.dockHidden = true;
    console.log("Running as menu bar app");
  }

  enableDockMode(): void {
    show();
    this.dockHidden = false;
    console.log("Running as dock app");
  }

  toggle(): void {
    if (isVisible()) {
      this.enableMenuBarMode();
    } else {
      this.enableDockMode();
    }
  }
}

const app = new MenuBarApp();
app.enableMenuBarMode();
```

### Dynamic Dock Menu

```typescript
import { setMenu, type MenuItem } from "runtime:dock";

interface RecentFile {
  name: string;
  path: string;
}

function updateDockMenu(recentFiles: RecentFile[]): void {
  const menuItems: MenuItem[] = [
    { id: "new", label: "New Document" },
    { id: "open", label: "Open..." },
  ];

  if (recentFiles.length > 0) {
    menuItems.push({ type: "separator" });
    menuItems.push({
      label: "Recent Files",
      submenu: recentFiles.map((file) => ({
        id: `recent:${file.path}`,
        label: file.name,
      })),
    });
  }

  menuItems.push({ type: "separator" });
  menuItems.push({ id: "preferences", label: "Preferences..." });

  setMenu(menuItems);
}

// Usage
updateDockMenu([
  { name: "report.pdf", path: "/docs/report.pdf" },
  { name: "notes.txt", path: "/docs/notes.txt" },
]);
```

### Dynamic Icon Based on State

```typescript
import { setIcon, setBadge } from "runtime:dock";

type AppState = "idle" | "syncing" | "error" | "success";

const icons: Record<AppState, string> = {
  idle: "./icons/app-idle.png",
  syncing: "./icons/app-syncing.png",
  error: "./icons/app-error.png",
  success: "./icons/app-success.png",
};

function updateAppState(state: AppState): void {
  setIcon(icons[state]);

  switch (state) {
    case "syncing":
      setBadge("⟳");
      break;
    case "error":
      setBadge("!");
      break;
    case "success":
      setBadge("✓");
      break;
    default:
      setBadge("");
  }
}

// Usage
updateAppState("syncing");
// Later...
updateAppState("success");
```

## Platform Notes

All `runtime:dock` functions are **macOS-only**:

| Platform | Behavior |
|----------|----------|
| macOS | Full functionality |
| Windows | No-op (functions return safely but do nothing) |
| Linux | No-op (functions return safely but do nothing) |

For cross-platform notification badges, consider using system tray icons via `runtime:window` which works on all platforms.
