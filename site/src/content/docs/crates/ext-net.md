---
title: "ext_net"
description: Network operations extension providing the runtime:net module.
slug: crates/ext-net
---

The `ext_net` crate provides HTTP fetch and network operations for Forge applications through the `runtime:net` module.

## Overview

ext_net handles:

- **HTTP fetch** - GET, POST, PUT, DELETE, etc.
- **Request configuration** - Headers, body, timeout
- **Response handling** - JSON, text, bytes
- **Capability-based security** - URL-based permission checks

## Module: `runtime:net`

```typescript
import {
  fetchJson,
  fetchText,
  fetchBytes
} from "runtime:net";
```

## Key Types

### Error Types

```rust
enum NetErrorCode {
    Io = 1000,
    PermissionDenied = 1001,
    InvalidUrl = 1002,
    Timeout = 1003,
    ConnectionFailed = 1004,
    RequestFailed = 1005,
}

struct NetError {
    code: NetErrorCode,
    message: String,
}
```

### Request/Response Types

```rust
struct FetchOpts {
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

struct FetchResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

struct FetchBytesResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
```

### Capability Types

```rust
struct NetCapabilities {
    allowed_hosts: Vec<String>,
    denied_hosts: Vec<String>,
}

trait NetCapabilityChecker {
    fn check_url(&self, url: &str) -> bool;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_net_fetch` | `fetch(url, opts?)` | HTTP fetch returning text |
| `op_net_fetch_json` | `fetchJson(url, opts?)` | HTTP fetch parsing JSON |
| `op_net_fetch_bytes` | `fetchBytes(url, opts?)` | HTTP fetch returning bytes |

## File Structure

```text
crates/ext_net/
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
pub struct FetchResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_net_fetch(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<FetchOpts>,
) -> Result<FetchResponse, NetError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_net", "runtime:net")
        .ts_path("ts/init.ts")
        .ops(&["op_net_fetch", "op_net_fetch_json", "op_net_fetch_bytes"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_net extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `reqwest` | HTTP client |
| `url` | URL parsing |
| `tokio` | Async runtime |
| `serde` | JSON serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [runtime:net API](/docs/api/runtime-net) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Code generation library
