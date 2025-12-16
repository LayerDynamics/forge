---
title: "ext_protocol"
description: Custom protocol handler extension providing the runtime:protocol module.
slug: crates/ext-protocol
---

The `ext_protocol` crate provides custom URL protocol handling for Forge applications through the `runtime:protocol` module.

## Overview

ext_protocol handles:

- **Protocol registration** - Register custom URL schemes
- **Protocol handling** - Handle incoming protocol requests
- **Deep linking** - App activation via URLs
- **Protocol association** - OS-level protocol registration

## Module: `runtime:protocol`

```typescript
import {
  register,
  unregister,
  isRegistered,
  onProtocolRequest,
  setAsDefaultProtocolClient
} from "runtime:protocol";
```

## Key Types

### Error Types

```rust
enum ProtocolErrorCode {
    Generic = 9900,
    RegisterFailed = 9901,
    UnregisterFailed = 9902,
    AlreadyRegistered = 9903,
    InvalidProtocol = 9904,
    PermissionDenied = 9905,
}

struct ProtocolError {
    code: ProtocolErrorCode,
    message: String,
}
```

### Protocol Types

```rust
struct ProtocolRegistration {
    scheme: String,
    handler: ProtocolHandler,
}

struct ProtocolRequest {
    url: String,
    scheme: String,
    host: Option<String>,
    path: String,
    query: HashMap<String, String>,
}

struct ProtocolResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

struct ProtocolState {
    registrations: HashMap<String, ProtocolRegistration>,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_protocol_register` | `register(scheme, handler)` | Register protocol handler |
| `op_protocol_unregister` | `unregister(scheme)` | Remove handler |
| `op_protocol_is_registered` | `isRegistered(scheme)` | Check registration |
| `op_protocol_on_request` | `onProtocolRequest(callback)` | Handle requests |
| `op_protocol_set_default` | `setAsDefaultProtocolClient(scheme)` | Set as OS default |
| `op_protocol_remove_default` | `removeAsDefaultProtocolClient(scheme)` | Remove as OS default |

## Usage Examples

### Registering a Protocol

```typescript
import { register } from "runtime:protocol";

// Register myapp:// protocol
await register("myapp", async (request) => {
  console.log(`Received: ${request.url}`);
  console.log(`Path: ${request.path}`);
  console.log(`Query:`, request.query);

  return {
    status: 200,
    headers: { "Content-Type": "text/html" },
    body: new TextEncoder().encode("<h1>Hello from myapp://</h1>")
  };
});
```

### Deep Linking

```typescript
import { register, setAsDefaultProtocolClient } from "runtime:protocol";

// Register as handler for myapp:// URLs
await setAsDefaultProtocolClient("myapp");

// Handle incoming URLs
await register("myapp", async (request) => {
  // myapp://open?file=/path/to/file
  if (request.path === "open" && request.query.file) {
    await openFile(request.query.file);
  }

  // myapp://settings
  if (request.path === "settings") {
    await showSettings();
  }

  return { status: 200 };
});
```

### Custom Content Protocol

```typescript
import { register } from "runtime:protocol";

// Serve custom content via asset:// protocol
await register("asset", async (request) => {
  const path = request.path;

  // Load asset from app bundle
  const content = await loadAsset(path);
  const mimeType = getMimeType(path);

  return {
    status: content ? 200 : 404,
    headers: { "Content-Type": mimeType },
    body: content || new Uint8Array()
  };
});
```

### Protocol Events

```typescript
import { onProtocolRequest } from "runtime:protocol";

// Listen for any protocol requests
const unsubscribe = await onProtocolRequest((request) => {
  console.log(`Protocol request: ${request.scheme}://${request.path}`);

  // Log or handle at app level
  analytics.track("protocol_request", {
    scheme: request.scheme,
    path: request.path
  });
});
```

## OS Integration

| Platform | Registration Location |
|----------|----------------------|
| macOS | `Info.plist` CFBundleURLTypes |
| Windows | Registry HKEY_CLASSES_ROOT |
| Linux | `.desktop` file MimeType |

## File Structure

```text
crates/ext_protocol/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct ProtocolRequest {
    pub url: String,
    pub scheme: String,
    pub host: Option<String>,
    pub path: String,
}

#[weld_struct]
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_protocol_register(
    state: Rc<RefCell<OpState>>,
    #[string] scheme: String,
) -> Result<(), ProtocolError> {
    // implementation
}

#[weld_op]
#[op2]
pub fn op_protocol_is_registered(
    state: Rc<RefCell<OpState>>,
    #[string] scheme: String,
) -> Result<bool, ProtocolError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_protocol", "runtime:protocol")
        .ts_path("ts/init.ts")
        .ops(&["op_protocol_register", "op_protocol_unregister", "op_protocol_is_registered", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_protocol extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `url` | URL parsing |
| `serde` | Serialization |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_shell](/docs/crates/ext-shell) - Shell integration
- [ext_ipc](/docs/crates/ext-ipc) - IPC communication
- [Architecture](/docs/architecture) - Full system architecture
