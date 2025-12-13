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

use ext_ui::{
    init_ui_capabilities, init_ui_state, FileDialogOpts, FromDenoCmd, MenuEvent, MenuItem,
    MessageDialogOpts, OpenOpts, ToRendererCmd, TrayOpts, UiEvent,
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
    include_str!("../../../sdk/preload.js")
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
        if specifier.starts_with("host:") {
            let module_name = &specifier[5..]; // strip "host:"
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
                code.into()
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
    let mut app_dir = PathBuf::from("apps/example-deno-app");
    let mut dev_mode = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--app-dir" => {
                app_dir = PathBuf::from(args.next().expect("--app-dir requires a path"));
            }
            "--dev" => {
                dev_mode = true;
            }
            _ => {}
        }
    }

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
    let (fs_caps, net_caps, sys_caps, ui_caps, process_caps, wasm_caps) =
        create_capability_adapters(capabilities.clone());

    // Create IPC channels for Deno <-> Host <-> Renderer communication
    let (to_deno_tx, to_deno_rx) = tokio::sync::mpsc::channel::<UiEvent>(256);
    let (to_renderer_tx, mut to_renderer_rx) = tokio::sync::mpsc::channel::<ToRendererCmd>(256);
    let (from_deno_tx, mut from_deno_rx) = tokio::sync::mpsc::channel::<FromDenoCmd>(64);
    let (menu_events_tx, menu_events_rx) = tokio::sync::mpsc::channel::<MenuEvent>(64);

    // Build Deno runtime with extensions (host:*)
    let module_loader = Rc::new(ForgeModuleLoader::new(app_dir.clone()));
    let mut js = JsRuntime::new(RuntimeOptions {
        module_loader: Some(module_loader),
        extensions: vec![
            ext_fs::fs_extension(),
            ext_net::net_extension(),
            ext_sys::sys_extension(),
            ext_ui::ui_extension(),
            ext_process::process_extension(),
            ext_wasm::wasm_extension(),
        ],
        ..Default::default()
    });

    // Initialize all extension state with capability adapters
    {
        let op_state = js.op_state();
        let mut state = op_state.borrow_mut();

        // Initialize UI state (including menu events channel)
        // Use to_deno_rx - the receiver side of the channel that IPC handler sends to
        init_ui_state(
            &mut state,
            to_renderer_tx.clone(),
            to_deno_rx,
            from_deno_tx.clone(),
            menu_events_rx,
        );

        // Initialize FS state with capability checker
        ext_fs::init_fs_state(&mut state, Some(fs_caps));

        // Initialize Net state with capability checker
        ext_net::init_net_state(&mut state, Some(net_caps));

        // Initialize Sys state with capability checker
        ext_sys::init_sys_state(&mut state, Some(sys_caps));

        // Initialize UI capabilities
        init_ui_capabilities(&mut state, Some(ui_caps));

        // Initialize Process state with capability checker
        let max_processes = capabilities.get_max_processes();
        ext_process::init_process_state(&mut state, Some(process_caps), Some(max_processes));

        // Initialize WASM state with capability checker
        let max_wasm_instances = capabilities.get_max_wasm_instances();
        ext_wasm::init_wasm_state(&mut state, Some(wasm_caps), Some(max_wasm_instances));
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
    #[allow(unused_assignments)]
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

                // Regular menu event - forward to Deno
                let map = menu_id_map_for_thread.lock().unwrap();
                if let Some((item_id, label)) = map.get(&event.id) {
                    let menu_event = MenuEvent {
                        menu_id: "app".to_string(),
                        item_id: item_id.clone(),
                        label: label.clone(),
                    };
                    tracing::debug!("Menu event: item_id={}, label={}", item_id, label);
                    let _ = menu_events_tx.blocking_send(menu_event);
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
                            let _ = to_deno_tx_clone.try_send(UiEvent {
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

                let buttons = if let Some(btns) = &opts.buttons {
                    if btns.len() == 1 {
                        MessageButtons::Ok
                    } else if btns.len() == 2 && btns[0].to_lowercase() == "cancel" {
                        MessageButtons::OkCancel
                    } else if btns.len() == 2 && btns[0].to_lowercase() == "yes" {
                        MessageButtons::YesNo
                    } else {
                        MessageButtons::OkCancel
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

                _app_menu = Some(menu);
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
                    let _ = to_deno_tx.try_send(UiEvent {
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
                    let _ = to_deno_tx.try_send(UiEvent {
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
                    let _ = to_deno_tx.try_send(UiEvent {
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
                    let _ = to_deno_tx.try_send(UiEvent {
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

            _ => {}
        }
    });
}
