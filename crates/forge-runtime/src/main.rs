//! Forge Runtime - Desktop application runtime using Deno + WebView
//!
//! The Forge runtime embeds a Deno JavaScript runtime within a native window,
//! enabling desktop applications built with web technologies while maintaining
//! full access to system APIs through Rust extensions.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              Tao Event Loop (OS)                │
//! └───────────┬─────────────────────┬───────────────┘
//!             │                     │
//!     ┌───────▼────────┐    ┌──────▼─────────┐
//!     │ Deno JsRuntime │    │  Wry WebView   │
//!     │  (src/main.ts) │◄──►│ (web/index.html)│
//!     └────────┬───────┘    └────────────────┘
//!              │
//!      ┌───────┴────────┐
//!      │   Extensions   │
//!      │  (runtime:*)   │
//!      └────────────────┘
//! ```
//!
//! # Initialization Flow
//!
//! 1. **Parse manifest.app.toml** - Read app configuration, permissions
//! 2. **Initialize capabilities** - Create permission adapters from manifest
//! 3. **Build extension registry** - Register all ext_* crates by tier (0-3)
//! 4. **Create Deno runtime** - Instantiate JsRuntime with extensions
//! 5. **Setup module loader** - Map `runtime:*` specifiers to extensions
//! 6. **Execute src/main.ts** - Run app entry point
//! 7. **Create event loop** - Start tao event loop with windows
//! 8. **Asset serving** - Serve web/ via `app://` protocol
//!
//! # Extension System
//!
//! Extensions are initialized in tiers based on complexity:
//! - **Tier 0** (ExtensionOnly): No state (e.g., ext_timers)
//! - **Tier 1** (SimpleState): Basic state (e.g., ext_fs, ext_net)
//! - **Tier 2** (CapabilityBased): Requires capability adapters (most extensions)
//! - **Tier 3** (ComplexContext): Needs channels, app info (ext_ipc, ext_window)
//!
//! See [`ext_registry`] for the complete extension registry.
//!
//! # IPC Communication
//!
//! **Renderer → Deno:**
//! ```text
//! window.host.send(channel, data)
//!   → WebView IPC
//!   → mpsc channel
//!   → Deno runtime.windowEvents()
//! ```
//!
//! **Deno → Renderer:**
//! ```text
//! runtime.sendToWindow(windowId, channel, data)
//!   → evaluate_script()
//!   → window.__host_dispatch(channel, data)
//! ```
//!
//! # Hot Module Reload (Dev Mode)
//!
//! When launched with `--dev`:
//! - WebSocket server on port 3001
//! - File watcher on `web/` directory
//! - Auto-reload on changes
//! - All permissions allowed
//!
//! # Asset Serving
//!
//! **Dev Mode:** Assets served from filesystem (`web/` directory)
//! **Production:** Assets embedded in binary (via `FORGE_EMBED_DIR` at build time)
//!
//! See [`ForgeAssetProvider`] for implementation.
//!
//! # Module Loading
//!
//! The [`ForgeModuleLoader`] maps specifiers:
//! - `runtime:fs` → `ext:runtime_fs/init.js`
//! - `runtime:window` → `ext:runtime_window/init.js`
//! - File URLs → Filesystem paths
//!
//! # Environment Variables
//!
//! - `FORGE_LOG` - Log level (default: "info")
//! - `RUST_BACKTRACE` - Enable backtraces on panic
//!
//! # Examples
//!
//! See `examples/` directory for sample apps:
//! - `examples/react-app` - React-based UI
//! - `examples/svelte-app` - Svelte components
//! - `examples/text-editor` - Simple editor

use anyhow::{Context, Result};
use deno_ast::{MediaType, ParseParams};
use deno_core::error::ModuleLoaderError;
use deno_core::{
    JsRuntime, ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleSourceCode,
    ModuleSpecifier, ResolutionKind, RuntimeOptions,
};
use futures_util::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::WindowBuilder;
use tokio_tungstenite::tungstenite::Message;
use wry::http::{Response, StatusCode};
use wry::WebViewBuilder;

use ext_ipc::{IpcEvent, ToRendererCmd};
use ext_window::{
    add_context_menu_items, add_menu_items_with_tracking, add_tray_menu_items,
    create_default_tray_icon, mime_for, AssetProvider, ChannelChecker,
    FileDialogOpts as WinFileDialogOpts, MenuEvent as WinMenuEvent, MenuItem as WinMenuItem,
    MessageDialogOpts as WinMessageDialogOpts, TrayOpts as WinTrayOpts, WindowCmd, WindowManager,
    WindowManagerConfig, WindowOpts, WindowSystemEvent, CONTEXT_MENU_TIMEOUT_SECS,
};

mod capabilities;
mod crash;
mod ext_registry;

use capabilities::{create_capability_adapters, Capabilities, Permissions};
use ext_registry::{ExtensionInitContext, ExtensionRegistry};

/// Application manifest (manifest.app.toml)
///
/// Defines app metadata, window configuration, and permissions.
/// Loaded at runtime startup.
#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    /// App metadata (name, version, identifier)
    pub app: App,
    /// Default window configuration (optional)
    pub windows: Option<Windows>,
    /// Permissions/capabilities section. Accepts both `permissions` and `capabilities` keys.
    #[serde(alias = "capabilities")]
    pub permissions: Option<Permissions>,
}
/// Application metadata
///
/// Required fields for app identification and distribution.
#[derive(Debug, Deserialize, Clone)]
pub struct App {
    /// Display name of the application
    pub name: String,
    /// Unique identifier (reverse-DNS format, e.g., "com.example.myapp")
    pub identifier: String,
    /// Semantic version (e.g., "1.0.0")
    pub version: String,
    /// Enable crash reporting (default: false)
    pub crash_reporting: Option<bool>,
    /// Directory for crash reports (default: OS temp dir)
    pub crash_report_dir: Option<String>,
}
/// Default window configuration
///
/// Settings applied to windows created without explicit options.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Windows {
    /// Default window width in pixels (default: 800)
    pub width: Option<u32>,
    /// Default window height in pixels (default: 600)
    pub height: Option<u32>,
    /// Whether windows are resizable (default: true)
    pub resizable: Option<bool>,
}

fn preload_js() -> &'static str {
    // Generated from sdk/preload.ts at build time (transpiled to JS)
    include_str!(concat!(env!("OUT_DIR"), "/preload.js"))
}

// Include generated assets module (for release builds with embedded assets)
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

// ============================================================================
// WindowManager Support Types
// ============================================================================

/// Asset provider for WindowManager
///
/// Serves web assets from embedded data (production) or filesystem (dev).
struct ForgeAssetProvider {
    /// Base directory for filesystem assets
    app_dir: PathBuf,
}

impl AssetProvider for ForgeAssetProvider {
    fn get_asset(&self, path: &str) -> Option<Vec<u8>> {
        // First try embedded assets
        if ASSET_EMBEDDED {
            if let Some(bytes) = get_asset(path) {
                return Some(bytes.to_vec());
            }
        }
        // Fallback to filesystem
        let file_path = self.app_dir.join("web").join(path);
        std::fs::read(&file_path).ok()
    }

    fn is_embedded(&self) -> bool {
        ASSET_EMBEDDED
    }
}

/// Channel checker wrapper for WindowManager
///
/// Enforces IPC channel permissions based on manifest configuration.
struct ForgeChannelChecker {
    /// Capability checker with compiled permission rules
    capabilities: Capabilities,
}

impl ChannelChecker for ForgeChannelChecker {
    fn check_channel(&self, channel: &str, allowed: Option<&[String]>) -> Result<(), String> {
        self.capabilities
            .check_channel(channel, allowed)
            .map_err(|e| e.to_string())
    }
}

// ============================================================================
// Module Loader for ES Modules
// ============================================================================

/// Run lightweight checks against window helpers to ensure host builds match UI expectations.
fn warm_up_window_helpers(app_dir: &Path, app_name: &str, default_channels: &[String]) {
    let title: Cow<'_, str> = Cow::Owned(app_name.to_string());

    // Touch builders and MIME detection so changes surface as build errors, not runtime surprises.
    let _builder = WindowBuilder::new().with_title(title.clone().into_owned());
    let _webview_builder = WebViewBuilder::new();
    let _default_icon = create_default_tray_icon();
    let _response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_for("index.html"))
        .body(Vec::<u8>::new())
        .ok();

    let window_opts = WindowOpts {
        title: Some(title.into_owned()),
        width: Some(800),
        height: Some(600),
        channels: if default_channels.is_empty() {
            None
        } else {
            Some(default_channels.to_vec())
        },
        ..Default::default()
    };
    let _ = window_opts;

    // Prime menu/tray builders with sample data to validate helper wiring.
    let sample_items = vec![WinMenuItem {
        id: Some("quit".to_string()),
        label: "Quit".to_string(),
        accelerator: Some("CmdOrCtrl+Q".to_string()),
        enabled: Some(true),
        checked: None,
        submenu: None,
        item_type: None,
    }];

    let mut id_map: HashMap<muda::MenuId, (String, String)> = HashMap::new();
    let mut ctx_ids: HashSet<muda::MenuId> = HashSet::new();
    let menu = muda::Menu::new();
    add_menu_items_with_tracking(&menu, &sample_items, &mut id_map);
    add_context_menu_items(&menu, &sample_items, &mut id_map, &mut ctx_ids);
    add_tray_menu_items(&menu, &sample_items, &mut HashMap::new(), "tray-warmup");

    let _tray_opts = WinTrayOpts {
        icon: None,
        tooltip: Some(format!("{app_name} running")),
        menu: Some(sample_items.clone()),
    };

    let _file_dialog = WinFileDialogOpts {
        title: Some(format!("Select a file for {app_name}")),
        default_path: Some(app_dir.join("web").to_string_lossy().to_string()),
        filters: None,
        multiple: Some(false),
        directory: Some(false),
    };

    let _message_dialog = WinMessageDialogOpts {
        title: Some(format!("{app_name} status")),
        message: "Forge host initialized".to_string(),
        kind: Some("info".to_string()),
        buttons: Some(vec!["OK".to_string()]),
    };

    let _mime = mime_for("index.html");
}

/// Module loader for Forge runtime
///
/// Maps `runtime:*` specifiers to extension init modules.
/// Handles both extension modules and file URLs.
struct ForgeModuleLoader {
    /// Base directory for resolving file URLs
    #[allow(dead_code)]
    app_dir: PathBuf,
}

impl ForgeModuleLoader {
    fn new(app_dir: PathBuf) -> Self {
        Self { app_dir }
    }
}

impl deno_core::ModuleLoader for ForgeModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        // Handle runtime:* imports by mapping to ext:runtime_*/init.js
        if let Some(module_name) = specifier.strip_prefix("runtime:") {
            let ext_specifier = format!("ext:runtime_{}/init.js", module_name);
            return ModuleSpecifier::parse(&ext_specifier)
                .map_err(|e| ModuleLoaderError::generic(format!("Invalid specifier: {}", e)));
        }

        // For relative imports, resolve against referrer
        deno_core::resolve_import(specifier, referrer)
            .map_err(|e| ModuleLoaderError::generic(e.to_string()))
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        // Extension modules (ext:*) are handled by deno_core automatically
        if module_specifier.scheme() == "ext" {
            return ModuleLoadResponse::Sync(Err(ModuleLoaderError::generic(format!(
                "Extension module should be handled by deno_core: {}",
                module_specifier
            ))));
        }

        let module_specifier = module_specifier.clone();

        ModuleLoadResponse::Sync((move || {
            let path = module_specifier.to_file_path().map_err(|_| {
                ModuleLoaderError::generic(format!(
                    "Cannot convert to file path: {}",
                    module_specifier
                ))
            })?;

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (deno_core::ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::Json => (deno_core::ModuleType::Json, false),
                _ => {
                    return Err(ModuleLoaderError::generic(format!(
                        "Unknown file extension: {:?}",
                        path.extension()
                    )));
                }
            };

            let code = std::fs::read_to_string(&path).map_err(|e| {
                ModuleLoaderError::generic(format!("Failed to read {}: {}", path.display(), e))
            })?;

            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.clone(),
                    text: code.into(),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })
                .map_err(|e| ModuleLoaderError::generic(e.to_string()))?;

                let transpiled = parsed
                    .transpile(
                        &deno_ast::TranspileOptions::default(),
                        &deno_ast::TranspileModuleOptions::default(),
                        &deno_ast::EmitOptions::default(),
                    )
                    .map_err(|e| ModuleLoaderError::generic(e.to_string()))?;

                transpiled.into_source().text
            } else {
                code
            };

            let module = deno_core::ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                &module_specifier,
                None,
            );
            Ok(module)
        })())
    }
}

/// HMR (Hot Module Replacement) server for dev mode
/// Watches web directory for changes and sends reload signals to connected clients
async fn run_hmr_server(port: u16, watch_dir: PathBuf) {
    use tokio::net::TcpListener;
    use tokio::sync::broadcast;

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("HMR server failed to bind to port {}: {}", port, e);
            return;
        }
    };

    tracing::info!("HMR server listening on ws://127.0.0.1:{}", port);

    // Broadcast channel for file change notifications
    let (tx, _) = broadcast::channel::<String>(16);
    let tx_for_watcher = tx.clone();

    // File watcher thread
    let watch_dir_clone = watch_dir.clone();
    std::thread::spawn(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut watcher: RecommendedWatcher =
            match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        for path in event.paths {
                            let path_str = path.display().to_string();
                            let _ = notify_tx.send(path_str);
                        }
                    }
                }
            }) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

        if let Err(e) = watcher.watch(&watch_dir_clone, RecursiveMode::Recursive) {
            tracing::error!("Failed to watch directory: {}", e);
            return;
        }

        tracing::debug!("HMR watching {}", watch_dir_clone.display());

        // Forward file change events to broadcast channel
        for path in notify_rx {
            let msg = if path.ends_with(".css") {
                format!("css:{}", path)
            } else {
                format!("reload:{}", path)
            };
            let _ = tx_for_watcher.send(msg);
        }
    });

    // Accept WebSocket connections
    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            let mut rx = tx.subscribe();

            tokio::spawn(async move {
                let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        tracing::debug!("WebSocket handshake failed for {}: {}", addr, e);
                        return;
                    }
                };

                tracing::debug!("HMR client connected: {}", addr);
                let (mut write, mut read) = ws_stream.split();

                // Handle incoming messages and broadcast changes
                loop {
                    tokio::select! {
                        // Forward file changes to client
                        msg = rx.recv() => {
                            match msg {
                                Ok(change) => {
                                    if write.send(Message::Text(change)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        // Handle client messages (ping/pong, close)
                        client_msg = read.next() => {
                            match client_msg {
                                Some(Ok(Message::Close(_))) | None => break,
                                Some(Ok(Message::Ping(data))) => {
                                    let _ = write.send(Message::Pong(data)).await;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                tracing::debug!("HMR client disconnected: {}", addr);
            });
        }
    }
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    let _guard = rt.enter();

    sync_main(rt)
}

fn sync_main(rt: tokio::runtime::Runtime) -> Result<()> {
    // Initialize tracing with env-filter support
    // Use FORGE_LOG env var for log level configuration, default to "info"
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_env("FORGE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    // Parse args: --app-dir <dir> --dev
    let mut args = env::args().skip(1);
    let mut app_dir: Option<PathBuf> = None;
    let mut dev_mode = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--app-dir" => {
                app_dir = Some(PathBuf::from(
                    args.next().expect("--app-dir requires a path"),
                ));
            }
            "--dev" => {
                dev_mode = true;
            }
            _ => {}
        }
    }

    // Auto-detect app_dir from bundle if not provided via arguments
    let app_dir = match app_dir {
        Some(dir) => dir,
        None => {
            // Try to detect if running from a macOS/Linux bundle
            if let Ok(exe_path) = env::current_exe() {
                let exe_path = exe_path.canonicalize().unwrap_or(exe_path);

                // macOS: /path/to/App.app/Contents/MacOS/binary
                // Resources at: /path/to/App.app/Contents/Resources/app/
                if let Some(macos_dir) = exe_path.parent() {
                    if macos_dir.file_name().is_some_and(|n| n == "MacOS") {
                        if let Some(contents_dir) = macos_dir.parent() {
                            let resources_app = contents_dir.join("Resources").join("app");
                            if resources_app.exists() {
                                tracing::info!(
                                    "Auto-detected app directory: {}",
                                    resources_app.display()
                                );
                                resources_app
                            } else {
                                anyhow::bail!(
                                    "Bundle detected but Resources/app not found at: {}",
                                    resources_app.display()
                                );
                            }
                        } else {
                            anyhow::bail!("Could not find Contents directory in bundle");
                        }
                    } else {
                        anyhow::bail!(
                            "Usage: forge-runtime --app-dir <path> [--dev]\n\
                            Note: When running from a macOS bundle, app_dir is auto-detected."
                        );
                    }
                } else {
                    anyhow::bail!("Usage: forge-runtime --app-dir <path> [--dev]");
                }
            } else {
                anyhow::bail!("Usage: forge-runtime --app-dir <path> [--dev]");
            }
        }
    };

    let manifest_path = app_dir.join("manifest.app.toml");
    let manifest_txt = rt
        .block_on(tokio::fs::read_to_string(&manifest_path))
        .with_context(|| format!("reading manifest at {}", manifest_path.display()))?;
    let manifest: Manifest = toml::from_str(&manifest_txt).context("parsing manifest")?;

    tracing::info!(
        "Starting app: {} v{}",
        manifest.app.name,
        manifest.app.version
    );

    // Initialize crash reporting
    let crash_report_dir = manifest
        .app
        .crash_report_dir
        .clone()
        .unwrap_or_else(|| app_dir.join("crashes").to_string_lossy().to_string());
    crash::init_crash_reporting(
        manifest.app.crash_reporting.unwrap_or(false),
        &crash_report_dir,
        &manifest.app.name,
    );

    // Log crash reporting status
    if crash::is_enabled() {
        if let Some(dir) = crash::get_report_dir() {
            tracing::info!(
                "Crash reporting enabled, reports will be saved to: {}",
                dir.display()
            );
        }
    }

    // Initialize capabilities from manifest permissions
    let capabilities = Capabilities::from_permissions(manifest.permissions.as_ref(), dev_mode)
        .context("initializing capabilities")?;

    if dev_mode {
        tracing::info!("Running in dev mode - all permissions allowed");
    }

    // Create capability adapters for each extension
    let adapters = create_capability_adapters(capabilities.clone());

    // Create IPC channels for Deno <-> Host <-> Renderer communication
    let (to_deno_tx, to_deno_rx) = tokio::sync::mpsc::channel::<IpcEvent>(256);
    let (to_renderer_tx, mut to_renderer_rx) = tokio::sync::mpsc::channel::<ToRendererCmd>(256);

    // Create channels for ext_window (native window operations)
    let (window_cmd_tx, mut window_cmd_rx) = tokio::sync::mpsc::channel::<WindowCmd>(64);
    let (window_events_tx, window_events_rx) = tokio::sync::mpsc::channel::<WindowSystemEvent>(64);
    let (window_menu_events_tx, window_menu_events_rx) =
        tokio::sync::mpsc::channel::<WinMenuEvent>(64);

    // Build Deno runtime with extensions (runtime:*) using the extension registry
    let module_loader = Rc::new(ForgeModuleLoader::new(app_dir.clone()));
    let registry = ExtensionRegistry::new();

    tracing::info!(
        "Extension registry loaded with {} extensions",
        registry.count()
    );

    let mut js = JsRuntime::new(RuntimeOptions {
        module_loader: Some(module_loader),
        extensions: registry.build_extensions(None), // None = all extensions enabled
        ..Default::default()
    });

    // Build app info for extensions
    let app_info = ext_app::AppInfo {
        name: manifest.app.name.clone(),
        version: manifest.app.version.clone(),
        identifier: manifest.app.identifier.clone(),
        is_packaged: false, // TODO: detect packaged mode
        exe_path: std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().to_string()),
        resource_path: Some(app_dir.to_string_lossy().to_string()),
    };

    // Initialize all extension state using the registry
    {
        let op_state = js.op_state();
        let mut state = op_state.borrow_mut();

        // Initialize IPC and Window manually (they consume channel receivers)
        ext_registry::init_ipc_manually(
            &mut state,
            to_renderer_tx.clone(),
            to_deno_rx,
            Some(&adapters),
        );

        ext_registry::init_window_manually(
            &mut state,
            window_cmd_tx.clone(),
            window_events_rx,
            window_menu_events_rx,
            Some(&adapters),
        );

        // Build initialization context for remaining extensions
        let init_ctx = ExtensionInitContext {
            adapters: Some(adapters),
            capabilities: Some(std::sync::Arc::new(capabilities.clone())),
            ipc: None,    // Already consumed above
            window: None, // Already consumed above
            app_info: Some(app_info.clone()),
            dev_mode,
        };

        // Initialize all other extension states via registry (skips ipc/window)
        if let Err(e) = registry.init_all_states(&mut state, &init_ctx) {
            tracing::warn!("Extension state init warning: {}", e);
            // Continue - some extensions (ipc, window) are expected to skip
        }
    }

    // Load the app's main.ts as an ES module (but don't evaluate yet)
    let main_ts_path = app_dir
        .join("src/main.ts")
        .canonicalize()
        .with_context(|| {
            format!(
                "Cannot find main.ts at {}",
                app_dir.join("src/main.ts").display()
            )
        })?;
    let main_specifier = ModuleSpecifier::from_file_path(&main_ts_path)
        .map_err(|_| anyhow::anyhow!("Invalid path: {}", main_ts_path.display()))?;

    tracing::info!("Executing {}", main_ts_path.display());

    // Load the main module
    let module_id = rt.block_on(js.load_main_es_module(&main_specifier))?;

    // Custom user events for the tao event loop
    #[derive(Debug)]
    enum UserEvent {
        // IPC: Deno -> Renderer
        ToRenderer(ToRendererCmd),
        // ext_window commands (runtime:window) - handled by WindowManager
        WindowCmd(WindowCmd),
    }

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Start HMR server in dev mode
    if dev_mode {
        let hmr_port = 35729;
        let web_dir_for_hmr = app_dir.join("web");
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create HMR runtime");
            rt.block_on(async move {
                run_hmr_server(hmr_port, web_dir_for_hmr).await;
            });
        });
    }

    // Spawn task: forward renderer commands from Deno to event loop
    thread::spawn({
        let proxy = proxy.clone();
        move || {
            while let Some(cmd) = to_renderer_rx.blocking_recv() {
                if let Err(e) = proxy.send_event(UserEvent::ToRenderer(cmd)) {
                    tracing::error!("Failed to send ToRenderer event: {:?}", e);
                }
            }
        }
    });

    // Spawn task: forward ext_window WindowCmd to event loop
    thread::spawn({
        let proxy = proxy.clone();
        move || {
            while let Some(cmd) = window_cmd_rx.blocking_recv() {
                println!("forge_runtime: forwarding WindowCmd {:?}", cmd);
                tracing::debug!(
                    "forge_runtime: forwarding WindowCmd to event loop: {:?}",
                    cmd
                );
                if let Err(e) = proxy.send_event(UserEvent::WindowCmd(cmd)) {
                    tracing::error!("Failed to send WindowCmd event: {:?}", e);
                }
            }
        }
    });

    // Clone app_dir for use in the event loop closure
    let app_dir_clone = app_dir.clone();

    // Get default channels from capabilities for new windows
    let default_channels = capabilities.get_default_channels();

    warm_up_window_helpers(
        &app_dir_clone,
        &manifest.app.name,
        default_channels.as_deref().unwrap_or(&[]),
    );

    // Create WindowManager to handle all window operations
    use std::sync::Arc;
    let window_manager_config = WindowManagerConfig {
        app_dir: app_dir.clone(),
        dev_mode,
        app_name: manifest.app.name.clone(),
        default_channels: default_channels.clone(),
    };
    let asset_provider = Arc::new(ForgeAssetProvider {
        app_dir: app_dir.clone(),
    });
    let channel_checker: Option<Arc<dyn ChannelChecker>> = Some(Arc::new(ForgeChannelChecker {
        capabilities: capabilities.clone(),
    }));
    let mut window_manager: WindowManager<UserEvent> = WindowManager::new(
        window_manager_config,
        window_events_tx.clone(),
        to_deno_tx.clone(),
        window_cmd_tx.clone(),
        channel_checker,
        preload_js().to_string(),
        asset_provider,
    );

    // Get shared state from WindowManager for menu event thread
    let menu_id_map = window_manager.menu_id_map();
    let pending_ctx_menu = window_manager.pending_ctx_menu();

    // We'll use the runtime directly in the event loop for polling
    // The spawned tasks use the runtime context from rt.enter() in main()

    // Track module evaluation state
    let mut module_eval_started = false;
    let mut module_eval_done = false;
    let mut module_eval_receiver = None;

    // Set up menu event receiver from muda and forward to Deno
    let menu_id_map_for_thread = menu_id_map.clone();
    let pending_ctx_menu_for_thread = pending_ctx_menu.clone();
    std::thread::spawn(move || {
        use std::time::Duration;
        let receiver = muda::MenuEvent::receiver();
        let timeout = Duration::from_secs(1); // Check every second for stale menus

        loop {
            // Use recv_timeout to periodically check for stale context menus
            match receiver.recv_timeout(timeout) {
                Ok(event) => {
                    // First, check if this is a context menu selection
                    {
                        let mut pending = pending_ctx_menu_for_thread.lock().unwrap();
                        if let Some((ref ids, _, _)) = *pending {
                            if ids.contains(&event.id) {
                                // This is a context menu selection - respond and clear
                                let map = menu_id_map_for_thread.lock().unwrap();
                                if let Some((item_id, label)) = map.get(&event.id) {
                                    tracing::debug!(
                                        "Context menu selection: item_id={}, label={}",
                                        item_id,
                                        label
                                    );
                                    if let Some((_, sender, _)) = pending.take() {
                                        let _ = sender.send(Some(item_id.clone()));
                                    }
                                }
                                continue; // Don't forward to regular menu events
                            }
                        }
                    }

                    // Regular menu event - forward to Deno via ext_window
                    let map = menu_id_map_for_thread.lock().unwrap();
                    if let Some((item_id, label)) = map.get(&event.id) {
                        tracing::debug!("Menu event: item_id={}, label={}", item_id, label);
                        let win_menu_event = WinMenuEvent {
                            menu_id: "app".to_string(),
                            item_id: item_id.clone(),
                            label: label.clone(),
                        };
                        let _ = window_menu_events_tx.blocking_send(win_menu_event);
                    } else {
                        tracing::warn!("Menu event for unknown MenuId: {:?}", event.id);
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Check for stale pending context menus (user dismissed without selecting)
                    let mut pending = pending_ctx_menu_for_thread.lock().unwrap();
                    if let Some((_, _, ref shown_at)) = *pending {
                        if shown_at.elapsed().as_secs() >= CONTEXT_MENU_TIMEOUT_SECS {
                            tracing::debug!(
                                "Context menu timed out after {} seconds",
                                CONTEXT_MENU_TIMEOUT_SECS
                            );
                            if let Some((_, sender, _)) = pending.take() {
                                let _ = sender.send(None);
                            }
                        }
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    tracing::debug!("Menu event channel disconnected");
                    break;
                }
            }
        }
    });

    tracing::debug!("starting tao event loop");
    event_loop.run(move |event, event_loop_target, control| {
        // Use Poll mode so we can continuously poll the JsRuntime
        *control = ControlFlow::Poll;

        tracing::debug!("event_loop tick: {:?}", event);

        match event {
            // Poll the JsRuntime on each iteration when idle
            Event::MainEventsCleared => {
                if !module_eval_started {
                    module_eval_receiver = Some(js.mod_evaluate(module_id));
                    module_eval_started = true;
                    tracing::debug!("Module evaluation started");
                }

                if !module_eval_done {
                    let result = rt.block_on(async {
                        tokio::time::timeout(
                            std::time::Duration::from_millis(10),
                            js.run_event_loop(deno_core::PollEventLoopOptions {
                                wait_for_inspector: false,
                                pump_v8_message_loop: true,
                            }),
                        )
                        .await
                    });

                    match result {
                        Ok(Ok(_)) => {
                            module_eval_done = true;
                            if let Some(eval) = module_eval_receiver.take() {
                                let _ = rt.block_on(eval);
                            }
                            tracing::debug!("Module evaluation completed");
                        }
                        Ok(Err(e)) => {
                            tracing::error!("JsRuntime event loop error: {:?}", e);
                            if let Some(eval) = module_eval_receiver.take() {
                                let _ = rt.block_on(eval);
                            }
                            module_eval_done = true;
                        }
                        Err(_timeout) => {
                            // Timeout - still processing, continue polling
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::ToRenderer(ToRendererCmd::Send {
                window_id,
                channel,
                payload,
            })) => {
                // Use WindowManager's send_to_renderer (handles channel filtering internally)
                window_manager.send_to_renderer(&window_id, &channel, &payload.to_string());
            }

            // =========================================================================
            // Window system events
            // =========================================================================
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                // Delegate to WindowManager (sends close event and cleans up)
                if !window_manager.handle_close_requested(window_id) {
                    tracing::warn!("Close requested for unknown window: {:?}", window_id);
                }

                // Exit if all windows are closed
                if window_manager.is_empty() {
                    *control = ControlFlow::Exit;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Focused(focused),
                window_id,
                ..
            } => {
                if let Some(win_id) = window_manager.get_window_id(&window_id) {
                    let event_type = if focused { "focus" } else { "blur" };
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({}),
                        event_type: Some(event_type.to_string()),
                    });
                    tracing::debug!("Window {} {}", win_id, event_type);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
                ..
            } => {
                if let Some(win_id) = window_manager.get_window_id(&window_id) {
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({
                            "width": size.width,
                            "height": size.height
                        }),
                        event_type: Some("resize".to_string()),
                    });
                    tracing::debug!(
                        "Window {} resized to {}x{}",
                        win_id,
                        size.width,
                        size.height
                    );
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Moved(position),
                window_id,
                ..
            } => {
                if let Some(win_id) = window_manager.get_window_id(&window_id) {
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({
                            "x": position.x,
                            "y": position.y
                        }),
                        event_type: Some("move".to_string()),
                    });
                    tracing::debug!(
                        "Window {} moved to ({}, {})",
                        win_id,
                        position.x,
                        position.y
                    );
                }
            }

            // =========================================================================
            // ext_window event handlers (runtime:window) - delegated to WindowManager
            // =========================================================================
            Event::UserEvent(UserEvent::WindowCmd(cmd)) => {
                tracing::debug!("event_loop: handling WindowCmd {:?}", cmd);
                println!("event_loop: handling WindowCmd");
                window_manager.handle_cmd(cmd, event_loop_target);
            }

            _ => {}
        }
    });
}
