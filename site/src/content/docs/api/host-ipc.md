---
title: "host:ipc"
description: Inter-process communication between Deno and window renderers.
---

The `host:ipc` module provides inter-process communication (IPC) between your Deno application and window renderers (WebViews).

## Overview

IPC enables bidirectional messaging:
- **Deno to Renderer**: Send data to windows using `sendToWindow()`
- **Renderer to Deno**: Receive events using `windowEvents()` or callbacks

---

## Types

### IpcEvent

Event received from a renderer:

```typescript
interface IpcEvent {
  /** Window ID that sent the event */
  windowId: string;
  /** Channel name for the event */
  channel: string;
  /** Event payload data */
  payload: unknown;
  /** Event type for window system events */
  type?: "close" | "focus" | "blur" | "resize" | "move";
}
```

### Callback Types

```typescript
/** Callback for all IPC events */
type IpcEventCallback = (event: IpcEvent) => void;

/** Callback for channel-specific handlers */
type ChannelCallback = (payload: unknown, windowId: string) => void;
```

---

## Sending Messages

### sendToWindow(windowId, channel, payload?)

Send a message to a specific window's renderer. The message is received by `window.host.on(channel, callback)` in the WebView.

```typescript
import { sendToWindow } from "host:ipc";

// Send data to a specific window
await sendToWindow("main-window", "update", { count: 42 });

// Send a simple notification
await sendToWindow("main-window", "refresh");
```

### broadcast(windowIds, channel, payload?)

Send a message to multiple windows simultaneously:

```typescript
import { broadcast } from "host:ipc";

// Send to multiple windows
await broadcast(
  ["main", "settings", "preview"],
  "theme-changed",
  { theme: "dark" }
);
```

---

## Receiving Events

### Using Async Generators

#### windowEvents()

Async generator that yields all window events:

```typescript
import { windowEvents } from "host:ipc";

for await (const event of windowEvents()) {
  console.log(`[${event.windowId}] ${event.channel}:`, event.payload);

  if (event.type === "close") {
    console.log("Window closed");
    break;
  }
}
```

#### windowEventsFor(windowId)

Filter events for a specific window:

```typescript
import { windowEventsFor } from "host:ipc";

// Only process events from the main window
for await (const event of windowEventsFor("main")) {
  console.log("Main window event:", event.channel);
}
```

#### channelEvents(channel)

Filter events for a specific channel:

```typescript
import { channelEvents } from "host:ipc";

// Only process "button-click" events
for await (const event of channelEvents("button-click")) {
  console.log("Button clicked in window:", event.windowId);
}
```

### Using Callbacks

#### onEvent(callback)

Register a callback for all IPC events. Returns an unsubscribe function:

```typescript
import { onEvent } from "host:ipc";

const unsubscribe = onEvent((event) => {
  console.log(`Event: ${event.channel} from ${event.windowId}`);
});

// Later, to stop listening:
unsubscribe();
```

#### onChannel(channel, callback)

Register a callback for events on a specific channel:

```typescript
import { onChannel } from "host:ipc";

const unsubscribe = onChannel("user-action", (payload, windowId) => {
  console.log(`User action from ${windowId}:`, payload);
});

// Later, to stop listening:
unsubscribe();
```

### Low-Level Receive

#### recvWindowEvent()

Receive the next event (blocking). Returns `null` when the channel closes:

```typescript
import { recvWindowEvent } from "host:ipc";

const event = await recvWindowEvent();
if (event) {
  console.log(`Received: ${event.channel} from ${event.windowId}`);
}
```

---

## Renderer API

In your WebView (renderer), use the `window.host` API to communicate with Deno:

### Sending from Renderer

```javascript
// Send a message to Deno
window.host.send("button-click", { buttonId: "submit" });

// Send with no payload
window.host.send("refresh");
```

### Receiving in Renderer

```javascript
// Listen for messages from Deno
window.host.on("update", (payload) => {
  console.log("Received update:", payload);
});

// Remove listener
window.host.off("update", handler);
```

---

## Complete Example

### Deno Side (main.ts)

```typescript
import { createWindow } from "host:window";
import { sendToWindow, onChannel } from "host:ipc";

// Create the main window
const win = await createWindow({
  title: "My App",
  width: 800,
  height: 600,
});

// Listen for button clicks from the renderer
onChannel("button-click", async (payload, windowId) => {
  const { buttonId } = payload as { buttonId: string };
  console.log(`Button ${buttonId} clicked in ${windowId}`);

  // Send response back to the window
  await sendToWindow(windowId, "button-response", {
    success: true,
    message: `Processed ${buttonId}`,
  });
});

// Listen for window close
onChannel("close", async (_, windowId) => {
  console.log(`Window ${windowId} requested close`);
});
```

### Renderer Side (index.html)

```html
<!DOCTYPE html>
<html>
<body>
  <button id="submit">Submit</button>
  <div id="status"></div>

  <script>
    const submitBtn = document.getElementById("submit");
    const status = document.getElementById("status");

    // Send click event to Deno
    submitBtn.addEventListener("click", () => {
      window.host.send("button-click", { buttonId: "submit" });
    });

    // Receive response from Deno
    window.host.on("button-response", (payload) => {
      status.textContent = payload.message;
    });
  </script>
</body>
</html>
```

---

## Patterns

### Request-Response Pattern

```typescript
import { sendToWindow, onChannel } from "host:ipc";

// Deno: Handle requests and send responses
onChannel("fetch-data", async (payload, windowId) => {
  const { requestId, query } = payload as { requestId: string; query: string };

  // Process the request
  const result = await fetchFromDatabase(query);

  // Send response with matching requestId
  await sendToWindow(windowId, "fetch-response", {
    requestId,
    data: result,
  });
});
```

### Multi-Window Sync

```typescript
import { broadcast, onEvent } from "host:ipc";

// Track all window IDs
const windows = new Set<string>();

onEvent((event) => {
  windows.add(event.windowId);

  // Broadcast state changes to all windows
  if (event.channel === "state-change") {
    broadcast(
      Array.from(windows),
      "state-update",
      event.payload
    );
  }
});
```

### Event Routing

```typescript
import { onChannel } from "host:ipc";

// Route different channels to different handlers
onChannel("auth", handleAuth);
onChannel("data", handleData);
onChannel("settings", handleSettings);

function handleAuth(payload: unknown, windowId: string) {
  // Handle authentication events
}

function handleData(payload: unknown, windowId: string) {
  // Handle data events
}

function handleSettings(payload: unknown, windowId: string) {
  // Handle settings events
}
```
