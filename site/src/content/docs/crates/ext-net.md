---
title: "ext_net"
description: Network operations extension providing the host:net module.
---

The `ext_net` crate provides HTTP fetch and network operations for Forge applications through the `host:net` module.

## Overview

ext_net handles:

- **HTTP fetch** - GET, POST, PUT, DELETE, etc.
- **Request configuration** - Headers, body, timeout
- **Response handling** - JSON, text, bytes
- **Capability-based security** - URL-based permission checks

## Module: `host:net`

```typescript
import {
  fetchJson,
  fetchText,
  fetchBytes
} from "host:net";
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

## Related

- [host:net API](/docs/api/host-net) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Build system
