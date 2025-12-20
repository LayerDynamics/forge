/**
 * @module runtime:trace
 *
 * Lightweight tracing extension for span-based performance tracking.
 *
 * Features:
 * - Manual span lifecycle: start() -> end() with unique ID tracking
 * - High-resolution duration measurement (Rust Instant)
 * - instant() for point-in-time events (zero duration)
 * - flush() for batch export of completed spans
 *
 * Error: SpanNotFound thrown when end() called with invalid span ID.
 *
 * Architecture:
 * - TraceState in OpState tracks active spans (HashMap) and finished spans (Vec)
 * - Span IDs are monotonic u64 counters
 * - No parent-child relationships (flat tracing)
 * - No permissions required
 *
 * Example:
 * ```typescript
 * import { start, end, flush } from "runtime:trace";
 *
 * const id = start("fetchData", { url });
 * try {
 *   const data = await fetch(url);
 *   end(id, { status: "ok" });
 * } catch (e) {
 *   end(id, { error: e.message });
 * }
 *
 * // Periodic export
 * setInterval(() => {
 *   const spans = flush();
 *   console.log(`Exported ${spans.length} spans`);
 * }, 60000);
 * ```
 */

export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

/**
 * Represents a completed trace span with timing and metadata.
 *
 * Returned by `end()`, `instant()`, and `flush()` operations.
 */
export interface SpanRecord {
  /** Unique span ID (matches ID returned by `start()`) */
  id: bigint;
  /** Span name */
  name: string;
  /** Wall-clock timestamp (milliseconds since UNIX epoch) */
  started_at: bigint;
  /** Elapsed duration in milliseconds (0 for instant events) */
  duration_ms: number;
  /** Optional arbitrary attributes (JSON-serializable) */
  attributes?: unknown;
  /** Optional result data (JSON-serializable) */
  result?: unknown;
}

declare const Deno: {
  core: {
    ops: {
      op_trace_info(): ExtensionInfo;
      op_trace_start(name: string, attributes?: unknown): bigint;
      op_trace_end(id: bigint, result?: unknown): SpanRecord;
      op_trace_instant(name: string, attributes?: unknown): SpanRecord;
      op_trace_flush(): SpanRecord[];
    };
  };
};

const { core } = Deno;

/**
 * Get extension information (name, version, status).
 * @returns Extension metadata
 */
export function info(): ExtensionInfo {
  return core.ops.op_trace_info();
}

/**
 * Start a trace span and return its unique ID.
 *
 * Call end() with the returned ID to finish the span. Use try/finally to ensure
 * end() is always called.
 *
 * @param name - Span name
 * @param attributes - Optional JSON-serializable attributes
 * @returns Unique span ID (pass to end())
 */
export function start(name: string, attributes?: unknown): bigint {
  return core.ops.op_trace_start(name, attributes);
}

/**
 * End a trace span and return the completed record.
 *
 * @param id - Span ID from start()
 * @param result - Optional result data
 * @returns Completed span record with duration
 * @throws {TraceError} SpanNotFound if ID is invalid
 */
export function end(id: bigint, result?: unknown): SpanRecord {
  return core.ops.op_trace_end(id, result);
}

/**
 * Record a point-in-time event with zero duration.
 *
 * @param name - Event name
 * @param attributes - Optional event metadata
 * @returns Span record with duration_ms: 0
 */
export function instant(name: string, attributes?: unknown): SpanRecord {
  return core.ops.op_trace_instant(name, attributes);
}

/**
 * Retrieve all finished spans and clear the buffer.
 *
 * Drains the finished spans buffer. Subsequent calls return only spans finished
 * after the previous flush.
 *
 * @returns Array of all finished span records since last flush
 */
export function flush(): SpanRecord[] {
  return core.ops.op_trace_flush();
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  start: { args: []; result: void };
  end: { args: []; result: void };
  instant: { args: []; result: void };
  flush: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "start" | "end" | "instant" | "flush";

/** Hook callback types */
type BeforeHookCallback<T extends OpName> = (args: OpArgs<T>) => void | Promise<void>;
type AfterHookCallback<T extends OpName> = (result: OpResult<T>, args: OpArgs<T>) => void | Promise<void>;
type ErrorHookCallback<T extends OpName> = (error: Error, args: OpArgs<T>) => void | Promise<void>;

/** Internal hook storage */
const _hooks = {
  before: new Map<OpName, Set<BeforeHookCallback<OpName>>>(),
  after: new Map<OpName, Set<AfterHookCallback<OpName>>>(),
  error: new Map<OpName, Set<ErrorHookCallback<OpName>>>(),
};

/**
 * Register a callback to be called before an operation executes.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the operation arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onBefore<T extends OpName>(
  opName: T,
  callback: BeforeHookCallback<T>
): () => void {
  if (!_hooks.before.has(opName)) {
    _hooks.before.set(opName, new Set());
  }
  _hooks.before.get(opName)!.add(callback as BeforeHookCallback<OpName>);
  return () => _hooks.before.get(opName)?.delete(callback as BeforeHookCallback<OpName>);
}

/**
 * Register a callback to be called after an operation completes successfully.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the result and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onAfter<T extends OpName>(
  opName: T,
  callback: AfterHookCallback<T>
): () => void {
  if (!_hooks.after.has(opName)) {
    _hooks.after.set(opName, new Set());
  }
  _hooks.after.get(opName)!.add(callback as AfterHookCallback<OpName>);
  return () => _hooks.after.get(opName)?.delete(callback as AfterHookCallback<OpName>);
}

/**
 * Register a callback to be called when an operation throws an error.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the error and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onError<T extends OpName>(
  opName: T,
  callback: ErrorHookCallback<T>
): () => void {
  if (!_hooks.error.has(opName)) {
    _hooks.error.set(opName, new Set());
  }
  _hooks.error.get(opName)!.add(callback as ErrorHookCallback<OpName>);
  return () => _hooks.error.get(opName)?.delete(callback as ErrorHookCallback<OpName>);
}

/** Internal: Invoke before hooks for an operation */
async function _invokeBeforeHooks<T extends OpName>(opName: T, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.before.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(args);
    }
  }
}

/** Internal: Invoke after hooks for an operation */
async function _invokeAfterHooks<T extends OpName>(opName: T, result: OpResult<T>, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.after.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(result, args);
    }
  }
}

/** Internal: Invoke error hooks for an operation */
async function _invokeErrorHooks<T extends OpName>(opName: T, error: Error, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.error.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(error, args);
    }
  }
}

/**
 * Remove all hooks for a specific operation or all operations.
 * @param opName - Optional: specific operation to clear hooks for
 */
export function removeAllHooks(opName?: OpName): void {
  if (opName) {
    _hooks.before.delete(opName);
    _hooks.after.delete(opName);
    _hooks.error.delete(opName);
  } else {
    _hooks.before.clear();
    _hooks.after.clear();
    _hooks.error.clear();
  }
}

/** Handler function type */
type HandlerFn = (...args: unknown[]) => unknown | Promise<unknown>;

/** Internal handler storage */
const _handlers = new Map<string, HandlerFn>();

/**
 * Register a custom handler that can be invoked by name.
 * @param name - Unique name for the handler
 * @param handler - Handler function to register
 * @throws Error if a handler with the same name already exists
 */
export function registerHandler(name: string, handler: HandlerFn): void {
  if (_handlers.has(name)) {
    throw new Error(`Handler '${name}' already registered`);
  }
  _handlers.set(name, handler);
}

/**
 * Invoke a registered handler by name.
 * @param name - Name of the handler to invoke
 * @param args - Arguments to pass to the handler
 * @returns The handler's return value
 * @throws Error if no handler with the given name exists
 */
export async function invokeHandler(name: string, ...args: unknown[]): Promise<unknown> {
  const handler = _handlers.get(name);
  if (!handler) {
    throw new Error(`Handler '${name}' not found`);
  }
  return await handler(...args);
}

/**
 * List all registered handler names.
 * @returns Array of handler names
 */
export function listHandlers(): string[] {
  return Array.from(_handlers.keys());
}

/**
 * Remove a registered handler.
 * @param name - Name of the handler to remove
 * @returns true if the handler was removed, false if it didn't exist
 */
export function removeHandler(name: string): boolean {
  return _handlers.delete(name);
}

/**
 * Check if a handler is registered.
 * @param name - Name of the handler to check
 * @returns true if the handler exists
 */
export function hasHandler(name: string): boolean {
  return _handlers.has(name);
}

