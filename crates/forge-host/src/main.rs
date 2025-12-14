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
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::{WindowBuilder, WindowId};
use tokio_tungstenite::tungstenite::Message;
use wry::http::{Response, StatusCode};
use wry::WebViewBuilder;

use ext_ipc::{init_ipc_capabilities, init_ipc_state, IpcEvent, ToRendererCmd};
use ext_ui::{
    init_ui_capabilities, init_ui_state, FileDialogOpts, FromDenoCmd, MenuEvent, MenuItem,
    MessageDialogOpts, OpenOpts, TrayOpts,
};

use ext_window::{
    init_window_capabilities, init_window_state, FileDialogOpts as WinFileDialogOpts,
    MenuEvent as WinMenuEvent, MenuItem as WinMenuItem, MessageDialogOpts as WinMessageDialogOpts,
    NativeHandle, Position, Size, TrayOpts as WinTrayOpts, WindowCmd, WindowOpts,
    WindowState as WinWindowState, WindowSystemEvent,
};

mod capabilities;
mod crash;
use capabilities::{create_capability_adapters, Capabilities, Permissions};

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    pub app: App,
    pub windows: Option<Windows>,
    pub permissions: Option<Permissions>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct App {
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub crash_reporting: Option<bool>,
    pub crash_report_dir: Option<String>,
}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Windows {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
}

fn preload_js() -> &'static str {
    // Generated from sdk/preload.ts at build time (transpiled to JS)
    include_str!(concat!(env!("OUT_DIR"), "/preload.js"))
}

fn mime_for(path: &str) -> &'static str {
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

// Include generated assets module (for release builds with embedded assets)
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

// ============================================================================
// Module Loader for ES Modules
// ============================================================================

/// Custom module loader that handles:
/// - `host:*` specifiers → maps to extension modules (ext:host_*/init.js)
/// - File paths with TypeScript transpilation
struct ForgeModuleLoader {
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
        // Handle host:* imports by mapping to ext:host_*/init.js
        if let Some(module_name) = specifier.strip_prefix("host:") {
            let ext_specifier = format!("ext:host_{}/init.js", module_name);
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
    // Create tokio runtime manually (not using #[tokio::main])
    // This allows us to call block_on from within the tao event loop
    // without tokio detecting runtime nesting
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    // Use enter() to set up the runtime context for spawning,
    // but don't use block_on so we're not inside a blocking call
    let _guard = rt.enter();

    // Run the sync setup and event loop
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

    let app_dir =
        app_dir.ok_or_else(|| anyhow::anyhow!("Usage: forge-host --app-dir <path> [--dev]"))?;

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
    let (from_deno_tx, mut from_deno_rx) = tokio::sync::mpsc::channel::<FromDenoCmd>(64);
    let (menu_events_tx, menu_events_rx) = tokio::sync::mpsc::channel::<MenuEvent>(64);

    // Create channels for ext_window (native window operations)
    let (window_cmd_tx, mut window_cmd_rx) = tokio::sync::mpsc::channel::<WindowCmd>(64);
    let (window_events_tx, window_events_rx) = tokio::sync::mpsc::channel::<WindowSystemEvent>(64);
    let (window_menu_events_tx, window_menu_events_rx) =
        tokio::sync::mpsc::channel::<WinMenuEvent>(64);

    // Build Deno runtime with extensions (host:*)
    let module_loader = Rc::new(ForgeModuleLoader::new(app_dir.clone()));
    let mut js = JsRuntime::new(RuntimeOptions {
        module_loader: Some(module_loader),
        extensions: vec![
            ext_fs::fs_extension(),
            ext_ipc::ipc_extension(),
            ext_net::net_extension(),
            ext_sys::sys_extension(),
            ext_ui::ui_extension(),
            ext_window::window_extension(),
            ext_process::process_extension(),
            ext_wasm::wasm_extension(),
        ],
        ..Default::default()
    });

    // Initialize all extension state with capability adapters
    {
        let op_state = js.op_state();
        let mut state = op_state.borrow_mut();

        // Initialize IPC state (renderer <-> Deno communication)
        init_ipc_state(&mut state, to_renderer_tx.clone(), to_deno_rx);

        // Initialize UI state (commands from Deno to host, menu events)
        init_ui_state(&mut state, from_deno_tx.clone(), menu_events_rx);

        // Initialize FS state with capability checker
        ext_fs::init_fs_state(&mut state, Some(adapters.fs));

        // Initialize Net state with capability checker
        ext_net::init_net_state(&mut state, Some(adapters.net));

        // Initialize Sys state with capability checker
        ext_sys::init_sys_state(&mut state, Some(adapters.sys));

        // Initialize UI capabilities
        init_ui_capabilities(&mut state, Some(adapters.ui));

        // Initialize IPC capabilities
        init_ipc_capabilities(&mut state, Some(adapters.ipc));

        // Initialize Process state with capability checker
        let max_processes = capabilities.get_max_processes();
        ext_process::init_process_state(&mut state, Some(adapters.process), Some(max_processes));

        // Initialize WASM state with capability checker
        let max_wasm_instances = capabilities.get_max_wasm_instances();
        ext_wasm::init_wasm_state(&mut state, Some(adapters.wasm), Some(max_wasm_instances));

        // Initialize Window state with capability checker
        init_window_state(
            &mut state,
            window_cmd_tx.clone(),
            window_events_rx,
            window_menu_events_rx,
        );
        init_window_capabilities(&mut state, Some(adapters.window));
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

    // Start module evaluation (but don't wait - the tao event loop needs to run concurrently)
    let _eval_receiver = js.mod_evaluate(module_id);

    // Custom user events for the tao event loop
    enum UserEvent {
        ToRenderer(ToRendererCmd),
        CreateWindow(OpenOpts, tokio::sync::oneshot::Sender<String>),
        CloseWindow(String, tokio::sync::oneshot::Sender<bool>),
        SetWindowTitle(String, String),
        ShowOpenDialog(
            FileDialogOpts,
            tokio::sync::oneshot::Sender<Option<Vec<String>>>,
        ),
        ShowSaveDialog(FileDialogOpts, tokio::sync::oneshot::Sender<Option<String>>),
        ShowMessageDialog(MessageDialogOpts, tokio::sync::oneshot::Sender<usize>),
        // Menu events
        SetAppMenu(Vec<MenuItem>, tokio::sync::oneshot::Sender<bool>),
        ShowContextMenu(
            Option<String>,
            Vec<MenuItem>,
            tokio::sync::oneshot::Sender<Option<String>>,
        ),
        // Tray events
        CreateTray(TrayOpts, tokio::sync::oneshot::Sender<String>),
        UpdateTray(String, TrayOpts, tokio::sync::oneshot::Sender<bool>),
        DestroyTray(String, tokio::sync::oneshot::Sender<bool>),

        // === ext_window events ===
        // Window lifecycle (from host:window)
        WinCreate(
            WindowOpts,
            tokio::sync::oneshot::Sender<Result<String, String>>,
        ),
        WinClose(String, tokio::sync::oneshot::Sender<bool>),
        WinMinimize(String),
        WinMaximize(String),
        WinUnmaximize(String),
        WinRestore(String),
        WinSetFullscreen(String, bool),
        WinFocus(String),

        // Window properties
        WinGetPosition(
            String,
            tokio::sync::oneshot::Sender<Result<Position, String>>,
        ),
        WinSetPosition(String, i32, i32),
        WinGetSize(String, tokio::sync::oneshot::Sender<Result<Size, String>>),
        WinSetSize(String, u32, u32),
        WinGetTitle(String, tokio::sync::oneshot::Sender<Result<String, String>>),
        WinSetTitle(String, String),
        WinSetResizable(String, bool),
        WinSetDecorations(String, bool),
        WinSetAlwaysOnTop(String, bool),
        WinSetVisible(String, bool),

        // State queries
        WinGetState(
            String,
            tokio::sync::oneshot::Sender<Result<WinWindowState, String>>,
        ),

        // Dialogs (from host:window)
        WinShowOpenDialog(
            WinFileDialogOpts,
            tokio::sync::oneshot::Sender<Option<Vec<String>>>,
        ),
        WinShowSaveDialog(
            WinFileDialogOpts,
            tokio::sync::oneshot::Sender<Option<String>>,
        ),
        WinShowMessageDialog(WinMessageDialogOpts, tokio::sync::oneshot::Sender<usize>),

        // Menus (from host:window)
        WinSetAppMenu(Vec<WinMenuItem>, tokio::sync::oneshot::Sender<bool>),
        WinShowContextMenu(
            Option<String>,
            Vec<WinMenuItem>,
            tokio::sync::oneshot::Sender<Option<String>>,
        ),

        // Tray (from host:window)
        WinCreateTray(WinTrayOpts, tokio::sync::oneshot::Sender<String>),
        WinUpdateTray(String, WinTrayOpts, tokio::sync::oneshot::Sender<bool>),
        WinDestroyTray(String, tokio::sync::oneshot::Sender<bool>),

        // Native handle
        WinGetNativeHandle(
            String,
            tokio::sync::oneshot::Sender<Result<NativeHandle, String>>,
        ),
    }

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Start HMR server in dev mode
    if dev_mode {
        let hmr_port = 35729;
        let web_dir_for_hmr = app_dir.join("web");
        tokio::spawn(async move {
            run_hmr_server(hmr_port, web_dir_for_hmr).await;
        });
    }

    // Spawn task: forward renderer commands from Deno to event loop
    tokio::task::spawn({
        let proxy = proxy.clone();
        async move {
            while let Some(cmd) = to_renderer_rx.recv().await {
                let _ = proxy.send_event(UserEvent::ToRenderer(cmd));
            }
        }
    });

    // Spawn task: handle Deno commands (CreateWindow, etc.)
    tokio::task::spawn({
        let proxy = proxy.clone();
        async move {
            while let Some(cmd) = from_deno_rx.recv().await {
                match cmd {
                    FromDenoCmd::CreateWindow { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::CreateWindow(opts, respond));
                    }
                    FromDenoCmd::CloseWindow { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::CloseWindow(window_id, respond));
                    }
                    FromDenoCmd::SetWindowTitle { window_id, title } => {
                        let _ = proxy.send_event(UserEvent::SetWindowTitle(window_id, title));
                    }
                    FromDenoCmd::ShowOpenDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::ShowOpenDialog(opts, respond));
                    }
                    FromDenoCmd::ShowSaveDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::ShowSaveDialog(opts, respond));
                    }
                    FromDenoCmd::ShowMessageDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::ShowMessageDialog(opts, respond));
                    }
                    // Menu commands
                    FromDenoCmd::SetAppMenu { items, respond } => {
                        let _ = proxy.send_event(UserEvent::SetAppMenu(items, respond));
                    }
                    FromDenoCmd::ShowContextMenu {
                        window_id,
                        items,
                        respond,
                    } => {
                        let _ =
                            proxy.send_event(UserEvent::ShowContextMenu(window_id, items, respond));
                    }
                    // Tray commands
                    FromDenoCmd::CreateTray { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::CreateTray(opts, respond));
                    }
                    FromDenoCmd::UpdateTray {
                        tray_id,
                        opts,
                        respond,
                    } => {
                        let _ = proxy.send_event(UserEvent::UpdateTray(tray_id, opts, respond));
                    }
                    FromDenoCmd::DestroyTray { tray_id, respond } => {
                        let _ = proxy.send_event(UserEvent::DestroyTray(tray_id, respond));
                    }
                }
            }
        }
    });

    // Spawn task: handle ext_window WindowCmd
    tokio::task::spawn({
        let proxy = proxy.clone();
        async move {
            while let Some(cmd) = window_cmd_rx.recv().await {
                match cmd {
                    // Window lifecycle
                    WindowCmd::Create { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::WinCreate(opts, respond));
                    }
                    WindowCmd::Close { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinClose(window_id, respond));
                    }
                    WindowCmd::Minimize { window_id } => {
                        let _ = proxy.send_event(UserEvent::WinMinimize(window_id));
                    }
                    WindowCmd::Maximize { window_id } => {
                        let _ = proxy.send_event(UserEvent::WinMaximize(window_id));
                    }
                    WindowCmd::Unmaximize { window_id } => {
                        let _ = proxy.send_event(UserEvent::WinUnmaximize(window_id));
                    }
                    WindowCmd::Restore { window_id } => {
                        let _ = proxy.send_event(UserEvent::WinRestore(window_id));
                    }
                    WindowCmd::SetFullscreen {
                        window_id,
                        fullscreen,
                    } => {
                        let _ =
                            proxy.send_event(UserEvent::WinSetFullscreen(window_id, fullscreen));
                    }
                    WindowCmd::Focus { window_id } => {
                        let _ = proxy.send_event(UserEvent::WinFocus(window_id));
                    }

                    // Window properties
                    WindowCmd::GetPosition { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinGetPosition(window_id, respond));
                    }
                    WindowCmd::SetPosition { window_id, x, y } => {
                        let _ = proxy.send_event(UserEvent::WinSetPosition(window_id, x, y));
                    }
                    WindowCmd::GetSize { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinGetSize(window_id, respond));
                    }
                    WindowCmd::SetSize {
                        window_id,
                        width,
                        height,
                    } => {
                        let _ = proxy.send_event(UserEvent::WinSetSize(window_id, width, height));
                    }
                    WindowCmd::GetTitle { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinGetTitle(window_id, respond));
                    }
                    WindowCmd::SetTitle { window_id, title } => {
                        let _ = proxy.send_event(UserEvent::WinSetTitle(window_id, title));
                    }
                    WindowCmd::SetResizable {
                        window_id,
                        resizable,
                    } => {
                        let _ = proxy.send_event(UserEvent::WinSetResizable(window_id, resizable));
                    }
                    WindowCmd::SetDecorations {
                        window_id,
                        decorations,
                    } => {
                        let _ =
                            proxy.send_event(UserEvent::WinSetDecorations(window_id, decorations));
                    }
                    WindowCmd::SetAlwaysOnTop {
                        window_id,
                        always_on_top,
                    } => {
                        let _ = proxy
                            .send_event(UserEvent::WinSetAlwaysOnTop(window_id, always_on_top));
                    }
                    WindowCmd::SetVisible { window_id, visible } => {
                        let _ = proxy.send_event(UserEvent::WinSetVisible(window_id, visible));
                    }

                    // State queries
                    WindowCmd::GetState { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinGetState(window_id, respond));
                    }

                    // Dialogs
                    WindowCmd::ShowOpenDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::WinShowOpenDialog(opts, respond));
                    }
                    WindowCmd::ShowSaveDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::WinShowSaveDialog(opts, respond));
                    }
                    WindowCmd::ShowMessageDialog { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::WinShowMessageDialog(opts, respond));
                    }

                    // Menus
                    WindowCmd::SetAppMenu { items, respond } => {
                        let _ = proxy.send_event(UserEvent::WinSetAppMenu(items, respond));
                    }
                    WindowCmd::ShowContextMenu {
                        window_id,
                        items,
                        respond,
                    } => {
                        let _ = proxy
                            .send_event(UserEvent::WinShowContextMenu(window_id, items, respond));
                    }

                    // Tray
                    WindowCmd::CreateTray { opts, respond } => {
                        let _ = proxy.send_event(UserEvent::WinCreateTray(opts, respond));
                    }
                    WindowCmd::UpdateTray {
                        tray_id,
                        opts,
                        respond,
                    } => {
                        let _ = proxy.send_event(UserEvent::WinUpdateTray(tray_id, opts, respond));
                    }
                    WindowCmd::DestroyTray { tray_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinDestroyTray(tray_id, respond));
                    }

                    // Native handle
                    WindowCmd::GetNativeHandle { window_id, respond } => {
                        let _ = proxy.send_event(UserEvent::WinGetNativeHandle(window_id, respond));
                    }
                }
            }
        }
    });

    // Clone app_dir for use in the event loop closure
    let app_dir_clone = app_dir.clone();

    let mut webviews: HashMap<String, wry::WebView> = HashMap::new();
    let mut windows: HashMap<WindowId, String> = HashMap::new();
    let mut tao_windows: HashMap<String, tao::window::Window> = HashMap::new();
    let mut window_channels: HashMap<String, Option<Vec<String>>> = HashMap::new();
    let mut window_counter: u64 = 0;

    // Tray icon storage (using tray-icon crate)
    let mut trays: HashMap<String, tray_icon::TrayIcon> = HashMap::new();
    let mut tray_counter: u64 = 0;

    // App menu storage (using muda) - kept alive to prevent menu from being dropped
    let mut _app_menu: Option<muda::Menu> = None;

    // Menu ID mapping: muda's internal MenuId -> (user_id, label)
    // This is shared between the menu event thread and the main event loop
    use std::sync::{Arc, Mutex};
    let menu_id_map: Arc<Mutex<HashMap<muda::MenuId, (String, String)>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Pending context menu response: (set of MenuIds for this context menu, response channel)
    // When a menu event matches one of these IDs, we respond and clear the pending state
    type PendingContextMenu = Option<(
        std::collections::HashSet<muda::MenuId>,
        tokio::sync::oneshot::Sender<Option<String>>,
    )>;
    let pending_ctx_menu: Arc<Mutex<PendingContextMenu>> = Arc::new(Mutex::new(None));

    // Get default channels from capabilities for new windows
    let default_channels = capabilities.get_default_channels();

    // We'll use the runtime directly in the event loop for polling
    // The spawned tasks use the runtime context from rt.enter() in main()

    // Track if module evaluation has completed
    let mut module_eval_done = false;

    // Set up menu event receiver from muda and forward to Deno
    let menu_id_map_for_thread = menu_id_map.clone();
    let pending_ctx_menu_for_thread = pending_ctx_menu.clone();
    std::thread::spawn(move || {
        let receiver = muda::MenuEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                // First, check if this is a context menu selection
                {
                    let mut pending = pending_ctx_menu_for_thread.lock().unwrap();
                    if let Some((ref ids, _)) = *pending {
                        if ids.contains(&event.id) {
                            // This is a context menu selection - respond and clear
                            let map = menu_id_map_for_thread.lock().unwrap();
                            if let Some((item_id, label)) = map.get(&event.id) {
                                tracing::debug!(
                                    "Context menu selection: item_id={}, label={}",
                                    item_id,
                                    label
                                );
                                if let Some((_, sender)) = pending.take() {
                                    let _ = sender.send(Some(item_id.clone()));
                                }
                            }
                            continue; // Don't forward to regular menu events
                        }
                    }
                }

                // Regular menu event - forward to Deno (both ext_ui and ext_window)
                let map = menu_id_map_for_thread.lock().unwrap();
                if let Some((item_id, label)) = map.get(&event.id) {
                    // Send to ext_ui menu events channel
                    let menu_event = MenuEvent {
                        menu_id: "app".to_string(),
                        item_id: item_id.clone(),
                        label: label.clone(),
                    };
                    tracing::debug!("Menu event: item_id={}, label={}", item_id, label);
                    let _ = menu_events_tx.blocking_send(menu_event);

                    // Also send to ext_window menu events channel
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
        }
    });

    event_loop.run(move |event, event_loop_target, control| {
        // Use Poll mode so we can continuously poll the JsRuntime
        *control = ControlFlow::Poll;

        match event {
            // Poll the JsRuntime on each iteration when idle
            Event::MainEventsCleared => {
                if !module_eval_done {
                    // Use rt.block_on() directly since we're not inside an async context
                    // (we only used rt.enter() in main(), not block_on())
                    let result = rt.block_on(async {
                        // Use a short timeout so we don't block the UI event loop too long
                        tokio::time::timeout(
                            std::time::Duration::from_millis(10),
                            js.run_event_loop(deno_core::PollEventLoopOptions {
                                wait_for_inspector: false,
                                pump_v8_message_loop: true,
                            })
                        ).await
                    });

                    match result {
                        Ok(Ok(_)) => {
                            // Event loop completed (no more pending ops)
                            module_eval_done = true;
                            tracing::debug!("Module evaluation completed");
                        }
                        Ok(Err(e)) => {
                            tracing::error!("JsRuntime event loop error: {:?}", e);
                            module_eval_done = true;
                        }
                        Err(_timeout) => {
                            // Timeout - still processing, continue polling
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::CreateWindow(opts, respond)) => {
                let width = opts.width.unwrap_or(1024);
                let height = opts.height.unwrap_or(768);

                let window = WindowBuilder::new()
                    .with_title(&manifest.app.name)
                    .with_inner_size(tao::dpi::LogicalSize::new(width, height))
                    .build(event_loop_target)
                    .expect("Failed to create window");

                window_counter += 1;
                let win_id = format!("win-{}", window_counter);

                // Determine channel allowlist for this window
                // Priority: window-specific opts > manifest default > None (allow all in dev)
                let win_channels = opts.channels.clone().or_else(|| default_channels.clone());

                // Build WebView with custom app:// protocol
                let mut builder = WebViewBuilder::new();

                // Inject preload script for window.host bridge
                builder = builder.with_initialization_script(preload_js());

                // IPC handler: messages from renderer -> Deno
                // Channel filtering happens here for incoming messages
                let to_deno_tx_clone = to_deno_tx.clone();
                let win_id_for_ipc = win_id.clone();
                let ipc_capabilities = capabilities.clone();
                let ipc_allowed_channels = win_channels.clone();
                builder = builder.with_ipc_handler(move |msg| {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(msg.body()) {
                        let channel = val.get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Check if channel is allowed using capabilities.check_channel
                        let channel_check = ipc_capabilities.check_channel(
                            &channel,
                            ipc_allowed_channels.as_deref()
                        );

                        if channel_check.is_ok() {
                            let payload = val.get("payload")
                                .cloned()
                                .unwrap_or(serde_json::json!(null));
                            let _ = to_deno_tx_clone.try_send(IpcEvent {
                                window_id: win_id_for_ipc.clone(),
                                channel,
                                payload,
                                event_type: None,
                            });
                        } else {
                            tracing::warn!(
                                "Blocked IPC message on channel '{}' from window {} - not in allowlist",
                                channel, win_id_for_ipc
                            );
                        }
                    }
                });

                // Custom app:// protocol handler with CSP
                let app_dir_for_protocol = app_dir_clone.clone();
                let is_dev_mode = dev_mode;
                builder = builder.with_custom_protocol("app".into(), move |_ctx, request| {
                    let uri = request.uri().to_string();
                    let mut path = uri
                        .strip_prefix("app://")
                        .unwrap_or("")
                        .trim_start_matches('/')
                        .trim_end_matches('/');

                    // Handle relative URL resolution: if path looks like "file.html/resource",
                    // extract just the resource part (browser resolved relative to document URL)
                    if let Some(slash_pos) = path.find('/') {
                        let first_part = &path[..slash_pos];
                        // If the first part looks like an HTML file, this is a relative resource
                        if first_part.ends_with(".html") || first_part.ends_with(".htm") {
                            path = &path[slash_pos + 1..];
                        }
                    }

                    // Content-Security-Policy: strict in production, relaxed in dev
                    let csp = if is_dev_mode {
                        // Dev mode: allow ws:// for HMR, localhost for dev server, CDNs for libs
                        "default-src 'self' app:; \
                         script-src 'self' app: 'unsafe-inline' 'unsafe-eval' https://unpkg.com https://cdn.jsdelivr.net; \
                         style-src 'self' app: 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net; \
                         connect-src 'self' app: ws://localhost:* ws://127.0.0.1:* http://localhost:* http://127.0.0.1:* https://*; \
                         img-src 'self' app: data: blob: https:; \
                         font-src 'self' app: data: https:;"
                    } else {
                        // Production: strict CSP
                        "default-src 'self' app:; \
                         script-src 'self' app:; \
                         style-src 'self' app: 'unsafe-inline'; \
                         img-src 'self' app: data: blob:; \
                         font-src 'self' app: data:; \
                         connect-src 'self' app:;"
                    };

                    // First try embedded assets (release mode)
                    if ASSET_EMBEDDED {
                        if let Some(bytes) = get_asset(path) {
                            return Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", mime_for(path))
                                .header("Content-Security-Policy", csp)
                                .header("X-Content-Type-Options", "nosniff")
                                .body(Cow::Owned(bytes.to_vec()))
                                .unwrap();
                        }
                    }

                    // Fallback to filesystem (dev mode)
                    let file_path = app_dir_for_protocol.join("web").join(path);
                    tracing::debug!("Protocol: uri={} path={} file={} exists={}",
                        uri, path, file_path.display(), file_path.exists());
                    if file_path.exists() {
                        match std::fs::read(&file_path) {
                            Ok(bytes) => {
                                return Response::builder()
                                    .status(StatusCode::OK)
                                    .header("Content-Type", mime_for(path))
                                    .header("Content-Security-Policy", csp)
                                    .header("X-Content-Type-Options", "nosniff")
                                    .body(Cow::Owned(bytes))
                                    .unwrap();
                            }
                            Err(e) => {
                                tracing::error!("Failed to read {}: {}", file_path.display(), e);
                            }
                        }
                    }

                    // 404 Not Found
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .body(Cow::Owned(format!("Not found: {}", path).into_bytes()))
                        .unwrap()
                });

                // Set the initial URL
                let start_url = opts.url.as_deref().unwrap_or("app://index.html");
                builder = builder.with_url(start_url);

                let webview = builder.build(&window).expect("Failed to create webview");

                // Track window ID -> win_id mapping
                windows.insert(window.id(), win_id.clone());
                webviews.insert(win_id.clone(), webview);
                // Store window handle for later operations (like setTitle)
                tao_windows.insert(win_id.clone(), window);
                // Store channel allowlist for outgoing message filtering
                window_channels.insert(win_id.clone(), win_channels);

                tracing::info!("Created window {} at {}", win_id, start_url);
                let _ = respond.send(win_id);
            }

            Event::UserEvent(UserEvent::ToRenderer(ToRendererCmd::Send { window_id, channel, payload })) => {
                // Check if channel is allowed for this window (outgoing messages)
                let win_allowed_channels = window_channels.get(&window_id).and_then(|c| c.clone());
                let channel_check = capabilities.check_channel(
                    &channel,
                    win_allowed_channels.as_deref()
                );

                if channel_check.is_ok() {
                    if let Some(wv) = webviews.get(&window_id) {
                        let js = format!(
                            "window.__host_dispatch && window.__host_dispatch({{channel:{:?},payload:{}}});",
                            channel, payload
                        );
                        let _ = wv.evaluate_script(&js);
                    }
                } else {
                    tracing::warn!(
                        "Blocked outgoing IPC message on channel '{}' to window {} - not in allowlist",
                        channel, window_id
                    );
                }
            }

            Event::UserEvent(UserEvent::CloseWindow(window_id, respond)) => {
                let success = if let Some(tao_id) = windows.iter()
                    .find(|(_, v)| **v == window_id)
                    .map(|(k, _)| *k)
                {
                    windows.remove(&tao_id);
                    webviews.remove(&window_id);
                    tao_windows.remove(&window_id);
                    window_channels.remove(&window_id);
                    tracing::info!("Window {} closed programmatically", window_id);
                    true
                } else {
                    false
                };
                let _ = respond.send(success);

                // Exit if all windows are closed
                if webviews.is_empty() {
                    *control = ControlFlow::Exit;
                }
            }

            Event::UserEvent(UserEvent::SetWindowTitle(window_id, title)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_title(&title);
                    tracing::debug!("Set window {} title to '{}'", window_id, title);
                } else {
                    tracing::warn!("SetWindowTitle: window {} not found", window_id);
                }
            }

            Event::UserEvent(UserEvent::ShowOpenDialog(opts, respond)) => {
                let mut dialog = rfd::FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }
                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }
                if let Some(filters) = &opts.filters {
                    for filter in filters {
                        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
                        dialog = dialog.add_filter(&filter.name, &extensions);
                    }
                }

                let result = if opts.directory.unwrap_or(false) {
                    dialog.pick_folder().map(|p| vec![p.to_string_lossy().to_string()])
                } else if opts.multiple.unwrap_or(false) {
                    dialog.pick_files().map(|paths| {
                        paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect()
                    })
                } else {
                    dialog.pick_file().map(|p| vec![p.to_string_lossy().to_string()])
                };

                let _ = respond.send(result);
            }

            Event::UserEvent(UserEvent::ShowSaveDialog(opts, respond)) => {
                let mut dialog = rfd::FileDialog::new();

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }
                if let Some(path) = &opts.default_path {
                    dialog = dialog.set_directory(path);
                }
                if let Some(filters) = &opts.filters {
                    for filter in filters {
                        let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
                        dialog = dialog.add_filter(&filter.name, &extensions);
                    }
                }

                let result = dialog.save_file().map(|p| p.to_string_lossy().to_string());
                let _ = respond.send(result);
            }

            Event::UserEvent(UserEvent::ShowMessageDialog(opts, respond)) => {
                use rfd::{MessageDialog, MessageLevel, MessageButtons};

                let level = match opts.kind.as_deref() {
                    Some("error") => MessageLevel::Error,
                    Some("warning") => MessageLevel::Warning,
                    _ => MessageLevel::Info,
                };

                // Build buttons with custom labels when provided
                // rfd supports: Ok, OkCancel, YesNo, YesNoCancel, and custom variants
                let buttons = if let Some(btns) = &opts.buttons {
                    match btns.len() {
                        0 => MessageButtons::Ok,
                        1 => MessageButtons::OkCustom(btns[0].clone()),
                        2 => MessageButtons::OkCancelCustom(btns[0].clone(), btns[1].clone()),
                        n => {
                            if n > 3 {
                                tracing::warn!(
                                    "Message dialog supports at most 3 buttons, got {}. Extra buttons will be ignored.",
                                    n
                                );
                            }
                            MessageButtons::YesNoCancelCustom(
                                btns[0].clone(),
                                btns[1].clone(),
                                btns.get(2).cloned().unwrap_or_else(|| "Cancel".to_string()),
                            )
                        }
                    }
                } else {
                    MessageButtons::Ok
                };

                let mut dialog = MessageDialog::new()
                    .set_level(level)
                    .set_buttons(buttons)
                    .set_description(&opts.message);

                if let Some(title) = &opts.title {
                    dialog = dialog.set_title(title);
                }

                let result = dialog.show();
                // Map rfd result to button index
                let button_idx = match result {
                    rfd::MessageDialogResult::Ok => 0,
                    rfd::MessageDialogResult::Cancel => 0,
                    rfd::MessageDialogResult::Yes => 1,
                    rfd::MessageDialogResult::No => 0,
                    rfd::MessageDialogResult::Custom(s) => {
                        // Find the button index
                        if let Some(btns) = &opts.buttons {
                            btns.iter().position(|b| b == &s).unwrap_or(0)
                        } else {
                            0
                        }
                    }
                };
                let _ = respond.send(button_idx);
            }

            // ================================================================
            // Menu Events
            // ================================================================

            Event::UserEvent(UserEvent::SetAppMenu(items, respond)) => {
                // Clear old menu ID mappings
                {
                    let mut map = menu_id_map.lock().unwrap();
                    map.clear();
                }

                // Build menu using muda, tracking menu IDs
                let menu = muda::Menu::new();

                // Helper function to add items to a menu, returning menu ID mappings
                fn add_menu_items_with_tracking(
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
                            // Track the menu ID mapping
                            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
                            let _ = menu.append(&check_item);
                        } else {
                            let menu_item = muda::MenuItem::new(
                                &item.label,
                                item.enabled.unwrap_or(true),
                                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                            );
                            // Track the menu ID mapping
                            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
                            let _ = menu.append(&menu_item);
                        }
                    }
                }

                fn add_submenu_items_with_tracking(
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
                            // Track the menu ID mapping
                            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                            id_map.insert(check_item.id().clone(), (user_id, item.label.clone()));
                            let _ = submenu.append(&check_item);
                        } else {
                            let menu_item = muda::MenuItem::new(
                                &item.label,
                                item.enabled.unwrap_or(true),
                                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                            );
                            // Track the menu ID mapping
                            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                            id_map.insert(menu_item.id().clone(), (user_id, item.label.clone()));
                            let _ = submenu.append(&menu_item);
                        }
                    }
                }

                // Build menu and populate ID map
                {
                    let mut map = menu_id_map.lock().unwrap();
                    add_menu_items_with_tracking(&menu, &items, &mut map);
                    tracing::debug!("Registered {} menu items for event tracking", map.len());
                }

                // On macOS, set as the app menu; on other platforms, attach to windows
                #[cfg(target_os = "macos")]
                {
                    menu.init_for_nsapp();
                }

                #[cfg(target_os = "windows")]
                {
                    use tao::platform::windows::WindowExtWindows;
                    // For Windows, attach menu to each window
                    for window in tao_windows.values() {
                        unsafe {
                            let _ = menu.init_for_hwnd(window.hwnd() as isize);
                        }
                    }
                }

                #[cfg(target_os = "linux")]
                {
                    use gtk::prelude::*;
                    use tao::platform::unix::WindowExtUnix;
                    // For Linux, attach menu to each GTK window
                    for window in tao_windows.values() {
                        let gtk_win = window.gtk_window();
                        let gtk_win_ref: &gtk::Window = gtk_win.upcast_ref();
                        let _ = menu.init_for_gtk_window(gtk_win_ref, None::<&gtk::Box>);
                    }
                }

                // Keep menu alive to prevent it from being dropped
                _app_menu = Some(menu);
                // Reference to suppress unused_assignments warning while keeping menu alive
                let _ = _app_menu.is_some();
                tracing::info!("Set app menu with {} items", items.len());
                let _ = respond.send(true);
            }

            Event::UserEvent(UserEvent::ShowContextMenu(window_id, items, respond)) => {
                use muda::ContextMenu;

                tracing::debug!("ShowContextMenu requested with {} items", items.len());

                // Cancel any pending context menu response
                {
                    let mut pending = pending_ctx_menu.lock().unwrap();
                    if let Some((_, old_sender)) = pending.take() {
                        let _ = old_sender.send(None);
                    }
                }

                if items.is_empty() {
                    let _ = respond.send(None);
                    return;
                }

                // Build the context menu
                let menu = muda::Menu::new();
                let mut ctx_menu_ids: std::collections::HashSet<muda::MenuId> = std::collections::HashSet::new();

                // Helper function to add items and collect their MenuIds
                fn add_context_menu_items(
                    menu: &muda::Menu,
                    items: &[MenuItem],
                    id_map: &mut HashMap<muda::MenuId, (String, String)>,
                    ctx_ids: &mut std::collections::HashSet<muda::MenuId>,
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

                fn add_context_submenu_items(
                    submenu: &muda::Submenu,
                    items: &[MenuItem],
                    id_map: &mut HashMap<muda::MenuId, (String, String)>,
                    ctx_ids: &mut std::collections::HashSet<muda::MenuId>,
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

                // Add items and collect IDs
                {
                    let mut map = menu_id_map.lock().unwrap();
                    add_context_menu_items(&menu, &items, &mut map, &mut ctx_menu_ids);
                }

                // Store the pending response with the set of IDs
                {
                    let mut pending = pending_ctx_menu.lock().unwrap();
                    *pending = Some((ctx_menu_ids, respond));
                }

                // Show the context menu at cursor position
                // Use window_id to find the appropriate window, or use the first window
                let target_tao_window = if let Some(ref wid) = window_id {
                    tao_windows.get(wid)
                } else {
                    tao_windows.values().next()
                };

                if let Some(tao_win) = target_tao_window {
                    #[cfg(target_os = "macos")]
                    {
                        use tao::platform::macos::WindowExtMacOS;
                        unsafe {
                            menu.show_context_menu_for_nsview(tao_win.ns_view() as _, None::<muda::dpi::Position>);
                        }
                    }

                    #[cfg(target_os = "windows")]
                    {
                        use tao::platform::windows::WindowExtWindows;
                        unsafe {
                            menu.show_context_menu_for_hwnd(tao_win.hwnd() as isize, None::<muda::dpi::Position>);
                        }
                    }

                    #[cfg(target_os = "linux")]
                    {
                        use gtk::prelude::*;
                        use tao::platform::unix::WindowExtUnix;
                        let gtk_win = tao_win.gtk_window();
                        let gtk_win_ref: &gtk::Window = gtk_win.upcast_ref();
                        menu.show_context_menu_for_gtk_window(gtk_win_ref, None::<muda::dpi::Position>);
                    }

                    tracing::info!("Showed context menu with {} items", items.len());
                } else {
                    tracing::warn!("No window found to show context menu");
                    // No window - respond with None
                    let mut pending = pending_ctx_menu.lock().unwrap();
                    if let Some((_, sender)) = pending.take() {
                        let _ = sender.send(None);
                    }
                }
            }

            // ================================================================
            // Tray Events
            // ================================================================

            Event::UserEvent(UserEvent::CreateTray(opts, respond)) => {
                use tray_icon::{TrayIconBuilder, Icon};

                // Helper function to create a default tray icon (simple gray square)
                fn create_default_tray_icon() -> Icon {
                    let size = 22u32;
                    let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);
                    for _ in 0..(size * size) {
                        // Medium gray with full opacity
                        rgba_data.extend_from_slice(&[128, 128, 128, 255]);
                    }
                    Icon::from_rgba(rgba_data, size, size).expect("Failed to create default icon")
                }

                // Helper function to add menu items to tray menu
                fn add_tray_menu_items(
                    menu: &muda::Menu,
                    items: &[MenuItem],
                    id_map: &mut HashMap<muda::MenuId, (String, String)>,
                    tray_id: &str,
                ) {
                    fn add_items_recursive(
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
                                add_submenu_recursive(&submenu, submenu_items, id_map, tray_id);
                                let _ = menu.append(&submenu);
                            } else if item.item_type.as_deref() == Some("checkbox") {
                                let check_item = muda::CheckMenuItem::new(
                                    &item.label,
                                    item.enabled.unwrap_or(true),
                                    item.checked.unwrap_or(false),
                                    item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                                );
                                // Track menu ID with tray prefix for event routing
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
                                // Track menu ID with tray prefix for event routing
                                let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                                let menu_id = format!("{}:{}", tray_id, user_id);
                                id_map.insert(menu_item.id().clone(), (menu_id, item.label.clone()));
                                let _ = menu.append(&menu_item);
                            }
                        }
                    }

                    fn add_submenu_recursive(
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
                                add_submenu_recursive(&nested_submenu, nested_items, id_map, tray_id);
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

                    add_items_recursive(menu, items, id_map, tray_id);
                }

                tray_counter += 1;
                let tray_id_str = format!("tray-{}", tray_counter);

                // Load icon from file or use default
                let icon = if let Some(ref icon_path) = opts.icon {
                    // Resolve icon path relative to app directory
                    let full_path = if std::path::Path::new(icon_path).is_absolute() {
                        std::path::PathBuf::from(icon_path)
                    } else {
                        app_dir.join(icon_path)
                    };

                    match std::fs::read(&full_path) {
                        Ok(bytes) => {
                            // Decode image using image crate
                            match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    // Resize for tray icon (22x22 is standard for macOS menu bar)
                                    let resized = img.resize_exact(22, 22, image::imageops::FilterType::Lanczos3);
                                    let rgba = resized.to_rgba8();
                                    let (width, height) = rgba.dimensions();
                                    match Icon::from_rgba(rgba.into_raw(), width, height) {
                                        Ok(icon) => {
                                            tracing::debug!("Loaded tray icon from: {:?}", full_path);
                                            icon
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to create icon from decoded image: {}", e);
                                            create_default_tray_icon()
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to decode image {:?}: {}", full_path, e);
                                    create_default_tray_icon()
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read icon file {:?}: {}", full_path, e);
                            create_default_tray_icon()
                        }
                    }
                } else {
                    create_default_tray_icon()
                };

                let mut builder = TrayIconBuilder::new().with_icon(icon);

                if let Some(ref tooltip) = opts.tooltip {
                    builder = builder.with_tooltip(tooltip);
                }

                // Build tray menu if provided
                if let Some(ref menu_items) = opts.menu {
                    if !menu_items.is_empty() {
                        let menu = muda::Menu::new();

                        // Add menu items and track IDs for event mapping
                        {
                            let mut map = menu_id_map.lock().unwrap();
                            add_tray_menu_items(&menu, menu_items, &mut map, &tray_id_str);
                        }

                        builder = builder.with_menu(Box::new(menu));
                        tracing::debug!("Added menu with {} items to tray", menu_items.len());
                    }
                }

                match builder.build() {
                    Ok(tray) => {
                        trays.insert(tray_id_str.clone(), tray);
                        tracing::info!("Created tray icon: {}", tray_id_str);
                        let _ = respond.send(tray_id_str);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create tray: {}", e);
                        let _ = respond.send(String::new());
                    }
                }
            }

            Event::UserEvent(UserEvent::UpdateTray(tray_id, opts, respond)) => {
                if let Some(tray) = trays.get_mut(&tray_id) {
                    if let Some(ref tooltip) = opts.tooltip {
                        let _ = tray.set_tooltip(Some(tooltip));
                    }
                    tracing::debug!("Updated tray: {}", tray_id);
                    let _ = respond.send(true);
                } else {
                    tracing::warn!("Tray not found: {}", tray_id);
                    let _ = respond.send(false);
                }
            }

            Event::UserEvent(UserEvent::DestroyTray(tray_id, respond)) => {
                if trays.remove(&tray_id).is_some() {
                    tracing::info!("Destroyed tray: {}", tray_id);
                    let _ = respond.send(true);
                } else {
                    tracing::warn!("Tray not found: {}", tray_id);
                    let _ = respond.send(false);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                // Find and remove the window
                if let Some(win_id) = windows.remove(&window_id) {
                    // Send close event to Deno before removing
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({}),
                        event_type: Some("close".to_string()),
                    });
                    webviews.remove(&win_id);
                    tao_windows.remove(&win_id);
                    window_channels.remove(&win_id);
                    tracing::info!("Window {} closed", win_id);
                }

                // Exit if all windows are closed
                if webviews.is_empty() {
                    *control = ControlFlow::Exit;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Focused(focused),
                window_id,
                ..
            } => {
                if let Some(win_id) = windows.get(&window_id) {
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
                if let Some(win_id) = windows.get(&window_id) {
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({
                            "width": size.width,
                            "height": size.height
                        }),
                        event_type: Some("resize".to_string()),
                    });
                    tracing::debug!("Window {} resized to {}x{}", win_id, size.width, size.height);
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Moved(position),
                window_id,
                ..
            } => {
                if let Some(win_id) = windows.get(&window_id) {
                    let _ = to_deno_tx.try_send(IpcEvent {
                        window_id: win_id.clone(),
                        channel: "__window__".to_string(),
                        payload: serde_json::json!({
                            "x": position.x,
                            "y": position.y
                        }),
                        event_type: Some("move".to_string()),
                    });
                    tracing::debug!("Window {} moved to ({}, {})", win_id, position.x, position.y);
                }
            }

            // =========================================================================
            // ext_window event handlers (host:window)
            // =========================================================================

            Event::UserEvent(UserEvent::WinCreate(opts, respond)) => {
                // Create window with tao
                let width = opts.width.unwrap_or(800);
                let height = opts.height.unwrap_or(600);
                let title = opts.title.clone().unwrap_or_else(|| manifest.app.name.clone());

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

                match win_builder.build(event_loop_target) {
                    Ok(window) => {
                        window_counter += 1;
                        let win_id = format!("win-{}", window_counter);
                        let win_channels = opts.channels.clone().or_else(|| default_channels.clone());

                        // Build WebView with custom app:// protocol (same pattern as CreateWindow)
                        let mut wv_builder = WebViewBuilder::new();
                        wv_builder = wv_builder.with_initialization_script(preload_js());

                        // IPC handler
                        let to_deno_tx_clone = to_deno_tx.clone();
                        let win_id_for_ipc = win_id.clone();
                        let ipc_capabilities = capabilities.clone();
                        let ipc_allowed_channels = win_channels.clone();
                        wv_builder = wv_builder.with_ipc_handler(move |msg| {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(msg.body()) {
                                let channel = val.get("channel")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                let channel_check = ipc_capabilities.check_channel(
                                    &channel,
                                    ipc_allowed_channels.as_deref()
                                );

                                if channel_check.is_ok() {
                                    let payload = val.get("payload")
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
                        let app_dir_for_protocol = app_dir_clone.clone();
                        let is_dev_mode = dev_mode;
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
                                 script-src 'self' app: 'unsafe-inline' 'unsafe-eval' https://unpkg.com https://cdn.jsdelivr.net; \
                                 style-src 'self' app: 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net; \
                                 connect-src 'self' app: ws://localhost:* ws://127.0.0.1:* http://localhost:* http://127.0.0.1:* https://*; \
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

                            // Try embedded assets first
                            if ASSET_EMBEDDED {
                                if let Some(bytes) = get_asset(path) {
                                    return Response::builder()
                                        .status(StatusCode::OK)
                                        .header("Content-Type", mime_for(path))
                                        .header("Content-Security-Policy", csp)
                                        .header("X-Content-Type-Options", "nosniff")
                                        .body(Cow::Owned(bytes.to_vec()))
                                        .unwrap();
                                }
                            }

                            // Fallback to filesystem
                            let file_path = app_dir_for_protocol.join("web").join(path);
                            if file_path.exists() {
                                if let Ok(bytes) = std::fs::read(&file_path) {
                                    return Response::builder()
                                        .status(StatusCode::OK)
                                        .header("Content-Type", mime_for(path))
                                        .header("Content-Security-Policy", csp)
                                        .header("X-Content-Type-Options", "nosniff")
                                        .body(Cow::Owned(bytes))
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

                        match wv_builder.build(&window) {
                            Ok(webview) => {
                                let tao_window_id = window.id();
                                windows.insert(tao_window_id, win_id.clone());
                                webviews.insert(win_id.clone(), webview);
                                tao_windows.insert(win_id.clone(), window);
                                window_channels.insert(win_id.clone(), win_channels);

                                let _ = window_events_tx.try_send(WindowSystemEvent {
                                    window_id: win_id.clone(),
                                    event_type: "create".to_string(),
                                    payload: serde_json::json!({}),
                                });

                                tracing::info!("ext_window: Created window {} at {}", win_id, start_url);
                                let _ = respond.send(Ok(win_id));
                            }
                            Err(e) => {
                                tracing::error!("ext_window: Failed to build webview: {}", e);
                                let _ = respond.send(Err(format!("Failed to build webview: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("ext_window: Failed to create window: {}", e);
                        let _ = respond.send(Err(format!("Failed to create window: {}", e)));
                    }
                }
            }

            Event::UserEvent(UserEvent::WinClose(window_id, respond)) => {
                if let Some(window) = tao_windows.remove(&window_id) {
                    let tao_id = window.id();
                    windows.remove(&tao_id);
                    webviews.remove(&window_id);
                    window_channels.remove(&window_id);

                    let _ = window_events_tx.try_send(WindowSystemEvent {
                        window_id: window_id.clone(),
                        event_type: "close".to_string(),
                        payload: serde_json::json!({}),
                    });

                    tracing::info!("ext_window: Window {} closed", window_id);
                    let _ = respond.send(true);
                } else {
                    let _ = respond.send(false);
                }
            }

            Event::UserEvent(UserEvent::WinMinimize(window_id)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_minimized(true);
                    let _ = window_events_tx.try_send(WindowSystemEvent {
                        window_id: window_id.clone(),
                        event_type: "minimize".to_string(),
                        payload: serde_json::json!({}),
                    });
                }
            }

            Event::UserEvent(UserEvent::WinMaximize(window_id)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_maximized(true);
                    let _ = window_events_tx.try_send(WindowSystemEvent {
                        window_id: window_id.clone(),
                        event_type: "maximize".to_string(),
                        payload: serde_json::json!({}),
                    });
                }
            }

            Event::UserEvent(UserEvent::WinUnmaximize(window_id)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_maximized(false);
                    let _ = window_events_tx.try_send(WindowSystemEvent {
                        window_id: window_id.clone(),
                        event_type: "restore".to_string(),
                        payload: serde_json::json!({}),
                    });
                }
            }

            Event::UserEvent(UserEvent::WinRestore(window_id)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_minimized(false);
                    let _ = window_events_tx.try_send(WindowSystemEvent {
                        window_id: window_id.clone(),
                        event_type: "restore".to_string(),
                        payload: serde_json::json!({}),
                    });
                }
            }

            Event::UserEvent(UserEvent::WinSetFullscreen(window_id, fullscreen)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    if fullscreen {
                        window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
                    } else {
                        window.set_fullscreen(None);
                    }
                }
            }

            Event::UserEvent(UserEvent::WinFocus(window_id)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_focus();
                }
            }

            Event::UserEvent(UserEvent::WinGetPosition(window_id, respond)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    let pos = window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(0, 0));
                    let _ = respond.send(Ok(Position { x: pos.x, y: pos.y }));
                } else {
                    let _ = respond.send(Err(format!("Window not found: {}", window_id)));
                }
            }

            Event::UserEvent(UserEvent::WinSetPosition(window_id, x, y)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_outer_position(tao::dpi::LogicalPosition::new(x, y));
                }
            }

            Event::UserEvent(UserEvent::WinGetSize(window_id, respond)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    let size = window.inner_size();
                    let _ = respond.send(Ok(Size {
                        width: size.width,
                        height: size.height,
                    }));
                } else {
                    let _ = respond.send(Err(format!("Window not found: {}", window_id)));
                }
            }

            Event::UserEvent(UserEvent::WinSetSize(window_id, width, height)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_inner_size(tao::dpi::LogicalSize::new(width, height));
                }
            }

            Event::UserEvent(UserEvent::WinGetTitle(window_id, respond)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    let title = window.title();
                    let _ = respond.send(Ok(title));
                } else {
                    let _ = respond.send(Err(format!("Window not found: {}", window_id)));
                }
            }

            Event::UserEvent(UserEvent::WinSetTitle(window_id, title)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_title(&title);
                }
            }

            Event::UserEvent(UserEvent::WinSetResizable(window_id, resizable)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_resizable(resizable);
                }
            }

            Event::UserEvent(UserEvent::WinSetDecorations(window_id, decorations)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_decorations(decorations);
                }
            }

            Event::UserEvent(UserEvent::WinSetAlwaysOnTop(window_id, always_on_top)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_always_on_top(always_on_top);
                }
            }

            Event::UserEvent(UserEvent::WinSetVisible(window_id, visible)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    window.set_visible(visible);
                }
            }

            Event::UserEvent(UserEvent::WinGetState(window_id, respond)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    let state = WinWindowState {
                        is_visible: window.is_visible(),
                        is_focused: window.is_focused(),
                        is_fullscreen: window.fullscreen().is_some(),
                        is_maximized: window.is_maximized(),
                        is_minimized: window.is_minimized(),
                        is_resizable: window.is_resizable(),
                        has_decorations: window.is_decorated(),
                        is_always_on_top: window.is_always_on_top(),
                    };
                    let _ = respond.send(Ok(state));
                } else {
                    let _ = respond.send(Err(format!("Window not found: {}", window_id)));
                }
            }

            Event::UserEvent(UserEvent::WinShowOpenDialog(opts, respond)) => {
                // Convert WinFileDialogOpts to ext_ui FileDialogOpts for compatibility
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

                let result = if opts.directory.unwrap_or(false) {
                    dialog.pick_folder().map(|p| vec![p.to_string_lossy().to_string()])
                } else if opts.multiple.unwrap_or(false) {
                    dialog.pick_files().map(|paths| {
                        paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect()
                    })
                } else {
                    dialog.pick_file().map(|p| vec![p.to_string_lossy().to_string()])
                };

                let _ = respond.send(result);
            }

            Event::UserEvent(UserEvent::WinShowSaveDialog(opts, respond)) => {
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

                let result = dialog.save_file().map(|p| p.to_string_lossy().to_string());
                let _ = respond.send(result);
            }

            Event::UserEvent(UserEvent::WinShowMessageDialog(opts, respond)) => {
                let level = match opts.kind.as_deref() {
                    Some("warning") => rfd::MessageLevel::Warning,
                    Some("error") => rfd::MessageLevel::Error,
                    _ => rfd::MessageLevel::Info,
                };

                // Build buttons with custom labels when provided
                // rfd supports: Ok, OkCancel, YesNo, YesNoCancel, and custom variants
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
                // Map result to button index
                let idx = match result {
                    rfd::MessageDialogResult::Ok => 0,
                    rfd::MessageDialogResult::Cancel => 1,
                    rfd::MessageDialogResult::Yes => 0,
                    rfd::MessageDialogResult::No => 1,
                    rfd::MessageDialogResult::Custom(_) => 0,
                };
                let _ = respond.send(idx);
            }

            Event::UserEvent(UserEvent::WinSetAppMenu(items, respond)) => {
                // Convert WinMenuItem to muda menu items (similar to ext_ui implementation)
                let menu = muda::Menu::new();

                fn add_win_menu_items(
                    menu: &muda::Menu,
                    items: &[WinMenuItem],
                    id_map: &mut HashMap<muda::MenuId, (String, String)>,
                ) {
                    for item in items {
                        if item.item_type.as_deref() == Some("separator") {
                            let sep = muda::PredefinedMenuItem::separator();
                            let _ = menu.append(&sep);
                        } else if let Some(ref submenu_items) = item.submenu {
                            let submenu = muda::Submenu::new(&item.label, true);
                            for sub_item in submenu_items {
                                if sub_item.item_type.as_deref() == Some("separator") {
                                    let sep = muda::PredefinedMenuItem::separator();
                                    let _ = submenu.append(&sep);
                                } else {
                                    let menu_item = muda::MenuItem::new(
                                        &sub_item.label,
                                        sub_item.enabled.unwrap_or(true),
                                        sub_item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                                    );
                                    let user_id = sub_item.id.clone().unwrap_or_else(|| sub_item.label.clone());
                                    id_map.insert(menu_item.id().clone(), (user_id, sub_item.label.clone()));
                                    let _ = submenu.append(&menu_item);
                                }
                            }
                            let _ = menu.append(&submenu);
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

                {
                    let mut map = menu_id_map.lock().unwrap();
                    add_win_menu_items(&menu, &items, &mut map);
                }

                #[cfg(target_os = "macos")]
                {
                    menu.init_for_nsapp();
                }

                let _ = respond.send(true);
            }

            Event::UserEvent(UserEvent::WinShowContextMenu(window_id, items, respond)) => {
                // Similar to ext_ui context menu
                let menu = muda::Menu::new();

                fn add_win_ctx_items(
                    menu: &muda::Menu,
                    items: &[WinMenuItem],
                    id_map: &mut HashMap<muda::MenuId, (String, String)>,
                ) {
                    for item in items {
                        if item.item_type.as_deref() == Some("separator") {
                            let sep = muda::PredefinedMenuItem::separator();
                            let _ = menu.append(&sep);
                        } else {
                            let menu_item = muda::MenuItem::new(
                                &item.label,
                                item.enabled.unwrap_or(true),
                                item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                            );
                            let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                            id_map.insert(menu_item.id().clone(), (user_id.clone(), item.label.clone()));
                            let _ = menu.append(&menu_item);
                        }
                    }
                }

                {
                    let mut map = menu_id_map.lock().unwrap();
                    add_win_ctx_items(&menu, &items, &mut map);
                }

                // Show context menu on the window if specified
                if let Some(ref wid) = window_id {
                    if let Some(window) = tao_windows.get(wid) {
                        #[cfg(target_os = "macos")]
                        {
                            use muda::ContextMenu;
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
                            use muda::ContextMenu;
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
                            use muda::ContextMenu;
                            use tao::platform::unix::WindowExtUnix;
                            let gtk_win: &gtk::Window = window.gtk_window().upcast_ref();
                            let _ = menu.show_context_menu_for_gtk_window(
                                gtk_win,
                                None::<muda::dpi::Position>,
                            );
                        }
                    }
                }

                // For now, return empty string (context menus use events)
                let _ = respond.send(None);
            }

            Event::UserEvent(UserEvent::WinCreateTray(opts, respond)) => {
                use tray_icon::{Icon, TrayIconBuilder};

                // Helper function to create a default tray icon (simple gray square)
                fn create_default_tray_icon() -> Icon {
                    let size = 22u32;
                    let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);
                    for _ in 0..(size * size) {
                        // Medium gray with full opacity
                        rgba_data.extend_from_slice(&[128, 128, 128, 255]);
                    }
                    Icon::from_rgba(rgba_data, size, size).expect("Failed to create default icon")
                }

                // Similar to ext_ui tray creation
                tray_counter += 1;
                let tray_id_str = format!("tray-{}", tray_counter);

                let icon = if let Some(ref icon_path) = opts.icon {
                    let full_path = if std::path::Path::new(icon_path).is_absolute() {
                        std::path::PathBuf::from(icon_path)
                    } else {
                        app_dir.join(icon_path)
                    };

                    match std::fs::read(&full_path) {
                        Ok(bytes) => {
                            match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    let resized = img.resize_exact(22, 22, image::imageops::FilterType::Lanczos3);
                                    let rgba = resized.to_rgba8();
                                    let (width, height) = rgba.dimensions();
                                    Icon::from_rgba(rgba.into_raw(), width, height)
                                        .unwrap_or_else(|_| create_default_tray_icon())
                                }
                                Err(_) => create_default_tray_icon(),
                            }
                        }
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
                        // Add menu items
                        {
                            let mut map = menu_id_map.lock().unwrap();
                            for item in menu_items {
                                if item.item_type.as_deref() == Some("separator") {
                                    let sep = muda::PredefinedMenuItem::separator();
                                    let _ = menu.append(&sep);
                                } else {
                                    let menu_item = muda::MenuItem::new(
                                        &item.label,
                                        item.enabled.unwrap_or(true),
                                        item.accelerator.as_ref().and_then(|a| a.parse().ok()),
                                    );
                                    let user_id = item.id.clone().unwrap_or_else(|| item.label.clone());
                                    let menu_id = format!("{}:{}", tray_id_str, user_id);
                                    map.insert(menu_item.id().clone(), (menu_id, item.label.clone()));
                                    let _ = menu.append(&menu_item);
                                }
                            }
                        }
                        builder = builder.with_menu(Box::new(menu));
                    }
                }

                match builder.build() {
                    Ok(tray) => {
                        trays.insert(tray_id_str.clone(), tray);
                        let _ = respond.send(tray_id_str);
                    }
                    Err(_) => {
                        let _ = respond.send(String::new());
                    }
                }
            }

            Event::UserEvent(UserEvent::WinUpdateTray(tray_id, opts, respond)) => {
                if let Some(tray) = trays.get_mut(&tray_id) {
                    if let Some(ref tooltip) = opts.tooltip {
                        let _ = tray.set_tooltip(Some(tooltip));
                    }
                    let _ = respond.send(true);
                } else {
                    let _ = respond.send(false);
                }
            }

            Event::UserEvent(UserEvent::WinDestroyTray(tray_id, respond)) => {
                if trays.remove(&tray_id).is_some() {
                    let _ = respond.send(true);
                } else {
                    let _ = respond.send(false);
                }
            }

            Event::UserEvent(UserEvent::WinGetNativeHandle(window_id, respond)) => {
                if let Some(window) = tao_windows.get(&window_id) {
                    #[cfg(target_os = "macos")]
                    {
                        use tao::platform::macos::WindowExtMacOS;
                        let handle = window.ns_view() as u64;
                        let _ = respond.send(Ok(NativeHandle {
                            platform: "macos".to_string(),
                            handle,
                        }));
                    }
                    #[cfg(target_os = "windows")]
                    {
                        use tao::platform::windows::WindowExtWindows;
                        let handle = window.hwnd() as u64;
                        let _ = respond.send(Ok(NativeHandle {
                            platform: "windows".to_string(),
                            handle,
                        }));
                    }
                    #[cfg(target_os = "linux")]
                    {
                        // Native handle retrieval not yet implemented on Linux
                        // Linux supports both X11 and Wayland, requiring different APIs
                        let _ = respond.send(Err(
                            "Native handle retrieval is not yet supported on Linux. \
                            Consider using X11/Wayland-specific APIs directly if needed.".to_string()
                        ));
                    }
                } else {
                    let _ = respond.send(Err(format!("Window not found: {}", window_id)));
                }
            }

            _ => {}
        }
    });
}
