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