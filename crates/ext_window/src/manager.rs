//! WindowManager - Platform implementation for window operations
//!
//! This module contains the actual platform logic for windows, dialogs, menus, and trays.
//! forge-host creates a WindowManager instance and calls its methods from the event loop.

use crate::{
    FileDialogOpts, MenuItem, MessageDialogOpts, MonitorInfo, NativeHandle, Position, Size,
    TrayOpts, WindowCmd, WindowOpts, WindowState, WindowSystemEvent,
};
use deno_ast::{MediaType, ParseParams, TranspileModuleOptions, TranspileOptions};
use deno_core::ModuleSpecifier;
pub use ext_ipc::IpcEvent;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tao::event_loop::EventLoopWindowTarget;
use tao::window::{Window, WindowBuilder, WindowId};
use tokio::sync::mpsc;
use wry::http::{Response, StatusCode};
use wry::WebView;

/// Context menu timeout in seconds (user has this long to select an item)
pub const CONTEXT_MENU_TIMEOUT_SECS: u64 = 30;

/// Pending context menu state with timestamp for timeout handling
pub type PendingContextMenu = Option<(
    HashSet<muda::MenuId>,
    tokio::sync::oneshot::Sender<Option<String>>,
    Instant, // When the menu was shown
)>;

/// Configuration for WindowManager
pub struct WindowManagerConfig {
    pub app_dir: PathBuf,
    pub dev_mode: bool,
    pub app_name: String,
    pub default_channels: Option<Vec<String>>,
}

/// Asset provider trait - allows forge-host to provide embedded or filesystem assets
pub trait AssetProvider: Send + Sync {
    fn get_asset(&self, path: &str) -> Option<Vec<u8>>;
    fn is_embedded(&self) -> bool;
}

/// WindowManager handles all platform window operations
pub struct WindowManager<U: 'static> {
    // State
    window_counter: u64,
    tray_counter: u64,
    windows: HashMap<WindowId, String>,
    webviews: HashMap<String, WebView>,
    tao_windows: HashMap<String, Window>,
    window_channels: HashMap<String, Option<Vec<String>>>,
    trays: HashMap<String, tray_icon::TrayIcon>,
    menu_id_map: Arc<Mutex<HashMap<muda::MenuId, (String, String)>>>,
    pending_ctx_menu: Arc<Mutex<PendingContextMenu>>,
    app_menu: Option<muda::Menu>,

    // Config
    config: WindowManagerConfig,

    // Channels back to Deno
    window_events_tx: mpsc::Sender<WindowSystemEvent>,

    // IPC sender for renderer -> Deno messages
    to_deno_tx: mpsc::Sender<IpcEvent>,

    // Capability checker for channel filtering
    capabilities: Option<Arc<dyn ChannelChecker>>,

    // Preload JS (provided by forge-host)
    preload_js: String,

    // Asset provider (provided by forge-host)
    asset_provider: Arc<dyn AssetProvider>,

    // Phantom for UserEvent type
    _phantom: std::marker::PhantomData<U>,
}

/// Trait for checking channel permissions
pub trait ChannelChecker: Send + Sync {
    fn check_channel(&self, channel: &str, allowed: Option<&[String]>) -> Result<(), String>;
}

impl<U: 'static> WindowManager<U> {
    /// Create a new WindowManager
    pub fn new(
        config: WindowManagerConfig,
        window_events_tx: mpsc::Sender<WindowSystemEvent>,
        to_deno_tx: mpsc::Sender<IpcEvent>,
        capabilities: Option<Arc<dyn ChannelChecker>>,
        preload_js: String,
        asset_provider: Arc<dyn AssetProvider>,
    ) -> Self {
        Self {
            window_counter: 0,
            tray_counter: 0,
            windows: HashMap::new(),
            webviews: HashMap::new(),
            tao_windows: HashMap::new(),
            window_channels: HashMap::new(),
            trays: HashMap::new(),
            menu_id_map: Arc::new(Mutex::new(HashMap::new())),
            pending_ctx_menu: Arc::new(Mutex::new(None)),
            app_menu: None,
            config,
            window_events_tx,
            to_deno_tx,
            capabilities,
            preload_js,
            asset_provider,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the menu ID map for use in menu event thread
    pub fn menu_id_map(&self) -> Arc<Mutex<HashMap<muda::MenuId, (String, String)>>> {
        self.menu_id_map.clone()
    }

    /// Get the pending context menu state for use in menu event thread
    pub fn pending_ctx_menu(&self) -> Arc<Mutex<PendingContextMenu>> {
        self.pending_ctx_menu.clone()
    }

    /// Check if all windows are closed
    pub fn is_empty(&self) -> bool {
        self.webviews.is_empty()
    }

    /// Get window ID from tao WindowId
    pub fn get_window_id(&self, tao_id: &WindowId) -> Option<&String> {
        self.windows.get(tao_id)
    }

    /// Get webview by window ID
    pub fn get_webview(&self, window_id: &str) -> Option<&WebView> {
        self.webviews.get(window_id)
    }

    /// Get tao window by window ID
    pub fn get_tao_window(&self, window_id: &str) -> Option<&Window> {
        self.tao_windows.get(window_id)
    }

    /// Get channel allowlist for a window
    pub fn get_window_channels(&self, window_id: &str) -> Option<&Option<Vec<String>>> {
        self.window_channels.get(window_id)
    }

    /// Insert a webview into the manager (for ext_ui compatibility)
    pub fn insert_webview(&mut self, window_id: String, webview: WebView) {
        self.webviews.insert(window_id, webview);
    }

    /// Insert a tao window into the manager (for ext_ui compatibility)
    pub fn insert_tao_window(&mut self, window_id: String, window: Window) {
        let tao_id = window.id();
        self.windows.insert(tao_id, window_id.clone());
        self.tao_windows.insert(window_id, window);
    }

    /// Insert window channel allowlist (for ext_ui compatibility)
    pub fn insert_window_channels(&mut self, window_id: String, channels: Option<Vec<String>>) {
        self.window_channels.insert(window_id, channels);
    }

    /// Remove window by ID (for ext_ui compatibility)
    pub fn remove_window(&mut self, window_id: &str) -> bool {
        if let Some(window) = self.tao_windows.remove(window_id) {
            let tao_id = window.id();
            self.windows.remove(&tao_id);
            self.webviews.remove(window_id);
            self.window_channels.remove(window_id);
            true
        } else {
            false
        }
    }

    /// Get the next window ID counter (for ext_ui compatibility)
    pub fn next_window_id(&mut self) -> u64 {
        self.window_counter += 1;
        self.window_counter
    }

    /// Get app menu reference (for ext_ui compatibility)
    pub fn app_menu(&self) -> &Option<muda::Menu> {
        &self.app_menu
    }

    /// Set app menu (for ext_ui compatibility)
    pub fn set_app_menu_raw(&mut self, menu: Option<muda::Menu>) {
        self.app_menu = menu;
    }

    /// Iterate over all tao windows (for ext_ui platform-specific code)
    pub fn iter_tao_windows(&self) -> impl Iterator<Item = &Window> {
        self.tao_windows.values()
    }

    /// Get tray by ID (for ext_ui compatibility)
    pub fn get_tray(&self, tray_id: &str) -> Option<&tray_icon::TrayIcon> {
        self.trays.get(tray_id)
    }

    /// Get mutable tray by ID (for ext_ui compatibility)
    pub fn get_tray_mut(&mut self, tray_id: &str) -> Option<&mut tray_icon::TrayIcon> {
        self.trays.get_mut(tray_id)
    }

    /// Insert tray (for ext_ui compatibility)
    pub fn insert_tray(&mut self, tray_id: String, tray: tray_icon::TrayIcon) {
        self.trays.insert(tray_id, tray);
    }

    /// Remove tray (for ext_ui compatibility)
    pub fn remove_tray(&mut self, tray_id: &str) -> bool {
        self.trays.remove(tray_id).is_some()
    }

    /// Get the next tray ID counter (for ext_ui compatibility)
    pub fn next_tray_id(&mut self) -> u64 {
        self.tray_counter += 1;
        self.tray_counter
    }

    /// Get app directory (for ext_ui compatibility)
    pub fn app_dir(&self) -> &PathBuf {
        &self.config.app_dir
    }

    /// Send message to renderer
    pub fn send_to_renderer(&self, window_id: &str, channel: &str, payload: &str) {
        // Check if channel is allowed for this window
        let win_allowed_channels = self.window_channels.get(window_id).and_then(|c| c.clone());

        if let Some(ref caps) = self.capabilities {
            if caps
                .check_channel(channel, win_allowed_channels.as_deref())
                .is_err()
            {
                tracing::warn!(
                    "Blocked outgoing IPC message on channel '{}' to window {} - not in allowlist",
                    channel,
                    window_id
                );
                return;
            }
        }

        if let Some(wv) = self.webviews.get(window_id) {
            let js = format!(
                "window.__host_dispatch && window.__host_dispatch({{channel:{:?},payload:{}}});",
                channel, payload
            );
            let _ = wv.evaluate_script(&js);
        }
    }

    // =========================================================================
    // Window Lifecycle
    // =========================================================================

    /// Create a new window
    pub fn create_window(
        &mut self,
        event_loop_target: &EventLoopWindowTarget<U>,
        opts: WindowOpts,
    ) -> Result<String, String> {
        tracing::debug!("create_window starting with opts: {:?}", opts);
        println!("create_window starting");
        let width = opts.width.unwrap_or(800);
        let height = opts.height.unwrap_or(600);
        let title = opts
            .title
            .clone()
            .unwrap_or_else(|| self.config.app_name.clone());

        let mut win_builder = WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(tao::dpi::LogicalSize::new(width, height));

        if let (Some(x), Some(y)) = (opts.x, opts.y) {
            win_builder = win_builder.with_position(tao::dpi::LogicalPosition::new(x, y));
        }
        if let Some(resizable) = opts.resizable {
            win_builder = win_builder.with_resizable(resizable);
        }
        if let Some(decorations) = opts.decorations {
            win_builder = win_builder.with_decorations(decorations);
        }
        if let Some(visible) = opts.visible {
            win_builder = win_builder.with_visible(visible);
        }
        if let Some(transparent) = opts.transparent {
            win_builder = win_builder.with_transparent(transparent);
        }
        if let Some(always_on_top) = opts.always_on_top {
            win_builder = win_builder.with_always_on_top(always_on_top);
        }
        if let (Some(min_w), Some(min_h)) = (opts.min_width, opts.min_height) {
            win_builder = win_builder.with_min_inner_size(tao::dpi::LogicalSize::new(min_w, min_h));
        }
        if let (Some(max_w), Some(max_h)) = (opts.max_width, opts.max_height) {
            win_builder = win_builder.with_max_inner_size(tao::dpi::LogicalSize::new(max_w, max_h));
        }

        let window = win_builder
            .build(event_loop_target)
            .map_err(|e| format!("Failed to create window: {}", e))?;

        self.window_counter += 1;
        let win_id = format!("win-{}", self.window_counter);
        let win_channels = opts
            .channels
            .clone()
            .or_else(|| self.config.default_channels.clone());

        // Build WebView
        let mut wv_builder = wry::WebViewBuilder::new();
        wv_builder = wv_builder.with_initialization_script(&self.preload_js);

        // IPC handler
        let to_deno_tx_clone = self.to_deno_tx.clone();
        let win_id_for_ipc = win_id.clone();
        let capabilities = self.capabilities.clone();
        let ipc_allowed_channels = win_channels.clone();
        wv_builder = wv_builder.with_ipc_handler(move |msg| {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(msg.body()) {
                let channel = val
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let allowed = if let Some(ref caps) = capabilities {
                    caps.check_channel(&channel, ipc_allowed_channels.as_deref())
                        .is_ok()
                } else {
                    true
                };

                if allowed {
                    let payload = val
                        .get("payload")
                        .cloned()
                        .unwrap_or(serde_json::json!(null));
                    let _ = to_deno_tx_clone.try_send(IpcEvent {
                        window_id: win_id_for_ipc.clone(),
                        channel,
                        payload,
                        event_type: None,
                    });
                }
            }
        });

        // Custom app:// protocol handler
        let app_dir = self.config.app_dir.clone();
        let is_dev_mode = self.config.dev_mode;
        let asset_provider = self.asset_provider.clone();
        wv_builder = wv_builder.with_custom_protocol("app".into(), move |_ctx, request| {
            let uri = request.uri().to_string();
            let mut path = uri
                .strip_prefix("app://")
                .unwrap_or("")
                .trim_start_matches('/')
                .trim_end_matches('/');

            if let Some(slash_pos) = path.find('/') {
                let first_part = &path[..slash_pos];
                if first_part.ends_with(".html") || first_part.ends_with(".htm") {
                    path = &path[slash_pos + 1..];
                }
            }

            let csp = if is_dev_mode {
                "default-src 'self' app:; \
                 script-src 'self' app: 'unsafe-inline' 'unsafe-eval' https://unpkg.com https://cdn.jsdelivr.net https://esm.sh; \
                 style-src 'self' app: 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net; \
                 connect-src 'self' app: ws://localruntime:* ws://127.0.0.1:* http://localruntime:* http://127.0.0.1:* https://*; \
                 img-src 'self' app: data: blob: https:; \
                 font-src 'self' app: data: https:;"
            } else {
                "default-src 'self' app:; \
                 script-src 'self' app:; \
                 style-src 'self' app: 'unsafe-inline'; \
                 img-src 'self' app: data: blob:; \
                 font-src 'self' app: data:; \
                 connect-src 'self' app:;"
            };

            // Try asset provider first (handles both embedded and filesystem)
            if let Some(bytes) = asset_provider.get_asset(path) {
                let (content_type, body) = maybe_transpile_ts(path, bytes);
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", content_type)
                    .header("Content-Security-Policy", csp)
                    .header("X-Content-Type-Options", "nosniff")
                    .body(Cow::Owned(body))
                    .unwrap();
            }

            // Fallback to direct filesystem (for dev mode if provider doesn't handle path)
            let file_path = app_dir.join("web").join(path);
            if file_path.exists() {
                if let Ok(bytes) = std::fs::read(&file_path) {
                    let (content_type, body) = maybe_transpile_ts(path, bytes);
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", content_type)
                        .header("Content-Security-Policy", csp)
                        .header("X-Content-Type-Options", "nosniff")
                        .body(Cow::Owned(body))
                        .unwrap();
                }
            }

            // 404
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Cow::Owned(format!("Not found: {}", path).into_bytes()))
                .unwrap()
        });

        // Set URL
        let start_url = opts.url.as_deref().unwrap_or("app://index.html");
        wv_builder = wv_builder.with_url(start_url);

        let webview = wv_builder
            .build(&window)
            .map_err(|e| format!("Failed to build webview: {}", e))?;

        let tao_window_id = window.id();
        self.windows.insert(tao_window_id, win_id.clone());
        self.webviews.insert(win_id.clone(), webview);
        self.tao_windows.insert(win_id.clone(), window);
        self.window_channels.insert(win_id.clone(), win_channels);

        if opts.devtools.unwrap_or(false) {
            let _ = self.open_devtools(&win_id);
        }

        let _ = self.window_events_tx.try_send(WindowSystemEvent {
            window_id: win_id.clone(),
            event_type: "create".to_string(),
            payload: serde_json::json!({}),
        });

        tracing::debug!("Created window {} at {}", win_id, start_url);
        Ok(win_id)
    }

    /// Close a window by ID
    pub fn close_window(&mut self, window_id: &str) -> bool {
        if let Some(window) = self.tao_windows.remove(window_id) {
            let tao_id = window.id();
            self.windows.remove(&tao_id);
            self.webviews.remove(window_id);
            self.window_channels.remove(window_id);

            let _ = self.window_events_tx.try_send(WindowSystemEvent {
                window_id: window_id.to_string(),
                event_type: "close".to_string(),
                payload: serde_json::json!({}),
            });

            tracing::debug!("Window {} closed", window_id);
            true
        } else {
            false
        }
    }

    /// Minimize a window
    pub fn minimize_window(&self, window_id: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_minimized(true);
            let _ = self.window_events_tx.try_send(WindowSystemEvent {
                window_id: window_id.to_string(),
                event_type: "minimize".to_string(),
                payload: serde_json::json!({}),
            });
        }
    }

    /// Maximize a window
    pub fn maximize_window(&self, window_id: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_maximized(true);
            let _ = self.window_events_tx.try_send(WindowSystemEvent {
                window_id: window_id.to_string(),
                event_type: "maximize".to_string(),
                payload: serde_json::json!({}),
            });
        }
    }

    /// Unmaximize a window
    pub fn unmaximize_window(&self, window_id: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_maximized(false);
            let _ = self.window_events_tx.try_send(WindowSystemEvent {
                window_id: window_id.to_string(),
                event_type: "restore".to_string(),
                payload: serde_json::json!({}),
            });
        }
    }

    /// Restore a window from minimized state
    pub fn restore_window(&self, window_id: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_minimized(false);
            let _ = self.window_events_tx.try_send(WindowSystemEvent {
                window_id: window_id.to_string(),
                event_type: "restore".to_string(),
                payload: serde_json::json!({}),
            });
        }
    }

    /// Set fullscreen mode
    pub fn set_fullscreen(&self, window_id: &str, fullscreen: bool) {
        if let Some(window) = self.tao_windows.get(window_id) {
            if fullscreen {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            }
        }
    }

    /// Focus a window
    pub fn focus_window(&self, window_id: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_focus();
        }
    }

    // =========================================================================
    // Window Properties
    // =========================================================================

    /// Get window position
    pub fn get_position(&self, window_id: &str) -> Result<Position, String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            let pos = window
                .outer_position()
                .unwrap_or(tao::dpi::PhysicalPosition::new(0, 0));
            Ok(Position { x: pos.x, y: pos.y })
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Set window position
    pub fn set_position(&self, window_id: &str, x: i32, y: i32) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_outer_position(tao::dpi::LogicalPosition::new(x, y));
        }
    }

    /// Get window size
    pub fn get_size(&self, window_id: &str) -> Result<Size, String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            let size = window.inner_size();
            Ok(Size {
                width: size.width,
                height: size.height,
            })
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Set window size
    pub fn set_size(&self, window_id: &str, width: u32, height: u32) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_inner_size(tao::dpi::LogicalSize::new(width, height));
        }
    }

    /// Get window title
    pub fn get_title(&self, window_id: &str) -> Result<String, String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            Ok(window.title())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Set window title
    pub fn set_title(&self, window_id: &str, title: &str) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_title(title);
        }
    }

    /// Set window resizable
    pub fn set_resizable(&self, window_id: &str, resizable: bool) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_resizable(resizable);
        }
    }

    /// Set window decorations
    pub fn set_decorations(&self, window_id: &str, decorations: bool) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_decorations(decorations);
        }
    }

    /// Set always on top
    pub fn set_always_on_top(&self, window_id: &str, always_on_top: bool) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_always_on_top(always_on_top);
        }
    }

    /// Set window visibility
    pub fn set_visible(&self, window_id: &str, visible: bool) {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_visible(visible);
        }
    }

    /// Get window state
    pub fn get_state(&self, window_id: &str) -> Result<WindowState, String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            Ok(WindowState {
                is_visible: window.is_visible(),
                is_focused: window.is_focused(),
                is_fullscreen: window.fullscreen().is_some(),
                is_maximized: window.is_maximized(),
                is_minimized: window.is_minimized(),
                is_resizable: window.is_resizable(),
                has_decorations: window.is_decorated(),
                is_always_on_top: window.is_always_on_top(),
            })
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    // =========================================================================
    // Dialogs
    // =========================================================================

    /// Show file open dialog
    pub fn show_open_dialog(&self, opts: FileDialogOpts) -> Option<Vec<String>> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(ref title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(ref default_path) = opts.default_path {
            dialog = dialog.set_directory(default_path);
        }
        if let Some(ref filters) = opts.filters {
            for filter in filters {
                let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
                dialog = dialog.add_filter(&filter.name, &exts);
            }
        }

        if opts.directory.unwrap_or(false) {
            dialog
                .pick_folder()
                .map(|p| vec![p.to_string_lossy().to_string()])
        } else if opts.multiple.unwrap_or(false) {
            dialog.pick_files().map(|paths| {
                paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect()
            })
        } else {
            dialog
                .pick_file()
                .map(|p| vec![p.to_string_lossy().to_string()])
        }
    }

    /// Show file save dialog
    pub fn show_save_dialog(&self, opts: FileDialogOpts) -> Option<String> {
        let mut dialog = rfd::FileDialog::new();

        if let Some(ref title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(ref default_path) = opts.default_path {
            dialog = dialog.set_file_name(default_path);
        }
        if let Some(ref filters) = opts.filters {
            for filter in filters {
                let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
                dialog = dialog.add_filter(&filter.name, &exts);
            }
        }

        dialog.save_file().map(|p| p.to_string_lossy().to_string())
    }

    /// Show message dialog
    pub fn show_message_dialog(&self, opts: MessageDialogOpts) -> usize {
        let level = match opts.kind.as_deref() {
            Some("warning") => rfd::MessageLevel::Warning,
            Some("error") => rfd::MessageLevel::Error,
            _ => rfd::MessageLevel::Info,
        };

        let buttons = if let Some(ref btns) = opts.buttons {
            match btns.len() {
                0 => rfd::MessageButtons::Ok,
                1 => rfd::MessageButtons::OkCustom(btns[0].clone()),
                2 => rfd::MessageButtons::OkCancelCustom(btns[0].clone(), btns[1].clone()),
                n => {
                    if n > 3 {
                        tracing::warn!(
                            "Message dialog supports at most 3 buttons, got {}. Extra buttons will be ignored.",
                            n
                        );
                    }
                    rfd::MessageButtons::YesNoCancelCustom(
                        btns[0].clone(),
                        btns[1].clone(),
                        btns.get(2).cloned().unwrap_or_else(|| "Cancel".to_string()),
                    )
                }
            }
        } else {
            rfd::MessageButtons::Ok
        };

        let dialog = rfd::MessageDialog::new()
            .set_level(level)
            .set_title(opts.title.as_deref().unwrap_or(""))
            .set_description(&opts.message)
            .set_buttons(buttons);

        let result = dialog.show();
        match result {
            rfd::MessageDialogResult::Ok => 0,
            rfd::MessageDialogResult::Cancel => 1,
            rfd::MessageDialogResult::Yes => 0,
            rfd::MessageDialogResult::No => 1,
            rfd::MessageDialogResult::Custom(_) => 0,
        }
    }

    // =========================================================================
    // Menus
    // =========================================================================

    /// Set the application menu
    pub fn set_app_menu(&mut self, items: Vec<MenuItem>) -> bool {
        // Clear old menu ID mappings
        {
            let mut map = self.menu_id_map.lock().unwrap();
            map.clear();
        }

        let menu = muda::Menu::new();

        // Build menu and populate ID map
        {
            let mut map = self.menu_id_map.lock().unwrap();
            add_menu_items_with_tracking(&menu, &items, &mut map);
            tracing::debug!("Registered {} menu items for event tracking", map.len());
        }

        // Platform-specific initialization
        #[cfg(target_os = "macos")]
        {
            menu.init_for_nsapp();
        }

        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            for window in self.tao_windows.values() {
                unsafe {
                    let _ = menu.init_for_hwnd(window.hwnd() as isize);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            use tao::platform::unix::WindowExtUnix;
            for window in self.tao_windows.values() {
                let gtk_win = window.gtk_window();
                let gtk_win_ref: &gtk::Window = gtk_win.upcast_ref();
                let _ = menu.init_for_gtk_window(gtk_win_ref, None::<&gtk::Box>);
            }
        }

        // Keep menu alive
        if self.app_menu.is_some() {
            tracing::debug!("Replacing existing app menu");
        }
        self.app_menu = Some(menu);
        tracing::debug!(
            "Set app menu with {} items (menu active: {})",
            items.len(),
            self.app_menu.is_some()
        );
        true
    }

    /// Show a context menu
    pub fn show_context_menu(
        &mut self,
        window_id: Option<&str>,
        items: Vec<MenuItem>,
        respond: tokio::sync::oneshot::Sender<Option<String>>,
    ) {
        use muda::ContextMenu;

        tracing::debug!("ShowContextMenu requested with {} items", items.len());

        // Cancel any pending context menu response
        {
            let mut pending = self.pending_ctx_menu.lock().unwrap();
            if let Some((_, old_sender, _)) = pending.take() {
                let _ = old_sender.send(None);
            }
        }

        if items.is_empty() {
            let _ = respond.send(None);
            return;
        }

        // Build the context menu
        let menu = muda::Menu::new();
        let mut ctx_menu_ids: HashSet<muda::MenuId> = HashSet::new();

        // Add items and collect IDs
        {
            let mut map = self.menu_id_map.lock().unwrap();
            add_context_menu_items(&menu, &items, &mut map, &mut ctx_menu_ids);
        }

        // Store the pending response with the set of IDs and timestamp
        {
            let mut pending = self.pending_ctx_menu.lock().unwrap();
            *pending = Some((ctx_menu_ids, respond, Instant::now()));
        }

        // Show context menu on the window
        let target_window = if let Some(wid) = window_id {
            self.tao_windows.get(wid)
        } else {
            self.tao_windows.values().next()
        };

        if let Some(window) = target_window {
            #[cfg(target_os = "macos")]
            {
                use tao::platform::macos::WindowExtMacOS;
                unsafe {
                    menu.show_context_menu_for_nsview(
                        window.ns_view() as _,
                        None::<muda::dpi::Position>,
                    );
                }
            }
            #[cfg(target_os = "windows")]
            {
                use tao::platform::windows::WindowExtWindows;
                unsafe {
                    let _ = menu.show_context_menu_for_hwnd(
                        window.hwnd() as _,
                        None::<muda::dpi::Position>,
                    );
                }
            }
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                use tao::platform::unix::WindowExtUnix;
                let gtk_win: &gtk::Window = window.gtk_window().upcast_ref();
                menu.show_context_menu_for_gtk_window(gtk_win, None::<muda::dpi::Position>);
            }

            tracing::debug!("Showed context menu with {} items", items.len());
        } else {
            tracing::warn!("No window found to show context menu");
            let mut pending = self.pending_ctx_menu.lock().unwrap();
            if let Some((_, sender, _)) = pending.take() {
                let _ = sender.send(None);
            }
        }
    }

    // =========================================================================
    // Tray
    // =========================================================================

    /// Create a system tray icon
    pub fn create_tray(&mut self, opts: TrayOpts) -> String {
        use tray_icon::{Icon, TrayIconBuilder};

        self.tray_counter += 1;
        let tray_id_str = format!("tray-{}", self.tray_counter);

        let icon = if let Some(ref icon_path) = opts.icon {
            let full_path = if std::path::Path::new(icon_path).is_absolute() {
                std::path::PathBuf::from(icon_path)
            } else {
                self.config.app_dir.join(icon_path)
            };

            match std::fs::read(&full_path) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let resized =
                            img.resize_exact(22, 22, image::imageops::FilterType::Lanczos3);
                        let rgba = resized.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        Icon::from_rgba(rgba.into_raw(), width, height)
                            .unwrap_or_else(|_| create_default_tray_icon())
                    }
                    Err(_) => create_default_tray_icon(),
                },
                Err(_) => create_default_tray_icon(),
            }
        } else {
            create_default_tray_icon()
        };

        let mut builder = TrayIconBuilder::new().with_icon(icon);

        if let Some(ref tooltip) = opts.tooltip {
            builder = builder.with_tooltip(tooltip);
        }

        if let Some(ref menu_items) = opts.menu {
            if !menu_items.is_empty() {
                let menu = muda::Menu::new();
                {
                    let mut map = self.menu_id_map.lock().unwrap();
                    add_tray_menu_items(&menu, menu_items, &mut map, &tray_id_str);
                }
                builder = builder.with_menu(Box::new(menu));
            }
        }

        match builder.build() {
            Ok(tray) => {
                self.trays.insert(tray_id_str.clone(), tray);
                tracing::debug!("Created tray icon: {}", tray_id_str);
                tray_id_str
            }
            Err(e) => {
                tracing::error!("Failed to create tray: {}", e);
                String::new()
            }
        }
    }

    /// Update an existing tray icon
    pub fn update_tray(&mut self, tray_id: &str, opts: TrayOpts) -> bool {
        if let Some(tray) = self.trays.get_mut(tray_id) {
            if let Some(ref tooltip) = opts.tooltip {
                let _ = tray.set_tooltip(Some(tooltip));
            }
            tracing::debug!("Updated tray: {}", tray_id);
            true
        } else {
            tracing::warn!("Tray not found: {}", tray_id);
            false
        }
    }

    /// Destroy a tray icon
    pub fn destroy_tray(&mut self, tray_id: &str) -> bool {
        if self.trays.remove(tray_id).is_some() {
            tracing::debug!("Destroyed tray: {}", tray_id);
            true
        } else {
            tracing::warn!("Tray not found: {}", tray_id);
            false
        }
    }

    // =========================================================================
    // Native Handle
    // =========================================================================

    /// Get native window handle
    pub fn get_native_handle(&self, window_id: &str) -> Result<NativeHandle, String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            #[cfg(target_os = "macos")]
            {
                use tao::platform::macos::WindowExtMacOS;
                let handle = window.ns_view() as u64;
                return Ok(NativeHandle {
                    platform: "macos".to_string(),
                    handle,
                });
            }
            #[cfg(target_os = "windows")]
            {
                use tao::platform::windows::WindowExtWindows;
                let handle = window.hwnd() as u64;
                return Ok(NativeHandle {
                    platform: "windows".to_string(),
                    handle,
                });
            }
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::*;
                use tao::platform::unix::WindowExtUnix;
                let gtk_window = window.gtk_window();
                let handle = if let Some(gdk_window) = gtk_window.window() {
                    gdk_window.as_ptr() as u64
                } else {
                    gtk_window.as_ptr() as u64
                };
                return Ok(NativeHandle {
                    platform: "linux".to_string(),
                    handle,
                });
            }
            #[allow(unreachable_code)]
            Err("Platform not supported".to_string())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    // =========================================================================
    // Enhanced Window Operations
    // =========================================================================

    /// Open developer tools for a window
    pub fn open_devtools(&self, window_id: &str) -> Result<(), String> {
        if let Some(webview) = self.webviews.get(window_id) {
            #[cfg(any(debug_assertions, feature = "devtools"))]
            {
                webview.open_devtools();
                Ok(())
            }
            #[cfg(not(any(debug_assertions, feature = "devtools")))]
            {
                let _ = webview;
                Err("DevTools not supported on this platform/build".to_string())
            }
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Close developer tools for a window
    pub fn close_devtools(&self, window_id: &str) -> Result<(), String> {
        if let Some(webview) = self.webviews.get(window_id) {
            #[cfg(any(debug_assertions, feature = "devtools"))]
            {
                webview.close_devtools();
                Ok(())
            }
            #[cfg(not(any(debug_assertions, feature = "devtools")))]
            {
                let _ = webview;
                Err("DevTools not supported on this platform/build".to_string())
            }
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Check if developer tools are open for a window
    pub fn is_devtools_open(&self, window_id: &str) -> Result<bool, String> {
        if let Some(webview) = self.webviews.get(window_id) {
            #[cfg(any(debug_assertions, feature = "devtools"))]
            {
                Ok(webview.is_devtools_open())
            }
            #[cfg(not(any(debug_assertions, feature = "devtools")))]
            {
                let _ = webview;
                Ok(false)
            }
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Evaluate JavaScript in a window's WebView
    pub fn eval_js(&self, window_id: &str, script: &str) -> Result<(), String> {
        if let Some(webview) = self.webviews.get(window_id) {
            webview
                .evaluate_script(script)
                .map_err(|e| format!("Failed to evaluate JavaScript: {}", e))?;
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Inject CSS into a window's WebView
    pub fn inject_css(&self, window_id: &str, css: &str) -> Result<(), String> {
        if let Some(webview) = self.webviews.get(window_id) {
            // Use JavaScript to inject CSS
            let escaped_css = css.replace('\\', "\\\\").replace('`', "\\`");
            let js = format!(
                r#"(function() {{
                    const style = document.createElement('style');
                    style.textContent = `{}`;
                    document.head.appendChild(style);
                }})();"#,
                escaped_css
            );
            webview
                .evaluate_script(&js)
                .map_err(|e| format!("Failed to inject CSS: {}", e))?;
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Set minimum window size
    pub fn set_min_size(&self, window_id: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_min_inner_size(Some(tao::dpi::LogicalSize::new(width, height)));
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Set maximum window size
    pub fn set_max_size(&self, window_id: &str, width: u32, height: u32) -> Result<(), String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            window.set_max_inner_size(Some(tao::dpi::LogicalSize::new(width, height)));
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Center window on screen
    pub fn center_window(&self, window_id: &str) -> Result<(), String> {
        if let Some(window) = self.tao_windows.get(window_id) {
            if let Some(monitor) = window.current_monitor() {
                let screen_size = monitor.size();
                let window_size = window.outer_size();
                let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                let y = (screen_size.height as i32 - window_size.height as i32) / 2;
                let monitor_pos = monitor.position();
                window.set_outer_position(tao::dpi::PhysicalPosition::new(
                    monitor_pos.x + x,
                    monitor_pos.y + y,
                ));
            }
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    /// Get information about all available monitors
    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        // Get monitors from any available window
        if let Some(window) = self.tao_windows.values().next() {
            window
                .available_monitors()
                .map(|monitor| {
                    let position = monitor.position();
                    let size = monitor.size();
                    MonitorInfo {
                        name: monitor.name(),
                        position: Position {
                            x: position.x,
                            y: position.y,
                        },
                        size: Size {
                            width: size.width,
                            height: size.height,
                        },
                        scale_factor: monitor.scale_factor(),
                        is_primary: window
                            .primary_monitor()
                            .map(|p| p.name() == monitor.name())
                            .unwrap_or(false),
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Window Events (called from event loop)
    // =========================================================================

    /// Handle window close requested event
    pub fn handle_close_requested(&mut self, tao_window_id: WindowId) -> bool {
        if let Some(win_id) = self.windows.remove(&tao_window_id) {
            let _ = self.to_deno_tx.try_send(IpcEvent {
                window_id: win_id.clone(),
                channel: "__window__".to_string(),
                payload: serde_json::json!({}),
                event_type: Some("close".to_string()),
            });
            self.webviews.remove(&win_id);
            self.tao_windows.remove(&win_id);
            self.window_channels.remove(&win_id);
            tracing::debug!("Window {} closed", win_id);
            true
        } else {
            false
        }
    }

    /// Handle window focus changed event
    pub fn handle_focus_changed(&self, tao_window_id: WindowId, focused: bool) {
        if let Some(win_id) = self.windows.get(&tao_window_id) {
            let event_type = if focused { "focus" } else { "blur" };
            let _ = self.to_deno_tx.try_send(IpcEvent {
                window_id: win_id.clone(),
                channel: "__window__".to_string(),
                payload: serde_json::json!({}),
                event_type: Some(event_type.to_string()),
            });
            tracing::debug!("Window {} {}", win_id, event_type);
        }
    }

    /// Handle window resized event
    pub fn handle_resized(&self, tao_window_id: WindowId, width: u32, height: u32) {
        if let Some(win_id) = self.windows.get(&tao_window_id) {
            let _ = self.to_deno_tx.try_send(IpcEvent {
                window_id: win_id.clone(),
                channel: "__window__".to_string(),
                payload: serde_json::json!({
                    "width": width,
                    "height": height
                }),
                event_type: Some("resize".to_string()),
            });
            tracing::debug!("Window {} resized to {}x{}", win_id, width, height);
        }
    }

    /// Handle window moved event
    pub fn handle_moved(&self, tao_window_id: WindowId, x: i32, y: i32) {
        if let Some(win_id) = self.windows.get(&tao_window_id) {
            let _ = self.to_deno_tx.try_send(IpcEvent {
                window_id: win_id.clone(),
                channel: "__window__".to_string(),
                payload: serde_json::json!({
                    "x": x,
                    "y": y
                }),
                event_type: Some("move".to_string()),
            });
            tracing::debug!("Window {} moved to ({}, {})", win_id, x, y);
        }
    }

    /// Handle a WindowCmd from ext_window ops
    /// This dispatches the command to the appropriate method
    pub fn handle_cmd(&mut self, cmd: WindowCmd, event_loop_target: &EventLoopWindowTarget<U>) {
        match cmd {
            // Window lifecycle
            WindowCmd::Create { opts, respond } => {
                tracing::debug!(
                    "WindowCmd::Create url={:?} size={:?}x{:?}",
                    opts.url,
                    opts.width,
                    opts.height
                );
                println!("WindowCmd::Create received");
                let result = self.create_window(event_loop_target, opts);
                match &result {
                    Ok(win_id) => tracing::info!("WindowManager created window {}", win_id),
                    Err(err) => tracing::error!("WindowManager failed to create window: {}", err),
                }
                let _ = respond.send(result);
            }
            WindowCmd::Close { window_id, respond } => {
                tracing::debug!("WindowCmd::Close {}", window_id);
                let result = self.close_window(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::Minimize { window_id } => {
                tracing::debug!("WindowCmd::Minimize {}", window_id);
                self.minimize_window(&window_id);
            }
            WindowCmd::Maximize { window_id } => {
                self.maximize_window(&window_id);
            }
            WindowCmd::Unmaximize { window_id } => {
                self.unmaximize_window(&window_id);
            }
            WindowCmd::Restore { window_id } => {
                self.restore_window(&window_id);
            }
            WindowCmd::SetFullscreen {
                window_id,
                fullscreen,
            } => {
                self.set_fullscreen(&window_id, fullscreen);
            }
            WindowCmd::Focus { window_id } => {
                self.focus_window(&window_id);
            }

            // Window properties
            WindowCmd::GetPosition { window_id, respond } => {
                let result = self.get_position(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::SetPosition { window_id, x, y } => {
                self.set_position(&window_id, x, y);
            }
            WindowCmd::GetSize { window_id, respond } => {
                let result = self.get_size(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::SetSize {
                window_id,
                width,
                height,
            } => {
                self.set_size(&window_id, width, height);
            }
            WindowCmd::GetTitle { window_id, respond } => {
                let result = self.get_title(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::SetTitle { window_id, title } => {
                self.set_title(&window_id, &title);
            }
            WindowCmd::SetResizable {
                window_id,
                resizable,
            } => {
                self.set_resizable(&window_id, resizable);
            }
            WindowCmd::SetDecorations {
                window_id,
                decorations,
            } => {
                self.set_decorations(&window_id, decorations);
            }
            WindowCmd::SetAlwaysOnTop {
                window_id,
                always_on_top,
            } => {
                self.set_always_on_top(&window_id, always_on_top);
            }
            WindowCmd::SetVisible { window_id, visible } => {
                self.set_visible(&window_id, visible);
            }

            // State queries
            WindowCmd::GetState { window_id, respond } => {
                let result = self.get_state(&window_id);
                let _ = respond.send(result);
            }

            // Dialogs
            WindowCmd::ShowOpenDialog { opts, respond } => {
                let result = self.show_open_dialog(opts);
                let _ = respond.send(result);
            }
            WindowCmd::ShowSaveDialog { opts, respond } => {
                let result = self.show_save_dialog(opts);
                let _ = respond.send(result);
            }
            WindowCmd::ShowMessageDialog { opts, respond } => {
                let result = self.show_message_dialog(opts);
                let _ = respond.send(result);
            }

            // Menus
            WindowCmd::SetAppMenu { items, respond } => {
                let result = self.set_app_menu(items);
                let _ = respond.send(result);
            }
            WindowCmd::ShowContextMenu {
                window_id,
                items,
                respond,
            } => {
                self.show_context_menu(window_id.as_deref(), items, respond);
            }

            // Tray
            WindowCmd::CreateTray { opts, respond } => {
                let result = self.create_tray(opts);
                let _ = respond.send(result);
            }
            WindowCmd::UpdateTray {
                tray_id,
                opts,
                respond,
            } => {
                let result = self.update_tray(&tray_id, opts);
                let _ = respond.send(result);
            }
            WindowCmd::DestroyTray { tray_id, respond } => {
                let result = self.destroy_tray(&tray_id);
                let _ = respond.send(result);
            }

            // Native handle
            WindowCmd::GetNativeHandle { window_id, respond } => {
                let result = self.get_native_handle(&window_id);
                let _ = respond.send(result);
            }

            // Enhanced Window Operations
            WindowCmd::OpenDevTools { window_id, respond } => {
                let result = self.open_devtools(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::CloseDevTools { window_id, respond } => {
                let result = self.close_devtools(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::IsDevToolsOpen { window_id, respond } => {
                let result = self.is_devtools_open(&window_id);
                let _ = respond.send(result);
            }
            WindowCmd::EvalJs {
                window_id,
                script,
                respond,
            } => {
                let result = self.eval_js(&window_id, &script);
                let _ = respond.send(result);
            }
            WindowCmd::InjectCss {
                window_id,
                css,
                respond,
            } => {
                let result = self.inject_css(&window_id, &css);
                let _ = respond.send(result);
            }
            WindowCmd::SetMinSize {
                window_id,
                width,
                height,
            } => {
                let _ = self.set_min_size(&window_id, width, height);
            }
            WindowCmd::SetMaxSize {
                window_id,
                width,
                height,
            } => {
                let _ = self.set_max_size(&window_id, width, height);
            }
            WindowCmd::Center { window_id } => {
                let _ = self.center_window(&window_id);
            }
            WindowCmd::GetMonitors { respond } => {
                let result = self.get_monitors();
                let _ = respond.send(result);
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn is_ts_path(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default(),
        "ts" | "tsx" | "mts" | "cts"
    )
}

fn transpile_ts_to_js(path: &str, source: String) -> Result<String, String> {
    let specifier = ModuleSpecifier::parse(&format!("app://{}", path))
        .map_err(|e| format!("Invalid specifier: {e}"))?;
    let media_type = MediaType::from_path(Path::new(path));

    let parsed = deno_ast::parse_module(ParseParams {
        specifier,
        text: source.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(|e| e.to_string())?;

    parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &deno_ast::EmitOptions::default(),
        )
        .map_err(|e| e.to_string())
        .map(|output| output.into_source().text)
}

fn maybe_transpile_ts(path: &str, bytes: Vec<u8>) -> (String, Vec<u8>) {
    let content_type = mime_for(path).to_string();

    if !is_ts_path(path) {
        return (content_type, bytes);
    }

    match String::from_utf8(bytes.clone()) {
        Ok(source) => match transpile_ts_to_js(path, source.clone()) {
            Ok(js) => (
                "text/javascript; charset=utf-8".to_string(),
                js.into_bytes(),
            ),
            Err(_) => (content_type, source.into_bytes()),
        },
        Err(_) => (content_type, bytes),
    }
}

/// Get MIME type for a file path based on extension
pub fn mime_for(path: &str) -> &'static str {
    if let Some(ext) = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        match ext {
            "html" | "htm" => "text/html; charset=utf-8",
            "js" | "mjs" => "text/javascript; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "json" => "application/json",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "txt" => "text/plain; charset=utf-8",
            "wasm" => "application/wasm",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}

/// Create a default gray tray icon (22x22 pixels)
pub fn create_default_tray_icon() -> tray_icon::Icon {
    let size = 22u32;
    let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..(size * size) {
        rgba_data.extend_from_slice(&[128, 128, 128, 255]);
    }
    tray_icon::Icon::from_rgba(rgba_data, size, size).expect("Failed to create default icon")
}

/// Add menu items to a Menu and track their IDs for event handling
pub fn add_menu_items_with_tracking(
    menu: &muda::Menu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = menu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref submenu_items) = item.submenu {
            let submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_submenu_items_with_tracking(&submenu, submenu_items, id_map);
            let _ = menu.append(&submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
            let _ = menu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
            let _ = menu.append(&menu_item);
        }
    }
}

/// Add menu items to a Submenu and track their IDs for event handling
pub fn add_submenu_items_with_tracking(
    submenu: &muda::Submenu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = submenu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref nested_items) = item.submenu {
            let nested_submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_submenu_items_with_tracking(&nested_submenu, nested_items, id_map);
            let _ = submenu.append(&nested_submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
            let _ = submenu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
            let _ = submenu.append(&menu_item);
        }
    }
}

/// Add context menu items and track their IDs for event handling
pub fn add_context_menu_items(
    menu: &muda::Menu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
    ctx_ids: &mut HashSet<muda::MenuId>,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = menu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref nested_items) = item.submenu {
            let submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_context_submenu_items(&submenu, nested_items, id_map, ctx_ids);
            let _ = menu.append(&submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
            ctx_ids.insert(check_item.id().clone());
            let _ = menu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
            ctx_ids.insert(menu_item.id().clone());
            let _ = menu.append(&menu_item);
        }
    }
}

/// Add context menu submenu items and track their IDs for event handling
pub fn add_context_submenu_items(
    submenu: &muda::Submenu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
    ctx_ids: &mut HashSet<muda::MenuId>,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = submenu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref nested_items) = item.submenu {
            let nested_submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_context_submenu_items(&nested_submenu, nested_items, id_map, ctx_ids);
            let _ = submenu.append(&nested_submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
            ctx_ids.insert(check_item.id().clone());
            let _ = submenu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
            ctx_ids.insert(menu_item.id().clone());
            let _ = submenu.append(&menu_item);
        }
    }
}

/// Add tray menu items and track their IDs for event handling
pub fn add_tray_menu_items(
    menu: &muda::Menu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
    tray_id: &str,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = menu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref submenu_items) = item.submenu {
            let submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_tray_submenu_items(&submenu, submenu_items, id_map, tray_id);
            let _ = menu.append(&submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            let menu_id = format!("{}:{}", tray_id, user_id);
            id_map.insert(check_item.id().clone(), (menu_id, item.label.clone()));
            let _ = menu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            let menu_id = format!("{}:{}", tray_id, user_id);
            id_map.insert(menu_item.id().clone(), (menu_id, item.label.clone()));
            let _ = menu.append(&menu_item);
        }
    }
}

/// Add tray submenu items and track their IDs for event handling
pub fn add_tray_submenu_items(
    submenu: &muda::Submenu,
    items: &[MenuItem],
    id_map: &mut HashMap<muda::MenuId, (String, String)>,
    tray_id: &str,
) {
    for item in items {
        if item.item_type.as_deref() == Some("separator") {
            let _ = submenu.append(&muda::PredefinedMenuItem::separator());
        } else if let Some(ref nested_items) = item.submenu {
            let nested_submenu = muda::Submenu::new(&item.label, item.enabled.unwrap_or(true));
            add_tray_submenu_items(&nested_submenu, nested_items, id_map, tray_id);
            let _ = submenu.append(&nested_submenu);
        } else if item.item_type.as_deref() == Some("checkbox") {
            let check_item = muda::CheckMenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.checked.unwrap_or(false),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            let menu_id = format!("{}:{}", tray_id, user_id);
            id_map.insert(check_item.id().clone(), (menu_id, item.label.clone()));
            let _ = submenu.append(&check_item);
        } else {
            let menu_item = muda::MenuItem::new(
                &item.label,
                item.enabled.unwrap_or(true),
                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
            );
            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
            let menu_id = format!("{}:{}", tray_id, user_id);
            id_map.insert(menu_item.id().clone(), (menu_id, item.label.clone()));
            let _ = submenu.append(&menu_item);
        }
    }
}
