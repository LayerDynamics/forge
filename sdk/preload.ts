// Preload script injected into WebView renderer
// Provides window.host bridge for renderer <-> Deno communication

// Type declarations for the global window extensions
declare global {
  interface Window {
    __host_dispatch?: (msg: { channel: string; payload: unknown }) => void;
    host?: HostBridge;
    ipc?: {
      postMessage(message: string): void;
    };
  }
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

(() => {
  if (typeof window !== "undefined") {
    const listeners = new Map<string, ListenerCallback[]>();

    // Internal dispatch function called by host when sending messages to renderer
    window.__host_dispatch = function (msg: { channel: string; payload: unknown }) {
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
    window.host = {
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
        if (window.ipc) {
          window.ipc.postMessage(JSON.stringify({ channel, payload }));
        }
      },

      send(channel: string, payload?: unknown) {
        if (window.ipc) {
          window.ipc.postMessage(JSON.stringify({ channel, payload }));
        } else {
          console.warn("[host.send] IPC not available");
        }
      },
    };

    console.debug("[preload] window.host bridge ready");

    // HMR (Hot Module Replacement) client - connects to dev server for live reload
    // Only runs in dev mode (when app:// protocol is used and HMR server is available)
    if (location.protocol === "app:") {
      const HMR_PORT = 35729;
      let hmrSocket: WebSocket | null = null;
      let reconnectAttempts = 0;
      const MAX_RECONNECT_ATTEMPTS = 5;

      function connectHMR() {
        try {
          hmrSocket = new WebSocket(`ws://127.0.0.1:${HMR_PORT}`);

          hmrSocket.onopen = () => {
            console.debug("[HMR] Connected to dev server");
            reconnectAttempts = 0;
          };

          hmrSocket.onmessage = (event) => {
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

          hmrSocket.onclose = () => {
            console.debug("[HMR] Disconnected from dev server");
            hmrSocket = null;

            // Attempt reconnect
            if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
              reconnectAttempts++;
              setTimeout(connectHMR, 1000 * reconnectAttempts);
            }
          };

          hmrSocket.onerror = () => {
            // Silent error - HMR is optional in dev mode
            console.debug("[HMR] Connection error (dev server may not be running)");
          };
        } catch {
          // HMR not available - this is fine, it's optional
          console.debug("[HMR] Failed to connect (dev server may not be running)");
        }
      }

      // Start HMR connection after a short delay to let the page load
      setTimeout(connectHMR, 500);
    }
  }
})();
