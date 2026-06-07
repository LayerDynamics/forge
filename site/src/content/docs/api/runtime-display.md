---
title: "runtime:display"
description: "Display and monitor information for Forge applications"
slug: api/runtime-display
---

Display and monitor information for Forge applications. Provides monitor enumeration, cursor position tracking, and display change event subscriptions.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_display](/docs/crates/ext-display) for implementation details.

## Features

- Enumerate all connected monitors
- Get primary monitor and monitor by ID
- Track cursor position in virtual screen coordinates
- Subscribe to monitor connection/disconnection events
- Detect resolution, scale factor, and refresh rate changes

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Application                              │
├─────────────────────────────────────────────────────────────┤
│  getAll()  getPrimary()  getCursorPosition()  subscribe()   │
├─────────────────────────────────────────────────────────────┤
│                    runtime:display                           │
├─────────────────────────────────────────────────────────────┤
│          tao/wry Monitor API  |  Platform APIs               │
│  (MonitorHandle, Position)   |  (AppleScript, Win32, xdotool)│
└─────────────────────────────────────────────────────────────┘
```

## Import

```typescript
import {
  // Query functions
  getAll,
  getPrimary,
  getById,
  getAtPoint,
  getCursorPosition,
  getCount,
  // Subscription
  subscribe,
  unsubscribe,
  nextEvent,
  getSubscriptions,
  // Convenience
  getDisplayInfo,
  getMonitorAtCursor,
  watchDisplays,
  formatRefreshRate,
  formatResolution,
  // Types
  type MonitorInfo,
  type CursorPosition,
  type DisplayEvent,
  type SubscribeOptions,
} from "runtime:display";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.display.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
info(): ExtensionInfo
echo(message: string): string
getAll(): MonitorInfo[]
getPrimary(): MonitorInfo | null
getById(id: string): MonitorInfo | null
getAtPoint(x: number, y: number): MonitorInfo | null
getCursorPosition(): CursorPosition
getCount(): number
subscribe(options: SubscribeOptions =
nextEvent(subscriptionId: string): Promise<DisplayEvent | null>
unsubscribe(subscriptionId: string): void
getSubscriptions(): SubscriptionInfo[]
getDisplayInfo():
getMonitorAtCursor(): MonitorInfo | null
watchDisplays( callback: (event: DisplayEvent) => void, options: SubscribeOptions =
formatRefreshRate(millihertz: number): string
formatResolution(size: Size, scaleFactor?: number): string
all(): MonitorInfo[]
monitors(): MonitorInfo[]
primary(): MonitorInfo | null
count(): number
cursor(): CursorPosition
```
<!-- /forge:api -->

### Query Functions

#### getAll()

Get all connected monitors.

**Returns:** `MonitorInfo[]`

**Example:**

```typescript
import { getAll } from "runtime:display";

const monitors = getAll();
for (const monitor of monitors) {
  console.log(`${monitor.name}: ${monitor.size.width}x${monitor.size.height}`);
  console.log(`  Position: (${monitor.position.x}, ${monitor.position.y})`);
  console.log(`  Scale: ${monitor.scale_factor}x`);
  console.log(`  Primary: ${monitor.is_primary}`);
}
```

---

#### getPrimary()

Get the primary monitor.

**Returns:** `MonitorInfo | null`

**Example:**

```typescript
import { getPrimary } from "runtime:display";

const primary = getPrimary();
if (primary) {
  console.log(`Primary display: ${primary.size.width}x${primary.size.height}`);
  console.log(`Scale factor: ${primary.scale_factor}x`);
}
```

---

#### getById(id)

Get a monitor by its unique ID.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | Monitor ID (format: "name:x,y") |

**Returns:** `MonitorInfo | null`

**Example:**

```typescript
import { getById, getAll } from "runtime:display";

const monitors = getAll();
const monitor = getById(monitors[0].id);
if (monitor) {
  console.log(`Found: ${monitor.name}`);
}
```

---

#### getAtPoint(x, y)

Get the monitor at a specific screen coordinate.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `x` | `number` | X coordinate in virtual screen space |
| `y` | `number` | Y coordinate in virtual screen space |

**Returns:** `MonitorInfo | null`

**Example:**

```typescript
import { getAtPoint } from "runtime:display";

// Find monitor at top-left of screen
const monitor = getAtPoint(0, 0);
if (monitor) {
  console.log(`Monitor at origin: ${monitor.name}`);
}
```

---

#### getCursorPosition()

Get the current cursor position.

**Returns:** `CursorPosition`

**Platform Notes:**
- macOS: Uses AppleScript
- Windows: Uses Win32 API `GetCursorPos`
- Linux: Uses `xdotool` (must be installed)

**Example:**

```typescript
import { getCursorPosition } from "runtime:display";

const pos = getCursorPosition();
console.log(`Cursor at: (${pos.x}, ${pos.y})`);
if (pos.monitor_id) {
  console.log(`On monitor: ${pos.monitor_id}`);
}
```

---

#### getCount()

Get the number of connected monitors.

**Returns:** `number`

**Example:**

```typescript
import { getCount } from "runtime:display";

const count = getCount();
console.log(`${count} monitor(s) connected`);
```

### Subscription API

#### subscribe(options?)

Subscribe to display change events. Monitors for connection, disconnection, and property changes.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options.intervalMs` | `number` | Polling interval (min: 500ms, default: 1000ms) |

**Returns:** `Promise<string>` - Subscription ID

**Example:**

```typescript
import { subscribe, nextEvent, unsubscribe } from "runtime:display";

// Start monitoring (poll every second)
const subId = await subscribe({ intervalMs: 1000 });

// Listen for events
while (true) {
  const event = await nextEvent(subId);
  if (!event) break;

  switch (event.type) {
    case "MonitorConnected":
      console.log(`New monitor: ${event.data.monitor.name}`);
      break;
    case "MonitorDisconnected":
      console.log(`Monitor removed: ${event.data.monitor_id}`);
      break;
    case "MonitorChanged":
      console.log(`Monitor changed: ${event.data.changes.join(", ")}`);
      break;
  }
}

// Stop monitoring
unsubscribe(subId);
```

---

#### nextEvent(subscriptionId)

Get the next display event from a subscription.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `subscriptionId` | `string` | ID from subscribe() |

**Returns:** `Promise<DisplayEvent | null>` - Event or null if cancelled

---

#### unsubscribe(subscriptionId)

Cancel a display subscription.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `subscriptionId` | `string` | ID from subscribe() |

**Throws:** Error if subscription ID is invalid

---

#### getSubscriptions()

List all active display subscriptions.

**Returns:** `SubscriptionInfo[]`

**Example:**

```typescript
import { getSubscriptions } from "runtime:display";

const subs = getSubscriptions();
for (const sub of subs) {
  console.log(`Subscription ${sub.id}: ${sub.event_count} events`);
  console.log(`  Active: ${sub.is_active}`);
  console.log(`  Interval: ${sub.interval_ms}ms`);
}
```

### Convenience Functions

#### getDisplayInfo()

Get a complete display configuration summary.

**Returns:** `{ count, primary, monitors, virtualSize }`

**Example:**

```typescript
import { getDisplayInfo } from "runtime:display";

const info = getDisplayInfo();
console.log(`${info.count} display(s)`);
console.log(`Virtual screen: ${info.virtualSize.width}x${info.virtualSize.height}`);
if (info.primary) {
  console.log(`Primary: ${info.primary.name}`);
}
```

---

#### getMonitorAtCursor()

Get the monitor the cursor is currently on.

**Returns:** `MonitorInfo | null`

**Example:**

```typescript
import { getMonitorAtCursor } from "runtime:display";

const monitor = getMonitorAtCursor();
if (monitor) {
  console.log(`Cursor is on: ${monitor.name}`);
}
```

---

#### watchDisplays(callback, options?)

Watch for display changes with a callback function.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callback` | `(event: DisplayEvent) => void` | Event handler |
| `options` | `SubscribeOptions` | Subscription options |

**Returns:** `Promise<() => void>` - Stop function

**Example:**

```typescript
import { watchDisplays } from "runtime:display";

const stop = await watchDisplays((event) => {
  console.log(`Display event: ${event.type}`);
});

// Later, stop watching
stop();
```

---

#### formatRefreshRate(millihertz)

Format refresh rate from millihertz to a human-readable string.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `millihertz` | `number` | Refresh rate in millihertz |

**Returns:** `string` - Formatted string (e.g., "60 Hz")

---

#### formatResolution(size, scaleFactor?)

Format monitor resolution as a string.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `size` | `Size` | Size with width and height |
| `scaleFactor` | `number` | Optional scale factor |

**Returns:** `string` - Formatted string

**Example:**

```typescript
import { formatResolution, getPrimary } from "runtime:display";

const primary = getPrimary();
if (primary) {
  console.log(formatResolution(primary.size, primary.scale_factor));
  // "1920x1080 (3840x2160 @2x)" for HiDPI
}
```

## Type Definitions

```typescript
/** A 2D position in screen coordinates */
interface Position {
  x: number;
  y: number;
}

/** A 2D size in pixels */
interface Size {
  width: number;
  height: number;
}

/** Information about a connected monitor */
interface MonitorInfo {
  /** Unique identifier (format: "name:x,y") */
  id: string;
  /** Human-readable name (may be null) */
  name: string | null;
  /** Position in virtual screen coordinates */
  position: Position;
  /** Size in pixels */
  size: Size;
  /** DPI scale factor (1.0 = 100%, 2.0 = 200% HiDPI) */
  scale_factor: number;
  /** Whether this is the primary monitor */
  is_primary: boolean;
  /** Refresh rate in millihertz (60000 = 60Hz), null if unavailable */
  refresh_rate_millihertz: number | null;
}

/** Current cursor position */
interface CursorPosition {
  x: number;
  y: number;
  /** Monitor ID the cursor is on (if determinable) */
  monitor_id: string | null;
}

/** Types of monitor property changes */
type MonitorChangeType =
  | "ScaleFactor"
  | "Position"
  | "Size"
  | "RefreshRate"
  | "Primary";

/** Display event */
type DisplayEvent =
  | { type: "MonitorConnected"; data: { monitor: MonitorInfo } }
  | { type: "MonitorDisconnected"; data: { monitor_id: string } }
  | { type: "MonitorChanged"; data: { monitor: MonitorInfo; changes: MonitorChangeType[] } };

/** Subscription options */
interface SubscribeOptions {
  /** Polling interval in milliseconds (min: 500, default: 1000) */
  intervalMs?: number;
}

/** Subscription information */
interface SubscriptionInfo {
  id: string;
  interval_ms: number;
  is_active: boolean;
  event_count: number;
}
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:display";

onBefore("getAll", () => {
  console.log("Querying monitors...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:display";

onAfter("subscribe", (result) => {
  console.log("Subscribed with ID:", result);
});
```

**Available operation names:** `"info"`, `"echo"`, `"getAll"`, `"getPrimary"`, `"getById"`, `"getAtPoint"`, `"getCursorPosition"`, `"getCount"`, `"subscribe"`, `"unsubscribe"`, `"nextEvent"`, `"subscriptions"`

## Complete Examples

### Multi-Monitor Window Positioning

```typescript
import { getAll, getPrimary, getDisplayInfo } from "runtime:display";
import { create } from "runtime:window";

// Position window on secondary monitor
async function openOnSecondaryMonitor() {
  const monitors = getAll();
  const secondary = monitors.find(m => !m.is_primary);

  if (secondary) {
    await create({
      title: "Secondary Window",
      x: secondary.position.x + 100,
      y: secondary.position.y + 100,
      width: 800,
      height: 600,
    });
  } else {
    console.log("No secondary monitor available");
  }
}

// Center window on primary monitor
async function centerOnPrimary() {
  const primary = getPrimary();
  if (!primary) return;

  const windowWidth = 800;
  const windowHeight = 600;

  await create({
    title: "Centered Window",
    x: primary.position.x + (primary.size.width - windowWidth) / 2,
    y: primary.position.y + (primary.size.height - windowHeight) / 2,
    width: windowWidth,
    height: windowHeight,
  });
}
```

### Display Configuration Monitor

```typescript
import { watchDisplays, getDisplayInfo, formatResolution, formatRefreshRate } from "runtime:display";

async function monitorDisplayConfig() {
  // Show initial state
  const info = getDisplayInfo();
  console.log(`\nDisplay Configuration:`);
  console.log(`Virtual screen: ${info.virtualSize.width}x${info.virtualSize.height}`);
  console.log(`\nMonitors (${info.count}):`);

  for (const m of info.monitors) {
    console.log(`  ${m.name || "Unknown"} ${m.is_primary ? "(Primary)" : ""}`);
    console.log(`    Resolution: ${formatResolution(m.size, m.scale_factor)}`);
    console.log(`    Position: (${m.position.x}, ${m.position.y})`);
    if (m.refresh_rate_millihertz) {
      console.log(`    Refresh: ${formatRefreshRate(m.refresh_rate_millihertz)}`);
    }
  }

  // Watch for changes
  console.log("\nWatching for display changes...\n");

  const stop = await watchDisplays((event) => {
    switch (event.type) {
      case "MonitorConnected":
        console.log(`+ Monitor connected: ${event.data.monitor.name}`);
        console.log(`  ${formatResolution(event.data.monitor.size)}`);
        break;

      case "MonitorDisconnected":
        console.log(`- Monitor disconnected: ${event.data.monitor_id}`);
        break;

      case "MonitorChanged":
        console.log(`~ Monitor changed: ${event.data.monitor.name}`);
        console.log(`  Changes: ${event.data.changes.join(", ")}`);
        if (event.data.changes.includes("ScaleFactor")) {
          console.log(`  New scale: ${event.data.monitor.scale_factor}x`);
        }
        if (event.data.changes.includes("Size")) {
          console.log(`  New size: ${formatResolution(event.data.monitor.size)}`);
        }
        break;
    }
  });

  return stop;
}
```

### Cursor Position Tracker

```typescript
import { getCursorPosition, getAtPoint, getMonitorAtCursor } from "runtime:display";

function trackCursor() {
  let lastMonitorId: string | null = null;

  setInterval(() => {
    const pos = getCursorPosition();
    const monitor = getMonitorAtCursor();

    // Detect monitor crossing
    if (monitor && monitor.id !== lastMonitorId) {
      console.log(`Cursor moved to monitor: ${monitor.name || monitor.id}`);
      lastMonitorId = monitor.id;
    }

    // Optional: log position
    // console.log(`Cursor: (${pos.x}, ${pos.y})`);
  }, 100);
}
```

### Responsive Layout Based on Display

```typescript
import { getPrimary, watchDisplays } from "runtime:display";

interface LayoutConfig {
  fontSize: number;
  sidebarWidth: number;
  compactMode: boolean;
}

function calculateLayout(): LayoutConfig {
  const primary = getPrimary();
  if (!primary) {
    return { fontSize: 14, sidebarWidth: 250, compactMode: false };
  }

  // Adjust for HiDPI
  const effectiveWidth = primary.size.width / primary.scale_factor;
  const effectiveHeight = primary.size.height / primary.scale_factor;

  return {
    fontSize: primary.scale_factor >= 2 ? 16 : 14,
    sidebarWidth: effectiveWidth > 1920 ? 300 : 200,
    compactMode: effectiveWidth < 1280,
  };
}

async function initResponsiveLayout(onLayoutChange: (config: LayoutConfig) => void) {
  // Initial layout
  onLayoutChange(calculateLayout());

  // Watch for display changes
  const stop = await watchDisplays((event) => {
    if (event.type === "MonitorChanged") {
      const changes = event.data.changes;
      if (changes.includes("ScaleFactor") || changes.includes("Size")) {
        onLayoutChange(calculateLayout());
      }
    }
  });

  return stop;
}
```

## Aliases

The module provides convenient aliases:

```typescript
import {
  all,       // alias for getAll
  monitors,  // alias for getAll
  primary,   // alias for getPrimary
  count,     // alias for getCount
  cursor,    // alias for getCursorPosition
} from "runtime:display";
```

## Best Practices

### Cache Monitor Info for Frequent Access

```typescript
// Good - cache for frequent position calculations
let cachedMonitors = getAll();

// Refresh on display changes
watchDisplays(() => {
  cachedMonitors = getAll();
});

function findMonitorForWindow(x: number, y: number) {
  return cachedMonitors.find(m =>
    x >= m.position.x &&
    x < m.position.x + m.size.width &&
    y >= m.position.y &&
    y < m.position.y + m.size.height
  );
}
```

### Handle Missing xdotool on Linux

```typescript
import { getCursorPosition } from "runtime:display";

function safeCursorPosition() {
  try {
    return getCursorPosition();
  } catch (e) {
    console.warn("Cursor position unavailable (install xdotool on Linux)");
    return { x: 0, y: 0, monitor_id: null };
  }
}
```

### Clean Up Subscriptions

```typescript
const subscriptions: string[] = [];

// Track subscriptions
const subId = await subscribe();
subscriptions.push(subId);

// Clean up on exit
function cleanup() {
  for (const id of subscriptions) {
    try {
      unsubscribe(id);
    } catch {
      // Already unsubscribed
    }
  }
}
```
