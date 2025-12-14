use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_ipc", "host:ipc")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_ipc_send",
            "op_ipc_recv",
        ])
        .generate_sdk_types("sdk")
        .dts_generator(generate_host_ipc_types)
        .build()
        .expect("Failed to build host_ipc extension");
}

fn generate_host_ipc_types() -> String {
    r#"// Auto-generated TypeScript definitions for host:ipc module
// Generated from ext_ipc/build.rs - do not edit manually

declare module "host:ipc" {
  // ============================================================================
  // Event Types
  // ============================================================================

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

  /** Callback function for IPC event handlers */
  export type IpcEventCallback = (event: IpcEvent) => void;

  /** Callback function for channel-specific handlers */
  export type ChannelCallback = (payload: unknown, windowId: string) => void;

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
   */
  export function sendToWindow(
    windowId: string,
    channel: string,
    payload?: unknown
  ): Promise<void>;

  /**
   * Receive the next event from any window (blocking).
   * Returns null when no more events are available (channel closed).
   */
  export function recvWindowEvent(): Promise<IpcEvent | null>;

  // ============================================================================
  // Async Generator Functions
  // ============================================================================

  /**
   * Async generator that yields all window events.
   * Use this in a for-await loop to process events as they arrive.
   */
  export function windowEvents(): AsyncGenerator<IpcEvent, void, unknown>;

  /**
   * Filter events for a specific window.
   * @param windowId - The window ID to filter for
   */
  export function windowEventsFor(
    windowId: string
  ): AsyncGenerator<IpcEvent, void, unknown>;

  /**
   * Filter events for a specific channel.
   * @param channel - The channel name to filter for
   */
  export function channelEvents(
    channel: string
  ): AsyncGenerator<IpcEvent, void, unknown>;

  // ============================================================================
  // Callback-based Event Listeners
  // ============================================================================

  /**
   * Register a callback for all IPC events.
   * Returns an unsubscribe function.
   *
   * @param callback - Function called for each event
   * @returns Unsubscribe function
   */
  export function onEvent(callback: IpcEventCallback): () => void;

  /**
   * Register a callback for events on a specific channel.
   * Returns an unsubscribe function.
   *
   * @param channel - The channel name to listen for
   * @param callback - Function called with (payload, windowId) for each event
   * @returns Unsubscribe function
   */
  export function onChannel(channel: string, callback: ChannelCallback): () => void;

  // ============================================================================
  // Broadcast Functions
  // ============================================================================

  /**
   * Broadcast a message to multiple windows.
   *
   * @param windowIds - Array of window IDs to send to
   * @param channel - The channel name for the message
   * @param payload - Optional payload data to send
   */
  export function broadcast(
    windowIds: string[],
    channel: string,
    payload?: unknown
  ): Promise<void>;
}
"#.to_string()
}
