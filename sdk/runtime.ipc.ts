// runtime:ipc module - TypeScript wrapper for IPC operations
// This is the single source of truth for the runtime:ipc SDK

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_ipc_send(windowId: string, channel: string, payload: unknown): Promise<void>;
      op_ipc_recv(): Promise<IpcEvent | null>;
    };
  };
};

/**
 * Event received from renderer (WebView) to Deno.
 * These events are sent via `window.host.send()` in the renderer.
 */
export interface IpcEvent {
  /** Window ID that sent the event */
  windowId: string;
  /** Channel name for the event */
  channel: string;
  /** Event payload data */
  payload: unknown;
  /** Event type for window system events */
  type?: "close" | "focus" | "blur" | "resize" | "move";
}

/**
 * Callback function for IPC event handlers
 */
export type IpcEventCallback = (event: IpcEvent) => void;

/**
 * Callback function for channel-specific handlers
 */
export type ChannelCallback = (payload: unknown, windowId: string) => void;

const core = Deno.core;

// ============================================================================
// Core Functions
// ============================================================================

/**
 * Send a message to a specific window's renderer.
 * The message will be received by `window.host.on(channel, callback)` in the WebView.
 *
 * @param windowId - The unique ID of the window to send the message to
 * @param channel - The channel name for the message
 * @param payload - Optional payload data to send
 *
 * @example
 * ```ts
 * import { sendToWindow } from "runtime:ipc";
 *
 * // Send data to a specific window
 * await sendToWindow("main-window", "update", { count: 42 });
 *
 * // Send a simple notification
 * await sendToWindow("main-window", "refresh");
 * ```
 */
export async function sendToWindow(
  windowId: string,
  channel: string,
  payload?: unknown
): Promise<void> {
  return await core.ops.op_ipc_send(windowId, channel, payload ?? null);
}

/**
 * Receive the next event from any window (blocking).
 * Returns null when no more events are available (channel closed).
 *
 * @returns The next IPC event, or null if the channel is closed
 *
 * @example
 * ```ts
 * import { recvWindowEvent } from "runtime:ipc";
 *
 * const event = await recvWindowEvent();
 * if (event) {
 *   console.log(`Received: ${event.channel} from ${event.windowId}`);
 * }
 * ```
 */
export async function recvWindowEvent(): Promise<IpcEvent | null> {
  return await core.ops.op_ipc_recv();
}

// ============================================================================
// Async Generator Functions
// ============================================================================

/**
 * Async generator that yields all window events.
 * Use this in a for-await loop to process events as they arrive.
 *
 * @example
 * ```ts
 * import { windowEvents } from "runtime:ipc";
 *
 * for await (const event of windowEvents()) {
 *   console.log(`[${event.windowId}] ${event.channel}:`, event.payload);
 *
 *   if (event.type === "close") {
 *     console.log("Window closed");
 *     break;
 *   }
 * }
 * ```
 */
export async function* windowEvents(): AsyncGenerator<IpcEvent, void, unknown> {
  while (true) {
    const event = await recvWindowEvent();
    if (event === null) break;
    yield event;
  }
}

/**
 * Filter events for a specific window.
 *
 * @param windowId - The window ID to filter for
 *
 * @example
 * ```ts
 * import { windowEventsFor } from "runtime:ipc";
 *
 * // Only process events from the main window
 * for await (const event of windowEventsFor("main")) {
 *   console.log("Main window event:", event.channel);
 * }
 * ```
 */
export async function* windowEventsFor(
  windowId: string
): AsyncGenerator<IpcEvent, void, unknown> {
  for await (const event of windowEvents()) {
    if (event.windowId === windowId) {
      yield event;
    }
  }
}

/**
 * Filter events for a specific channel.
 *
 * @param channel - The channel name to filter for
 *
 * @example
 * ```ts
 * import { channelEvents } from "runtime:ipc";
 *
 * // Only process "button-click" events
 * for await (const event of channelEvents("button-click")) {
 *   console.log("Button clicked in window:", event.windowId);
 * }
 * ```
 */
export async function* channelEvents(
  channel: string
): AsyncGenerator<IpcEvent, void, unknown> {
  for await (const event of windowEvents()) {
    if (event.channel === channel) {
      yield event;
    }
  }
}

// ============================================================================
// Event Listener API (callback-based)
// ============================================================================

let listenerActive = false;
const eventCallbacks: IpcEventCallback[] = [];
const channelCallbacks: Map<string, ChannelCallback[]> = new Map();

/**
 * Register a callback for all IPC events.
 * Returns an unsubscribe function.
 *
 * @param callback - Function called for each event
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { onEvent } from "runtime:ipc";
 *
 * const unsubscribe = onEvent((event) => {
 *   console.log(`Event: ${event.channel} from ${event.windowId}`);
 * });
 *
 * // Later, to stop listening:
 * unsubscribe();
 * ```
 */
export function onEvent(callback: IpcEventCallback): () => void {
  eventCallbacks.push(callback);
  startEventLoop();

  return () => {
    const index = eventCallbacks.indexOf(callback);
    if (index !== -1) {
      eventCallbacks.splice(index, 1);
    }
  };
}

/**
 * Register a callback for events on a specific channel.
 * Returns an unsubscribe function.
 *
 * @param channel - The channel name to listen for
 * @param callback - Function called with (payload, windowId) for each event
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { onChannel } from "runtime:ipc";
 *
 * const unsubscribe = onChannel("user-action", (payload, windowId) => {
 *   console.log(`User action from ${windowId}:`, payload);
 * });
 *
 * // Later, to stop listening:
 * unsubscribe();
 * ```
 */
export function onChannel(channel: string, callback: ChannelCallback): () => void {
  if (!channelCallbacks.has(channel)) {
    channelCallbacks.set(channel, []);
  }
  channelCallbacks.get(channel)!.push(callback);
  startEventLoop();

  return () => {
    const callbacks = channelCallbacks.get(channel);
    if (callbacks) {
      const index = callbacks.indexOf(callback);
      if (index !== -1) {
        callbacks.splice(index, 1);
      }
    }
  };
}

/**
 * Start the internal event loop if not already running.
 * This is called automatically when registering callbacks.
 */
function startEventLoop(): void {
  if (listenerActive) return;

  listenerActive = true;
  (async () => {
    for await (const event of windowEvents()) {
      // Dispatch to global callbacks
      for (const cb of eventCallbacks) {
        try {
          cb(event);
        } catch (e) {
          console.error("Error in IPC event callback:", e);
        }
      }

      // Dispatch to channel-specific callbacks
      const callbacks = channelCallbacks.get(event.channel);
      if (callbacks) {
        for (const cb of callbacks) {
          try {
            cb(event.payload, event.windowId);
          } catch (e) {
            console.error(`Error in IPC channel callback (${event.channel}):`, e);
          }
        }
      }
    }
    listenerActive = false;
  })();
}

// ============================================================================
// Broadcast API
// ============================================================================

/**
 * Broadcast a message to multiple windows.
 *
 * @param windowIds - Array of window IDs to send to
 * @param channel - The channel name for the message
 * @param payload - Optional payload data to send
 *
 * @example
 * ```ts
 * import { broadcast } from "runtime:ipc";
 *
 * // Send to multiple windows
 * await broadcast(["main", "settings", "preview"], "theme-changed", { theme: "dark" });
 * ```
 */
export async function broadcast(
  windowIds: string[],
  channel: string,
  payload?: unknown
): Promise<void> {
  await Promise.all(
    windowIds.map((windowId) => sendToWindow(windowId, channel, payload))
  );
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  send: { args: []; result: void };
  recv: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "send" | "recv";

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

