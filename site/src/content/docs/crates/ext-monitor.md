---
title: "ext_monitor"
description: System monitoring extension providing the runtime:monitor module.
slug: crates/ext-monitor
---

The `ext_monitor` crate provides comprehensive system and runtime monitoring for Forge applications through the `runtime:monitor` module. Built on the [`sysinfo`](https://docs.rs/sysinfo) crate for cross-platform system information access.

## Overview

ext_monitor provides:

- **System Metrics** - CPU usage (total and per-core), memory (RAM + swap), disk usage (all mounted filesystems), network statistics (per-interface traffic counters), process information
- **Runtime Metrics** - Event loop latency measurement, process uptime tracking, V8 heap statistics
- **WebView Metrics** - Window count and visibility tracking (placeholder)
- **Subscription API** - Continuous metric collection at configurable intervals with async iterator pattern
- **Cross-Platform** - Unified API across macOS (sysctl), Windows (WMI), and Linux (/proc)

## Quick Start

```typescript
import { getCpu, getMemory, subscribe, nextSnapshot } from "runtime:monitor";

// Get current CPU usage (takes ~200ms for accurate measurement)
const cpu = await getCpu();
console.log(`CPU: ${cpu.total_percent.toFixed(1)}%`);

// Get memory usage (synchronous)
const mem = getMemory();
console.log(`Memory: ${(mem.used_bytes / 1024**3).toFixed(1)} GB`);

// Subscribe to continuous monitoring
const subId = await subscribe({
  intervalMs: 1000,
  includeCpu: true,
  includeMemory: true,
});

// Receive metric snapshots
for (let i = 0; i < 10; i++) {
  const snapshot = await nextSnapshot(subId);
  if (snapshot?.cpu) {
    console.log(`CPU: ${snapshot.cpu.total_percent.toFixed(1)}%`);
  }
}
```

## API Reference

### System Metrics

#### `getCpu()`

Get current CPU usage statistics. This is an **async** operation that takes ~200ms to complete because CPU usage requires measuring time delta.

**Returns:** `Promise<CpuUsage>`

```typescript
interface CpuUsage {
  total_percent: number;      // Total CPU usage (0-100)
  per_core: number[];         // Per-core usage percentages
  core_count: number;         // Number of CPU cores
  frequency_mhz: number | null; // CPU frequency in MHz (if available)
}
```

**Example:**

```typescript
import { getCpu } from "runtime:monitor";

const cpu = await getCpu();
console.log(`Total CPU: ${cpu.total_percent.toFixed(1)}%`);
console.log(`Cores: ${cpu.core_count}`);
cpu.per_core.forEach((usage, i) => {
  console.log(`  Core ${i}: ${usage.toFixed(1)}%`);
});
```

#### `getMemory()`

Get current memory usage statistics. This is a **synchronous** operation.

**Returns:** `MemoryUsage`

```typescript
interface MemoryUsage {
  total_bytes: number;        // Total physical memory
  used_bytes: number;         // Used memory
  free_bytes: number;         // Free memory
  available_bytes: number;    // Available memory (free + reclaimable)
  swap_total_bytes: number;   // Total swap
  swap_used_bytes: number;    // Used swap
}
```

**Example:**

```typescript
import { getMemory } from "runtime:monitor";

const mem = getMemory();
const usedGB = mem.used_bytes / (1024 ** 3);
const totalGB = mem.total_bytes / (1024 ** 3);
console.log(`Memory: ${usedGB.toFixed(1)} / ${totalGB.toFixed(1)} GB`);
```

#### `getDisks()`

Get disk usage for all mounted filesystems. This is a **synchronous** operation.

**Returns:** `DiskUsage[]`

```typescript
interface DiskUsage {
  mount_point: string;  // Mount point path
  device: string;       // Device name
  filesystem: string;   // Filesystem type
  total_bytes: number;  // Total capacity
  used_bytes: number;   // Used space
  free_bytes: number;   // Free space
}
```

**Example:**

```typescript
import { getDisks } from "runtime:monitor";

const disks = getDisks();
for (const disk of disks) {
  const usedPct = (disk.used_bytes / disk.total_bytes * 100).toFixed(1);
  console.log(`${disk.mount_point}: ${usedPct}% used`);
}
```

#### `getNetwork()`

Get network statistics for all interfaces. This is a **synchronous** operation. Returns cumulative statistics since system boot.

**Returns:** `NetworkStats[]`

```typescript
interface NetworkStats {
  interface: string;      // Interface name
  bytes_sent: number;     // Total bytes sent since boot
  bytes_recv: number;     // Total bytes received since boot
  packets_sent: number;   // Total packets sent since boot
  packets_recv: number;   // Total packets received since boot
}
```

**Example:**

```typescript
import { getNetwork } from "runtime:monitor";

const networks = getNetwork();
for (const net of networks) {
  const sentMB = net.bytes_sent / (1024 ** 2);
  const recvMB = net.bytes_recv / (1024 ** 2);
  console.log(`${net.interface}: sent ${sentMB.toFixed(1)} MB, recv ${recvMB.toFixed(1)} MB`);
}
```

#### `getProcessSelf()`

Get information about the current process (the Forge runtime). This is a **synchronous** operation.

**Returns:** `ProcessInfo`

```typescript
interface ProcessInfo {
  pid: number;                 // Process ID
  name: string;                // Process name
  cpu_percent: number;         // CPU usage percentage
  memory_rss_bytes: number;    // Resident memory (RSS)
  memory_virtual_bytes: number; // Virtual memory
  status: string;              // Process status
  start_time_secs: number;     // Start time (Unix timestamp)
  parent_pid: number | null;   // Parent process ID
}
```

**Example:**

```typescript
import { getProcessSelf } from "runtime:monitor";

const proc = getProcessSelf();
console.log(`PID: ${proc.pid}`);
console.log(`Memory: ${(proc.memory_rss_bytes / 1024 / 1024).toFixed(1)} MB`);
console.log(`Status: ${proc.status}`);
```

#### `getProcesses()`

Get a list of running processes sorted by CPU usage. Returns top 50 processes to prevent overwhelming the runtime. This is a **synchronous** operation.

**Returns:** `ProcessInfo[]`

**Example:**

```typescript
import { getProcesses } from "runtime:monitor";

const procs = getProcesses();
console.log("Top 5 CPU-consuming processes:");
for (const p of procs.slice(0, 5)) {
  console.log(`  ${p.name}: ${p.cpu_percent.toFixed(1)}%`);
}
```

### Runtime Metrics

#### `getRuntime()`

Get Deno runtime metrics including event loop latency and uptime. This is a **synchronous** operation.

**Returns:** `RuntimeMetrics`

```typescript
interface RuntimeMetrics {
  pending_ops_count: number;      // Pending async operations (placeholder)
  module_count: number;           // Loaded modules (placeholder)
  event_loop_latency_us: number;  // Event loop latency in microseconds
  uptime_secs: number;            // Process uptime in seconds
}
```

**Example:**

```typescript
import { getRuntime } from "runtime:monitor";

const runtime = getRuntime();
console.log(`Uptime: ${runtime.uptime_secs}s`);
console.log(`Event loop latency: ${runtime.event_loop_latency_us}μs`);
```

#### `getHeap()`

Get V8 heap statistics. Currently returns placeholder values - full V8 heap stats require direct isolate access which is not yet implemented. This is a **synchronous** operation.

**Returns:** `HeapStats`

```typescript
interface HeapStats {
  total_heap_size: number;          // Total heap size in bytes
  used_heap_size: number;           // Used heap size in bytes
  heap_size_limit: number;          // Heap size limit in bytes
  external_memory: number;          // External memory in bytes
  number_of_native_contexts: number; // Number of native contexts
}
```

### WebView Metrics

#### `getWebViews()`

Get WebView statistics across all windows. Currently returns placeholder values - full WebView metrics require coordination with ext_window which is not yet implemented. This is a **synchronous** operation.

**Returns:** `WebViewStats`

```typescript
interface WebViewStats {
  window_count: number;        // Total number of windows
  visible_count: number;       // Number of visible windows
  windows: WebViewMetrics[];   // Per-window metrics
}

interface WebViewMetrics {
  window_id: string;                   // Window ID
  is_visible: boolean;                 // Whether window is visible
  dom_node_count: number | null;       // DOM node count (if available)
  js_heap_size_bytes: number | null;   // JavaScript heap size
  js_heap_size_limit: number | null;   // JavaScript heap limit
}
```

### Subscription API

#### `subscribe(options)`

Subscribe to continuous metric updates. Creates a background task that collects metrics at the specified interval and sends them through a channel. Maximum 10 concurrent subscriptions allowed per runtime.

**Parameters:**

```typescript
interface SubscribeOptions {
  intervalMs?: number;        // Interval in milliseconds (minimum 100ms, default 1000ms)
  includeCpu?: boolean;       // Include CPU metrics (default: true)
  includeMemory?: boolean;    // Include memory metrics (default: true)
  includeRuntime?: boolean;   // Include runtime metrics (default: false)
  includeProcess?: boolean;   // Include current process info (default: false)
}
```

**Returns:** `Promise<string>` - Subscription ID

**Throws:**
- Error [9808] if intervalMs < 100ms
- Error [9805] if maximum 10 subscriptions exceeded

**Example:**

```typescript
import { subscribe } from "runtime:monitor";

// Subscribe to CPU and memory every 500ms
const subId = await subscribe({
  intervalMs: 500,
  includeCpu: true,
  includeMemory: true,
});
```

#### `nextSnapshot(subscriptionId)`

Get the next metric snapshot from a subscription. This is an **async** operation that waits for the next snapshot to be available. Returns null if the subscription has been cancelled.

**Returns:** `Promise<MetricSnapshot | null>`

```typescript
interface MetricSnapshot {
  timestamp_ms: number;         // Timestamp when metrics were collected
  cpu: CpuUsage | null;        // CPU metrics (if requested)
  memory: MemoryUsage | null;  // Memory metrics (if requested)
  runtime: RuntimeMetrics | null; // Runtime metrics (if requested)
  process: ProcessInfo | null; // Current process info (if requested)
}
```

**Example:**

```typescript
import { nextSnapshot } from "runtime:monitor";

const snapshot = await nextSnapshot(subId);
if (snapshot) {
  console.log(`Timestamp: ${snapshot.timestamp_ms}`);
  if (snapshot.cpu) {
    console.log(`CPU: ${snapshot.cpu.total_percent}%`);
  }
}
```

#### `unsubscribe(subscriptionId)`

Cancel a metric subscription. Stops the background metric collection and cleans up resources. Any pending `nextSnapshot()` calls will return null.

**Throws:** Error [9804] if subscription ID is invalid

**Example:**

```typescript
import { unsubscribe } from "runtime:monitor";

unsubscribe(subId);
```

#### `getSubscriptions()`

List all active subscriptions. This is a **synchronous** operation.

**Returns:** `SubscriptionInfo[]`

```typescript
interface SubscriptionInfo {
  id: string;              // Subscription ID
  interval_ms: number;     // Interval in milliseconds
  is_active: boolean;      // Whether subscription is active
  snapshot_count: number;  // Number of snapshots delivered
}
```

**Example:**

```typescript
import { getSubscriptions } from "runtime:monitor";

const subs = getSubscriptions();
for (const sub of subs) {
  console.log(`Subscription ${sub.id}: ${sub.snapshot_count} snapshots delivered`);
}
```

### Convenience Functions

#### `getSystemSnapshot()`

Get a complete system snapshot (CPU, memory, disk, network) at once. This is an **async** operation.

**Returns:** `Promise<{ cpu, memory, disks, network }>`

**Example:**

```typescript
import { getSystemSnapshot } from "runtime:monitor";

const sys = await getSystemSnapshot();
console.log(`CPU: ${sys.cpu.total_percent.toFixed(1)}%`);
console.log(`Memory: ${(sys.memory.used_bytes / 1024**3).toFixed(1)} GB`);
console.log(`Disks: ${sys.disks.length}`);
console.log(`Network interfaces: ${sys.network.length}`);
```

#### `formatBytes(bytes, decimals?)`

Format bytes as a human-readable string.

**Parameters:**
- `bytes` (number) - Number of bytes
- `decimals` (number, optional) - Number of decimal places (default: 1)

**Returns:** `string`

**Example:**

```typescript
import { formatBytes, getMemory } from "runtime:monitor";

const mem = getMemory();
console.log(`Used: ${formatBytes(mem.used_bytes)}`);  // "8.2 GB"
```

#### `monitorLoop(intervalMs, callback)`

Create a simple monitor loop that calls a callback with each snapshot. Returns a stop function to cancel monitoring.

**Parameters:**
- `intervalMs` (number) - Interval in milliseconds
- `callback` (function) - Function to call with each snapshot

**Returns:** `Promise<() => void>` - Stop function

**Example:**

```typescript
import { monitorLoop } from "runtime:monitor";

const stop = await monitorLoop(1000, (snapshot) => {
  console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
});

// Stop after 10 seconds
setTimeout(stop, 10000);
```

## Common Patterns

### System Monitoring Dashboard

```typescript
import { getSystemSnapshot, formatBytes } from "runtime:monitor";

async function showSystemStatus() {
  const sys = await getSystemSnapshot();

  console.log("=== System Status ===");
  console.log(`CPU: ${sys.cpu.total_percent.toFixed(1)}% (${sys.cpu.core_count} cores)`);
  console.log(`Memory: ${formatBytes(sys.memory.used_bytes)} / ${formatBytes(sys.memory.total_bytes)}`);
  console.log(`Disks: ${sys.disks.length} mounted`);

  for (const disk of sys.disks) {
    const usedPct = (disk.used_bytes / disk.total_bytes * 100).toFixed(1);
    console.log(`  ${disk.mount_point}: ${usedPct}% used`);
  }
}

await showSystemStatus();
```

### Continuous Monitoring with Alerts

```typescript
import { subscribe, nextSnapshot, unsubscribe } from "runtime:monitor";

const subId = await subscribe({
  intervalMs: 1000,
  includeCpu: true,
  includeMemory: true,
});

let highCpuCount = 0;

for (let i = 0; i < 60; i++) {  // Monitor for 60 seconds
  const snapshot = await nextSnapshot(subId);
  if (!snapshot) break;

  if (snapshot.cpu && snapshot.cpu.total_percent > 80) {
    highCpuCount++;
    if (highCpuCount >= 5) {
      console.warn("ALERT: CPU usage above 80% for 5 consecutive samples!");
    }
  } else {
    highCpuCount = 0;
  }

  if (snapshot.memory) {
    const memPct = (snapshot.memory.used_bytes / snapshot.memory.total_bytes) * 100;
    if (memPct > 90) {
      console.warn(`ALERT: Memory usage at ${memPct.toFixed(1)}%!`);
    }
  }
}

unsubscribe(subId);
```

### Performance Profiling

```typescript
import { getRuntime, getCpu, getProcessSelf } from "runtime:monitor";

async function profilePerformance(fn: () => Promise<void>) {
  const beforeCpu = await getCpu();
  const beforeProc = getProcessSelf();
  const beforeRuntime = getRuntime();

  await fn();

  const afterCpu = await getCpu();
  const afterProc = getProcessSelf();
  const afterRuntime = getRuntime();

  console.log("=== Performance Profile ===");
  console.log(`CPU delta: ${(afterCpu.total_percent - beforeCpu.total_percent).toFixed(1)}%`);
  console.log(`Memory delta: ${((afterProc.memory_rss_bytes - beforeProc.memory_rss_bytes) / 1024 / 1024).toFixed(1)} MB`);
  console.log(`Event loop latency: ${afterRuntime.event_loop_latency_us}μs`);
}

await profilePerformance(async () => {
  // Run your code here
  await someExpensiveOperation();
});
```

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
```

## Platform Support

| Platform | System Info Source | Status |
|----------|-------------------|--------|
| macOS (x64) | sysctl, host_statistics | ✅ Full support |
| macOS (ARM) | sysctl, host_statistics | ✅ Full support |
| Windows (x64) | WMI, Performance Counters | ✅ Full support |
| Windows (ARM) | WMI, Performance Counters | ✅ Full support |
| Linux (x64) | /proc, sysfs | ✅ Full support |
| Linux (ARM) | /proc, sysfs | ✅ Full support |

Platform-specific behavior is handled by the [`sysinfo`](https://docs.rs/sysinfo) crate.

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
```

### 3. Subscription Interval Too Fast

```typescript
// ❌ INCORRECT: Interval below 100ms minimum
const subId = await subscribe({ intervalMs: 50 });  // Error 9808

// ✅ CORRECT: Use minimum 100ms interval
const subId = await subscribe({ intervalMs: 100 });
```

## Related

- [ext_sys](/docs/crates/ext-sys) - System information extension
- [ext_process](/docs/crates/ext-process) - Process management extension
- [Architecture](/docs/architecture) - Full system architecture
