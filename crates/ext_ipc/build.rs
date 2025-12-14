use deno_ast::{EmitOptions, MediaType, ParseParams, TranspileModuleOptions, TranspileOptions};
use std::env;
use std::fs;
use std::path::Path;

/// Transpile TypeScript to JavaScript using deno_ast
fn transpile_ts(ts_code: &str, specifier: &str) -> String {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: deno_ast::ModuleSpecifier::parse(specifier).unwrap(),
        text: ts_code.into(),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .expect("Failed to parse TypeScript");

    let transpile_result = parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &EmitOptions::default(),
        )
        .expect("Failed to transpile TypeScript");

    transpile_result.into_source().text
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-changed=ts/init.ts");

    let ts_path = Path::new("ts/init.ts");
    if ts_path.exists() {
        let ts_code = fs::read_to_string(ts_path).expect("Failed to read ts/init.ts");
        let js_code = transpile_ts(&ts_code, "file:///init.ts");

        // Generate complete extension! macro invocation
        let extension_rs = format!(
            r#"deno_core::extension!(
    host_ipc,
    ops = [
        op_ipc_send,
        op_ipc_recv,
    ],
    esm_entry_point = "ext:host_ipc/init.js",
    esm = ["ext:host_ipc/init.js" = {{ source = {:?} }}]
);"#,
            js_code
        );
        fs::write(out_path.join("extension.rs"), extension_rs).expect("Failed to write extension.rs");
    }

    // Go up to workspace root and then to sdk directory
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let sdk_dir = workspace_root.join("sdk");
    let generated_dir = sdk_dir.join("generated");

    // Create generated directory if it doesn't exist
    fs::create_dir_all(&generated_dir).ok();

    // Generate type definitions
    let types = generate_host_ipc_types();

    let dest_path = generated_dir.join("host.ipc.d.ts");
    fs::write(&dest_path, types).unwrap();

    // Also write to OUT_DIR for reference
    let out_types_path = Path::new(&out_dir).join("host.ipc.d.ts");
    fs::write(&out_types_path, generate_host_ipc_types()).unwrap();

    println!("cargo:rerun-if-changed=src/lib.rs");
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
