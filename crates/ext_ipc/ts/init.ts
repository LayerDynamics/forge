// host:ipc module - TypeScript wrapper for IPC operations
// This is the single source of truth for the host:ipc SDK

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
 * import { sendToWindow } from "host:ipc";
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
 * import { recvWindowEvent } from "host:ipc";
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
 * import { windowEvents } from "host:ipc";
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
 * import { windowEventsFor } from "host:ipc";
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
 * import { channelEvents } from "host:ipc";
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
 * import { onEvent } from "host:ipc";
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
 * import { onChannel } from "host:ipc";
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
 * import { broadcast } from "host:ipc";
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
