// host:ui module - TypeScript wrapper for Deno core ops
// Note: IPC functions (sendToWindow, recvWindowEvent, windowEvents) have moved to host:ipc

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_ui_open_window(opts: OpenWindowOptions): Promise<string>;
      op_ui_close_window(windowId: string): Promise<boolean>;
      op_ui_set_window_title(windowId: string, title: string): Promise<void>;
      op_ui_dialog_open(opts: FileDialogOptions): Promise<string[] | null>;
      op_ui_dialog_save(opts: FileDialogOptions): Promise<string | null>;
      op_ui_dialog_message(opts: MessageDialogOptions): Promise<number>;
      op_ui_set_app_menu(items: MenuItem[]): Promise<boolean>;
      op_ui_show_context_menu(windowId: string | null, items: MenuItem[]): Promise<string>;
      op_ui_menu_recv(): Promise<MenuEvent | null>;
      op_ui_create_tray(opts: TrayOptions): Promise<string>;
      op_ui_update_tray(trayId: string, opts: TrayOptions): Promise<boolean>;
      op_ui_destroy_tray(trayId: string): Promise<boolean>;
    };
  };
};

interface OpenWindowOptions {
  url?: string;
  width?: number;
  height?: number;
  title?: string;
  resizable?: boolean;
  decorations?: boolean;
  channels?: string[];
}

// WindowEvent interface - use IpcEvent from host:ipc for IPC events
interface WindowEvent {
  windowId: string;
  channel: string;
  payload: unknown;
  type?: "close" | "focus" | "blur" | "resize" | "move";
}

// Note: WindowHandle now uses host:ipc for IPC methods
// The send/emit methods should be obtained from the window handle returned by openWindow
// or use sendToWindow from host:ipc directly
interface WindowHandle {
  readonly id: string;
  setTitle(title: string): Promise<void>;
  close(): Promise<boolean>;
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
  title?: string;
  message: string;
  kind?: "info" | "warning" | "error";
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

export async function openWindow(opts: OpenWindowOptions = {}): Promise<WindowHandle> {
  const windowId = await core.ops.op_ui_open_window(opts);

  // Note: For IPC (send/receive messages), use host:ipc module:
  //   import { sendToWindow, windowEvents } from "host:ipc";
  //   sendToWindow(windowId, channel, payload);
  const handle: WindowHandle = {
    id: windowId,

    async setTitle(title: string): Promise<void> {
      return await core.ops.op_ui_set_window_title(windowId, title);
    },

    async close(): Promise<boolean> {
      return await core.ops.op_ui_close_window(windowId);
    },
  };

  return handle;
}

export async function closeWindow(windowId: string): Promise<boolean> {
  return await core.ops.op_ui_close_window(windowId);
}

export async function setWindowTitle(windowId: string, title: string): Promise<void> {
  return await core.ops.op_ui_set_window_title(windowId, title);
}

// IPC functions (sendToWindow, recvWindowEvent, windowEvents) have moved to host:ipc module
// Import them from "host:ipc" instead

export const dialog = {
  async open(opts: FileDialogOptions = {}): Promise<string[] | null> {
    return await core.ops.op_ui_dialog_open(opts);
  },

  async save(opts: FileDialogOptions = {}): Promise<string | null> {
    return await core.ops.op_ui_dialog_save(opts);
  },

  async message(opts: MessageDialogOptions | string): Promise<number> {
    const options: MessageDialogOptions = typeof opts === "string" ? { message: opts } : opts;
    return await core.ops.op_ui_dialog_message(options);
  },

  async alert(message: string, title: string = "Alert"): Promise<number> {
    return await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["OK"]
    });
  },

  async confirm(message: string, title: string = "Confirm"): Promise<boolean> {
    const result = await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["Cancel", "OK"]
    });
    return result === 1;
  },

  async error(message: string, title: string = "Error"): Promise<number> {
    return await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "error",
      buttons: ["OK"]
    });
  },

  async warning(message: string, title: string = "Warning"): Promise<number> {
    return await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "warning",
      buttons: ["OK"]
    });
  }
};

export const showOpenDialog = dialog.open;
export const showSaveDialog = dialog.save;
export const showMessageDialog = dialog.message;

export async function setAppMenu(items: MenuItem[]): Promise<boolean> {
  return await core.ops.op_ui_set_app_menu(items);
}

export async function showContextMenu(items: MenuItem[], windowId?: string): Promise<string | null> {
  const result = await core.ops.op_ui_show_context_menu(windowId ?? null, items);
  return result === "" ? null : result;
}

export async function recvMenuEvent(): Promise<MenuEvent | null> {
  return await core.ops.op_ui_menu_recv();
}

export async function* menuEvents(): AsyncGenerator<MenuEvent, void, unknown> {
  while (true) {
    const event = await recvMenuEvent();
    if (event === null) break;
    yield event;
  }
}

let menuListenerActive = false;
const menuCallbacks: MenuCallback[] = [];

export function onMenu(callback: MenuCallback): () => void {
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
  setAppMenu,
  showContextMenu,
  events: menuEvents,
  onMenu,
};

export async function createTray(opts: TrayOptions = {}): Promise<TrayHandle> {
  const trayId = await core.ops.op_ui_create_tray(opts);

  return {
    id: trayId,

    async update(newOpts: TrayOptions): Promise<boolean> {
      return await core.ops.op_ui_update_tray(trayId, newOpts);
    },

    async destroy(): Promise<boolean> {
      return await core.ops.op_ui_destroy_tray(trayId);
    }
  };
}

export async function destroyTray(trayId: string): Promise<boolean> {
  return await core.ops.op_ui_destroy_tray(trayId);
}

export const tray = {
  create: createTray,
  destroy: destroyTray,
};
