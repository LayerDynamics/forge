# ext_monitor

System and runtime monitoring extension for Forge applications.

Provides real-time system metrics (CPU, memory, disk, network), Deno runtime metrics (event loop latency, uptime), process information, and subscription-based continuous monitoring. Built on the [`sysinfo`](https://docs.rs/sysinfo) crate for cross-platform system information access.

**Runtime Module:** `runtime:monitor`

## Features

- **System Metrics**: CPU usage (total and per-core), memory (RAM + swap), disk usage (all mounted filesystems), network statistics (per-interface traffic counters), process information (current process and system-wide top 50)
- **Runtime Metrics**: Event loop latency measurement, process uptime tracking, V8 heap statistics (placeholder - requires isolate access)
- **WebView Metrics**: Window count and visibility tracking (placeholder - requires ext_window integration)
- **Subscription API**: Continuous metric collection at configurable intervals (100ms minimum), selective metric inclusion, async iterator pattern for real-time monitoring, maximum 10 concurrent subscriptions
- **Cross-Platform**: Uses `sysinfo` crate for unified API across macOS (sysctl), Windows (WMI), and Linux (/proc)

## Usage Examples

### Basic System Snapshot

```typescript
import { getCpu, getMemory, getDisks, getNetwork } from "runtime:monitor";

// CPU usage (takes ~200ms for accurate measurement)
const cpu = await getCpu();
console.log(`CPU: ${cpu.total_percent.toFixed(1)}%`);
console.log(`Cores: ${cpu.core_count}`);
cpu.per_core.forEach((usage, i) => {
  console.log(`  Core ${i}: ${usage.toFixed(1)}%`);
});

// Memory usage (synchronous)
const mem = getMemory();
const usedGB = mem.used_bytes / (1024 ** 3);
const totalGB = mem.total_bytes / (1024 ** 3);
console.log(`Memory: ${usedGB.toFixed(1)} / ${totalGB.toFixed(1)} GB`);

// Disk usage
const disks = getDisks();
for (const disk of disks) {
  const usedPct = (disk.used_bytes / disk.total_bytes * 100).toFixed(1);
  console.log(`${disk.mount_point}: ${usedPct}% used`);
}

// Network statistics
const networks = getNetwork();
for (const net of networks) {
  const sentMB = net.bytes_sent / (1024 ** 2);
  const recvMB = net.bytes_recv / (1024 ** 2);
  console.log(`${net.interface}: sent ${sentMB.toFixed(1)} MB, recv ${recvMB.toFixed(1)} MB`);
}
```

### Process Monitoring

```typescript
import { getProcessSelf, getProcesses } from "runtime:monitor";

// Current process information
const proc = getProcessSelf();
console.log(`PID: ${proc.pid}`);
console.log(`Memory: ${(proc.memory_rss_bytes / 1024 / 1024).toFixed(1)} MB`);
console.log(`CPU: ${proc.cpu_percent.toFixed(1)}%`);
console.log(`Status: ${proc.status}`);

// Top processes by CPU usage (limited to 50)
const processes = getProcesses();
console.log("Top 5 CPU-consuming processes:");
for (const p of processes.slice(0, 5)) {
  console.log(`  ${p.name}: ${p.cpu_percent.toFixed(1)}%`);
}
```

### Runtime Metrics

```typescript
import { getRuntime, getHeap } from "runtime:monitor";

// Deno runtime metrics
const runtime = getRuntime();
console.log(`Uptime: ${runtime.uptime_secs}s`);
console.log(`Event loop latency: ${runtime.event_loop_latency_us}μs`);

// V8 heap statistics (placeholder - returns default values)
const heap = getHeap();
console.log(`Heap used: ${(heap.used_heap_size / 1024 / 1024).toFixed(1)} MB`);
```

### Subscription-Based Continuous Monitoring

```typescript
import { subscribe, nextSnapshot, unsubscribe } from "runtime:monitor";

// Start monitoring CPU and memory every 500ms
const subId = await subscribe({
  intervalMs: 500,
  includeCpu: true,
  includeMemory: true,
  includeRuntime: false,
  includeProcess: false,
});

// Receive 20 snapshots
for (let i = 0; i < 20; i++) {
  const snapshot = await nextSnapshot(subId);
  if (snapshot) {
    console.log(`[${new Date(snapshot.timestamp_ms).toISOString()}]`);
    if (snapshot.cpu) {
      console.log(`  CPU: ${snapshot.cpu.total_percent.toFixed(1)}%`);
    }
    if (snapshot.memory) {
      console.log(`  Memory: ${(snapshot.memory.used_bytes / 1024**3).toFixed(1)} GB`);
    }
  }
}

// Stop monitoring
unsubscribe(subId);
```

### Complete System Snapshot (Convenience Function)

```typescript
import { getSystemSnapshot, formatBytes } from "runtime:monitor";

// Collect all system metrics at once
const sys = await getSystemSnapshot();
console.log(`CPU: ${sys.cpu.total_percent.toFixed(1)}%`);
console.log(`Memory: ${formatBytes(sys.memory.used_bytes)}`);
console.log(`Disks: ${sys.disks.length} mounted`);
console.log(`Network interfaces: ${sys.network.length}`);
```

### Monitor Loop (Convenience Function)

```typescript
import { monitorLoop } from "runtime:monitor";

// Simple monitoring loop with callback
const stop = await monitorLoop(1000, (snapshot) => {
  console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
  console.log(`Memory: ${(snapshot.memory?.used_bytes / 1024**3).toFixed(1)} GB`);
});

// Stop after 10 seconds
setTimeout(stop, 10000);
```

### Multiple Subscriptions

```typescript
import { subscribe, nextSnapshot, unsubscribe, getSubscriptions } from "runtime:monitor";

// Create multiple subscriptions with different configurations
const cpuSubId = await subscribe({
  intervalMs: 500,
  includeCpu: true,
  includeMemory: false,
});

const memSubId = await subscribe({
  intervalMs: 1000,
  includeCpu: false,
  includeMemory: true,
});

// List active subscriptions
const subs = getSubscriptions();
console.log(`Active subscriptions: ${subs.length}`);
for (const sub of subs) {
  console.log(`  ${sub.id}: ${sub.snapshot_count} snapshots delivered`);
}

// Clean up
unsubscribe(cpuSubId);
unsubscribe(memSubId);
```

## Architecture

ext_monitor uses the `sysinfo` crate for cross-platform system information:

```text
┌──────────────────────────────────────────────────────────────┐
│ TypeScript Application (runtime:monitor)                     │
│  - getCpu(), getMemory(), getDisks()                         │
│  - subscribe(), nextSnapshot()                               │
└────────────────┬─────────────────────────────────────────────┘
                 │ Deno Ops (op_monitor_*)
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ ext_monitor Operations                                       │
│  - MonitorState: cached System, Disks, Networks              │
│  - EventLoopLatencyMeasurer: background latency tracking     │
│  - Subscriptions: HashMap<id, Subscription>                  │
└────────────────┬─────────────────────────────────────────────┘
                 │ sysinfo API calls
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ sysinfo crate                                                │
│  - System::new_with_specifics()                              │
│  - refresh_cpu_usage(), refresh_memory(), etc.               │
└────────────────┬─────────────────────────────────────────────┘
                 │ Platform-specific system APIs
                 ↓
┌──────────────────────────────────────────────────────────────┐
│ OS System Information APIs                                   │
│  - Linux: /proc filesystem, sysfs                            │
│  - macOS: sysctl, host_statistics, vm_stat                   │
│  - Windows: WMI, Performance Counters, PSAPI                 │
└──────────────────────────────────────────────────────────────┘
```

## Operations

The extension provides 17 operations across 5 categories:

### System Metrics (6 operations)

| Operation | TypeScript | Return Type | Description |
|-----------|-----------|-------------|-------------|
| `op_monitor_cpu` | `getCpu()` | `CpuUsage` | CPU usage statistics (async, ~200ms) |
| `op_monitor_memory` | `getMemory()` | `MemoryUsage` | RAM and swap statistics |
| `op_monitor_disk` | `getDisks()` | `DiskUsage[]` | All mounted filesystem usage |
| `op_monitor_network` | `getNetwork()` | `NetworkStats[]` | Network interface traffic |
| `op_monitor_process_self` | `getProcessSelf()` | `ProcessInfo` | Current process information |
| `op_monitor_processes` | `getProcesses()` | `ProcessInfo[]` | Top 50 processes by CPU |

### Runtime Metrics (2 operations)

| Operation | TypeScript | Return Type | Description |
|-----------|-----------|-------------|-------------|
| `op_monitor_runtime` | `getRuntime()` | `RuntimeMetrics` | Event loop latency, uptime |
| `op_monitor_heap` | `getHeap()` | `HeapStats` | V8 heap stats (placeholder) |

### WebView Metrics (1 operation)

| Operation | TypeScript | Return Type | Description |
|-----------|-----------|-------------|-------------|
| `op_monitor_webview` | `getWebViews()` | `WebViewStats` | Window metrics (placeholder) |

### Subscription API (4 operations)

| Operation | TypeScript | Return Type | Description |
|-----------|-----------|-------------|-------------|
| `op_monitor_subscribe` | `subscribe(options)` | `Promise<string>` | Create metric subscription (returns ID) |
| `op_monitor_next` | `nextSnapshot(id)` | `Promise<MetricSnapshot \| null>` | Get next snapshot (async) |
| `op_monitor_unsubscribe` | `unsubscribe(id)` | `void` | Cancel subscription |
| `op_monitor_subscriptions` | `getSubscriptions()` | `SubscriptionInfo[]` | List active subscriptions |

### Legacy Operations (2 operations, backward compatibility)

| Operation | TypeScript | Return Type | Description |
|-----------|-----------|-------------|-------------|
| `op_monitor_info` | `info()` | `ExtensionInfo` | Extension metadata |
| `op_monitor_echo` | `echo(message)` | `string` | Echo test operation |

## Error Handling

All operations use structured error codes:

| Code | Error | Description |
|------|-------|-------------|
| 9800 | Generic | General monitoring operation error |
| 9801 | QueryFailed | Failed to query system metrics |
| 9802 | ProcessNotFound | Process ID not found |
| 9803 | PermissionDenied | Insufficient permissions for operation |
| 9804 | InvalidSubscription | Subscription ID is invalid or expired |
| 9805 | SubscriptionLimitExceeded | Maximum 10 subscriptions exceeded |
| 9806 | WebViewMetricsUnavailable | WebView metrics not yet implemented |
| 9807 | PlatformNotSupported | Operation not supported on this platform |
| 9808 | InvalidInterval | Subscription interval < 100ms minimum |

```typescript
import { getCpu, subscribe } from "runtime:monitor";

// Handle CPU query errors
try {
  const cpu = await getCpu();
} catch (error) {
  // Error 9801: Query failed
  console.error("Failed to get CPU usage:", error);
}

// Handle subscription errors
try {
  const subId = await subscribe({ intervalMs: 50 });  // Too fast!
} catch (error) {
  // Error 9808: Invalid interval (minimum 100ms)
  console.error("Invalid subscription interval:", error);
}

// Handle subscription limit
try {
  // Create 11 subscriptions (max is 10)
  for (let i = 0; i < 11; i++) {
    await subscribe({ intervalMs: 1000 });
  }
} catch (error) {
  // Error 9805: Subscription limit exceeded
  console.error("Too many subscriptions:", error);
}
```

## Implementation Details

### CPU Measurement

CPU usage calculation requires two measurements with a time interval because it's calculated as `(cpu_time_delta / wall_time_delta)`. The `getCpu()` operation:

1. Calls `system.refresh_cpu_usage()` to establish baseline
2. Sleeps for 200ms to allow CPU time to accumulate
3. Calls `system.refresh_cpu_usage()` again to measure delta
4. Returns per-core and averaged total CPU percentages

This is why `getCpu()` is async and takes ~200ms to complete.

### State Management

`MonitorState` is stored in Deno's `OpState` and contains:
- `System`: Main system information cache (CPU, memory, processes)
- `Disks`: Cached disk/filesystem information
- `Networks`: Cached network interface information
- `EventLoopLatencyMeasurer`: Background latency tracking (Arc<AtomicU64>)
- `subscriptions`: HashMap of active metric subscriptions

The cache is refreshed on each operation call to get current values.

### Subscription Architecture

Each subscription spawns a tokio background task with:
- **Dedicated System instance**: Avoids `Rc<RefCell<>>` borrow conflicts in async tasks
- **tokio::time::interval ticker**: Periodic metric collection
- **mpsc::channel**: Sends snapshots to subscriber (32-message buffer)
- **CancellationToken**: Graceful shutdown on unsubscribe
- **Arc<AtomicU64> counter**: Tracks delivered snapshot count

The background task collects metrics at the specified interval and sends `MetricSnapshot` messages through the channel. The subscriber receives them via `nextSnapshot()`, which temporarily borrows the receiver from the subscription HashMap.

### Event Loop Latency Measurement

The `EventLoopLatencyMeasurer` spawns a background task that:
1. Schedules a 10ms sleep via `tokio::time::sleep`
2. Measures actual wake-up time deviation from expected 10ms
3. Stores latency in `Arc<AtomicU64>` for lock-free access
4. Repeats every 500ms

High latency indicates the event loop is blocked by long-running operations. This measurement is started automatically when `getRuntime()` is first called or when a subscription includes runtime metrics.

### Process Limits

`getProcesses()` returns only the top 50 processes sorted by CPU usage to prevent overwhelming the runtime. Full process list access would require querying thousands of processes on typical systems, which is expensive and would cause significant performance impact.

## Platform Support

| Platform | System Info Source | Status |
|----------|-------------------|--------|
| macOS (x64) | sysctl, host_statistics | ✅ Full support |
| macOS (ARM) | sysctl, host_statistics | ✅ Full support |
| Windows (x64) | WMI, Performance Counters | ✅ Full support |
| Windows (ARM) | WMI, Performance Counters | ✅ Full support |
| Linux (x64) | /proc, sysfs | ✅ Full support |
| Linux (ARM) | /proc, sysfs | ✅ Full support |

Platform-specific behavior is handled by the `sysinfo` crate, which provides a unified API across all platforms.

## Common Pitfalls

### 1. Forgetting CPU is Async

```typescript
// ❌ INCORRECT: Missing await
const cpu = getCpu();  // Returns Promise<CpuUsage>, not CpuUsage
console.log(cpu.total_percent);  // undefined

// ✅ CORRECT: Await the promise
const cpu = await getCpu();
console.log(cpu.total_percent);  // Works
```

### 2. Not Cleaning Up Subscriptions

```typescript
// ❌ RISKY: Subscription leak
async function monitor() {
  const subId = await subscribe({ intervalMs: 1000 });
  // ... do some monitoring ...
  // Forgot to unsubscribe - background task keeps running
}

// ✅ CORRECT: Always unsubscribe
async function monitor() {
  const subId = await subscribe({ intervalMs: 1000 });
  try {
    // ... do some monitoring ...
  } finally {
    unsubscribe(subId);  // Clean up
  }
}

// ✅ BETTER: Use monitorLoop convenience function
const stop = await monitorLoop(1000, (snapshot) => {
  // ... handle snapshot ...
});
// Call stop() when done - automatically cleans up
```

### 3. Subscription Interval Too Fast

```typescript
// ❌ INCORRECT: Interval below 100ms minimum
const subId = await subscribe({ intervalMs: 50 });  // Error 9808

// ✅ CORRECT: Use minimum 100ms interval
const subId = await subscribe({ intervalMs: 100 });

// ✅ RECOMMENDED: Use 500ms+ for most use cases
const subId = await subscribe({ intervalMs: 1000 });
```

### 4. Assuming Synchronous Memory/Disk Operations Return Current Values

```typescript
// ⚠️ IMPORTANT: Memory/disk/network are cached snapshots
const mem1 = getMemory();
const mem2 = getMemory();  // Same cached values if called too quickly

// The sysinfo cache is refreshed on each call, but OS caching
// means values may not change between very rapid calls

// ✅ CORRECT: Use subscriptions for continuous monitoring
const subId = await subscribe({
  intervalMs: 1000,
  includeMemory: true,
});
```

## Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `deno_core` | 0.373 | Op definitions and runtime integration |
| `sysinfo` | Latest | Cross-platform system information |
| `tokio` | 1.x | Async runtime for subscriptions |
| `tokio_util` | 0.7 | CancellationToken for subscription cleanup |
| `thiserror` | 2.x | Error type definitions |
| `deno_error` | 0.x | JavaScript error conversion |
| `serde` | 1.x | Serialization for metrics |
| `tracing` | 0.1 | Logging and diagnostics |
| `forge-weld-macro` | 0.1 | TypeScript binding generation |

## Testing

```bash
# Run all tests
cargo test -p ext_monitor

# Run with output
cargo test -p ext_monitor -- --nocapture

# With debug logging
RUST_LOG=ext_monitor=debug cargo test -p ext_monitor -- --nocapture
```

## Related Extensions

- `ext_sys` - System information extension (app info, clipboard, notifications)
- `ext_process` - Process spawning and management extension
- `ext_window` - Window management (required for WebView metrics integration)

## See Also

- [sysinfo crate](https://docs.rs/sysinfo) - System information library
- [Forge Architecture](/docs/architecture) - Full system architecture
