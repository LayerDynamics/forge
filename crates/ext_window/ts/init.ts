// host:window module - TypeScript wrapper for native window operations

// Deno.core type declaration with all 37 ops
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

// ============================================================================
// Type Definitions
// ============================================================================

interface WindowOptions {
  url?: string;
  width?: number;
  height?: number;
  title?: string;
  resizable?: boolean;
  decorations?: boolean;
  visible?: boolean;
  transparent?: boolean;
  alwaysOnTop?: boolean;
  x?: number;
  y?: number;
  minWidth?: number;
  minHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
  channels?: string[];
}

interface Position {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

interface NativeHandle {
  /** Platform type: "windows", "macos", "linux-x11", "linux-wayland", or "linux" (placeholder) */
  platform: string;
  /**
   * Raw handle value (HWND on Windows, NSView* on macOS, X11 window ID on Linux).
   * Note: On Linux without X11/Wayland detection, returns 0 as a placeholder.
   * Typed as number since Rust u64 serializes to JS number (safe for values up to 2^53).
   */
  handle: number;
}

interface WindowSystemEvent {
  windowId: string;
  type: "close" | "focus" | "blur" | "resize" | "move" | "minimize" | "maximize" | "restore";
  payload: unknown;
}

interface Window {
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

interface FileFilter {
  name: string;
  extensions: string[];
}

interface FileDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
  multiple?: boolean;
  directory?: boolean;
}

interface MessageDialogOptions {
  /** Dialog title */
  title?: string;
  /** Message to display */
  message: string;
  /** Dialog type: affects the icon shown */
  kind?: "info" | "warning" | "error";
  /**
   * Button labels to display.
   * NOTE: The underlying rfd library only supports preset button configurations
   * (Ok, OkCancel, YesNo, etc.). Custom button labels may be ignored or mapped
   * to the closest preset. The return value is the index of the clicked button.
   * For reliable cross-platform behavior, use convenience methods like
   * dialog.alert(), dialog.confirm(), dialog.error(), dialog.warning().
   */
  buttons?: string[];
}

interface MenuItem {
  id?: string;
  label: string;
  accelerator?: string;
  enabled?: boolean;
  checked?: boolean;
  submenu?: MenuItem[];
  type?: "normal" | "checkbox" | "separator";
}

interface MenuEvent {
  menuId: string;
  itemId: string;
  label: string;
}

interface TrayOptions {
  icon?: string;
  tooltip?: string;
  menu?: MenuItem[];
}

interface TrayHandle {
  readonly id: string;
  update(opts: TrayOptions): Promise<boolean>;
  destroy(): Promise<boolean>;
}

type MenuCallback = (event: MenuEvent) => void;

const core = Deno.core;

// ============================================================================
// Window Functions
// ============================================================================

/**
 * Create a new window and return a Window handle
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

    // Events
    async *events(): AsyncGenerator<WindowSystemEvent, void, unknown> {
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

/**
 * Close a window by ID
 */
export async function closeWindow(windowId: string): Promise<boolean> {
  return await core.ops.op_window_close(windowId);
}

/**
 * Receive the next window system event
 */
async function recvWindowEvent(): Promise<WindowSystemEvent | null> {
  return await core.ops.op_window_events_recv();
}

/**
 * Async iterator for window system events from all windows
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

export const dialog = {
  /**
   * Show an open file dialog. Returns null if cancelled.
   */
  async open(opts: FileDialogOptions = {}): Promise<string[] | null> {
    return await core.ops.op_window_dialog_open(opts);
  },

  /**
   * Show a save file dialog. Returns null if cancelled.
   */
  async save(opts: FileDialogOptions = {}): Promise<string | null> {
    return await core.ops.op_window_dialog_save(opts);
  },

  /**
   * Show a message dialog. Returns the index of the clicked button.
   * Note: Custom button labels may not be fully supported on all platforms.
   * Use alert(), confirm(), error(), warning() for reliable cross-platform behavior.
   */
  async message(opts: MessageDialogOptions | string): Promise<number> {
    const options: MessageDialogOptions = typeof opts === "string" ? { message: opts } : opts;
    return await core.ops.op_window_dialog_message(options);
  },

  /**
   * Show an alert dialog (convenience wrapper)
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
   * Show a confirm dialog (convenience wrapper)
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
   * Show an error dialog (convenience wrapper)
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
   * Show a warning dialog (convenience wrapper)
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

async function recvMenuEvent(): Promise<MenuEvent | null> {
  return await core.ops.op_window_menu_recv();
}

async function* menuEvents(): AsyncGenerator<MenuEvent, void, unknown> {
  while (true) {
    const event = await recvMenuEvent();
    if (event === null) break;
    yield event;
  }
}

let menuListenerActive = false;
const menuCallbacks: MenuCallback[] = [];

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

export const menu = {
  /**
   * Set the application menu bar
   */
  async setAppMenu(items: MenuItem[]): Promise<boolean> {
    return await core.ops.op_window_set_app_menu(items);
  },

  /**
   * Show a context menu at the current cursor position
   */
  async showContextMenu(items: MenuItem[], windowId?: string): Promise<string | null> {
    const result = await core.ops.op_window_show_context_menu(windowId ?? null, items);
    return result === "" ? null : result;
  },

  /**
   * Async iterator for menu events
   */
  events: menuEvents,

  /**
   * Register a callback for menu events. Returns unsubscribe function.
   */
  onMenu
};

// ============================================================================
// Tray Namespace
// ============================================================================

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

async function destroyTray(trayId: string): Promise<boolean> {
  return await core.ops.op_window_destroy_tray(trayId);
}

export const tray = {
  /**
   * Create a system tray icon
   */
  create: createTray,

  /**
   * Destroy a tray by ID
   */
  destroy: destroyTray
};

// ============================================================================
// Re-exports for convenience
// ============================================================================

export type {
  WindowOptions,
  Position,
  Size,
  NativeHandle,
  WindowSystemEvent,
  Window,
  FileFilter,
  FileDialogOptions,
  MessageDialogOptions,
  MenuItem,
  MenuEvent,
  TrayOptions,
  TrayHandle
};
