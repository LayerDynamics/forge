// Timer extension for Deno runtime
// Provides setTimeout, setInterval, clearTimeout, clearInterval

// deno-lint-ignore no-explicit-any
const core = (Deno as any).core;

export interface TimerResult {
  id: number;
}

export interface TimerCallback {
  callback: (...args: unknown[]) => void;
  args: unknown[];
  repeat: boolean;
  delay: number;
}

// Map of timer IDs to callbacks
const timerCallbacks = new Map<number, TimerCallback>();

// Active interval timers that need to keep running
const activeIntervals = new Set<number>();

/**
 * Set a timeout - executes callback after delay
 */
function setTimeout(callback: (...args: unknown[]) => void, delay?: number, ...args: unknown[]): number {
  const delayMs = Math.max(0, delay ?? 0);

  // Create timer in Rust
  const result: TimerResult = core.ops.op_host_timer_create({
    delay_ms: delayMs,
    repeat: false,
  });

  const timerId = result.id;

  // Store callback info
  timerCallbacks.set(timerId, {
    callback,
    args,
    repeat: false,
    delay: delayMs,
  });

  // Start async sleep and execute callback when done
  runTimer(timerId, delayMs, false);

  return timerId;
}

/**
 * Set an interval - executes callback repeatedly
 */
function setInterval(callback: (...args: unknown[]) => void, delay?: number, ...args: unknown[]): number {
  const delayMs = Math.max(0, delay ?? 0);

  // Create timer in Rust
  const result: TimerResult = core.ops.op_host_timer_create({
    delay_ms: delayMs,
    repeat: true,
  });

  const timerId = result.id;

  // Store callback info
  timerCallbacks.set(timerId, {
    callback,
    args,
    repeat: true,
    delay: delayMs,
  });

  // Mark as active interval
  activeIntervals.add(timerId);

  // Start interval loop
  runTimer(timerId, delayMs, true);

  return timerId;
}

/**
 * Clear a timeout
 */
function clearTimeout(timerId: number): void {
  if (timerId === undefined || timerId === null) return;

  // Remove from our tracking
  timerCallbacks.delete(timerId);
  activeIntervals.delete(timerId);

  // Cancel in Rust
  core.ops.op_host_timer_cancel(timerId);
}

/**
 * Clear an interval
 */
function clearInterval(timerId: number): void {
  // Same implementation as clearTimeout
  clearTimeout(timerId);
}

/**
 * Run a timer (async)
 */
async function runTimer(timerId: number, delay: number, repeat: boolean): Promise<void> {
  while (true) {
    // Wait for the delay
    const completed = await core.ops.op_host_timer_sleep(timerId, delay);

    if (!completed) {
      // Timer was cancelled
      return;
    }

    // Get callback info
    const info = timerCallbacks.get(timerId);
    if (!info) {
      // Timer was cleared
      return;
    }

    // Execute callback
    try {
      info.callback(...info.args);
    } catch (e) {
      console.error("Timer callback error:", e);
    }

    if (!repeat) {
      // One-shot timer, clean up
      timerCallbacks.delete(timerId);
      core.ops.op_host_timer_cancel(timerId);
      return;
    }

    // Check if interval is still active
    if (!activeIntervals.has(timerId)) {
      return;
    }

    // Continue loop for interval
  }
}

// Install globals
// deno-lint-ignore no-explicit-any
(globalThis as any).setTimeout = setTimeout;
// deno-lint-ignore no-explicit-any
(globalThis as any).clearTimeout = clearTimeout;
// deno-lint-ignore no-explicit-any
(globalThis as any).setInterval = setInterval;
// deno-lint-ignore no-explicit-any
(globalThis as any).clearInterval = clearInterval;

export { setTimeout, clearTimeout, setInterval, clearInterval };


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  timerCreate: { args: []; result: void };
  timerCancel: { args: []; result: void };
  timerSleep: { args: []; result: void };
  timerExists: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "timerCreate" | "timerCancel" | "timerSleep" | "timerExists";

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

