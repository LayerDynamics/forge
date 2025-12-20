use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_window", "runtime:window")
        .ts_path("ts/init.ts")
        .ops(&[
            // Window Lifecycle (10 ops)
            "op_window_create",
            "op_window_close",
            "op_window_minimize",
            "op_window_maximize",
            "op_window_unmaximize",
            "op_window_restore",
            "op_window_set_fullscreen",
            "op_window_is_fullscreen",
            "op_window_focus",
            "op_window_is_focused",
            // Window Properties (16 ops)
            "op_window_get_position",
            "op_window_set_position",
            "op_window_get_size",
            "op_window_set_size",
            "op_window_get_title",
            "op_window_set_title",
            "op_window_set_resizable",
            "op_window_is_resizable",
            "op_window_set_decorations",
            "op_window_has_decorations",
            "op_window_set_always_on_top",
            "op_window_is_always_on_top",
            "op_window_set_visible",
            "op_window_is_visible",
            "op_window_is_maximized",
            "op_window_is_minimized",
            // Dialogs (3 ops)
            "op_window_dialog_open",
            "op_window_dialog_save",
            "op_window_dialog_message",
            // Menus (3 ops)
            "op_window_set_app_menu",
            "op_window_show_context_menu",
            "op_window_menu_recv",
            // Tray (3 ops)
            "op_window_create_tray",
            "op_window_update_tray",
            "op_window_destroy_tray",
            // Events & Native (2 ops)
            "op_window_events_recv",
            "op_window_get_native_handle",
            // Enhanced Window Ops (9 ops)
            "op_window_open_devtools",
            "op_window_close_devtools",
            "op_window_is_devtools_open",
            "op_window_eval_js",
            "op_window_inject_css",
            "op_window_set_min_size",
            "op_window_set_max_size",
            "op_window_center",
            "op_window_get_monitors",
        ])
        .generate_sdk_types("sdk")
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .dts_generator(generate_host_window_types)
        .enable_extensibility()
        .build()
        .expect("Failed to build host_window extension");
}

fn generate_host_window_types() -> String {
    r#"// Auto-generated TypeScript definitions for runtime:window module
// Generated from ext_window/build.rs - do not edit manually

declare module "runtime:window" {
  // ============================================================================
  // Window Types
  // ============================================================================

  /** Options for creating a new window */
  export interface WindowOptions {
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
    /** Whether window is visible on creation (default: true) */
    visible?: boolean;
    /** Whether window is transparent */
    transparent?: boolean;
    /** Whether window is always on top */
    alwaysOnTop?: boolean;
    /** Initial X position */
    x?: number;
    /** Initial Y position */
    y?: number;
    /** Minimum window width */
    minWidth?: number;
    /** Minimum window height */
    minHeight?: number;
    /** Maximum window width */
    maxWidth?: number;
    /** Maximum window height */
    maxHeight?: number;
    /** Channel allowlist for IPC - only these channels can be used */
    channels?: string[];
  }

  /** Window position */
  export interface Position {
    x: number;
    y: number;
  }

  /** Window size */
  export interface Size {
    width: number;
    height: number;
  }

  /** Native window handle (platform-specific) */
  export interface NativeHandle {
    /** Platform type: "windows", "macos", "linux-x11", "linux-wayland" */
    platform: string;
    /** Raw handle value (HWND, NSView*, X11 window, etc.) */
    handle: bigint;
  }

  /** Window system event */
  export interface WindowSystemEvent {
    /** The window that emitted the event */
    windowId: string;
    /** Event type */
    type: "close" | "focus" | "blur" | "resize" | "move" | "minimize" | "maximize" | "restore";
    /** Event payload (e.g., size for resize, position for move) */
    payload: unknown;
  }

  /** Window handle with methods for manipulating the window */
  export interface Window {
    /** Unique window identifier */
    readonly id: string;

    // Lifecycle
    /** Close this window */
    close(): Promise<boolean>;
    /** Minimize this window */
    minimize(): Promise<void>;
    /** Maximize this window */
    maximize(): Promise<void>;
    /** Restore from maximized state */
    unmaximize(): Promise<void>;
    /** Restore from minimized state */
    restore(): Promise<void>;
    /** Focus this window */
    focus(): Promise<void>;

    // Position & Size
    /** Get window position */
    getPosition(): Promise<Position>;
    /** Set window position */
    setPosition(x: number, y: number): Promise<void>;
    /** Get window size */
    getSize(): Promise<Size>;
    /** Set window size */
    setSize(width: number, height: number): Promise<void>;

    // Title
    /** Get window title */
    getTitle(): Promise<string>;
    /** Set window title */
    setTitle(title: string): Promise<void>;

    // State queries
    /** Check if fullscreen */
    isFullscreen(): Promise<boolean>;
    /** Set fullscreen mode */
    setFullscreen(fullscreen: boolean): Promise<void>;
    /** Check if focused */
    isFocused(): Promise<boolean>;
    /** Check if maximized */
    isMaximized(): Promise<boolean>;
    /** Check if minimized */
    isMinimized(): Promise<boolean>;
    /** Check if visible */
    isVisible(): Promise<boolean>;
    /** Check if resizable */
    isResizable(): Promise<boolean>;
    /** Check if has decorations */
    hasDecorations(): Promise<boolean>;
    /** Check if always on top */
    isAlwaysOnTop(): Promise<boolean>;

    // Configuration
    /** Set whether window is resizable */
    setResizable(resizable: boolean): Promise<void>;
    /** Set whether window has decorations */
    setDecorations(decorations: boolean): Promise<void>;
    /** Set whether window is always on top */
    setAlwaysOnTop(alwaysOnTop: boolean): Promise<void>;
    /** Set window visibility */
    setVisible(visible: boolean): Promise<void>;
    /** Show window (alias for setVisible(true)) */
    show(): Promise<void>;
    /** Hide window (alias for setVisible(false)) */
    hide(): Promise<void>;

    // Native
    /** Get native window handle */
    getNativeHandle(): Promise<NativeHandle>;

    // Events
    /** Async iterator for events from this window */
    events(): AsyncGenerator<WindowSystemEvent, void, unknown>;
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

  /** Tray handle with methods for manipulating the tray */
  export interface TrayHandle {
    /** Unique tray identifier */
    readonly id: string;
    /** Update tray options */
    update(options: TrayOptions): Promise<boolean>;
    /** Destroy this tray */
    destroy(): Promise<boolean>;
  }

  // ============================================================================
  // Window Functions
  // ============================================================================

  /** Create a new window and return a Window handle */
  export function createWindow(options?: WindowOptions): Promise<Window>;

  /** Close a window by ID */
  export function closeWindow(windowId: string): Promise<boolean>;

  /** Async iterator for window system events from all windows */
  export function windowEvents(): AsyncGenerator<WindowSystemEvent, void, unknown>;

  // ============================================================================
  // Dialog Namespace
  // ============================================================================

  export const dialog: {
    /** Show an open file dialog. Returns null if cancelled. */
    open(options?: FileDialogOptions): Promise<string[] | null>;
    /** Show a save file dialog. Returns null if cancelled. */
    save(options?: FileDialogOptions): Promise<string | null>;
    /** Show a message dialog. Returns the index of the clicked button. */
    message(options: MessageDialogOptions | string): Promise<number>;
    /** Show an alert dialog (convenience wrapper) */
    alert(message: string, title?: string): Promise<number>;
    /** Show a confirm dialog (convenience wrapper) */
    confirm(message: string, title?: string): Promise<boolean>;
    /** Show an error dialog (convenience wrapper) */
    error(message: string, title?: string): Promise<number>;
    /** Show a warning dialog (convenience wrapper) */
    warning(message: string, title?: string): Promise<number>;
  };

  // ============================================================================
  // Menu Namespace
  // ============================================================================

  export const menu: {
    /** Set the application menu bar */
    setAppMenu(items: MenuItem[]): Promise<boolean>;
    /** Show a context menu at the current cursor position */
    showContextMenu(items: MenuItem[], windowId?: string): Promise<string | null>;
    /** Async iterator for menu events */
    events(): AsyncGenerator<MenuEvent, void, unknown>;
    /** Register a callback for menu events. Returns unsubscribe function. */
    onMenu(callback: (event: MenuEvent) => void): () => void;
  };

  // ============================================================================
  // Tray Namespace
  // ============================================================================

  export const tray: {
    /** Create a system tray icon */
    create(options?: TrayOptions): Promise<TrayHandle>;
    /** Destroy a tray by ID */
    destroy(trayId: string): Promise<boolean>;
  };
}
"#
    .to_string()
}
