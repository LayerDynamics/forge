use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Generate TypeScript type definitions for host:ui module
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

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
