//! Application lifecycle extension for Forge
//!
//! Provides application-level operations including:
//! - Application quit/exit/relaunch
//! - App metadata (version, name, identifier)
//! - Special path retrieval
//! - Single instance locking
//! - Window visibility control
//! - Badge count management
//! - Locale information

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

// Include the generated extension code from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

// ============================================================================
// Error Types
// ============================================================================

/// Error codes for app operations (8300-8319)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorCode {
    /// Failed to quit application (8300)
    QuitFailed = 8300,
    /// Failed to exit application (8301)
    ExitFailed = 8301,
    /// Failed to relaunch application (8302)
    RelaunchFailed = 8302,
    /// Failed to get app info (8303)
    InfoFailed = 8303,
    /// Failed to get path (8304)
    PathFailed = 8304,
    /// Single instance lock failed (8305)
    LockFailed = 8305,
    /// Failed to focus application (8306)
    FocusFailed = 8306,
    /// Failed to hide application (8307)
    HideFailed = 8307,
    /// Failed to show application (8308)
    ShowFailed = 8308,
    /// Failed to set badge count (8309)
    BadgeFailed = 8309,
    /// Failed to set user model ID (8310)
    UserModelIdFailed = 8310,
    /// Invalid path type (8311)
    InvalidPathType = 8311,
    /// Permission denied (8312)
    PermissionDenied = 8312,
    /// Operation not supported (8313)
    NotSupported = 8313,
    /// App state not initialized (8314)
    NotInitialized = 8314,
}

impl std::fmt::Display for AppErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

/// Errors that can occur during app operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum AppError {
    #[error("[{code}] Failed to quit application: {message}")]
    #[class(generic)]
    QuitFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to exit application: {message}")]
    #[class(generic)]
    ExitFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to relaunch application: {message}")]
    #[class(generic)]
    RelaunchFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to get app info: {message}")]
    #[class(generic)]
    InfoFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to get path: {message}")]
    #[class(generic)]
    PathFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Single instance lock failed: {message}")]
    #[class(generic)]
    LockFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to focus application: {message}")]
    #[class(generic)]
    FocusFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to hide application: {message}")]
    #[class(generic)]
    HideFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to show application: {message}")]
    #[class(generic)]
    ShowFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to set badge count: {message}")]
    #[class(generic)]
    BadgeFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Failed to set user model ID: {message}")]
    #[class(generic)]
    UserModelIdFailed { code: AppErrorCode, message: String },

    #[error("[{code}] Invalid path type: {message}")]
    #[class(generic)]
    InvalidPathType { code: AppErrorCode, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: AppErrorCode, message: String },

    #[error("[{code}] Operation not supported: {message}")]
    #[class(generic)]
    NotSupported { code: AppErrorCode, message: String },

    #[error("[{code}] App state not initialized: {message}")]
    #[class(generic)]
    NotInitialized { code: AppErrorCode, message: String },
}

impl AppError {
    pub fn quit_failed(message: impl Into<String>) -> Self {
        Self::QuitFailed {
            code: AppErrorCode::QuitFailed,
            message: message.into(),
        }
    }

    pub fn exit_failed(message: impl Into<String>) -> Self {
        Self::ExitFailed {
            code: AppErrorCode::ExitFailed,
            message: message.into(),
        }
    }

    pub fn relaunch_failed(message: impl Into<String>) -> Self {
        Self::RelaunchFailed {
            code: AppErrorCode::RelaunchFailed,
            message: message.into(),
        }
    }

    pub fn info_failed(message: impl Into<String>) -> Self {
        Self::InfoFailed {
            code: AppErrorCode::InfoFailed,
            message: message.into(),
        }
    }

    pub fn path_failed(message: impl Into<String>) -> Self {
        Self::PathFailed {
            code: AppErrorCode::PathFailed,
            message: message.into(),
        }
    }

    pub fn lock_failed(message: impl Into<String>) -> Self {
        Self::LockFailed {
            code: AppErrorCode::LockFailed,
            message: message.into(),
        }
    }

    pub fn focus_failed(message: impl Into<String>) -> Self {
        Self::FocusFailed {
            code: AppErrorCode::FocusFailed,
            message: message.into(),
        }
    }

    pub fn hide_failed(message: impl Into<String>) -> Self {
        Self::HideFailed {
            code: AppErrorCode::HideFailed,
            message: message.into(),
        }
    }

    pub fn show_failed(message: impl Into<String>) -> Self {
        Self::ShowFailed {
            code: AppErrorCode::ShowFailed,
            message: message.into(),
        }
    }

    pub fn badge_failed(message: impl Into<String>) -> Self {
        Self::BadgeFailed {
            code: AppErrorCode::BadgeFailed,
            message: message.into(),
        }
    }

    pub fn user_model_id_failed(message: impl Into<String>) -> Self {
        Self::UserModelIdFailed {
            code: AppErrorCode::UserModelIdFailed,
            message: message.into(),
        }
    }

    pub fn invalid_path_type(message: impl Into<String>) -> Self {
        Self::InvalidPathType {
            code: AppErrorCode::InvalidPathType,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: AppErrorCode::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn not_supported(message: impl Into<String>) -> Self {
        Self::NotSupported {
            code: AppErrorCode::NotSupported,
            message: message.into(),
        }
    }

    pub fn not_initialized(message: impl Into<String>) -> Self {
        Self::NotInitialized {
            code: AppErrorCode::NotInitialized,
            message: message.into(),
        }
    }
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Trait for checking app operation permissions
pub trait AppCapabilityChecker: Send + Sync + 'static {
    /// Check if quitting is allowed
    fn can_quit(&self) -> bool {
        true
    }

    /// Check if relaunching is allowed
    fn can_relaunch(&self) -> bool {
        true
    }

    /// Check if getting app info is allowed
    fn can_get_info(&self) -> bool {
        true
    }

    /// Check if getting paths is allowed
    fn can_get_paths(&self) -> bool {
        true
    }

    /// Check if single instance locking is allowed
    fn can_lock(&self) -> bool {
        true
    }

    /// Check if window control is allowed
    fn can_control_windows(&self) -> bool {
        true
    }
}

/// Default capability checker that allows all operations
pub struct DefaultAppCapabilityChecker;

impl AppCapabilityChecker for DefaultAppCapabilityChecker {}

// ============================================================================
// Types
// ============================================================================

/// Application information stored in state
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Application identifier (e.g., com.example.app)
    pub identifier: String,
    /// Whether app is running packaged
    pub is_packaged: bool,
    /// Path to the application executable
    pub exe_path: Option<String>,
    /// Path to the application resources
    pub resource_path: Option<String>,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "Forge App".to_string(),
            version: "0.1.0".to_string(),
            identifier: "com.forge.app".to_string(),
            is_packaged: false,
            exe_path: std::env::current_exe()
                .ok()
                .map(|p| p.to_string_lossy().to_string()),
            resource_path: None,
        }
    }
}

/// Types of special paths that can be requested
#[weld_enum]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PathType {
    /// User's home directory
    Home,
    /// Application data directory
    AppData,
    /// User's documents directory
    Documents,
    /// User's downloads directory
    Downloads,
    /// User's desktop directory
    Desktop,
    /// User's music directory
    Music,
    /// User's pictures directory
    Pictures,
    /// User's videos directory
    Videos,
    /// Temporary directory
    Temp,
    /// Application executable path
    Exe,
    /// Application resources path
    Resources,
    /// Application log directory
    Logs,
    /// Application cache directory
    Cache,
}

impl std::str::FromStr for PathType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "home" => Ok(PathType::Home),
            "appdata" | "app_data" | "appData" => Ok(PathType::AppData),
            "documents" => Ok(PathType::Documents),
            "downloads" => Ok(PathType::Downloads),
            "desktop" => Ok(PathType::Desktop),
            "music" => Ok(PathType::Music),
            "pictures" => Ok(PathType::Pictures),
            "videos" => Ok(PathType::Videos),
            "temp" | "tmp" => Ok(PathType::Temp),
            "exe" | "executable" => Ok(PathType::Exe),
            "resources" | "resource" => Ok(PathType::Resources),
            "logs" | "log" => Ok(PathType::Logs),
            "cache" => Ok(PathType::Cache),
            _ => Err(AppError::invalid_path_type(format!(
                "Unknown path type: {}",
                s
            ))),
        }
    }
}

/// Locale information
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleInfo {
    /// Language code (e.g., "en")
    pub language: String,
    /// Country code (e.g., "US")
    pub country: Option<String>,
    /// Full locale string (e.g., "en-US")
    pub locale: String,
}

/// Application command for controlling the app lifecycle
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Request graceful quit
    Quit,
    /// Force exit with code
    Exit(i32),
    /// Relaunch the application
    Relaunch,
    /// Focus the application
    Focus,
    /// Hide all windows
    Hide,
    /// Show all windows
    Show,
    /// Set badge count
    SetBadge(Option<u32>),
}

/// State for single instance locking
pub struct SingleInstanceState {
    /// Whether lock is held
    pub locked: AtomicBool,
    /// Lock file path
    pub lock_path: Option<String>,
}

impl Default for SingleInstanceState {
    fn default() -> Self {
        Self {
            locked: AtomicBool::new(false),
            lock_path: None,
        }
    }
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize app state in the OpState
pub fn init_app_state<C: AppCapabilityChecker>(
    state: &mut OpState,
    app_info: AppInfo,
    cmd_tx: Option<mpsc::Sender<AppCommand>>,
    checker: Option<C>,
) {
    state.put(app_info);

    if let Some(tx) = cmd_tx {
        state.put(tx);
    }

    state.put(Arc::new(SingleInstanceState::default()));

    let checker: Box<dyn AppCapabilityChecker> = match checker {
        Some(c) => Box::new(c),
        None => Box::new(DefaultAppCapabilityChecker),
    };
    state.put(checker);
}

// ============================================================================
// Operations
// ============================================================================

/// Quit the application gracefully
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_quit(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_quit() {
            return Err(AppError::permission_denied("Quitting is not allowed"));
        }
    }

    info!("Application quit requested");

    // Try to send quit command
    let tx_opt = {
        let state = state.borrow();
        state.try_borrow::<mpsc::Sender<AppCommand>>().cloned()
    };

    if let Some(tx) = tx_opt {
        tx.send(AppCommand::Quit)
            .await
            .map_err(|e| AppError::quit_failed(e.to_string()))?;
    } else {
        warn!("No app command channel available, using std::process::exit");
        std::process::exit(0);
    }

    Ok(())
}

/// Force exit the application (no cleanup)
#[weld_op]
#[op2(fast)]
pub fn op_app_exit(#[smi] exit_code: i32) -> Result<(), AppError> {
    info!("Application exit requested with code: {}", exit_code);
    std::process::exit(exit_code);
}

/// Relaunch the application
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_relaunch(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_relaunch() {
            return Err(AppError::permission_denied("Relaunching is not allowed"));
        }
    }

    info!("Application relaunch requested");

    let exe_path = {
        let state = state.borrow();
        let app_info = state.borrow::<AppInfo>();
        app_info.exe_path.clone()
    };

    let exe = exe_path.ok_or_else(|| AppError::relaunch_failed("Executable path not available"))?;

    // Spawn new process
    std::process::Command::new(&exe)
        .spawn()
        .map_err(|e| AppError::relaunch_failed(e.to_string()))?;

    // Exit current process
    std::process::exit(0);
}

/// Get the application version
#[weld_op]
#[op2]
#[string]
pub fn op_app_get_version(state: &mut OpState) -> Result<String, AppError> {
    let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
    if !checker.can_get_info() {
        return Err(AppError::permission_denied("Getting app info is not allowed"));
    }
    let _ = checker;

    let app_info = state.borrow::<AppInfo>();
    Ok(app_info.version.clone())
}

/// Get the application name
#[weld_op]
#[op2]
#[string]
pub fn op_app_get_name(state: &mut OpState) -> Result<String, AppError> {
    let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
    if !checker.can_get_info() {
        return Err(AppError::permission_denied("Getting app info is not allowed"));
    }
    let _ = checker;

    let app_info = state.borrow::<AppInfo>();
    Ok(app_info.name.clone())
}

/// Get the application identifier
#[weld_op]
#[op2]
#[string]
pub fn op_app_get_identifier(state: &mut OpState) -> Result<String, AppError> {
    let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
    if !checker.can_get_info() {
        return Err(AppError::permission_denied("Getting app info is not allowed"));
    }
    let _ = checker;

    let app_info = state.borrow::<AppInfo>();
    Ok(app_info.identifier.clone())
}

/// Get a special system path
#[weld_op]
#[op2]
#[string]
pub fn op_app_get_path(
    state: &mut OpState,
    #[string] path_type: String,
) -> Result<String, AppError> {
    let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
    if !checker.can_get_paths() {
        return Err(AppError::permission_denied("Getting paths is not allowed"));
    }
    let _ = checker;

    let app_info = state.borrow::<AppInfo>();

    let path_type: PathType = path_type.parse()?;

    let path = match path_type {
        PathType::Home => dirs::home_dir(),
        PathType::AppData => dirs::data_dir(),
        PathType::Documents => dirs::document_dir(),
        PathType::Downloads => dirs::download_dir(),
        PathType::Desktop => dirs::desktop_dir(),
        PathType::Music => dirs::audio_dir(),
        PathType::Pictures => dirs::picture_dir(),
        PathType::Videos => dirs::video_dir(),
        PathType::Temp => Some(std::env::temp_dir()),
        PathType::Exe => app_info.exe_path.as_ref().map(std::path::PathBuf::from),
        PathType::Resources => app_info.resource_path.as_ref().map(std::path::PathBuf::from),
        PathType::Logs => dirs::data_dir().map(|p| p.join(&app_info.identifier).join("logs")),
        PathType::Cache => dirs::cache_dir().map(|p| p.join(&app_info.identifier)),
    };

    path.map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| AppError::path_failed(format!("Path not available: {:?}", path_type)))
}

/// Check if the application is running packaged
#[weld_op]
#[op2(fast)]
pub fn op_app_is_packaged(state: &mut OpState) -> Result<bool, AppError> {
    let app_info = state.borrow::<AppInfo>();
    Ok(app_info.is_packaged)
}

/// Get the system locale
#[weld_op]
#[op2]
#[serde]
pub fn op_app_get_locale() -> Result<LocaleInfo, AppError> {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());

    let parts: Vec<&str> = locale.split('-').collect();
    let language = parts.first().map(|s| s.to_string()).unwrap_or_else(|| "en".to_string());
    let country = parts.get(1).map(|s| s.to_string());

    Ok(LocaleInfo {
        language,
        country,
        locale,
    })
}

/// Request single instance lock
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_request_single_instance_lock(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<bool, AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_lock() {
            return Err(AppError::permission_denied("Single instance locking is not allowed"));
        }
    }

    debug!("Requesting single instance lock");

    let (identifier, single_instance_state) = {
        let state = state.borrow();
        let app_info = state.borrow::<AppInfo>();
        let si_state = state.borrow::<Arc<SingleInstanceState>>().clone();
        (app_info.identifier.clone(), si_state)
    };

    // Check if already locked
    if single_instance_state.locked.load(Ordering::SeqCst) {
        return Ok(true);
    }

    // Try to create lock file
    let lock_dir = dirs::cache_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(std::env::temp_dir);

    let lock_path = lock_dir.join(format!("{}.lock", identifier));

    // Try to create exclusive lock
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(_) => {
            single_instance_state.locked.store(true, Ordering::SeqCst);
            debug!("Single instance lock acquired: {:?}", lock_path);
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            debug!("Single instance lock already held by another instance");
            Ok(false)
        }
        Err(e) => {
            error!("Failed to create lock file: {}", e);
            Err(AppError::lock_failed(e.to_string()))
        }
    }
}

/// Release single instance lock
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_release_single_instance_lock(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    debug!("Releasing single instance lock");

    let (identifier, single_instance_state) = {
        let state = state.borrow();
        let app_info = state.borrow::<AppInfo>();
        let si_state = state.borrow::<Arc<SingleInstanceState>>().clone();
        (app_info.identifier.clone(), si_state)
    };

    if !single_instance_state.locked.load(Ordering::SeqCst) {
        return Ok(());
    }

    let lock_dir = dirs::cache_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(std::env::temp_dir);

    let lock_path = lock_dir.join(format!("{}.lock", identifier));

    if lock_path.exists() {
        std::fs::remove_file(&lock_path).map_err(|e| {
            error!("Failed to remove lock file: {}", e);
            AppError::lock_failed(e.to_string())
        })?;
    }

    single_instance_state.locked.store(false, Ordering::SeqCst);
    debug!("Single instance lock released");

    Ok(())
}

/// Bring the application to the foreground
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_focus(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_control_windows() {
            return Err(AppError::permission_denied("Window control is not allowed"));
        }
    }

    debug!("Focusing application");

    let tx_opt = {
        let state = state.borrow();
        state.try_borrow::<mpsc::Sender<AppCommand>>().cloned()
    };

    if let Some(tx) = tx_opt {
        tx.send(AppCommand::Focus)
            .await
            .map_err(|e| AppError::focus_failed(e.to_string()))?;
    } else {
        // Platform-specific fallback
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("osascript")
                .args(["-e", "tell application \"System Events\" to set frontmost of the first process whose unix id is (do shell script \"echo $PPID\") to true"])
                .spawn()
                .map_err(|e| AppError::focus_failed(e.to_string()))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            warn!("Focus not supported without app command channel");
        }
    }

    Ok(())
}

/// Hide all application windows
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_hide(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_control_windows() {
            return Err(AppError::permission_denied("Window control is not allowed"));
        }
    }

    debug!("Hiding application");

    let tx_opt = {
        let state = state.borrow();
        state.try_borrow::<mpsc::Sender<AppCommand>>().cloned()
    };

    if let Some(tx) = tx_opt {
        tx.send(AppCommand::Hide)
            .await
            .map_err(|e| AppError::hide_failed(e.to_string()))?;
    } else {
        warn!("Hide not supported without app command channel");
    }

    Ok(())
}

/// Show all application windows
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_show(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
) -> Result<(), AppError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn AppCapabilityChecker>>();
        if !checker.can_control_windows() {
            return Err(AppError::permission_denied("Window control is not allowed"));
        }
    }

    debug!("Showing application");

    let tx_opt = {
        let state = state.borrow();
        state.try_borrow::<mpsc::Sender<AppCommand>>().cloned()
    };

    if let Some(tx) = tx_opt {
        tx.send(AppCommand::Show)
            .await
            .map_err(|e| AppError::show_failed(e.to_string()))?;
    } else {
        warn!("Show not supported without app command channel");
    }

    Ok(())
}

/// Set the dock/taskbar badge count
/// Pass -1 to clear the badge, or 0+ to set the count
#[weld_op(async)]
#[op2(async)]
pub async fn op_app_set_badge_count(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[smi] count: i32,
) -> Result<(), AppError> {
    debug!("Setting badge count: {}", count);
    let count: Option<u32> = if count < 0 { None } else { Some(count as u32) };

    let tx_opt = {
        let state = state.borrow();
        state.try_borrow::<mpsc::Sender<AppCommand>>().cloned()
    };

    if let Some(tx) = tx_opt {
        tx.send(AppCommand::SetBadge(count))
            .await
            .map_err(|e| AppError::badge_failed(e.to_string()))?;
    } else {
        // Platform-specific fallback
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let badge_text = count.map(|c| c.to_string()).unwrap_or_default();
            Command::new("osascript")
                .args([
                    "-e",
                    &format!(
                        "tell application \"System Events\" to set dock tile text of application process \"forge-runtime\" to \"{}\"",
                        badge_text
                    ),
                ])
                .spawn()
                .map_err(|e| AppError::badge_failed(e.to_string()))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            warn!("Badge count not supported without app command channel");
        }
    }

    Ok(())
}

/// Set the Windows App User Model ID
#[weld_op]
#[op2(fast)]
pub fn op_app_set_user_model_id(
    _state: &mut OpState,
    #[string] _app_id: String,
) -> Result<(), AppError> {
    debug!("Setting user model ID: {}", _app_id);

    #[cfg(target_os = "windows")]
    {
        // Windows-specific implementation would go here
        // Would use SetCurrentProcessExplicitAppUserModelID
        return Err(AppError::not_supported(
            "User model ID requires Windows-specific implementation",
        ));
    }

    #[cfg(not(target_os = "windows"))]
    {
        // No-op on non-Windows platforms
        Ok(())
    }
}

// ============================================================================
// Extension Export
// ============================================================================

/// Get the app extension for registration with Deno runtime
pub fn app_extension() -> Extension {
    runtime_app::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(AppErrorCode::QuitFailed as i32, 8300);
        assert_eq!(AppErrorCode::ExitFailed as i32, 8301);
        assert_eq!(AppErrorCode::RelaunchFailed as i32, 8302);
        assert_eq!(AppErrorCode::InfoFailed as i32, 8303);
        assert_eq!(AppErrorCode::PathFailed as i32, 8304);
        assert_eq!(AppErrorCode::LockFailed as i32, 8305);
        assert_eq!(AppErrorCode::NotInitialized as i32, 8314);
    }

    #[test]
    fn test_path_type_parsing() {
        assert_eq!("home".parse::<PathType>().unwrap(), PathType::Home);
        assert_eq!("documents".parse::<PathType>().unwrap(), PathType::Documents);
        assert_eq!("downloads".parse::<PathType>().unwrap(), PathType::Downloads);
        assert_eq!("temp".parse::<PathType>().unwrap(), PathType::Temp);
        assert_eq!("tmp".parse::<PathType>().unwrap(), PathType::Temp);
        assert_eq!("appData".parse::<PathType>().unwrap(), PathType::AppData);
        assert!("invalid".parse::<PathType>().is_err());
    }

    #[test]
    fn test_app_info_default() {
        let info = AppInfo::default();
        assert_eq!(info.name, "Forge App");
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.identifier, "com.forge.app");
        assert!(!info.is_packaged);
    }

    #[test]
    fn test_locale_info_serialization() {
        let locale = LocaleInfo {
            language: "en".to_string(),
            country: Some("US".to_string()),
            locale: "en-US".to_string(),
        };
        let json = serde_json::to_string(&locale).unwrap();
        assert!(json.contains("en-US"));
        assert!(json.contains("US"));
    }

    #[test]
    fn test_default_capability_checker() {
        let checker = DefaultAppCapabilityChecker;
        assert!(checker.can_quit());
        assert!(checker.can_relaunch());
        assert!(checker.can_get_info());
        assert!(checker.can_get_paths());
        assert!(checker.can_lock());
        assert!(checker.can_control_windows());
    }
}
