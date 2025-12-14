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
    // Generate TypeScript type definitions for host:ui module
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Transpile ts/init.ts and generate extension.rs
    println!("cargo:rerun-if-changed=ts/init.ts");

    let ts_path = Path::new("ts/init.ts");
    if ts_path.exists() {
        let ts_code = fs::read_to_string(ts_path).expect("Failed to read ts/init.ts");
        let js_code = transpile_ts(&ts_code, "file:///init.ts");

        // Generate complete extension! macro invocation
        // Note: IPC ops (op_ui_window_send, op_ui_window_recv) have moved to ext_ipc
        let extension_rs = format!(
            r#"deno_core::extension!(
    host_ui,
    ops = [
        op_ui_open_window,
        op_ui_close_window,
        op_ui_set_window_title,
        op_ui_dialog_open,
        op_ui_dialog_save,
        op_ui_dialog_message,
        op_ui_set_app_menu,
        op_ui_show_context_menu,
        op_ui_menu_recv,
        op_ui_create_tray,
        op_ui_update_tray,
        op_ui_destroy_tray,
    ],
    esm_entry_point = "ext:host_ui/init.js",
    esm = ["ext:host_ui/init.js" = {{ source = {:?} }}]
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
    let types = generate_host_ui_types();

    let dest_path = generated_dir.join("host.ui.d.ts");
    fs::write(&dest_path, types).unwrap();

    // Also write to OUT_DIR for reference
    let out_path = Path::new(&out_dir).join("host.ui.d.ts");
    fs::write(&out_path, generate_host_ui_types()).unwrap();

    println!("cargo:rerun-if-changed=src/lib.rs");
}

fn generate_host_ui_types() -> String {
    r#"// Auto-generated TypeScript definitions for host:ui module
// Generated from ext_ui/build.rs - do not edit manually

declare module "host:ui" {
  // ============================================================================
  // Window Types
  // ============================================================================

  /** Options for opening a new window */
  export interface OpenWindowOptions {
    /** URL to load (default: "app://index.html") */
    url?: string;
    /** Window width in pixels */
    width?: number;
    /** Window height in pixels */
    height?: number;
    /** Window title */
    title?: string;
    /** Whether window is resizable (default: true) */
    resizable?: boolean;
    /** Whether window has decorations (default: true) */
    decorations?: boolean;
    /** Channel allowlist for IPC - only these channels can be used */
    channels?: string[];
  }

  /** Event received from a window renderer */
  export interface WindowEvent {
    /** The window that emitted the event */
    windowId: string;
    /** Event channel name */
    channel: string;
    /** Event payload data */
    payload: unknown;
    /** Window system event type (for close, focus, blur, resize, move events) */
    type?: "close" | "focus" | "blur" | "resize" | "move";
  }

  /** Window handle returned from openWindow */
  export interface Window {
    /** Unique window identifier */
    readonly id: string;
    /** Send a message to this window's renderer */
    send(channel: string, payload?: unknown): Promise<void>;
    /** Emit an event to both local listeners and this window */
    emit(channel: string, payload?: unknown): Promise<void>;
    /** Async iterator for events from this window */
    events(): AsyncGenerator<WindowEvent, void, unknown>;
    /** Listen for events on a specific channel */
    on(channel: string, callback: (payload: unknown) => void): () => void;
    /** Close this window */
    close(): Promise<boolean>;
    /** Set this window's title */
    setTitle(title: string): Promise<void>;
  }

  // ============================================================================
  // Dialog Types
  // ============================================================================

  /** File filter for open/save dialogs */
  export interface FileFilter {
    /** Display name for the filter (e.g., "Images") */
    name: string;
    /** File extensions without dots (e.g., ["png", "jpg"]) */
    extensions: string[];
  }

  /** Options for file open dialog */
  export interface FileDialogOptions {
    /** Dialog title */
    title?: string;
    /** Default starting path */
    defaultPath?: string;
    /** File type filters */
    filters?: FileFilter[];
    /** Allow selecting multiple files */
    multiple?: boolean;
    /** Select directories instead of files */
    directory?: boolean;
  }

  /** Options for message dialog */
  export interface MessageDialogOptions {
    /** Dialog title */
    title?: string;
    /** Message to display */
    message: string;
    /** Dialog kind: "info", "warning", or "error" */
    kind?: "info" | "warning" | "error";
    /** Custom button labels */
    buttons?: string[];
  }

  // ============================================================================
  // Menu Types
  // ============================================================================

  /** Menu item definition */
  export interface MenuItem {
    /** Unique identifier for this menu item */
    id?: string;
    /** Display label */
    label: string;
    /** Keyboard accelerator (e.g., "CmdOrCtrl+S") */
    accelerator?: string;
    /** Whether the item is enabled (default: true) */
    enabled?: boolean;
    /** Whether the item is checked (for checkbox items) */
    checked?: boolean;
    /** Submenu items */
    submenu?: MenuItem[];
    /** Item type: "normal", "checkbox", or "separator" */
    type?: "normal" | "checkbox" | "separator";
  }

  /** Event emitted when a menu item is clicked */
  export interface MenuEvent {
    /** Source of the event: "app", "context", or "tray" */
    menuId: string;
    /** The id of the clicked menu item */
    itemId: string;
    /** The label of the clicked menu item */
    label: string;
  }

  // ============================================================================
  // Tray Types
  // ============================================================================

  /** Options for creating a system tray icon */
  export interface TrayOptions {
    /** Path to icon file */
    icon?: string;
    /** Tooltip text shown on hover */
    tooltip?: string;
    /** Context menu for the tray icon */
    menu?: MenuItem[];
  }

  // ============================================================================
  // Window Functions
  // ============================================================================

  /** Open a new window and return a Window handle */
  export function openWindow(options?: OpenWindowOptions): Promise<Window>;

  /** Send a message to a specific window's renderer */
  export function sendToWindow(windowId: string, channel: string, payload?: unknown): Promise<void>;

  /** Async iterator for events from all windows */
  export function windowEvents(): AsyncGenerator<WindowEvent, void, unknown>;

  // ============================================================================
  // Dialog Functions
  // ============================================================================

  /** Show an open file dialog. Returns null if cancelled. */
  export function showOpenDialog(options?: FileDialogOptions): Promise<string[] | null>;

  /** Show a save file dialog. Returns null if cancelled. */
  export function showSaveDialog(options?: FileDialogOptions): Promise<string | null>;

  /** Show a message dialog. Returns the index of the clicked button. */
  export function showMessageDialog(options: MessageDialogOptions): Promise<number>;

  // ============================================================================
  // Menu Functions
  // ============================================================================

  /** Set the application menu bar */
  export function setAppMenu(items: MenuItem[]): Promise<boolean>;

  /** Show a context menu at the current cursor position. Returns clicked item ID or empty string. */
  export function showContextMenu(windowId: string | null, items: MenuItem[]): Promise<string>;

  /** Async iterator for menu events */
  export function menuEvents(): AsyncGenerator<MenuEvent, void, unknown>;

  /** Register a callback for menu events. Returns unsubscribe function. */
  export function onMenu(callback: (event: MenuEvent) => void): () => void;

  // ============================================================================
  // Tray Functions
  // ============================================================================

  /** Create a system tray icon. Returns the tray ID. */
  export function createTray(options?: TrayOptions): Promise<string>;

  /** Update an existing system tray icon */
  export function updateTray(trayId: string, options: TrayOptions): Promise<boolean>;

  /** Destroy a system tray icon */
  export function destroyTray(trayId: string): Promise<boolean>;
}
"#.to_string()
}
