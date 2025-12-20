// runtime:log module - structured logging bridge to host tracing.
// Also provides browser console forwarding via IPC.

interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

type LogLevel = "trace" | "debug" | "info" | "warn" | "warning" | "error";

declare const Deno: {
  core: {
    ops: {
      op_log_info(): ExtensionInfo;
      op_log_emit(level: string, message: string, fields?: Record<string, unknown>): void;
      op_ipc_send(windowId: string, channel: string, payload: unknown): Promise<void>;
    };
  };
};

const { core } = Deno;

// ============================================================================
// Host Logging (outputs to terminal via tracing)
// ============================================================================

export function info(): ExtensionInfo {
  return core.ops.op_log_info();
}

export function emit(level: LogLevel, message: string, fields?: Record<string, unknown>): void {
  core.ops.op_log_emit(level, message, fields ?? {});
}

export function trace(message: string, fields?: Record<string, unknown>): void {
  emit("trace", message, fields);
}

export function debug(message: string, fields?: Record<string, unknown>): void {
  emit("debug", message, fields);
}

export function infoLog(message: string, fields?: Record<string, unknown>): void {
  emit("info", message, fields);
}

export function warn(message: string, fields?: Record<string, unknown>): void {
  emit("warn", message, fields);
}

export function error(message: string, fields?: Record<string, unknown>): void {
  emit("error", message, fields);
}

// ============================================================================
// Browser Console Forwarding (outputs to browser DevTools)
// ============================================================================

/**
 * Configuration for browser console forwarding.
 * Set a default window ID to avoid passing it on every call.
 */
let defaultWindowId: string | null = null;

/**
 * Set the default window ID for browser console forwarding.
 * Once set, you can omit the windowId parameter in browserConsole calls.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, browserConsole } from "runtime:log";
 *
 * setDefaultWindow(win.id);
 * browserConsole.log("This appears in browser DevTools");
 * ```
 */
export function setDefaultWindow(windowId: string): void {
  defaultWindowId = windowId;
}

/**
 * Send a log message to the browser's DevTools console via IPC.
 * @param windowId - The window to send to (uses default if not specified)
 * @param level - Log level (trace, debug, info, warn, error)
 * @param message - The message to log
 * @param fields - Optional structured data to include
 */
async function sendToBrowser(
  windowId: string | undefined,
  level: LogLevel,
  message: string,
  fields?: Record<string, unknown>
): Promise<void> {
  const targetWindow = windowId ?? defaultWindowId;
  if (!targetWindow) {
    // Fallback to host logging if no window ID
    emit(level, `[no window] ${message}`, fields);
    return;
  }
  await core.ops.op_ipc_send(targetWindow, "__console__", {
    level,
    message,
    fields: fields ?? {},
  });
}

/**
 * Browser console logging utilities.
 * These send messages to the browser DevTools console via IPC.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, browserConsole } from "runtime:log";
 *
 * // Set default window once
 * setDefaultWindow(win.id);
 *
 * // Then log without specifying window
 * browserConsole.log("Hello from Deno backend!");
 * browserConsole.warn("This is a warning");
 * browserConsole.error("Something went wrong", { code: 500 });
 *
 * // Or specify window explicitly
 * browserConsole.log("Message to specific window", undefined, otherWindowId);
 * ```
 */
export const browserConsole = {
  /** Log a trace-level message to browser DevTools */
  trace: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "trace", message, fields),

  /** Log a debug-level message to browser DevTools */
  debug: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "debug", message, fields),

  /** Log an info-level message to browser DevTools */
  log: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "info", message, fields),

  /** Log an info-level message to browser DevTools */
  info: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "info", message, fields),

  /** Log a warning to browser DevTools */
  warn: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "warn", message, fields),

  /** Log an error to browser DevTools */
  error: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "error", message, fields),
};

/**
 * Dual-output logging - logs to BOTH host terminal AND browser DevTools.
 * Useful for development when you want to see logs in both places.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, dualLog } from "runtime:log";
 *
 * setDefaultWindow(win.id);
 * dualLog.info("Starting app..."); // Appears in terminal AND browser
 * ```
 */
export const dualLog = {
  trace: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("trace", message, fields);
    await sendToBrowser(windowId, "trace", message, fields);
  },

  debug: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("debug", message, fields);
    await sendToBrowser(windowId, "debug", message, fields);
  },

  info: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("info", message, fields);
    await sendToBrowser(windowId, "info", message, fields);
  },

  warn: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("warn", message, fields);
    await sendToBrowser(windowId, "warn", message, fields);
  },

  error: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("error", message, fields);
    await sendToBrowser(windowId, "error", message, fields);
  },
};
