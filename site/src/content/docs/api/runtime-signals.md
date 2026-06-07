---
title: "runtime:signals"
description: "Subscribe to OS signals (Unix only) for Forge applications"
slug: api/runtime-signals
---

Subscribe to operating system signals for Forge applications. Allows handling of signals like SIGTERM, SIGINT, and SIGHUP for graceful shutdown and process management.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_signals](/docs/crates/ext-signals) for implementation details.

## Platform Support

| Platform | Support |
|----------|---------|
| macOS | Full support |
| Linux | Full support |
| Windows | Limited (SIGINT only) |

## Import

```typescript
import { supportedSignals, subscribe, type SignalEvent, type SignalSubscription } from "runtime:signals";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.signals.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
supportedSignals(): string[]
subscribe(signals: string[]): Promise<SignalSubscription>
```
<!-- /forge:api -->

### supportedSignals()

Get list of supported signals on the current platform.

**Returns:** `string[]` - Array of supported signal names

**Example:**

```typescript
import { supportedSignals } from "runtime:signals";

const signals = supportedSignals();
console.log("Supported signals:", signals.join(", "));
// Unix: SIGTERM, SIGINT, SIGHUP, SIGUSR1, SIGUSR2, SIGQUIT, ...
// Windows: SIGINT
```

---

### subscribe(signals)

Subscribe to one or more signals. Returns a subscription that can be used to receive signal events.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `signals` | `string[]` | Array of signal names to subscribe to |

**Returns:** `Promise<SignalSubscription>` - Subscription handle

**Example:**

```typescript
import { subscribe } from "runtime:signals";

// Subscribe to termination signals
const subscription = await subscribe(["SIGTERM", "SIGINT"]);

// Handle signals
while (true) {
  const event = await subscription.next();
  if (!event) break;

  console.log(`Received signal: ${event.signal}`);

  if (event.signal === "SIGTERM" || event.signal === "SIGINT") {
    console.log("Graceful shutdown initiated...");
    await cleanup();
    break;
  }
}

// Unsubscribe when done
await subscription.unsubscribe();
```

## SignalSubscription Interface

The subscription object returned by `subscribe()` provides methods for receiving and managing signal events.

### subscription.id

The unique subscription ID (bigint).

### subscription.next()

Wait for the next signal event. Returns `null` when the subscription is closed or the app is shutting down.

**Returns:** `Promise<SignalEvent | null>` - Signal event or null

**Example:**

```typescript
const event = await subscription.next();
if (event) {
  console.log(`Signal: ${event.signal}`);
}
```

### subscription.unsubscribe()

Unsubscribe from signals and clean up resources.

**Returns:** `Promise<boolean>` - True if successfully unsubscribed

**Example:**

```typescript
const success = await subscription.unsubscribe();
console.log("Unsubscribed:", success);
```

## Type Definitions

```typescript
/**
 * Signal event received from the OS
 */
interface SignalEvent {
  /** Name of the signal (e.g., "SIGTERM", "SIGINT") */
  signal: string;
}

/**
 * Subscription handle for signal events
 */
interface SignalSubscription {
  /** Unique subscription ID */
  id: bigint;

  /** Wait for the next signal event */
  next(): Promise<SignalEvent | null>;

  /** Unsubscribe from signals */
  unsubscribe(): Promise<boolean>;
}
```

## Common Signals

### Unix Signals

| Signal | Description | Common Use |
|--------|-------------|------------|
| `SIGTERM` | Termination request | Graceful shutdown from system |
| `SIGINT` | Interrupt (Ctrl+C) | User interrupt from terminal |
| `SIGHUP` | Hangup | Terminal closed, config reload |
| `SIGUSR1` | User-defined 1 | Custom application behavior |
| `SIGUSR2` | User-defined 2 | Custom application behavior |
| `SIGQUIT` | Quit (Ctrl+\\) | Quit with core dump |

### Windows Signals

| Signal | Description | Common Use |
|--------|-------------|------------|
| `SIGINT` | Interrupt (Ctrl+C) | User interrupt |

## Lifecycle Hooks

Signal operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:signals";

onBefore("subscribe", (args) => {
  console.log("Subscribing to signals");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:signals";

onAfter("subscribe", (result) => {
  console.log("Subscribed with ID:", result);
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:signals";

onError("subscribe", (error) => {
  console.error("Signal subscription failed:", error.message);
});
```

**Available operation names:** `"supported"`, `"subscribe"`, `"next"`, `"unsubscribe"`

## Complete Example

```typescript
import { supportedSignals, subscribe } from "runtime:signals";

// Application state
let isShuttingDown = false;

// Cleanup function
async function cleanup() {
  console.log("Cleaning up...");

  // Close database connections
  // await db.close();

  // Save state
  // await state.save();

  // Close file handles
  // await files.closeAll();

  console.log("Cleanup complete");
}

// Main application
async function main() {
  // Check supported signals
  const supported = supportedSignals();
  console.log("Supported signals:", supported.join(", "));

  // Subscribe to termination signals
  const signalsToHandle = supported.filter(s =>
    ["SIGTERM", "SIGINT", "SIGHUP"].includes(s)
  );

  if (signalsToHandle.length === 0) {
    console.warn("No termination signals supported on this platform");
    return;
  }

  console.log(`Subscribing to: ${signalsToHandle.join(", ")}`);
  const subscription = await subscribe(signalsToHandle);

  console.log("Application running. Press Ctrl+C to exit.\n");

  // Signal handling loop
  while (!isShuttingDown) {
    const event = await subscription.next();

    if (!event) {
      console.log("Signal subscription ended");
      break;
    }

    console.log(`\nReceived signal: ${event.signal}`);

    switch (event.signal) {
      case "SIGTERM":
        console.log("SIGTERM received - initiating graceful shutdown");
        isShuttingDown = true;
        break;

      case "SIGINT":
        if (isShuttingDown) {
          console.log("Force quit - exiting immediately");
          process.exit(1);
        }
        console.log("SIGINT received - shutting down gracefully");
        console.log("(Press Ctrl+C again to force quit)");
        isShuttingDown = true;
        break;

      case "SIGHUP":
        console.log("SIGHUP received - reloading configuration");
        // await reloadConfig();
        break;

      default:
        console.log(`Unhandled signal: ${event.signal}`);
    }
  }

  // Cleanup
  await cleanup();
  await subscription.unsubscribe();

  console.log("Goodbye!");
}

// Run with error handling
main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
```

## Use Cases

### Graceful HTTP Server Shutdown

```typescript
import { subscribe } from "runtime:signals";

async function startServer() {
  // Start HTTP server
  const server = createServer();
  server.listen(3000);
  console.log("Server listening on port 3000");

  // Handle shutdown signals
  const subscription = await subscribe(["SIGTERM", "SIGINT"]);

  const event = await subscription.next();
  if (event) {
    console.log(`Received ${event.signal}, shutting down...`);

    // Stop accepting new connections
    server.close();

    // Wait for existing connections to finish
    await waitForConnections(server);

    console.log("Server shut down gracefully");
  }

  await subscription.unsubscribe();
}
```

### Configuration Reload on SIGHUP

```typescript
import { subscribe } from "runtime:signals";

let config = await loadConfig();

async function watchForReload() {
  const subscription = await subscribe(["SIGHUP"]);

  while (true) {
    const event = await subscription.next();
    if (!event) break;

    if (event.signal === "SIGHUP") {
      console.log("Reloading configuration...");
      config = await loadConfig();
      console.log("Configuration reloaded");
    }
  }
}
```

### Custom Signal Handlers

```typescript
import { subscribe, supportedSignals } from "runtime:signals";

// Check if SIGUSR1 is available
if (supportedSignals().includes("SIGUSR1")) {
  const subscription = await subscribe(["SIGUSR1"]);

  // SIGUSR1 could trigger a status dump
  (async () => {
    while (true) {
      const event = await subscription.next();
      if (!event) break;

      console.log("=== Application Status ===");
      console.log(`Memory: ${process.memoryUsage().heapUsed / 1024 / 1024} MB`);
      console.log(`Uptime: ${process.uptime()} seconds`);
      console.log("========================");
    }
  })();
}
```

## Best Practices

### Always Unsubscribe

```typescript
const subscription = await subscribe(["SIGTERM"]);
try {
  // Handle signals...
} finally {
  await subscription.unsubscribe();
}
```

### Check Platform Support

```typescript
const supported = supportedSignals();
const desired = ["SIGTERM", "SIGINT", "SIGHUP"];
const available = desired.filter(s => supported.includes(s));

if (available.length !== desired.length) {
  console.warn("Some signals not available on this platform");
}
```

### Handle Multiple Ctrl+C Presses

```typescript
let ctrlCCount = 0;

while (true) {
  const event = await subscription.next();
  if (!event) break;

  if (event.signal === "SIGINT") {
    ctrlCCount++;
    if (ctrlCCount === 1) {
      console.log("Shutting down gracefully... (Ctrl+C again to force)");
      // Start graceful shutdown
    } else {
      console.log("Force quitting!");
      process.exit(1);
    }
  }
}
```

### Don't Block the Signal Handler

```typescript
// Good - non-blocking cleanup
if (event.signal === "SIGTERM") {
  cleanup().catch(console.error);
  isShuttingDown = true;
}

// Avoid - blocking for too long may cause forceful termination
if (event.signal === "SIGTERM") {
  await veryLongOperation(); // May timeout
}
```
