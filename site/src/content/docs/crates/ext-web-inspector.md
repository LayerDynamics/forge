---
title: "ext_web_inspector"
description: WebView DevTools extension providing the runtime:web_inspector module for debugging and inspection.
slug: crates/ext-web-inspector
---

The `ext_web_inspector` crate provides WebView debugging and inspection capabilities for Forge applications through the `runtime:web_inspector` module.

## Overview

ext_web_inspector enables:

- **DevTools Integration** - Open Chrome DevTools for WebView debugging
- **Custom CDP Domains** - Forge-specific Chrome DevTools Protocol extensions
- **Runtime Inspection** - Inspect Deno runtime state from DevTools
- **Trace Visualization** - View performance traces in DevTools timeline
- **Signal Debugging** - Monitor reactive signal state changes
- **IPC Monitoring** - Track Deno ↔ WebView communication

## Module: `runtime:web_inspector`

```typescript
import {
  openDevTools,
  closeDevTools,
  isDevToolsOpen,
  attachCdpClient,
  sendCdpCommand,
  onCdpEvent
} from "runtime:web_inspector";
```

## Key Types

### Configuration Types

```typescript
interface DevToolsOptions {
  // Window ID to inspect
  windowId: number;

  // DevTools mode: "detached" | "docked" | "undocked"
  mode?: DevToolsMode;

  // Enable custom Forge CDP domains
  forgeExtensions?: boolean;
}

interface CdpClientOptions {
  // WebSocket URL for CDP connection
  wsUrl?: string;

  // Window ID (alternative to wsUrl)
  windowId?: number;

  // Custom domains to enable
  domains?: string[];
}
```

### CDP Types

```typescript
interface CdpMessage {
  id: number;
  method: string;
  params?: Record<string, unknown>;
}

interface CdpResponse {
  id: number;
  result?: unknown;
  error?: CdpError;
}

interface CdpError {
  code: number;
  message: string;
  data?: unknown;
}

interface CdpEvent {
  method: string;
  params: Record<string, unknown>;
}
```

## Custom CDP Domains

ext_web_inspector extends Chrome DevTools Protocol with Forge-specific domains:

### Forge.Monitor

Monitor system resources and app performance.

```typescript
// Enable monitoring
await sendCdpCommand("Forge.Monitor.enable", {
  metrics: ["cpu", "memory", "fps"]
});

// Receive metrics events
onCdpEvent("Forge.Monitor.metrics", (event) => {
  console.log(`CPU: ${event.cpu}%, Memory: ${event.memory}MB`);
});
```

### Forge.Trace

Access tracing spans and performance data.

```typescript
// Start trace capture
await sendCdpCommand("Forge.Trace.start", {
  categories: ["*"]
});

// Stop and get trace data
const trace = await sendCdpCommand("Forge.Trace.stop");
// Returns Chrome trace format for DevTools import
```

### Forge.Signals

Debug reactive signal state.

```typescript
// Enable signal debugging
await sendCdpCommand("Forge.Signals.enable");

// Receive signal change events
onCdpEvent("Forge.Signals.changed", (event) => {
  console.log(`Signal ${event.name}: ${event.oldValue} -> ${event.newValue}`);
});

// Get current signal state
const signals = await sendCdpCommand("Forge.Signals.getAll");
```

### Forge.Runtime

Inspect Deno runtime state.

```typescript
// Get runtime info
const info = await sendCdpCommand("Forge.Runtime.getInfo");
// Returns: { extensions, permissions, opState, ... }

// List loaded modules
const modules = await sendCdpCommand("Forge.Runtime.getModules");

// Evaluate in Deno context (not WebView)
const result = await sendCdpCommand("Forge.Runtime.evaluate", {
  expression: "Deno.version"
});
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_web_inspector_open` | `openDevTools(options)` | Open DevTools for window |
| `op_web_inspector_close` | `closeDevTools(windowId)` | Close DevTools |
| `op_web_inspector_is_open` | `isDevToolsOpen(windowId)` | Check DevTools state |
| `op_web_inspector_attach_cdp` | `attachCdpClient(options)` | Attach CDP client |
| `op_web_inspector_send_cdp` | `sendCdpCommand(method, params)` | Send CDP command |
| `op_web_inspector_on_cdp_event` | `onCdpEvent(method, callback)` | Subscribe to CDP events |

## Usage Examples

### Basic DevTools

```typescript
import { openDevTools, closeDevTools, isDevToolsOpen } from "runtime:web_inspector";

// Open DevTools for a window
await openDevTools({
  windowId: 1,
  mode: "detached",
  forgeExtensions: true
});

// Check if open
if (await isDevToolsOpen(1)) {
  console.log("DevTools is open");
}

// Close DevTools
await closeDevTools(1);
```

### Custom CDP Client

```typescript
import { attachCdpClient, sendCdpCommand, onCdpEvent } from "runtime:web_inspector";

// Attach to a window's CDP
await attachCdpClient({
  windowId: 1,
  domains: ["Forge.Monitor", "Forge.Signals"]
});

// Enable monitoring
await sendCdpCommand("Forge.Monitor.enable", {
  metrics: ["cpu", "memory"],
  interval: 1000  // Update every second
});

// Listen for metrics
onCdpEvent("Forge.Monitor.metrics", (metrics) => {
  console.log(`CPU: ${metrics.cpu}%, Heap: ${metrics.heapUsed}MB`);
});
```

### Performance Tracing

```typescript
import { sendCdpCommand } from "runtime:web_inspector";

// Capture a trace
await sendCdpCommand("Forge.Trace.start", {
  categories: ["forge.ipc", "forge.render", "v8"]
});

// ... run the operations you want to trace ...

const traceData = await sendCdpCommand("Forge.Trace.stop");

// Save for import into DevTools Performance panel
await Deno.writeTextFile("trace.json", JSON.stringify(traceData));
```

### Signal Debugging

```typescript
import { sendCdpCommand, onCdpEvent } from "runtime:web_inspector";

// Enable signal tracking
await sendCdpCommand("Forge.Signals.enable");

// Log all signal changes
onCdpEvent("Forge.Signals.changed", (event) => {
  console.log(`[Signal] ${event.name}:`, event.newValue);
  if (event.stack) {
    console.log("  Changed from:", event.stack);
  }
});

// Get snapshot of all signals
const signals = await sendCdpCommand("Forge.Signals.getAll");
console.log("Current signals:", signals);
```

## File Structure

```text
crates/ext_web_inspector/
├── src/
│   ├── lib.rs        # Extension implementation
│   ├── inspector.rs  # DevTools window management
│   ├── cdp.rs        # CDP protocol handling
│   └── domains/      # Custom Forge CDP domains
│       ├── monitor.rs
│       ├── trace.rs
│       ├── signals.rs
│       └── runtime.rs
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Error Codes

```rust
enum WebInspectorErrorCode {
    Generic = 8700,
    WindowNotFound = 8701,
    DevToolsUnavailable = 8702,
    CdpConnectionFailed = 8703,
    CdpCommandFailed = 8704,
    InvalidDomain = 8705,
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `wry` | WebView DevTools integration |
| `tokio-tungstenite` | WebSocket for CDP |
| `serde_json` | CDP message serialization |

## Platform Support

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| DevTools window | WebKit | Edge/Chromium | WebKit |
| CDP over WebSocket | Yes | Yes | Yes |
| Forge extensions | Yes | Yes | Yes |

## Related

- [runtime:trace](/docs/crates/ext-trace) - Performance tracing
- [runtime:monitor](/docs/crates/ext-monitor) - System monitoring
- [runtime:devtools](/docs/crates/ext-devtools) - Developer tools utilities
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) - CDP specification
