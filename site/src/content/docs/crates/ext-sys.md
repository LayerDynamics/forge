---
title: "ext_sys"
description: System operations extension providing the host:sys module.
---

The `ext_sys` crate provides system-level operations for Forge applications through the `host:sys` module.

## Overview

ext_sys handles:

- **System information** - OS, architecture, hostname
- **Clipboard** - Read and write text clipboard
- **Notifications** - Desktop notifications
- **Power/Battery** - Battery status and power info
- **Environment** - Environment variables
- **Capability-based security** - Permission checks per operation

## Module: `host:sys`

```typescript
import {
  info,
  clipboard,
  notify
} from "host:sys";
```

## Key Types

### Error Types

```rust
enum SysErrorCode {
    Io = 2000,
    PermissionDenied = 2001,
    NotSupported = 2002,
    ClipboardError = 2003,
    NotificationError = 2004,
}

struct SysError {
    code: SysErrorCode,
    message: String,
}
```

### Data Types

```rust
struct SystemInfo {
    os: String,
    arch: String,
    hostname: String,
    username: String,
    home_dir: Option<String>,
    temp_dir: String,
}

struct BatteryInfo {
    percentage: f32,
    is_charging: bool,
    time_to_full: Option<u64>,
    time_to_empty: Option<u64>,
}

struct PowerInfo {
    has_battery: bool,
    batteries: Vec<BatteryInfo>,
    is_on_ac: bool,
}

struct NotifyOpts {
    title: String,
    body: Option<String>,
    icon: Option<String>,
}
```

### Capability Types

```rust
struct SysCapabilities {
    clipboard_read: bool,
    clipboard_write: bool,
    notifications: bool,
    system_info: bool,
    power_info: bool,
}

trait SysCapabilityChecker {
    fn check_clipboard_read(&self) -> bool;
    fn check_clipboard_write(&self) -> bool;
    fn check_notifications(&self) -> bool;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_sys_info` | `info()` | Get system information |
| `op_sys_hostname` | `hostname()` | Get hostname |
| `op_sys_clipboard_read` | `clipboard.read()` | Read from clipboard |
| `op_sys_clipboard_write` | `clipboard.write(text)` | Write to clipboard |
| `op_sys_notify` | `notify(title, body?)` | Show notification |
| `op_sys_power` | `power()` | Get power/battery info |
| `op_sys_env` | `env(name)` | Get environment variable |
| `op_sys_env_all` | `envAll()` | Get all environment variables |

## Platform-Specific

### Notifications

| Platform | Implementation |
|----------|---------------|
| macOS | `mac-notification-sys` |
| Linux | `notify-rust` |
| Windows | Windows toast notifications |

### Clipboard

All platforms use `arboard` for clipboard access.

## File Structure

```text
crates/ext_sys/
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
| `arboard` | Clipboard access |
| `battery` | Battery status |
| `hostname` | Hostname retrieval |
| `dirs` | User directories |
| `mac-notification-sys` | macOS notifications |
| `notify-rust` | Linux notifications |
| `forge-weld` | Build-time code generation |

## Related

- [host:sys API](/docs/api/host-sys) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Build system
