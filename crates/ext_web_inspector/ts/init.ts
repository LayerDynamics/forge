// ext_web_inspector TypeScript bindings
// Auto-generated types and functions for runtime:web_inspector

const { core } = (Deno as any);

// ============================================================================
// Types
// ============================================================================

/** CDP domain identifier */
export type CdpDomain =
  | "Forge.Monitor"
  | "Forge.Trace"
  | "Forge.Signals"
  | "Forge.Runtime";

/** Inspector session information */
export interface InspectorSessionInfo {
  /** Window ID this session is attached to */
  windowId: string;
  /** Whether the session is connected */
  isConnected: boolean;
  /** Whether the custom Forge panel has been injected */
  panelInjected: boolean;
  /** Enabled CDP domains */
  enabledDomains: string[];
  /** Session creation timestamp (Unix millis) */
  createdAtMs: number;
}

/** Inspector event */
export interface InspectorEvent {
  /** Event type/name */
  eventType: string;
  /** Source domain */
  domain: string;
  /** Event payload */
  payload: unknown;
  /** Timestamp (Unix millis) */
  timestampMs: number;
}

/** Aggregated metrics from all Forge extensions */
export interface AggregatedMetrics {
  /** System metrics available */
  systemAvailable: boolean;
  /** Runtime metrics available */
  runtimeAvailable: boolean;
  /** Trace data available */
  traceAvailable: boolean;
  /** Number of active spans (if trace enabled) */
  activeSpanCount: number;
  /** Number of finished spans (if trace enabled) */
  finishedSpanCount: number;
  /** Signal subscriptions active */
  signalSubscriptions: number;
  /** Window count */
  windowCount: number;
  /** IPC channel count */
  ipcChannelCount: number;
}

/** Extension information */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
  supportedDomains: string[];
}

// ============================================================================
// CDP Types
// ============================================================================

/** CDP request message */
export interface CdpMessage {
  /** Request ID */
  id: number;
  /** Method name (e.g., "Forge.Monitor.getMetrics") */
  method: string;
  /** Optional parameters */
  params?: Record<string, unknown>;
}

/** CDP response */
export interface CdpResponse {
  /** Request ID (matches the request) */
  id: number;
  /** Result on success */
  result?: unknown;
  /** Error on failure */
  error?: CdpError;
}

/** CDP error */
export interface CdpError {
  /** Error code */
  code: number;
  /** Error message */
  message: string;
  /** Optional additional data */
  data?: unknown;
}

/** CDP event (server -> client) */
export interface CdpEvent {
  /** Event method (e.g., "Forge.Monitor.metricsUpdated") */
  method: string;
  /** Event parameters */
  params: unknown;
}

// ============================================================================
// Forge.Monitor Domain Types
// ============================================================================

export interface MonitorMetrics {
  cpu: {
    totalPercent: number;
    coreCount: number;
  };
  memory: {
    totalBytes: number;
    usedBytes: number;
    freeBytes: number;
  };
  eventLoop: {
    pendingOps: number;
    latencyUs: number;
  };
  timestamp: number;
}

export interface CpuUsage {
  totalPercent: number;
  perCore: number[];
  coreCount: number;
}

export interface MemoryUsage {
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  availableBytes: number;
}

export interface RuntimeMetrics {
  pendingOpsCount: number;
  eventLoopLatencyUs: number;
  uptimeSecs: number;
}

export interface ProfileOptions {
  /** Sample interval in microseconds */
  sampleIntervalUs?: number;
  /** Maximum duration in milliseconds */
  maxDurationMs?: number;
}

export interface ProfileData {
  profileId: string;
  samples: unknown[];
  durationMs: number;
}

// ============================================================================
// Forge.Trace Domain Types
// ============================================================================

export interface SpanInfo {
  id: string;
  name: string;
  target: string;
  level: string;
  startTimeUs: number;
  endTimeUs?: number;
  parentId?: string;
}

export interface SpanQueryOptions {
  /** Maximum number of spans to return */
  limit?: number;
  /** Filter by span name pattern */
  namePattern?: string;
  /** Filter by target pattern */
  targetPattern?: string;
  /** Filter by minimum level */
  minLevel?: string;
  /** Include only active (unfinished) spans */
  activeOnly?: boolean;
}

// ============================================================================
// Forge.Signals Domain Types
// ============================================================================

export interface SignalInfo {
  name: string;
  number: number;
  description: string;
}

export interface SignalSubscription {
  id: string;
  signal: string;
  active: boolean;
}

// ============================================================================
// Forge.Runtime Domain Types
// ============================================================================

export interface AppInfo {
  name: string;
  version: string;
  denoVersion: string;
  platform: string;
  arch: string;
}

export interface WindowInfo {
  id: string;
  title: string;
  visible: boolean;
}

export interface ExtensionStatus {
  name: string;
  status: "loaded" | "error" | "disabled";
}

export interface IpcChannel {
  name: string;
  messageCount: number;
}

// ============================================================================
// Core Functions
// ============================================================================

/** Get extension information */
export function info(): ExtensionInfo {
  return core.ops.op_web_inspector_info();
}

// ============================================================================
// Session Management
// ============================================================================

/**
 * Connect to the inspector for a window.
 *
 * Creates a new inspector session for the specified window, enabling
 * CDP communication and panel injection.
 *
 * @param windowId - The window ID to connect to
 * @returns Session information
 */
export async function connect(windowId: string): Promise<InspectorSessionInfo> {
  return await core.ops.op_web_inspector_connect(windowId);
}

/**
 * Disconnect from the inspector for a window.
 *
 * Closes the session and cleans up resources.
 *
 * @param windowId - The window ID to disconnect from
 */
export function disconnect(windowId: string): void {
  core.ops.op_web_inspector_disconnect(windowId);
}

/**
 * Check if the inspector is connected for a window.
 *
 * @param windowId - The window ID to check
 * @returns Whether the inspector is connected
 */
export function isConnected(windowId: string): boolean {
  return core.ops.op_web_inspector_is_connected(windowId);
}

/**
 * Get all active inspector sessions.
 *
 * @returns Array of session information
 */
export function sessions(): InspectorSessionInfo[] {
  return core.ops.op_web_inspector_sessions();
}

// ============================================================================
// CDP Communication
// ============================================================================

/**
 * Send a CDP command to a Forge domain.
 *
 * Sends a command to one of the custom Forge CDP domains (Forge.Monitor,
 * Forge.Trace, Forge.Signals, Forge.Runtime).
 *
 * @param windowId - The window ID
 * @param method - The CDP method (e.g., "Forge.Monitor.getMetrics")
 * @param params - Optional parameters
 * @returns The command result
 */
export async function sendCdp<T = unknown>(
  windowId: string,
  method: string,
  params?: Record<string, unknown>
): Promise<T> {
  return await core.ops.op_web_inspector_send_cdp(windowId, method, params);
}

/**
 * Enable a CDP domain for a session.
 *
 * Must be called before sending commands to a domain.
 *
 * @param windowId - The window ID
 * @param domain - The domain to enable (e.g., "Forge.Monitor")
 * @returns Whether the domain was newly enabled
 */
export function enableDomain(windowId: string, domain: CdpDomain): boolean {
  return core.ops.op_web_inspector_enable_domain(windowId, domain);
}

/**
 * Disable a CDP domain for a session.
 *
 * @param windowId - The window ID
 * @param domain - The domain to disable
 * @returns Whether the domain was previously enabled
 */
export function disableDomain(windowId: string, domain: CdpDomain): boolean {
  return core.ops.op_web_inspector_disable_domain(windowId, domain);
}

// ============================================================================
// Panel Injection
// ============================================================================

/**
 * Inject the Forge DevTools panel into the native inspector.
 *
 * This injects a custom "Forge" tab into the browser's DevTools that
 * displays Forge-specific metrics, traces, and runtime information.
 *
 * @param windowId - The window ID
 * @returns Whether injection was successful (false if already injected)
 */
export async function injectPanel(windowId: string): Promise<boolean> {
  return await core.ops.op_web_inspector_inject_panel(windowId);
}

/**
 * Check if the panel is injected for a window.
 *
 * @param windowId - The window ID
 * @returns Whether the panel is injected
 */
export function isPanelInjected(windowId: string): boolean {
  return core.ops.op_web_inspector_is_panel_injected(windowId);
}

// ============================================================================
// Metrics
// ============================================================================

/**
 * Get aggregated metrics from all Forge extensions.
 *
 * Returns a summary of available metrics from ext_monitor, ext_trace,
 * ext_signals, and other extensions.
 *
 * @returns Aggregated metrics
 */
export function getMetrics(): AggregatedMetrics {
  return core.ops.op_web_inspector_get_metrics();
}

// ============================================================================
// Event Subscription
// ============================================================================

/**
 * Subscribe to inspector events.
 *
 * Returns a subscription ID that can be used with `nextEvent()` and
 * `unsubscribeEvents()`.
 *
 * @returns Subscription ID
 */
export function subscribeEvents(): string {
  return core.ops.op_web_inspector_subscribe_events();
}

/**
 * Get the next event from a subscription.
 *
 * Blocks until an event is available or the subscription is closed.
 *
 * @param subscriptionId - The subscription ID from `subscribeEvents()`
 * @returns The next event, or null if the subscription is closed
 */
export async function nextEvent(
  subscriptionId: string
): Promise<InspectorEvent | null> {
  return await core.ops.op_web_inspector_next_event(subscriptionId);
}

/**
 * Unsubscribe from inspector events.
 *
 * @param subscriptionId - The subscription ID from `subscribeEvents()`
 */
export function unsubscribeEvents(subscriptionId: string): void {
  core.ops.op_web_inspector_unsubscribe_events(subscriptionId);
}

// ============================================================================
// Domain-Specific Helpers
// ============================================================================

/**
 * Forge.Monitor domain helpers
 */
export const Monitor = {
  /**
   * Enable the Monitor domain for a window.
   */
  async enable(windowId: string): Promise<void> {
    enableDomain(windowId, "Forge.Monitor");
    await sendCdp(windowId, "Forge.Monitor.enable");
  },

  /**
   * Disable the Monitor domain for a window.
   */
  async disable(windowId: string): Promise<void> {
    await sendCdp(windowId, "Forge.Monitor.disable");
    disableDomain(windowId, "Forge.Monitor");
  },

  /**
   * Get all metrics.
   */
  async getMetrics(windowId: string): Promise<MonitorMetrics> {
    return await sendCdp<MonitorMetrics>(windowId, "Forge.Monitor.getMetrics");
  },

  /**
   * Get CPU usage.
   */
  async getCpuUsage(windowId: string): Promise<CpuUsage> {
    return await sendCdp<CpuUsage>(windowId, "Forge.Monitor.getCpuUsage");
  },

  /**
   * Get memory usage.
   */
  async getMemoryUsage(windowId: string): Promise<MemoryUsage> {
    return await sendCdp<MemoryUsage>(windowId, "Forge.Monitor.getMemoryUsage");
  },

  /**
   * Get runtime metrics.
   */
  async getRuntimeMetrics(windowId: string): Promise<RuntimeMetrics> {
    return await sendCdp<RuntimeMetrics>(
      windowId,
      "Forge.Monitor.getRuntimeMetrics"
    );
  },

  /**
   * Start profiling.
   */
  async startProfiling(
    windowId: string,
    options?: ProfileOptions
  ): Promise<string> {
    const result = await sendCdp<{ profileId: string }>(
      windowId,
      "Forge.Monitor.startProfiling",
      options
    );
    return result.profileId;
  },

  /**
   * Stop profiling.
   */
  async stopProfiling(windowId: string, profileId: string): Promise<ProfileData> {
    return await sendCdp<ProfileData>(windowId, "Forge.Monitor.stopProfiling", {
      profileId,
    });
  },
};

/**
 * Forge.Trace domain helpers
 */
export const Trace = {
  /**
   * Enable the Trace domain for a window.
   */
  async enable(windowId: string): Promise<void> {
    enableDomain(windowId, "Forge.Trace");
    await sendCdp(windowId, "Forge.Trace.enable");
  },

  /**
   * Disable the Trace domain for a window.
   */
  async disable(windowId: string): Promise<void> {
    await sendCdp(windowId, "Forge.Trace.disable");
    disableDomain(windowId, "Forge.Trace");
  },

  /**
   * Get finished spans.
   */
  async getSpans(
    windowId: string,
    options?: SpanQueryOptions
  ): Promise<SpanInfo[]> {
    const result = await sendCdp<{ spans: SpanInfo[] }>(
      windowId,
      "Forge.Trace.getSpans",
      options
    );
    return result.spans;
  },

  /**
   * Get active (in-progress) spans.
   */
  async getActiveSpans(windowId: string): Promise<SpanInfo[]> {
    const result = await sendCdp<{ spans: SpanInfo[] }>(
      windowId,
      "Forge.Trace.getActiveSpans"
    );
    return result.spans;
  },

  /**
   * Clear all finished spans.
   */
  async clearSpans(windowId: string): Promise<number> {
    const result = await sendCdp<{ clearedCount: number }>(
      windowId,
      "Forge.Trace.clearSpans"
    );
    return result.clearedCount;
  },
};

/**
 * Forge.Signals domain helpers
 */
export const Signals = {
  /**
   * Enable the Signals domain for a window.
   */
  async enable(windowId: string): Promise<void> {
    enableDomain(windowId, "Forge.Signals");
    await sendCdp(windowId, "Forge.Signals.enable");
  },

  /**
   * Disable the Signals domain for a window.
   */
  async disable(windowId: string): Promise<void> {
    await sendCdp(windowId, "Forge.Signals.disable");
    disableDomain(windowId, "Forge.Signals");
  },

  /**
   * Get supported signals for the current platform.
   */
  async getSupportedSignals(windowId: string): Promise<SignalInfo[]> {
    const result = await sendCdp<{ signals: SignalInfo[] }>(
      windowId,
      "Forge.Signals.getSupportedSignals"
    );
    return result.signals;
  },

  /**
   * Get active signal subscriptions.
   */
  async getActiveSubscriptions(
    windowId: string
  ): Promise<SignalSubscription[]> {
    const result = await sendCdp<{ subscriptions: SignalSubscription[] }>(
      windowId,
      "Forge.Signals.getActiveSubscriptions"
    );
    return result.subscriptions;
  },
};

/**
 * Forge.Runtime domain helpers
 */
export const Runtime = {
  /**
   * Enable the Runtime domain for a window.
   */
  async enable(windowId: string): Promise<void> {
    enableDomain(windowId, "Forge.Runtime");
    await sendCdp(windowId, "Forge.Runtime.enable");
  },

  /**
   * Disable the Runtime domain for a window.
   */
  async disable(windowId: string): Promise<void> {
    await sendCdp(windowId, "Forge.Runtime.disable");
    disableDomain(windowId, "Forge.Runtime");
  },

  /**
   * Get application information.
   */
  async getAppInfo(windowId: string): Promise<AppInfo> {
    const result = await sendCdp<{ app: AppInfo }>(
      windowId,
      "Forge.Runtime.getAppInfo"
    );
    return result.app;
  },

  /**
   * Get all windows.
   */
  async getWindows(windowId: string): Promise<WindowInfo[]> {
    const result = await sendCdp<{ windows: WindowInfo[] }>(
      windowId,
      "Forge.Runtime.getWindows"
    );
    return result.windows;
  },

  /**
   * Get loaded extensions.
   */
  async getExtensions(windowId: string): Promise<ExtensionStatus[]> {
    const result = await sendCdp<{ extensions: ExtensionStatus[] }>(
      windowId,
      "Forge.Runtime.getExtensions"
    );
    return result.extensions;
  },

  /**
   * Get IPC channels.
   */
  async getIpcChannels(windowId: string): Promise<IpcChannel[]> {
    const result = await sendCdp<{ channels: IpcChannel[] }>(
      windowId,
      "Forge.Runtime.getIpcChannels"
    );
    return result.channels;
  },
};

// ============================================================================
// High-Level API
// ============================================================================

/**
 * Create an inspector instance for a window.
 *
 * This is a convenience class that manages the inspector session lifecycle.
 *
 * @example
 * ```ts
 * import * as webInspector from "runtime:web_inspector";
 *
 * const inspector = await webInspector.createInspector("main-window");
 * await inspector.injectPanel();
 *
 * // Get metrics
 * const metrics = await inspector.getMetrics();
 * console.log("CPU:", metrics.cpu.totalPercent);
 *
 * // Subscribe to events
 * for await (const event of inspector.events()) {
 *   console.log("Event:", event.eventType);
 * }
 *
 * // Clean up
 * inspector.dispose();
 * ```
 */
export class Inspector {
  private _windowId: string;
  private _eventSubscriptionId: string | null = null;
  private _disposed = false;

  private constructor(windowId: string) {
    this._windowId = windowId;
  }

  /** The window ID this inspector is attached to */
  get windowId(): string {
    return this._windowId;
  }

  /** Whether this inspector is connected */
  get isConnected(): boolean {
    return !this._disposed && isConnected(this._windowId);
  }

  /**
   * Create and connect an inspector for a window.
   */
  static async create(windowId: string): Promise<Inspector> {
    await connect(windowId);
    return new Inspector(windowId);
  }

  /**
   * Inject the Forge DevTools panel.
   */
  async injectPanel(): Promise<boolean> {
    this.ensureNotDisposed();
    return await injectPanel(this._windowId);
  }

  /**
   * Enable all Forge CDP domains.
   */
  async enableAllDomains(): Promise<void> {
    this.ensureNotDisposed();
    await Monitor.enable(this._windowId);
    await Trace.enable(this._windowId);
    await Signals.enable(this._windowId);
    await Runtime.enable(this._windowId);
  }

  /**
   * Get metrics from the Monitor domain.
   */
  async getMetrics(): Promise<MonitorMetrics> {
    this.ensureNotDisposed();
    return await Monitor.getMetrics(this._windowId);
  }

  /**
   * Get active spans from the Trace domain.
   */
  async getActiveSpans(): Promise<SpanInfo[]> {
    this.ensureNotDisposed();
    return await Trace.getActiveSpans(this._windowId);
  }

  /**
   * Get app info from the Runtime domain.
   */
  async getAppInfo(): Promise<AppInfo> {
    this.ensureNotDisposed();
    return await Runtime.getAppInfo(this._windowId);
  }

  /**
   * Async iterator for inspector events.
   */
  async *events(): AsyncIterableIterator<InspectorEvent> {
    this.ensureNotDisposed();

    if (!this._eventSubscriptionId) {
      this._eventSubscriptionId = subscribeEvents();
    }

    while (!this._disposed) {
      const event = await nextEvent(this._eventSubscriptionId);
      if (event === null) break;
      yield event;
    }
  }

  /**
   * Dispose of this inspector, cleaning up resources.
   */
  dispose(): void {
    if (this._disposed) return;

    if (this._eventSubscriptionId) {
      try {
        unsubscribeEvents(this._eventSubscriptionId);
      } catch {
        // Ignore errors during cleanup
      }
      this._eventSubscriptionId = null;
    }

    try {
      disconnect(this._windowId);
    } catch {
      // Ignore errors during cleanup
    }

    this._disposed = true;
  }

  private ensureNotDisposed(): void {
    if (this._disposed) {
      throw new Error("Inspector has been disposed");
    }
  }
}

/**
 * Create an inspector instance for a window.
 *
 * @param windowId - The window ID
 * @returns A connected Inspector instance
 */
export async function createInspector(windowId: string): Promise<Inspector> {
  return await Inspector.create(windowId);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Format bytes as human-readable string.
 */
export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

/**
 * Format microseconds as human-readable duration.
 */
export function formatDuration(us: number): string {
  if (us < 1000) return `${us} us`;
  if (us < 1_000_000) return `${(us / 1000).toFixed(2)} ms`;
  return `${(us / 1_000_000).toFixed(2)} s`;
}

/**
 * Format percentage.
 */
export function formatPercent(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}
