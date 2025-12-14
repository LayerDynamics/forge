// host:window module - Deno API for native window operations
// This is the single source of truth for the host:window SDK

// ============================================================================
// Type Definitions
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
  /** Platform type: "windows", "macos", "linux-x11", "linux-wayland", or "linux" (placeholder) */
  platform: string;
  /**
   * Raw handle value (HWND on Windows, NSView* on macOS, X11 window ID on Linux).
   * Note: On Linux without X11/Wayland detection, returns 0 as a placeholder.
   * Typed as number since Rust u64 serializes to JS number (safe for values up to 2^53).
   */
  handle: number;
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

/** File filter for open/save dialogs */
export interface FileFilter {
  /** Display name for the filter (e.g., "Images") */
  name: string;
  /** File extensions without dots (e.g., ["png", "jpg"]) */
  extensions: string[];
}

/** Options for file open/save dialog */
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

/** Window handle with methods for manipulating the window */
export interface Window {
  /** Unique window identifier */
  readonly id: string;
  // Lifecycle
  close(): Promise<boolean>;
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  unmaximize(): Promise<void>;
  restore(): Promise<void>;
  focus(): Promise<void>;
  // Position & Size
  getPosition(): Promise<Position>;
  setPosition(x: number, y: number): Promise<void>;
  getSize(): Promise<Size>;
  setSize(width: number, height: number): Promise<void>;
  // Title
  getTitle(): Promise<string>;
  setTitle(title: string): Promise<void>;
  // State
  isFullscreen(): Promise<boolean>;
  setFullscreen(fullscreen: boolean): Promise<void>;
  isFocused(): Promise<boolean>;
  isMaximized(): Promise<boolean>;
  isMinimized(): Promise<boolean>;
  isVisible(): Promise<boolean>;
  isResizable(): Promise<boolean>;
  hasDecorations(): Promise<boolean>;
  isAlwaysOnTop(): Promise<boolean>;
  // Configuration
  setResizable(resizable: boolean): Promise<void>;
  setDecorations(decorations: boolean): Promise<void>;
  setAlwaysOnTop(alwaysOnTop: boolean): Promise<void>;
  setVisible(visible: boolean): Promise<void>;
  show(): Promise<void>;
  hide(): Promise<void>;
  // Native
  getNativeHandle(): Promise<NativeHandle>;
  // Events
  events(): AsyncGenerator<WindowSystemEvent, void, unknown>;
}

/** Callback function for menu event handlers */
export type MenuCallback = (event: MenuEvent) => void;

// ============================================================================
// Deno Core Ops Declaration
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      // Window Lifecycle (10 ops)
      op_window_create(opts: WindowOptions): Promise<string>;
      op_window_close(windowId: string): Promise<boolean>;
      op_window_minimize(windowId: string): Promise<void>;
      op_window_maximize(windowId: string): Promise<void>;
      op_window_unmaximize(windowId: string): Promise<void>;
      op_window_restore(windowId: string): Promise<void>;
      op_window_set_fullscreen(windowId: string, fullscreen: boolean): Promise<void>;
      op_window_is_fullscreen(windowId: string): Promise<boolean>;
      op_window_focus(windowId: string): Promise<void>;
      op_window_is_focused(windowId: string): Promise<boolean>;
      // Window Properties (16 ops)
      op_window_get_position(windowId: string): Promise<Position>;
      op_window_set_position(windowId: string, x: number, y: number): Promise<void>;
      op_window_get_size(windowId: string): Promise<Size>;
      op_window_set_size(windowId: string, width: number, height: number): Promise<void>;
      op_window_get_title(windowId: string): Promise<string>;
      op_window_set_title(windowId: string, title: string): Promise<void>;
      op_window_set_resizable(windowId: string, resizable: boolean): Promise<void>;
      op_window_is_resizable(windowId: string): Promise<boolean>;
      op_window_set_decorations(windowId: string, decorations: boolean): Promise<void>;
      op_window_has_decorations(windowId: string): Promise<boolean>;
      op_window_set_always_on_top(windowId: string, alwaysOnTop: boolean): Promise<void>;
      op_window_is_always_on_top(windowId: string): Promise<boolean>;
      op_window_set_visible(windowId: string, visible: boolean): Promise<void>;
      op_window_is_visible(windowId: string): Promise<boolean>;
      op_window_is_maximized(windowId: string): Promise<boolean>;
      op_window_is_minimized(windowId: string): Promise<boolean>;
      // Dialogs (3 ops)
      op_window_dialog_open(opts: FileDialogOptions): Promise<string[] | null>;
      op_window_dialog_save(opts: FileDialogOptions): Promise<string | null>;
      op_window_dialog_message(opts: MessageDialogOptions): Promise<number>;
      // Menus (3 ops)
      op_window_set_app_menu(items: MenuItem[]): Promise<boolean>;
      op_window_show_context_menu(windowId: string | null, items: MenuItem[]): Promise<string>;
      op_window_menu_recv(): Promise<MenuEvent | null>;
      // Tray (3 ops)
      op_window_create_tray(opts: TrayOptions): Promise<string>;
      op_window_update_tray(trayId: string, opts: TrayOptions): Promise<boolean>;
      op_window_destroy_tray(trayId: string): Promise<boolean>;
      // Events & Native (2 ops)
      op_window_events_recv(): Promise<WindowSystemEvent | null>;
      op_window_get_native_handle(windowId: string): Promise<NativeHandle>;
    };
  };
};

const core = Deno.core;

// ============================================================================
// Window Functions
// ============================================================================

/**
 * Create a new window and return a Window handle.
 *
 * @param opts - Window options
 * @returns A Window handle for controlling the window
 *
 * @example
 * ```ts
 * import { createWindow } from "host:window";
 *
 * const win = await createWindow({
 *   url: "app://index.html",
 *   width: 1200,
 *   height: 800,
 *   title: "My App"
 * });
 *
 * // Later, manipulate the window
 * await win.setTitle("Updated Title");
 * const { width, height } = await win.getSize();
 * ```
 */
export async function createWindow(opts: WindowOptions = {}): Promise<Window> {
  const windowId = await core.ops.op_window_create(opts);

  const handle: Window = {
    id: windowId,

    // Lifecycle
    async close(): Promise<boolean> {
      return await core.ops.op_window_close(windowId);
    },
    async minimize(): Promise<void> {
      return await core.ops.op_window_minimize(windowId);
    },
    async maximize(): Promise<void> {
      return await core.ops.op_window_maximize(windowId);
    },
    async unmaximize(): Promise<void> {
      return await core.ops.op_window_unmaximize(windowId);
    },
    async restore(): Promise<void> {
      return await core.ops.op_window_restore(windowId);
    },
    async focus(): Promise<void> {
      return await core.ops.op_window_focus(windowId);
    },

    // Position & Size
    async getPosition(): Promise<Position> {
      return await core.ops.op_window_get_position(windowId);
    },
    async setPosition(x: number, y: number): Promise<void> {
      return await core.ops.op_window_set_position(windowId, x, y);
    },
    async getSize(): Promise<Size> {
      return await core.ops.op_window_get_size(windowId);
    },
    async setSize(width: number, height: number): Promise<void> {
      return await core.ops.op_window_set_size(windowId, width, height);
    },

    // Title
    async getTitle(): Promise<string> {
      return await core.ops.op_window_get_title(windowId);
    },
    async setTitle(title: string): Promise<void> {
      return await core.ops.op_window_set_title(windowId, title);
    },

    // State
    async isFullscreen(): Promise<boolean> {
      return await core.ops.op_window_is_fullscreen(windowId);
    },
    async setFullscreen(fullscreen: boolean): Promise<void> {
      return await core.ops.op_window_set_fullscreen(windowId, fullscreen);
    },
    async isFocused(): Promise<boolean> {
      return await core.ops.op_window_is_focused(windowId);
    },
    async isMaximized(): Promise<boolean> {
      return await core.ops.op_window_is_maximized(windowId);
    },
    async isMinimized(): Promise<boolean> {
      return await core.ops.op_window_is_minimized(windowId);
    },
    async isVisible(): Promise<boolean> {
      return await core.ops.op_window_is_visible(windowId);
    },
    async isResizable(): Promise<boolean> {
      return await core.ops.op_window_is_resizable(windowId);
    },
    async hasDecorations(): Promise<boolean> {
      return await core.ops.op_window_has_decorations(windowId);
    },
    async isAlwaysOnTop(): Promise<boolean> {
      return await core.ops.op_window_is_always_on_top(windowId);
    },

    // Configuration
    async setResizable(resizable: boolean): Promise<void> {
      return await core.ops.op_window_set_resizable(windowId, resizable);
    },
    async setDecorations(decorations: boolean): Promise<void> {
      return await core.ops.op_window_set_decorations(windowId, decorations);
    },
    async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
      return await core.ops.op_window_set_always_on_top(windowId, alwaysOnTop);
    },
    async setVisible(visible: boolean): Promise<void> {
      return await core.ops.op_window_set_visible(windowId, visible);
    },
    async show(): Promise<void> {
      return await core.ops.op_window_set_visible(windowId, true);
    },
    async hide(): Promise<void> {
      return await core.ops.op_window_set_visible(windowId, false);
    },

    // Native
    async getNativeHandle(): Promise<NativeHandle> {
      return await core.ops.op_window_get_native_handle(windowId);
    },

    // Events - filter for this window
    async *events(): AsyncGenerator<WindowSystemEvent, void, unknown> {
      for await (const event of windowEvents()) {
        if (event.windowId === windowId) {
          yield event;
        }
      }
    }
  };

  return handle;
}

/**
 * Close a window by ID.
 *
 * @param windowId - The window ID to close
 * @returns true if closed successfully
 *
 * @example
 * ```ts
 * import { closeWindow } from "host:window";
 *
 * await closeWindow("main-window");
 * ```
 */
export async function closeWindow(windowId: string): Promise<boolean> {
  return await core.ops.op_window_close(windowId);
}

/**
 * Receive the next window system event (blocking).
 * Returns null when no more events are available.
 *
 * @returns The next window event or null
 */
async function recvWindowEvent(): Promise<WindowSystemEvent | null> {
  return await core.ops.op_window_events_recv();
}

/**
 * Async generator for window system events from all windows.
 * Use in a for-await loop to process events as they arrive.
 *
 * @example
 * ```ts
 * import { windowEvents } from "host:window";
 *
 * for await (const event of windowEvents()) {
 *   console.log(`Window ${event.windowId}: ${event.type}`);
 *   if (event.type === "close") {
 *     console.log("Window closed");
 *   }
 * }
 * ```
 */
export async function* windowEvents(): AsyncGenerator<WindowSystemEvent, void, unknown> {
  while (true) {
    const event = await recvWindowEvent();
    if (event === null) break;
    yield event;
  }
}

// ============================================================================
// Dialog Namespace
// ============================================================================

/**
 * Dialog functions for file and message dialogs.
 *
 * @example
 * ```ts
 * import { dialog } from "host:window";
 *
 * // Open file dialog
 * const files = await dialog.open({
 *   title: "Select Images",
 *   filters: [{ name: "Images", extensions: ["png", "jpg"] }],
 *   multiple: true
 * });
 *
 * // Show confirmation
 * const confirmed = await dialog.confirm("Are you sure?");
 * ```
 */
export const dialog = {
  /**
   * Show an open file dialog.
   *
   * @param opts - Dialog options
   * @returns Array of selected file paths, or null if cancelled
   */
  async open(opts: FileDialogOptions = {}): Promise<string[] | null> {
    return await core.ops.op_window_dialog_open(opts);
  },

  /**
   * Show a save file dialog.
   *
   * @param opts - Dialog options
   * @returns Selected file path, or null if cancelled
   */
  async save(opts: FileDialogOptions = {}): Promise<string | null> {
    return await core.ops.op_window_dialog_save(opts);
  },

  /**
   * Show a message dialog.
   *
   * @param opts - Dialog options or message string
   * @returns Index of the clicked button
   */
  async message(opts: MessageDialogOptions | string): Promise<number> {
    const options: MessageDialogOptions = typeof opts === "string" ? { message: opts } : opts;
    return await core.ops.op_window_dialog_message(options);
  },

  /**
   * Show an alert dialog.
   *
   * @param message - Message to display
   * @param title - Dialog title
   * @returns Index of clicked button (always 0 for OK)
   */
  async alert(message: string, title: string = "Alert"): Promise<number> {
    return await core.ops.op_window_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["OK"]
    });
  },

  /**
   * Show a confirmation dialog.
   *
   * @param message - Message to display
   * @param title - Dialog title
   * @returns true if OK clicked, false if cancelled
   */
  async confirm(message: string, title: string = "Confirm"): Promise<boolean> {
    const result = await core.ops.op_window_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["Cancel", "OK"]
    });
    return result === 1;
  },

  /**
   * Show an error dialog.
   *
   * @param message - Error message to display
   * @param title - Dialog title
   */
  async error(message: string, title: string = "Error"): Promise<number> {
    return await core.ops.op_window_dialog_message({
      title,
      message,
      kind: "error",
      buttons: ["OK"]
    });
  },

  /**
   * Show a warning dialog.
   *
   * @param message - Warning message to display
   * @param title - Dialog title
   */
  async warning(message: string, title: string = "Warning"): Promise<number> {
    return await core.ops.op_window_dialog_message({
      title,
      message,
      kind: "warning",
      buttons: ["OK"]
    });
  }
};

// ============================================================================
// Menu Namespace
// ============================================================================

/**
 * Receive the next menu event (blocking).
 * Returns null when no more events are available.
 */
async function recvMenuEvent(): Promise<MenuEvent | null> {
  return await core.ops.op_window_menu_recv();
}

/**
 * Async generator for menu events.
 */
async function* menuEvents(): AsyncGenerator<MenuEvent, void, unknown> {
  while (true) {
    const event = await recvMenuEvent();
    if (event === null) break;
    yield event;
  }
}

let menuListenerActive = false;
const menuCallbacks: MenuCallback[] = [];

/**
 * Register a callback for menu events.
 *
 * @param callback - Function called for each menu event
 * @returns Unsubscribe function
 */
function onMenu(callback: MenuCallback): () => void {
  menuCallbacks.push(callback);

  if (!menuListenerActive) {
    menuListenerActive = true;
    (async () => {
      for await (const event of menuEvents()) {
        for (const cb of menuCallbacks) {
          try {
            cb(event);
          } catch (e) {
            console.error("Error in menu callback:", e);
          }
        }
      }
      menuListenerActive = false;
    })();
  }

  return () => {
    const index = menuCallbacks.indexOf(callback);
    if (index !== -1) {
      menuCallbacks.splice(index, 1);
    }
  };
}

/**
 * Menu functions for application and context menus.
 *
 * @example
 * ```ts
 * import { menu } from "host:window";
 *
 * // Set application menu
 * await menu.setAppMenu([
 *   {
 *     label: "File",
 *     submenu: [
 *       { id: "open", label: "Open", accelerator: "CmdOrCtrl+O" },
 *       { id: "save", label: "Save", accelerator: "CmdOrCtrl+S" },
 *       { type: "separator", label: "" },
 *       { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" }
 *     ]
 *   }
 * ]);
 *
 * // Listen for menu events
 * const unsub = menu.onMenu((event) => {
 *   console.log("Menu clicked:", event.itemId);
 * });
 * ```
 */
export const menu = {
  /**
   * Set the application menu bar.
   *
   * @param items - Menu items to display
   * @returns true if successful
   */
  async setAppMenu(items: MenuItem[]): Promise<boolean> {
    return await core.ops.op_window_set_app_menu(items);
  },

  /**
   * Show a context menu at the current cursor position.
   *
   * @param items - Menu items to display
   * @param windowId - Optional window to show menu in
   * @returns ID of clicked item, or null if cancelled
   */
  async showContextMenu(items: MenuItem[], windowId?: string): Promise<string | null> {
    const result = await core.ops.op_window_show_context_menu(windowId ?? null, items);
    return result === "" ? null : result;
  },

  /** Async generator for menu events */
  events: menuEvents,

  /** Register a callback for menu events */
  onMenu
};

// ============================================================================
// Tray Namespace
// ============================================================================

/**
 * Create a system tray icon.
 *
 * @param opts - Tray options
 * @returns TrayHandle for controlling the tray
 */
async function createTray(opts: TrayOptions = {}): Promise<TrayHandle> {
  const trayId = await core.ops.op_window_create_tray(opts);

  return {
    id: trayId,

    async update(newOpts: TrayOptions): Promise<boolean> {
      return await core.ops.op_window_update_tray(trayId, newOpts);
    },

    async destroy(): Promise<boolean> {
      return await core.ops.op_window_destroy_tray(trayId);
    }
  };
}

/**
 * Destroy a tray by ID.
 *
 * @param trayId - The tray ID to destroy
 * @returns true if destroyed successfully
 */
async function destroyTray(trayId: string): Promise<boolean> {
  return await core.ops.op_window_destroy_tray(trayId);
}

/**
 * System tray functions.
 *
 * @example
 * ```ts
 * import { tray } from "host:window";
 *
 * const myTray = await tray.create({
 *   icon: "assets/icon.png",
 *   tooltip: "My App",
 *   menu: [
 *     { id: "show", label: "Show Window" },
 *     { type: "separator", label: "" },
 *     { id: "quit", label: "Quit" }
 *   ]
 * });
 *
 * // Later, update or destroy
 * await myTray.update({ tooltip: "Updated!" });
 * await myTray.destroy();
 * ```
 */
export const tray = {
  /** Create a system tray icon */
  create: createTray,

  /** Destroy a tray by ID */
  destroy: destroyTray
};
