// host:ui module - Deno API for window management
// This is the single source of truth for the host:ui SDK

// Type definitions
export interface OpenWindowOptions {
  /** URL to load (default: "app://index.html") */
  url?: string;
  /** Window width in pixels */
  width?: number;
  /** Window height in pixels */
  height?: number;
  /** Window title */
  title?: string;
  /** Whether the window is resizable */
  resizable?: boolean;
  /** Whether to show window decorations */
  decorations?: boolean;
  /**
   * Channel allowlist for this window - only these channels can be used for IPC.
   * If not specified, uses the default from manifest.app.toml.
   * Use ["*"] to allow all channels, or [] to deny all channels.
   */
  channels?: string[];
}

export interface WindowEvent {
  /** The window that emitted the event */
  windowId: string;
  /** Event channel name */
  channel: string;
  /** Event payload data */
  payload: unknown;
  /** Event type for system window events */
  type?: "close" | "focus" | "blur" | "resize" | "move";
}

/** Menu event emitted when a menu item is clicked */
export interface MenuEvent {
  /** Source menu type: "app", "context", or "tray" */
  menuId: string;
  /** The menu item's id (from MenuItem.id) */
  itemId: string;
  /** The menu item's label */
  label: string;
}

export interface WindowHandle {
  /** Unique window identifier */
  id: string;
  /** Send a message to this window's renderer */
  send(channel: string, payload?: unknown): Promise<void>;
  /** Alias for send() - emit a message to this window's renderer */
  emit(channel: string, payload?: unknown): Promise<void>;
  /** Set the window title */
  setTitle(title: string): Promise<void>;
  /** Close this window */
  close(): Promise<boolean>;
  /** Async iterator for events from this window only */
  events(): AsyncGenerator<WindowEvent, void, unknown>;
}

export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface FileDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  multiple?: boolean;
  directory?: boolean;
}

export interface MessageDialogOptions {
  title?: string;
  message: string;
  kind?: "info" | "warning" | "error";
  buttons?: string[];
}

export interface MenuItem {
  /** Unique identifier for this menu item (used in events) */
  id?: string;
  /** Display text for the menu item */
  label: string;
  /** Keyboard accelerator (e.g., "CmdOrCtrl+S") */
  accelerator?: string;
  /** Whether the item is enabled (default: true) */
  enabled?: boolean;
  /** For checkbox items, whether it's checked */
  checked?: boolean;
  /** Nested menu items for submenus */
  submenu?: MenuItem[];
  /** Item type: "normal", "checkbox", or "separator" */
  type?: "normal" | "checkbox" | "separator";
}

export interface TrayOptions {
  /** Path to the tray icon file */
  icon?: string;
  /** Tooltip text shown on hover */
  tooltip?: string;
  /** Menu items to show when tray is clicked/right-clicked */
  menu?: MenuItem[];
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_ui_open_window(opts: OpenWindowOptions): Promise<string>;
      op_ui_close_window(windowId: string): Promise<boolean>;
      op_ui_set_window_title(windowId: string, title: string): Promise<void>;
      op_ui_window_send(windowId: string, channel: string, payload: unknown): Promise<void>;
      op_ui_window_recv(): Promise<WindowEvent | null>;
      op_ui_dialog_open(opts: FileDialogOptions): Promise<string[] | null>;
      op_ui_dialog_save(opts: FileDialogOptions): Promise<string | null>;
      op_ui_dialog_message(opts: MessageDialogOptions): Promise<number>;
      // Menu ops
      op_ui_set_app_menu(items: MenuItem[]): Promise<boolean>;
      op_ui_show_context_menu(windowId: string | null, items: MenuItem[]): Promise<string | null>;
      // Tray ops
      op_ui_create_tray(opts: TrayOptions): Promise<string>;
      op_ui_update_tray(trayId: string, opts: TrayOptions): Promise<boolean>;
      op_ui_destroy_tray(trayId: string): Promise<boolean>;
      // Menu event ops
      op_ui_menu_recv(): Promise<MenuEvent | null>;
    };
  };
};

/** Open a new window and return a handle for interacting with it */
export async function openWindow(opts?: OpenWindowOptions): Promise<WindowHandle> {
  const windowId = await Deno.core.ops.op_ui_open_window(opts || {});

  const handle: WindowHandle = {
    id: windowId,

    async send(channel: string, payload?: unknown): Promise<void> {
      return await Deno.core.ops.op_ui_window_send(windowId, channel, payload);
    },

    // emit is an alias for send (per SPEC: win.emit("ready", {}))
    async emit(channel: string, payload?: unknown): Promise<void> {
      return await Deno.core.ops.op_ui_window_send(windowId, channel, payload);
    },

    async setTitle(title: string): Promise<void> {
      return await Deno.core.ops.op_ui_set_window_title(windowId, title);
    },

    async close(): Promise<boolean> {
      return await Deno.core.ops.op_ui_close_window(windowId);
    },

    async *events(): AsyncGenerator<WindowEvent, void, unknown> {
      while (true) {
        const event = await recvWindowEvent();
        if (event === null) break;
        if (event.windowId === windowId) {
          yield event;
        }
      }
    }
  };

  return handle;
}

/** Close a window by its ID */
export async function closeWindow(windowId: string): Promise<boolean> {
  return await Deno.core.ops.op_ui_close_window(windowId);
}

/** Set the title of a window */
export async function setWindowTitle(windowId: string, title: string): Promise<void> {
  return await Deno.core.ops.op_ui_set_window_title(windowId, title);
}

/** Send a message to a specific window's renderer */
export async function sendToWindow(windowId: string, channel: string, payload?: unknown): Promise<void> {
  return await Deno.core.ops.op_ui_window_send(windowId, channel, payload);
}

/** Receive the next event from any window (blocking) */
export async function recvWindowEvent(): Promise<WindowEvent | null> {
  return await Deno.core.ops.op_ui_window_recv();
}

/** Async iterator for window events from all windows */
export async function* windowEvents(): AsyncGenerator<WindowEvent, void, unknown> {
  while (true) {
    const event = await recvWindowEvent();
    if (event === null) break;
    yield event;
  }
}

/** Dialog APIs for file and message dialogs */
export const dialog = {
  /** Show an open file/folder dialog */
  async open(opts: FileDialogOptions = {}): Promise<string[] | null> {
    return await Deno.core.ops.op_ui_dialog_open(opts);
  },

  /** Show a save file dialog */
  async save(opts: FileDialogOptions = {}): Promise<string | null> {
    return await Deno.core.ops.op_ui_dialog_save(opts);
  },

  /** Show a message dialog with custom buttons */
  async message(opts: MessageDialogOptions | string): Promise<number> {
    if (typeof opts === "string") {
      opts = { message: opts };
    }
    return await Deno.core.ops.op_ui_dialog_message(opts);
  },

  /** Show an alert dialog with OK button */
  async alert(message: string, title = "Alert"): Promise<number> {
    return await Deno.core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["OK"]
    });
  },

  /** Show a confirmation dialog, returns true if OK was clicked */
  async confirm(message: string, title = "Confirm"): Promise<boolean> {
    const result = await Deno.core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["Cancel", "OK"]
    });
    return result === 1;
  },

  /** Show an error dialog */
  async error(message: string, title = "Error"): Promise<number> {
    return await Deno.core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "error",
      buttons: ["OK"]
    });
  },

  /** Show a warning dialog */
  async warning(message: string, title = "Warning"): Promise<number> {
    return await Deno.core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "warning",
      buttons: ["OK"]
    });
  }
};

// Shorthand exports for common dialog operations
export const showOpenDialog = dialog.open;
export const showSaveDialog = dialog.save;
export const showMessageDialog = dialog.message;

// ============================================================================
// Menu APIs
// ============================================================================

/**
 * Set the application menu bar (shown at top of screen on macOS, in window on other platforms)
 * @param items Array of menu items defining the menu structure
 * @returns true if menu was set successfully
 */
export async function setAppMenu(items: MenuItem[]): Promise<boolean> {
  return await Deno.core.ops.op_ui_set_app_menu(items);
}

/**
 * Show a context menu at the current cursor position
 * @param items Array of menu items
 * @param windowId Optional window ID to associate the menu with
 * @returns The id of the selected menu item, or empty string if cancelled
 */
export async function showContextMenu(items: MenuItem[], windowId?: string): Promise<string | null> {
  const result = await Deno.core.ops.op_ui_show_context_menu(windowId ?? null, items);
  return result === "" ? null : result;
}

/**
 * Receive the next menu event (blocking)
 * @returns The next menu event or null if no more events
 */
export async function recvMenuEvent(): Promise<MenuEvent | null> {
  return await Deno.core.ops.op_ui_menu_recv();
}

/**
 * Async iterator for menu events from app menu, context menus, and tray menus
 */
export async function* menuEvents(): AsyncGenerator<MenuEvent, void, unknown> {
  while (true) {
    const event = await recvMenuEvent();
    if (event === null) break;
    yield event;
  }
}

// Track active menu listeners for cleanup
let menuListenerActive = false;
const menuCallbacks: Array<(event: MenuEvent) => void> = [];

/**
 * Register a callback for menu events
 * @param callback Function to call when a menu item is clicked
 * @returns Unsubscribe function to stop receiving events
 */
export function onMenu(callback: (event: MenuEvent) => void): () => void {
  menuCallbacks.push(callback);

  // Start the listener loop if not already running
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

  // Return unsubscribe function
  return () => {
    const index = menuCallbacks.indexOf(callback);
    if (index !== -1) {
      menuCallbacks.splice(index, 1);
    }
  };
}

/** Menu APIs namespace */
export const menu = {
  setAppMenu,
  showContextMenu,
  events: menuEvents,
  onMenu,
};

// ============================================================================
// Tray APIs
// ============================================================================

/** Handle for interacting with a system tray icon */
export interface TrayHandle {
  /** Unique tray identifier */
  id: string;
  /** Update the tray icon, tooltip, or menu */
  update(opts: TrayOptions): Promise<boolean>;
  /** Destroy this tray icon */
  destroy(): Promise<boolean>;
}

/**
 * Create a system tray icon
 * @param opts Tray options (icon, tooltip, menu)
 * @returns A handle for interacting with the tray
 */
export async function createTray(opts: TrayOptions = {}): Promise<TrayHandle> {
  const trayId = await Deno.core.ops.op_ui_create_tray(opts);

  return {
    id: trayId,

    async update(newOpts: TrayOptions): Promise<boolean> {
      return await Deno.core.ops.op_ui_update_tray(trayId, newOpts);
    },

    async destroy(): Promise<boolean> {
      return await Deno.core.ops.op_ui_destroy_tray(trayId);
    }
  };
}

/**
 * Destroy a system tray icon by ID
 * @param trayId The tray ID to destroy
 */
export async function destroyTray(trayId: string): Promise<boolean> {
  return await Deno.core.ops.op_ui_destroy_tray(trayId);
}

/** Tray APIs namespace */
export const tray = {
  create: createTray,
  destroy: destroyTray,
};
