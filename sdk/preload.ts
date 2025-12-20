// Preload script injected into WebView renderer
// Provides window.host bridge for renderer <-> Deno communication
// Note: This runs in the BROWSER context, not Deno - but we use globalThis for consistency

// Extend globalThis for browser environment
declare global {
  var __host_dispatch: ((msg: { channel: string; payload: unknown }) => void) | undefined;
  var host: HostBridge | undefined;
  var ipc: { postMessage(message: string): void } | undefined;
}

type ListenerCallback = (payload: unknown) => void;

interface HostBridge {
  /** Register a listener for messages from Deno */
  on(channel: string, cb: ListenerCallback): void;
  /** Remove a listener */
  off(channel: string, cb: ListenerCallback): void;
  /** Emit to local listeners (within renderer) */
  emit(channel: string, payload?: unknown): void;
  /** Send a message to Deno (via IPC) */
  send(channel: string, payload?: unknown): void;
}

// HMR connection function (hoisted to avoid inner declaration warning)
function connectHMR(
  hmrPort: number,
  maxAttempts: number,
  attemptRef: { current: number },
  socketRef: { current: WebSocket | null }
): void {
  try {
    socketRef.current = new WebSocket(`ws://127.0.0.1:${hmrPort}`);

    socketRef.current.onopen = () => {
      console.debug("[HMR] Connected to dev server");
      attemptRef.current = 0;
    };

    socketRef.current.onmessage = (event) => {
      const msg = event.data as string;

      if (msg.startsWith("css:")) {
        // Hot CSS reload - update stylesheets without full page reload
        const changedFile = msg.slice(4);
        console.debug("[HMR] CSS changed:", changedFile);

        document.querySelectorAll('link[rel="stylesheet"]').forEach((link) => {
          const href = link.getAttribute("href");
          if (href) {
            // Add cache-busting query param
            const newHref = href.split("?")[0] + "?t=" + Date.now();
            link.setAttribute("href", newHref);
          }
        });
      } else if (msg.startsWith("reload:")) {
        // Full page reload for non-CSS changes
        const changedFile = msg.slice(7);
        console.debug("[HMR] File changed, reloading:", changedFile);
        location.reload();
      }
    };

    socketRef.current.onclose = () => {
      console.debug("[HMR] Disconnected from dev server");
      socketRef.current = null;

      // Attempt reconnect
      if (attemptRef.current < maxAttempts) {
        attemptRef.current++;
        setTimeout(
          () => connectHMR(hmrPort, maxAttempts, attemptRef, socketRef),
          1000 * attemptRef.current
        );
      }
    };

    socketRef.current.onerror = () => {
      // Silent error - HMR is optional in dev mode
      console.debug("[HMR] Connection error (dev server may not be running)");
    };
  } catch {
    // HMR not available - this is fine, it's optional
    console.debug("[HMR] Failed to connect (dev server may not be running)");
  }
}

(() => {
  // Check we're in a browser context
  if (typeof document === "undefined") {
    return;
  }

  // Message queue for when IPC isn't ready yet
  const pendingMessages: string[] = [];
  let ipcReady = !!globalThis.ipc;

  // Function to send message, queuing if IPC not ready
  function sendViaIpc(msg: string) {
    if (globalThis.ipc) {
      globalThis.ipc.postMessage(msg);
    } else {
      pendingMessages.push(msg);
    }
  }

  // Poll for IPC availability and flush queue
  function checkIpcReady() {
    if (globalThis.ipc && !ipcReady) {
      ipcReady = true;
      while (pendingMessages.length > 0) {
        const msg = pendingMessages.shift()!;
        globalThis.ipc.postMessage(msg);
      }
    } else if (!globalThis.ipc) {
      // Keep polling
      setTimeout(checkIpcReady, 10);
    }
  }

  // Start polling if IPC not immediately available
  if (!globalThis.ipc) {
    checkIpcReady();
  }

  const listeners = new Map<string, ListenerCallback[]>();

  // Internal dispatch function called by host when sending messages to renderer
  globalThis.__host_dispatch = function (msg: { channel: string; payload: unknown }) {
    const { channel, payload } = msg;
    const arr = listeners.get(channel) || [];
    for (const cb of arr) {
      try {
        cb(payload);
      } catch (e) {
        console.error("[host.dispatch] Error in listener:", e);
      }
    }
  };

  // Public API exposed to renderer scripts
  globalThis.host = {
    on(channel: string, cb: ListenerCallback) {
      if (!listeners.has(channel)) {
        listeners.set(channel, []);
      }
      listeners.get(channel)!.push(cb);
    },

    off(channel: string, cb: ListenerCallback) {
      const arr = listeners.get(channel);
      if (arr) {
        const idx = arr.indexOf(cb);
        if (idx >= 0) arr.splice(idx, 1);
      }
    },

    emit(channel: string, payload?: unknown) {
      // Dispatch to local listeners first
      const arr = listeners.get(channel) || [];
      for (const cb of arr) {
        try {
          cb(payload);
        } catch (e) {
          console.error("[host.emit] Error:", e);
        }
      }
      // Also send to Deno backend
      sendViaIpc(JSON.stringify({ channel, payload }));
    },

    send(channel: string, payload?: unknown) {
      const msg = JSON.stringify({ channel, payload });
      sendViaIpc(msg);
    },
  };

  // Signal to backend that renderer is ready to receive messages.
  // Use setTimeout(0) to ensure __host_dispatch is fully registered.
  setTimeout(() => {
    if (globalThis.ipc) {
      globalThis.ipc.postMessage(JSON.stringify({
        channel: "__renderer_ready__",
        payload: {}
      }));
    }
  }, 0);

  // Built-in listener for __console__ channel - forwards Deno logs to browser DevTools
  globalThis.host.on("__console__", (payload: unknown) => {
    const msg = payload as { level: string; message: string; fields?: Record<string, unknown> };
    const { level, message, fields } = msg;
    const args: unknown[] = [message];
    if (fields && Object.keys(fields).length > 0) {
      args.push(fields);
    }
    switch (level) {
      case "trace":
      case "debug":
        console.debug(...args);
        break;
      case "info":
        console.info(...args);
        break;
      case "warn":
      case "warning":
        console.warn(...args);
        break;
      case "error":
        console.error(...args);
        break;
      default:
        console.log(...args);
    }
  });

  // HMR (Hot Module Replacement) client - connects to dev server for live reload
  // Only runs in dev mode (when app:// protocol is used and HMR server is available)
  if (location.protocol === "app:") {
    const HMR_PORT = 35729;
    const MAX_RECONNECT_ATTEMPTS = 5;
    const attemptRef = { current: 0 };
    const socketRef: { current: WebSocket | null } = { current: null };

    // Start HMR connection after a short delay to let the page load
    setTimeout(() => connectHMR(HMR_PORT, MAX_RECONNECT_ATTEMPTS, attemptRef, socketRef), 500);
  }
})();
