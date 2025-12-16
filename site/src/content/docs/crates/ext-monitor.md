---
title: "ext_monitor"
description: System monitoring extension providing the runtime:monitor module.
slug: crates/ext-monitor
---

The `ext_monitor` crate provides system resource monitoring for Forge applications through the `runtime:monitor` module.

## Overview

ext_monitor handles:

- **CPU usage** - System and per-process CPU metrics
- **Memory usage** - RAM and swap statistics
- **Disk usage** - Storage space and I/O
- **Network stats** - Bandwidth and connection info
- **Process monitoring** - Track running processes

## Module: `runtime:monitor`

```typescript
import {
  getCpuUsage,
  getMemoryUsage,
  getDiskUsage,
  getNetworkStats,
  getProcessList,
  getProcessInfo
} from "runtime:monitor";
```

## Key Types

### Error Types

```rust
enum MonitorErrorCode {
    Generic = 9800,
    QueryFailed = 9801,
    ProcessNotFound = 9802,
    PermissionDenied = 9803,
}

struct MonitorError {
    code: MonitorErrorCode,
    message: String,
}
```

### Metric Types

```rust
struct CpuUsage {
    total_percent: f64,
    per_core: Vec<f64>,
    core_count: u32,
}

struct MemoryUsage {
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    available_bytes: u64,
    swap_total_bytes: u64,
    swap_used_bytes: u64,
}

struct DiskUsage {
    mount_point: String,
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    filesystem: String,
}

struct NetworkStats {
    interface: String,
    bytes_sent: u64,
    bytes_recv: u64,
    packets_sent: u64,
    packets_recv: u64,
}

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_percent: f64,
    memory_bytes: u64,
    status: ProcessStatus,
    start_time: u64,
}

enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_monitor_cpu` | `getCpuUsage()` | Get CPU usage stats |
| `op_monitor_memory` | `getMemoryUsage()` | Get memory stats |
| `op_monitor_disk` | `getDiskUsage(path?)` | Get disk usage |
| `op_monitor_network` | `getNetworkStats()` | Get network stats |
| `op_monitor_processes` | `getProcessList()` | List all processes |
| `op_monitor_process_info` | `getProcessInfo(pid)` | Get process details |

## Usage Examples

### CPU Monitoring

```typescript
import { getCpuUsage } from "runtime:monitor";

const cpu = await getCpuUsage();
console.log(`Total CPU: ${cpu.total_percent.toFixed(1)}%`);
console.log(`Cores: ${cpu.core_count}`);

for (let i = 0; i < cpu.per_core.length; i++) {
  console.log(`  Core ${i}: ${cpu.per_core[i].toFixed(1)}%`);
}
```

### Memory Monitoring

```typescript
import { getMemoryUsage } from "runtime:monitor";

const mem = await getMemoryUsage();

const usedGB = (mem.used_bytes / 1024 / 1024 / 1024).toFixed(2);
const totalGB = (mem.total_bytes / 1024 / 1024 / 1024).toFixed(2);
const percent = ((mem.used_bytes / mem.total_bytes) * 100).toFixed(1);

console.log(`Memory: ${usedGB} GB / ${totalGB} GB (${percent}%)`);
```

### Disk Usage

```typescript
import { getDiskUsage } from "runtime:monitor";

const disks = await getDiskUsage();
for (const disk of disks) {
  const usedGB = (disk.used_bytes / 1024 / 1024 / 1024).toFixed(1);
  const totalGB = (disk.total_bytes / 1024 / 1024 / 1024).toFixed(1);
  console.log(`${disk.mount_point}: ${usedGB} GB / ${totalGB} GB`);
}
```

### Network Statistics

```typescript
import { getNetworkStats } from "runtime:monitor";

const networks = await getNetworkStats();
for (const net of networks) {
  const sentMB = (net.bytes_sent / 1024 / 1024).toFixed(2);
  const recvMB = (net.bytes_recv / 1024 / 1024).toFixed(2);
  console.log(`${net.interface}: ↑${sentMB} MB ↓${recvMB} MB`);
}
```

### Process Monitoring

```typescript
import { getProcessList, getProcessInfo } from "runtime:monitor";

// List top processes by CPU
const processes = await getProcessList();
const topCpu = processes
  .sort((a, b) => b.cpu_percent - a.cpu_percent)
  .slice(0, 10);

for (const proc of topCpu) {
  console.log(`${proc.name} (${proc.pid}): ${proc.cpu_percent.toFixed(1)}% CPU`);
}

// Get details for specific process
const info = await getProcessInfo(1234);
console.log(`Process ${info.name}:`);
console.log(`  PID: ${info.pid}`);
console.log(`  CPU: ${info.cpu_percent}%`);
console.log(`  Memory: ${(info.memory_bytes / 1024 / 1024).toFixed(1)} MB`);
```

## File Structure

```text
crates/ext_monitor/
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
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct CpuUsage {
    pub total_percent: f64,
    pub per_core: Vec<f64>,
    pub core_count: u32,
}

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_monitor_cpu(
    state: Rc<RefCell<OpState>>,
) -> Result<CpuUsage, MonitorError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_monitor", "runtime:monitor")
        .ts_path("ts/init.ts")
        .ops(&["op_monitor_cpu", "op_monitor_memory", "op_monitor_disk", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_monitor extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `sysinfo` | System information |
| `serde` | Serialization |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_sys](/docs/crates/ext-sys) - System information
- [ext_process](/docs/crates/ext-process) - Process management
- [Architecture](/docs/architecture) - Full system architecture
