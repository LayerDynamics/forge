// host:ui module - JavaScript wrapper for Deno core ops
const core = Deno.core;

export async function openWindow(opts = {}) {
  const windowId = await core.ops.op_ui_open_window(opts);

  const handle = {
    id: windowId,

    async send(channel, payload) {
      return await core.ops.op_ui_window_send(windowId, channel, payload);
    },

    async emit(channel, payload) {
      return await core.ops.op_ui_window_send(windowId, channel, payload);
    },

    async setTitle(title) {
      return await core.ops.op_ui_set_window_title(windowId, title);
    },

    async close() {
      return await core.ops.op_ui_close_window(windowId);
    },

    async *events() {
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

export async function closeWindow(windowId) {
  return await core.ops.op_ui_close_window(windowId);
}

export async function setWindowTitle(windowId, title) {
  return await core.ops.op_ui_set_window_title(windowId, title);
}

export async function sendToWindow(windowId, channel, payload) {
  return await core.ops.op_ui_window_send(windowId, channel, payload);
}

export async function recvWindowEvent() {
  return await core.ops.op_ui_window_recv();
}

export async function* windowEvents() {
  while (true) {
    const event = await recvWindowEvent();
    if (event === null) break;
    yield event;
  }
}

export const dialog = {
  async open(opts = {}) {
    return await core.ops.op_ui_dialog_open(opts);
  },

  async save(opts = {}) {
    return await core.ops.op_ui_dialog_save(opts);
  },

  async message(opts) {
    if (typeof opts === "string") {
      opts = { message: opts };
    }
    return await core.ops.op_ui_dialog_message(opts);
  },

  async alert(message, title = "Alert") {
    return await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["OK"]
    });
  },

  async confirm(message, title = "Confirm") {
    const result = await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "info",
      buttons: ["Cancel", "OK"]
    });
    return result === 1;
  },

  async error(message, title = "Error") {
    return await core.ops.op_ui_dialog_message({
      title,
      message,
      kind: "error",
      buttons: ["OK"]
    });
  },

  async warning(message, title = "Warning") {
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

export async function setAppMenu(items) {
  return await core.ops.op_ui_set_app_menu(items);
}

export async function showContextMenu(items, windowId) {
  const result = await core.ops.op_ui_show_context_menu(windowId ?? null, items);
  return result === "" ? null : result;
}

export async function recvMenuEvent() {
  return await core.ops.op_ui_menu_recv();
}

export async function* menuEvents() {
  while (true) {
    const event = await recvMenuEvent();
    if (event === null) break;
    yield event;
  }
}

let menuListenerActive = false;
const menuCallbacks = [];

export function onMenu(callback) {
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

export async function createTray(opts = {}) {
  const trayId = await core.ops.op_ui_create_tray(opts);

  return {
    id: trayId,

    async update(newOpts) {
      return await core.ops.op_ui_update_tray(trayId, newOpts);
    },

    async destroy() {
      return await core.ops.op_ui_destroy_tray(trayId);
    }
  };
}

export async function destroyTray(trayId) {
  return await core.ops.op_ui_destroy_tray(trayId);
}

export const tray = {
  create: createTray,
  destroy: destroyTray,
};
