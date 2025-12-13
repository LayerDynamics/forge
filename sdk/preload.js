// Preload script injected into WebView renderer
// Provides window.host bridge for renderer <-> Deno communication

(function() {
  if (typeof window !== "undefined") {
    var listeners = new Map();

    // Cross-platform IPC: wry uses different mechanisms on each platform
    // - macOS (WebKit): webkit.messageHandlers.ipc.postMessage()
    // - Windows (WebView2): window.chrome.webview.postMessage()
    // - Linux (WebKitGTK): window.ipc.postMessage()
    function postToHost(message) {
      if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.ipc) {
        // macOS WebKit
        window.webkit.messageHandlers.ipc.postMessage(message);
        return true;
      } else if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
        // Windows WebView2
        window.chrome.webview.postMessage(message);
        return true;
      } else if (window.ipc && window.ipc.postMessage) {
        // Linux WebKitGTK or fallback
        window.ipc.postMessage(message);
        return true;
      }
      return false;
    }

    // Check IPC availability
    var ipcAvailable = !!(
      (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.ipc) ||
      (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) ||
      (window.ipc && window.ipc.postMessage)
    );

    // Internal dispatch function called by host when sending messages to renderer
    window.__host_dispatch = function(msg) {
      var channel = msg.channel;
      var payload = msg.payload;
      console.log("[host.recv] channel:", channel, "payload:", payload);
      var arr = listeners.get(channel) || [];
      for (var i = 0; i < arr.length; i++) {
        try {
          arr[i](payload);
        } catch (e) {
          console.error("[host.dispatch] Error in listener:", e);
        }
      }
    };

    // Public API exposed to renderer scripts
    window.host = {
      on: function(channel, cb) {
        if (!listeners.has(channel)) {
          listeners.set(channel, []);
        }
        listeners.get(channel).push(cb);
      },

      off: function(channel, cb) {
        var arr = listeners.get(channel);
        if (arr) {
          var idx = arr.indexOf(cb);
          if (idx >= 0) arr.splice(idx, 1);
        }
      },

      emit: function(channel, payload) {
        console.log("[host.emit] channel:", channel, "payload:", payload);
        // Dispatch to local listeners first
        var arr = listeners.get(channel) || [];
        for (var i = 0; i < arr.length; i++) {
          try {
            arr[i](payload);
          } catch (e) {
            console.error("[host.emit] Error:", e);
          }
        }
        // Also send to Deno backend
        var msg = JSON.stringify({ channel: channel, payload: payload });
        if (postToHost(msg)) {
          console.log("[host.emit] sent to Deno");
        } else {
          console.warn("[host.emit] IPC not available, cannot send to Deno");
        }
      },

      send: function(channel, payload) {
        var msg = JSON.stringify({ channel: channel, payload: payload });
        if (!postToHost(msg)) {
          console.warn("[host.send] IPC not available");
        }
      }
    };

    console.log("[preload] window.host bridge ready, IPC available:", ipcAvailable);
    if (!ipcAvailable) {
      console.error("[preload] CRITICAL: No IPC mechanism found! Messages will not reach Deno.");
    }

    // HMR (Hot Module Replacement) client - connects to dev server for live reload
    // Only runs in dev mode (when app:// protocol is used and HMR server is available)
    if (location.protocol === "app:") {
      var HMR_PORT = 35729;
      var hmrSocket = null;
      var reconnectAttempts = 0;
      var MAX_RECONNECT_ATTEMPTS = 5;

      function connectHMR() {
        try {
          hmrSocket = new WebSocket("ws://127.0.0.1:" + HMR_PORT);

          hmrSocket.onopen = function() {
            console.log("[HMR] Connected to dev server");
            reconnectAttempts = 0;
          };

          hmrSocket.onmessage = function(event) {
            var msg = event.data;

            if (msg.indexOf("css:") === 0) {
              // Hot CSS reload - update stylesheets without full page reload
              var changedFile = msg.slice(4);
              console.log("[HMR] CSS changed:", changedFile);

              document.querySelectorAll('link[rel="stylesheet"]').forEach(function(link) {
                var href = link.getAttribute("href");
                if (href) {
                  // Add cache-busting query param
                  var newHref = href.split("?")[0] + "?t=" + Date.now();
                  link.setAttribute("href", newHref);
                }
              });
            } else if (msg.indexOf("reload:") === 0) {
              // Full page reload for non-CSS changes
              var changedFile = msg.slice(7);
              console.log("[HMR] File changed, reloading:", changedFile);
              location.reload();
            }
          };

          hmrSocket.onclose = function() {
            console.log("[HMR] Disconnected from dev server");
            hmrSocket = null;

            // Attempt reconnect
            if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
              reconnectAttempts++;
              setTimeout(connectHMR, 1000 * reconnectAttempts);
            }
          };

          hmrSocket.onerror = function() {
            // Silent error - HMR is optional in dev mode
            console.log("[HMR] Connection error (dev server may not be running)");
          };
        } catch (e) {
          // HMR not available - this is fine, it's optional
          console.log("[HMR] Failed to connect (dev server may not be running)");
        }
      }

      // Start HMR connection after a short delay to let the page load
      setTimeout(connectHMR, 500);
    }
  }
})();
