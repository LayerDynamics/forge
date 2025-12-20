/**
 * @module runtime:monitor
 *
 * System and runtime monitoring extension for Forge applications.
 *
 * Provides real-time system metrics (CPU, memory, disk, network), Deno runtime
 * metrics (event loop latency, uptime), process information, and subscription-based
 * continuous monitoring. Built on the `sysinfo` crate for cross-platform system
 * information access.
 *
 * **Runtime Module:** `runtime:monitor`
 *
 * ## Features
 *
 * ### System Metrics
 * - CPU usage (total and per-core percentages)
 * - Memory usage (RAM and swap statistics)
 * - Disk usage (all mounted filesystems)
 * - Network statistics (per-interface traffic counters)
 * - Process information (current process and system-wide)
 *
 * ### Runtime Metrics
 * - Event loop latency measurement
 * - Process uptime tracking
 * - V8 heap statistics (placeholder - full stats require isolate access)
 *
 * ### WebView Metrics
 * - Window count and visibility tracking (placeholder - requires ext_window integration)
 * - Per-window DOM and JavaScript heap stats (future feature)
 *
 * ### Subscription API
 * - Continuous metric collection at configurable intervals
 * - Selective metric inclusion (CPU, memory, runtime, process)
 * - Async iterator pattern for real-time monitoring
 * - Maximum 10 concurrent subscriptions per runtime
 *
 * ## Error Codes (9800-9808)
 *
 * | Code | Error | Description |
 * |------|-------|-------------|
 * | 9800 | Generic | General monitoring operation error |
 * | 9801 | QueryFailed | Failed to query system metrics |
 * | 9802 | ProcessNotFound | Process ID not found |
 * | 9803 | PermissionDenied | Insufficient permissions for operation |
 * | 9804 | InvalidSubscription | Subscription ID is invalid or expired |
 * | 9805 | SubscriptionLimitExceeded | Maximum 10 subscriptions exceeded |
 * | 9806 | WebViewMetricsUnavailable | WebView metrics not yet implemented |
 * | 9807 | PlatformNotSupported | Operation not supported on this platform |
 * | 9808 | InvalidInterval | Subscription interval < 100ms minimum |
 *
 * ## Quick Start
 *
 * ```typescript
 * import { getCpu, getMemory, subscribe, nextSnapshot } from "runtime:monitor";
 *
 * // Get current CPU usage (takes ~200ms for accurate measurement)
 * const cpu = await getCpu();
 * console.log(`CPU: ${cpu.total_percent.toFixed(1)}%`);
 *
 * // Get memory usage (synchronous)
 * const mem = getMemory();
 * console.log(`Memory: ${(mem.used_bytes / 1024**3).toFixed(1)} GB`);
 *
 * // Subscribe to continuous monitoring
 * const subId = await subscribe({
 *   intervalMs: 1000,
 *   includeCpu: true,
 *   includeMemory: true,
 * });
 *
 * // Receive metric snapshots
 * for (let i = 0; i < 10; i++) {
 *   const snapshot = await nextSnapshot(subId);
 *   if (snapshot?.cpu) {
 *     console.log(`CPU: ${snapshot.cpu.total_percent.toFixed(1)}%`);
 *   }
 * }
 * ```
 *
 * ## Architecture
 *
 * ext_monitor uses the `sysinfo` crate for cross-platform system metrics:
 *
 * ```text
 * TypeScript Application
 *   |
 *   | getCpu(), getMemory(), subscribe()
 *   v
 * runtime:monitor (ext_monitor)
 *   |
 *   | MonitorState (cached System, Disks, Networks)
 *   v
 * sysinfo crate
 *   |
 *   | Platform-specific APIs
 *   v
 * OS APIs (proc, sysctl, WMI, etc.)
 * ```
 *
 * ## Permission Model
 *
 * ext_monitor currently does not require explicit permissions. System metrics
 * are readable by any process. Future versions may add permission checks for:
 * - Reading other processes' information
 * - WebView metrics (requires ext_window coordination)
 *
 * ## Implementation Notes
 *
 * - **CPU Measurement**: `getCpu()` is async because `sysinfo` requires ~200ms
 *   between measurements for accurate CPU usage calculation
 * - **Memory/Disk/Network**: Synchronous operations using cached `MonitorState`
 * - **Subscriptions**: Background tokio tasks with dedicated `System` instances
 *   to avoid `Rc<RefCell<>>` borrow conflicts
 * - **Event Loop Latency**: Measured by scheduling a 10ms sleep and measuring
 *   actual wake-up time deviation
 * - **Process Limit**: `getProcesses()` returns top 50 processes by CPU usage
 *   to prevent overwhelming the runtime
 *
 * @example
 * ```typescript
 * import * as monitor from "runtime:monitor";
 *
 * // Complete system snapshot
 * const sys = await monitor.getSystemSnapshot();
 * console.log(`CPU: ${sys.cpu.total_percent.toFixed(1)}%`);
 * console.log(`Memory: ${monitor.formatBytes(sys.memory.used_bytes)}`);
 *
 * // Monitor with convenience loop
 * const stop = await monitor.monitorLoop(1000, (snapshot) => {
 *   console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
 * });
 *
 * // Stop after 10 seconds
 * setTimeout(stop, 10000);
 * ```
 */

// ============================================================================
// Deno Core Type Declarations
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      // Legacy operations (backward compatibility)
      op_monitor_info(): ExtensionInfo;
      op_monitor_echo(message: string): string;
      // System metrics
      op_monitor_cpu(): Promise<CpuUsage>;
      op_monitor_memory(): MemoryUsage;
      op_monitor_disk(): DiskUsage[];
      op_monitor_network(): NetworkStats[];
      op_monitor_process_self(): ProcessInfo;
      op_monitor_processes(): ProcessInfo[];
      // Runtime metrics
      op_monitor_runtime(): RuntimeMetrics;
      op_monitor_heap(): HeapStats;
      // WebView metrics
      op_monitor_webview(): WebViewStats;
      // Subscription API
      op_monitor_subscribe(options: SubscribeOptionsInternal): Promise<string>;
      op_monitor_next(subscriptionId: string): Promise<MetricSnapshot | null>;
      op_monitor_unsubscribe(subscriptionId: string): void;
      op_monitor_subscriptions(): SubscriptionInfo[];
    };
  };
};

const { core } = Deno;

// ============================================================================
// Extension Info Types (Legacy)
// ============================================================================

/**
 * Extension information for backward compatibility
 */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

// ============================================================================
// System Metric Types
// ============================================================================

/**
 * CPU usage statistics
 */
export interface CpuUsage {
  /** Total CPU usage percentage (0-100) */
  total_percent: number;
  /** Per-core CPU usage percentages */
  per_core: number[];
  /** Number of CPU cores */
  core_count: number;
  /** CPU frequency in MHz (if available) */
  frequency_mhz: number | null;
}

/**
 * Memory usage statistics
 */
export interface MemoryUsage {
  /** Total physical memory in bytes */
  total_bytes: number;
  /** Used memory in bytes */
  used_bytes: number;
  /** Free memory in bytes */
  free_bytes: number;
  /** Available memory in bytes (free + reclaimable) */
  available_bytes: number;
  /** Total swap in bytes */
  swap_total_bytes: number;
  /** Used swap in bytes */
  swap_used_bytes: number;
}

/**
 * Disk usage for a mount point
 */
export interface DiskUsage {
  /** Mount point path */
  mount_point: string;
  /** Device name */
  device: string;
  /** Filesystem type */
  filesystem: string;
  /** Total capacity in bytes */
  total_bytes: number;
  /** Used space in bytes */
  used_bytes: number;
  /** Free space in bytes */
  free_bytes: number;
}

/**
 * Network interface statistics
 */
export interface NetworkStats {
  /** Interface name */
  interface: string;
  /** Total bytes sent since system boot */
  bytes_sent: number;
  /** Total bytes received since system boot */
  bytes_recv: number;
  /** Total packets sent since system boot */
  packets_sent: number;
  /** Total packets received since system boot */
  packets_recv: number;
}

/**
 * Process information
 */
export interface ProcessInfo {
  /** Process ID */
  pid: number;
  /** Process name */
  name: string;
  /** CPU usage percentage */
  cpu_percent: number;
  /** Resident memory (RSS) in bytes */
  memory_rss_bytes: number;
  /** Virtual memory in bytes */
  memory_virtual_bytes: number;
  /** Process status (e.g., "Running", "Sleeping") */
  status: string;
  /** Process start time as Unix timestamp (seconds) */
  start_time_secs: number;
  /** Parent process ID (null if no parent) */
  parent_pid: number | null;
}

// ============================================================================
// Runtime Metric Types
// ============================================================================

/**
 * Deno runtime metrics
 */
export interface RuntimeMetrics {
  /** Number of pending async operations */
  pending_ops_count: number;
  /** Number of loaded modules */
  module_count: number;
  /** Event loop latency in microseconds */
  event_loop_latency_us: number;
  /** Process uptime in seconds */
  uptime_secs: number;
}

/**
 * V8 heap statistics
 */
export interface HeapStats {
  /** Total heap size in bytes */
  total_heap_size: number;
  /** Used heap size in bytes */
  used_heap_size: number;
  /** Heap size limit in bytes */
  heap_size_limit: number;
  /** External memory in bytes */
  external_memory: number;
  /** Number of native contexts */
  number_of_native_contexts: number;
}

// ============================================================================
// WebView Metric Types
// ============================================================================

/**
 * WebView metrics for a single window
 */
export interface WebViewMetrics {
  /** Window ID */
  window_id: string;
  /** Whether window is currently visible */
  is_visible: boolean;
  /** DOM node count (if available) */
  dom_node_count: number | null;
  /** JavaScript heap size in bytes */
  js_heap_size_bytes: number | null;
  /** JavaScript heap limit in bytes */
  js_heap_size_limit: number | null;
}

/**
 * Aggregated WebView statistics across all windows
 */
export interface WebViewStats {
  /** Total number of windows */
  window_count: number;
  /** Number of currently visible windows */
  visible_count: number;
  /** Per-window metrics */
  windows: WebViewMetrics[];
}

// ============================================================================
// Subscription Types
// ============================================================================

/** Internal subscription options format */
interface SubscribeOptionsInternal {
  interval_ms: number;
  include_cpu: boolean;
  include_memory: boolean;
  include_runtime: boolean;
  include_process: boolean;
}

/**
 * Options for metric subscription
 */
export interface SubscribeOptions {
  /** Interval between metric updates in milliseconds (minimum: 100ms, default: 1000ms) */
  intervalMs?: number;
  /** Whether to include CPU metrics (default: true) */
  includeCpu?: boolean;
  /** Whether to include memory metrics (default: true) */
  includeMemory?: boolean;
  /** Whether to include runtime metrics (default: false) */
  includeRuntime?: boolean;
  /** Whether to include process info for current process (default: false) */
  includeProcess?: boolean;
}

/**
 * Complete metric snapshot from a subscription
 */
export interface MetricSnapshot {
  /** Timestamp when metrics were collected (Unix milliseconds) */
  timestamp_ms: number;
  /** CPU metrics (if requested) */
  cpu: CpuUsage | null;
  /** Memory metrics (if requested) */
  memory: MemoryUsage | null;
  /** Runtime metrics (if requested) */
  runtime: RuntimeMetrics | null;
  /** Current process info (if requested) */
  process: ProcessInfo | null;
}

/**
 * Information about an active subscription
 */
export interface SubscriptionInfo {
  /** Unique subscription ID */
  id: string;
  /** Interval in milliseconds */
  interval_ms: number;
  /** Whether subscription is currently active */
  is_active: boolean;
  /** Number of snapshots delivered so far */
  snapshot_count: number;
}

// ============================================================================
// Legacy Operations (Backward Compatibility)
// ============================================================================

/**
 * Get extension information (legacy).
 * @returns Extension info object
 */
export function info(): ExtensionInfo {
  return core.ops.op_monitor_info();
}

/**
 * Echo a message back (legacy, for testing).
 * @param message - Message to echo
 * @returns The same message
 */
export function echo(message: string): string {
  return core.ops.op_monitor_echo(message);
}

// ============================================================================
// System Metric Functions
// ============================================================================

/**
 * Get current CPU usage statistics.
 *
 * Note: This function takes ~200ms to return accurate readings because
 * CPU usage is measured over a time interval.
 *
 * @returns CPU usage including per-core percentages
 *
 * @example
 * ```ts
 * import { getCpu } from "runtime:monitor";
 *
 * const cpu = await getCpu();
 * console.log(`Total CPU: ${cpu.total_percent.toFixed(1)}%`);
 * console.log(`Cores: ${cpu.core_count}`);
 * cpu.per_core.forEach((usage, i) => {
 *   console.log(`  Core ${i}: ${usage.toFixed(1)}%`);
 * });
 * ```
 */
export async function getCpu(): Promise<CpuUsage> {
  return await core.ops.op_monitor_cpu();
}

/**
 * Get current memory usage statistics.
 *
 * @returns Memory usage including total, used, free, and swap
 *
 * @example
 * ```ts
 * import { getMemory } from "runtime:monitor";
 *
 * const mem = getMemory();
 * const usedGB = mem.used_bytes / (1024 ** 3);
 * const totalGB = mem.total_bytes / (1024 ** 3);
 * console.log(`Memory: ${usedGB.toFixed(1)} / ${totalGB.toFixed(1)} GB`);
 * ```
 */
export function getMemory(): MemoryUsage {
  return core.ops.op_monitor_memory();
}

/**
 * Get disk usage for all mounted filesystems.
 *
 * @returns Array of disk usage info for each mount point
 *
 * @example
 * ```ts
 * import { getDisks } from "runtime:monitor";
 *
 * const disks = getDisks();
 * for (const disk of disks) {
 *   const usedPct = (disk.used_bytes / disk.total_bytes * 100).toFixed(1);
 *   console.log(`${disk.mount_point}: ${usedPct}% used`);
 * }
 * ```
 */
export function getDisks(): DiskUsage[] {
  return core.ops.op_monitor_disk();
}

/**
 * Get network statistics for all interfaces.
 *
 * @returns Array of network stats per interface
 *
 * @example
 * ```ts
 * import { getNetwork } from "runtime:monitor";
 *
 * const networks = getNetwork();
 * for (const net of networks) {
 *   const sentMB = net.bytes_sent / (1024 ** 2);
 *   const recvMB = net.bytes_recv / (1024 ** 2);
 *   console.log(`${net.interface}: sent ${sentMB.toFixed(1)} MB, recv ${recvMB.toFixed(1)} MB`);
 * }
 * ```
 */
export function getNetwork(): NetworkStats[] {
  return core.ops.op_monitor_network();
}

/**
 * Get information about the current process (the Forge runtime).
 *
 * @returns Process info including CPU, memory, and status
 *
 * @example
 * ```ts
 * import { getProcessSelf } from "runtime:monitor";
 *
 * const proc = getProcessSelf();
 * console.log(`PID: ${proc.pid}`);
 * console.log(`Memory: ${(proc.memory_rss_bytes / 1024 / 1024).toFixed(1)} MB`);
 * console.log(`Status: ${proc.status}`);
 * ```
 */
export function getProcessSelf(): ProcessInfo {
  return core.ops.op_monitor_process_self();
}

/**
 * Get a list of running processes sorted by CPU usage.
 *
 * Returns top 50 processes by CPU usage.
 *
 * @returns Array of process info sorted by CPU usage (descending)
 *
 * @example
 * ```ts
 * import { getProcesses } from "runtime:monitor";
 *
 * const procs = getProcesses();
 * console.log("Top 5 CPU-consuming processes:");
 * for (const p of procs.slice(0, 5)) {
 *   console.log(`  ${p.name}: ${p.cpu_percent.toFixed(1)}%`);
 * }
 * ```
 */
export function getProcesses(): ProcessInfo[] {
  return core.ops.op_monitor_processes();
}

// ============================================================================
// Runtime Metric Functions
// ============================================================================

/**
 * Get Deno runtime metrics including event loop latency.
 *
 * Event loop latency measures how long it takes for scheduled callbacks
 * to actually execute. High latency indicates the event loop is blocked.
 *
 * @returns Runtime metrics
 *
 * @example
 * ```ts
 * import { getRuntime } from "runtime:monitor";
 *
 * const runtime = getRuntime();
 * console.log(`Uptime: ${runtime.uptime_secs}s`);
 * console.log(`Event loop latency: ${runtime.event_loop_latency_us}us`);
 * ```
 */
export function getRuntime(): RuntimeMetrics {
  return core.ops.op_monitor_runtime();
}

/**
 * Get V8 heap statistics.
 *
 * Note: Currently returns placeholder values. Full V8 heap stats
 * require direct isolate access which is not yet implemented.
 *
 * @returns Heap statistics
 */
export function getHeap(): HeapStats {
  return core.ops.op_monitor_heap();
}

// ============================================================================
// WebView Metric Functions
// ============================================================================

/**
 * Get WebView statistics across all windows.
 *
 * Note: Currently returns placeholder values. Full WebView metrics
 * require coordination with ext_window which is not yet implemented.
 *
 * @returns WebView statistics for all windows
 */
export function getWebViews(): WebViewStats {
  return core.ops.op_monitor_webview();
}

// ============================================================================
// Subscription API
// ============================================================================

/**
 * Subscribe to continuous metric updates.
 *
 * Creates a subscription that collects metrics at the specified interval.
 * Use `nextSnapshot()` to receive each update, and `unsubscribe()` to stop.
 *
 * Maximum 10 concurrent subscriptions allowed per runtime.
 *
 * @param options - Subscription configuration
 * @returns Subscription ID to use with nextSnapshot/unsubscribe
 *
 * @example
 * ```ts
 * import { subscribe, nextSnapshot, unsubscribe } from "runtime:monitor";
 *
 * // Start monitoring CPU and memory every 500ms
 * const subId = await subscribe({
 *   intervalMs: 500,
 *   includeCpu: true,
 *   includeMemory: true,
 * });
 *
 * // Receive 10 snapshots
 * for (let i = 0; i < 10; i++) {
 *   const snapshot = await nextSnapshot(subId);
 *   if (snapshot) {
 *     console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
 *   }
 * }
 *
 * // Stop monitoring
 * unsubscribe(subId);
 * ```
 */
export async function subscribe(options: SubscribeOptions = {}): Promise<string> {
  const internalOptions: SubscribeOptionsInternal = {
    interval_ms: options.intervalMs ?? 1000,
    include_cpu: options.includeCpu ?? true,
    include_memory: options.includeMemory ?? true,
    include_runtime: options.includeRuntime ?? false,
    include_process: options.includeProcess ?? false,
  };
  return await core.ops.op_monitor_subscribe(internalOptions);
}

/**
 * Get the next metric snapshot from a subscription.
 *
 * This is an async operation that waits for the next snapshot to be available.
 * Returns null if the subscription has been cancelled.
 *
 * @param subscriptionId - ID returned from subscribe()
 * @returns Next metric snapshot or null if subscription ended
 *
 * @example
 * ```ts
 * const snapshot = await nextSnapshot(subId);
 * if (snapshot) {
 *   console.log(`Timestamp: ${snapshot.timestamp_ms}`);
 *   if (snapshot.cpu) {
 *     console.log(`CPU: ${snapshot.cpu.total_percent}%`);
 *   }
 * }
 * ```
 */
export async function nextSnapshot(subscriptionId: string): Promise<MetricSnapshot | null> {
  return await core.ops.op_monitor_next(subscriptionId);
}

/**
 * Cancel a metric subscription.
 *
 * Stops the background metric collection for this subscription.
 * Any pending nextSnapshot() calls will return null.
 *
 * @param subscriptionId - ID returned from subscribe()
 * @throws Error if subscription ID is invalid
 *
 * @example
 * ```ts
 * unsubscribe(subId);
 * ```
 */
export function unsubscribe(subscriptionId: string): void {
  core.ops.op_monitor_unsubscribe(subscriptionId);
}

/**
 * List all active subscriptions.
 *
 * @returns Array of subscription info objects
 *
 * @example
 * ```ts
 * import { getSubscriptions } from "runtime:monitor";
 *
 * const subs = getSubscriptions();
 * for (const sub of subs) {
 *   console.log(`Subscription ${sub.id}: ${sub.snapshot_count} snapshots delivered`);
 * }
 * ```
 */
export function getSubscriptions(): SubscriptionInfo[] {
  return core.ops.op_monitor_subscriptions();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Get a complete system snapshot (CPU, memory, disk, network).
 *
 * Convenience function that collects all system metrics at once.
 *
 * @returns Object with all system metrics
 *
 * @example
 * ```ts
 * import { getSystemSnapshot } from "runtime:monitor";
 *
 * const sys = await getSystemSnapshot();
 * console.log(`CPU: ${sys.cpu.total_percent.toFixed(1)}%`);
 * console.log(`Memory: ${(sys.memory.used_bytes / 1024**3).toFixed(1)} GB`);
 * console.log(`Disks: ${sys.disks.length}`);
 * console.log(`Network interfaces: ${sys.network.length}`);
 * ```
 */
export async function getSystemSnapshot(): Promise<{
  cpu: CpuUsage;
  memory: MemoryUsage;
  disks: DiskUsage[];
  network: NetworkStats[];
}> {
  const [cpu, memory, disks, network] = await Promise.all([
    getCpu(),
    Promise.resolve(getMemory()),
    Promise.resolve(getDisks()),
    Promise.resolve(getNetwork()),
  ]);
  return { cpu, memory, disks, network };
}

/**
 * Format bytes as a human-readable string.
 *
 * @param bytes - Number of bytes
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted string (e.g., "1.5 GB")
 *
 * @example
 * ```ts
 * import { formatBytes, getMemory } from "runtime:monitor";
 *
 * const mem = getMemory();
 * console.log(`Used: ${formatBytes(mem.used_bytes)}`);  // "8.2 GB"
 * ```
 */
export function formatBytes(bytes: number, decimals: number = 1): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(decimals)} ${sizes[i]}`;
}

/**
 * Create a simple monitor loop that logs metrics periodically.
 *
 * @param intervalMs - Interval in milliseconds
 * @param callback - Function to call with each snapshot
 * @returns Stop function to cancel monitoring
 *
 * @example
 * ```ts
 * import { monitorLoop } from "runtime:monitor";
 *
 * const stop = await monitorLoop(1000, (snapshot) => {
 *   console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
 * });
 *
 * // Later, stop monitoring
 * stop();
 * ```
 */
export async function monitorLoop(
  intervalMs: number,
  callback: (snapshot: MetricSnapshot) => void
): Promise<() => void> {
  const subId = await subscribe({
    intervalMs,
    includeCpu: true,
    includeMemory: true,
    includeRuntime: true,
  });

  let running = true;

  // Start async loop
  (async () => {
    while (running) {
      const snapshot = await nextSnapshot(subId);
      if (!snapshot || !running) break;
      callback(snapshot);
    }
  })();

  // Return stop function
  return () => {
    running = false;
    unsubscribe(subId);
  };
}

// ============================================================================
// Convenience Aliases
// ============================================================================

export { getCpu as cpu };
export { getMemory as memory };
export { getDisks as disks };
export { getNetwork as network };
export { getProcessSelf as self };
export { getProcesses as processes };
export { getRuntime as runtime };
export { getHeap as heap };
export { getWebViews as webviews };
