/**
 * @module runtime:console
 *
 * Console capture for Forge apps.
 *
 * Captures `console.*` output into a bounded ring buffer on the host so app
 * code can read recent logs back (for an in-app log viewer, diagnostics, or the
 * web inspector's console panel). Unlike `runtime:log` (which forwards to host
 * tracing and keeps nothing), this extension retains records and exposes
 * tail()/clear() to read them.
 *
 * Two sources feed the same buffer:
 * - Deno side: call install() to patch `console.*` so each call is also
 *   captured (the original console still runs).
 * - Renderer side: the WebView forwards console messages over the IPC bridge;
 *   the host parses them into records tagged "renderer".
 *
 * Example:
 * ```typescript
 * import { install, tail, clear } from "runtime:console";
 *
 * install();
 * console.warn("disk low", { freeMb: 12 });
 *
 * const recent = tail(50);
 * console.log(`captured ${recent.length} records`);
 * clear();
 * ```
 */

export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

/** Origin of a captured console record. */
export type ConsoleSource = "deno" | "renderer";

/** A single captured console call. */
export interface ConsoleRecord {
  /** Console level: "log", "info", "warn", "error", "debug". */
  level: string;
  /** Arguments passed to the console call, preserved as JSON values. */
  args: unknown[];
  /** Capture time in milliseconds since the UNIX epoch. */
  timestamp_ms: number;
  /** Whether the record came from the Deno side or the renderer. */
  source: ConsoleSource;
}

declare const Deno: {
  core: {
    ops: {
      op_console_info(): ExtensionInfo;
      op_console_push(level: string, args: unknown[], source?: string): void;
      op_console_tail(n: number): ConsoleRecord[];
      op_console_clear(): number;
    };
  };
};

const { core } = Deno;

const CONSOLE_LEVELS = ["log", "info", "warn", "error", "debug"] as const;
export type ConsoleLevel = (typeof CONSOLE_LEVELS)[number];

/** Original console methods, captured when install() runs so we can restore. */
let originals: Partial<Record<ConsoleLevel, (...args: unknown[]) => void>> | null = null;

/**
 * Get extension information (name, version, status).
 */
export function info(): ExtensionInfo {
  return core.ops.op_console_info();
}

/**
 * Record a console call into the host buffer.
 *
 * @param level - Console level ("log", "info", "warn", "error", "debug")
 * @param args - Arguments of the console call
 * @param source - Origin tag; defaults to "deno"
 */
export function push(level: string, args: unknown[], source: ConsoleSource = "deno"): void {
  core.ops.op_console_push(level, args, source);
}

/**
 * Patch the global `console.*` methods so every call is also captured into the
 * host buffer. The original console methods still run. Idempotent: calling it
 * again while installed is a no-op.
 */
export function install(): void {
  if (originals) return;
  const saved: Partial<Record<ConsoleLevel, (...args: unknown[]) => void>> = {};
  const target = globalThis.console as unknown as Record<string, (...args: unknown[]) => void>;

  for (const level of CONSOLE_LEVELS) {
    const original = target[level];
    saved[level] = original;
    target[level] = (...args: unknown[]) => {
      try {
        core.ops.op_console_push(level, args, "deno");
      } catch {
        // Never let capture break the app's logging.
      }
      if (typeof original === "function") {
        original.apply(target, args);
      }
    };
  }
  originals = saved;
}

/**
 * Restore the original `console.*` methods patched by install().
 */
export function uninstall(): void {
  if (!originals) return;
  const target = globalThis.console as unknown as Record<string, (...args: unknown[]) => void>;
  for (const level of CONSOLE_LEVELS) {
    const original = originals[level];
    if (typeof original === "function") {
      target[level] = original;
    }
  }
  originals = null;
}

/**
 * Return the most recent `n` captured records in chronological order.
 *
 * @param n - Maximum number of records to return
 */
export function tail(n: number): ConsoleRecord[] {
  return core.ops.op_console_tail(n);
}

/**
 * Clear all captured records.
 *
 * @returns The number of records removed
 */
export function clear(): number {
  return core.ops.op_console_clear();
}