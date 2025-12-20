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

interface ExtensionInfo {
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
