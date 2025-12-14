// host:ipc module - TypeScript wrapper for IPC operations

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
 * Event received from renderer (WebView) to Deno
 */
export interface IpcEvent {
  /** Window ID that sent the event */
  windowId: string;
  /** Channel name */
  channel: string;
  /** Event payload */
  payload: unknown;
  /** Event type for window system events: "close", "focus", "blur", "resize", "move" */
  type?: "close" | "focus" | "blur" | "resize" | "move";
}

const core = Deno.core;

/**
 * Send a message to a specific window's renderer
 * @param windowId - The ID of the window to send the message to
 * @param channel - The channel name for the message
 * @param payload - Optional payload to send
 */
export async function sendToWindow(windowId: string, channel: string, payload?: unknown): Promise<void> {
  return await core.ops.op_ipc_send(windowId, channel, payload);
}

/**
 * Receive the next event from any window
 * Returns null when no more events are available (channel closed)
 */
export async function recvWindowEvent(): Promise<IpcEvent | null> {
  return await core.ops.op_ipc_recv();
}

/**
 * Async generator that yields all window events
 * Use this in a for-await loop to process events as they arrive
 *
 * @example
 * ```ts
 * for await (const event of windowEvents()) {
 *   console.log(`Received from ${event.windowId}: ${event.channel}`, event.payload);
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
 * Filter events for a specific window
 * @param windowId - The window ID to filter for
 */
export async function* windowEventsFor(windowId: string): AsyncGenerator<IpcEvent, void, unknown> {
  for await (const event of windowEvents()) {
    if (event.windowId === windowId) {
      yield event;
    }
  }
}

/**
 * Filter events for a specific channel
 * @param channel - The channel name to filter for
 */
export async function* channelEvents(channel: string): AsyncGenerator<IpcEvent, void, unknown> {
  for await (const event of windowEvents()) {
    if (event.channel === channel) {
      yield event;
    }
  }
}
