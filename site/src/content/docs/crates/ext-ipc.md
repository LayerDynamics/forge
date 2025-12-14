---
title: "ext_ipc"
description: Inter-process communication extension providing the host:ipc module.
---

The `ext_ipc` crate provides inter-process communication (IPC) between the Deno runtime and WebView renderers through the `host:ipc` module.

## Overview

ext_ipc handles:

- **Deno → Renderer** - Send messages to windows via `sendToWindow()`
- **Renderer → Deno** - Receive events via `windowEvents()` or callbacks
- **Channel routing** - Route messages by channel name
- **Broadcast** - Send to multiple windows simultaneously
- **Capability-based security** - Channel allowlisting

## Module: `host:ipc`

```typescript
import {
  sendToWindow,
  onChannel,
  windowEvents,
  broadcast
} from "host:ipc";
```

## Key Types

### Event Types

```rust
struct IpcEvent {
    window_id: String,
    channel: String,
    payload: serde_json::Value,
    event_type: Option<String>,  // "close", "focus", etc.
}
```

### Command Types

```rust
struct ToRendererCmd {
    window_id: String,
    channel: String,
    payload: serde_json::Value,
}
```

### Error Types

```rust
enum IpcErrorCode {
    ChannelSend = 7000,
    ChannelRecv = 7001,
    WindowNotFound = 7002,
    Serialization = 7003,
}

struct IpcError {
    code: IpcErrorCode,
    message: String,
}
```

### State Types

```rust
struct IpcState {
    event_rx: mpsc::Receiver<IpcEvent>,
    cmd_tx: mpsc::Sender<ToRendererCmd>,
}

struct IpcCapabilities {
    allowed_channels: Vec<String>,
    denied_channels: Vec<String>,
}

trait IpcCapabilityChecker {
    fn check_channel(&self, channel: &str) -> bool;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_ipc_send` | `sendToWindow(id, channel, payload)` | Send to renderer |
| `op_ipc_recv` | `recvWindowEvent()` | Receive next event |

The TypeScript module builds higher-level APIs on these primitives:

```typescript
// Async generators
function* windowEvents(): AsyncGenerator<IpcEvent>;
function* windowEventsFor(windowId: string): AsyncGenerator<IpcEvent>;
function* channelEvents(channel: string): AsyncGenerator<IpcEvent>;

// Callbacks
function onEvent(callback: (event: IpcEvent) => void): () => void;
function onChannel(channel: string, callback: (payload, windowId) => void): () => void;

// Broadcast
async function broadcast(windowIds: string[], channel: string, payload?: unknown): Promise<void>;
```

## Message Flow

### Renderer → Deno

```text
Renderer                    Rust IPC                    Deno
────────────────────────────────────────────────────────────────
window.host.send()  ──►  WebView IPC handler  ──►  mpsc channel
                         (wry callback)             │
                                                    ▼
                                              windowEvents()
                                              onChannel()
```

### Deno → Renderer

```text
Deno                       Rust IPC                    Renderer
────────────────────────────────────────────────────────────────
sendToWindow()  ──►  op_ipc_send()  ──►  evaluate_script()
                                               │
                                               ▼
                                    window.__host_dispatch()
                                               │
                                               ▼
                                    window.host.on() callbacks
```

## File Structure

```text
crates/ext_ipc/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async channels |
| `serde`, `serde_json` | Message serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |

## Related

- [host:ipc API](/docs/api/host-ipc) - TypeScript API documentation
- [ext_window](/docs/crates/ext-window) - Window management
- [forge-host](/docs/crates/forge-host) - Runtime that bridges IPC
