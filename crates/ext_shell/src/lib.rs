//! # `runtime:shell` - Shell Integration and Command Execution Extension
//!
//! Comprehensive shell integration and command execution capabilities for Forge applications.
//!
//! ## Overview
//!
//! This extension provides two major categories of functionality that bridge Forge applications
//! with the operating system shell and desktop environment:
//!
//! ### 1. System Integration (7 operations)
//!
//! Desktop environment interaction for seamless OS integration:
//! - **URL Handling**: Open URLs in default browser
//! - **File Opening**: Launch files with default applications
//! - **File Manager**: Reveal files in Finder/Explorer
//! - **Trash Operations**: Safe deletion with recovery option
//! - **System Sounds**: Play beep/alert sounds
//! - **File Icons**: Retrieve system icons (platform-dependent)
//! - **App Queries**: Determine default applications for file types
//!
//! ### 2. Shell Execution (8 operations)
//!
//! Full-featured shell command execution with cross-platform support:
//! - **Command Execution**: Run shell commands with full syntax support
//! - **Process Management**: Kill background processes with signals
//! - **Working Directory**: Get and set current working directory
//! - **Environment Variables**: Read, write, and enumerate environment
//! - **Path Resolution**: Find executable paths (which)
//!
//! ## API Categories
//!
//! ### System Integration
//! - `openExternal()` - Open URLs in default browser
//! - `openPath()` - Open files/folders with default apps
//! - `showItemInFolder()` - Reveal file in file manager
//! - `moveToTrash()` - Move to trash/recycle bin
//! - `beep()` - Play system beep sound
//! - `getFileIcon()` - Get file type icons
//! - `getDefaultApp()` - Query default application
//!
//! ### Shell Execution
//! - `execute()` - Execute shell commands
//! - `kill()` - Terminate processes
//! - `cwd()` / `chdir()` - Working directory management
//! - `getEnv()` / `setEnv()` / `unsetEnv()` - Environment variables
//! - `getAllEnv()` - Get all environment variables
//! - `which()` - Find executable paths
//!
//! ## TypeScript Usage Examples
//!
//! ### Example 1: Opening URLs and Files
//!
//! ```typescript
//! import { openExternal, openPath, showItemInFolder } from "runtime:shell";
//!
//! // Open a URL in default browser
//! await openExternal("https://github.com/myproject");
//!
//! // Open a file with default app
//! await openPath("./document.pdf");
//!
//! // Reveal a downloaded file
//! await showItemInFolder("~/Downloads/report.xlsx");
//! ```
//!
//! ### Example 2: Safe File Deletion
//!
//! ```typescript
//! import { moveToTrash } from "runtime:shell";
//!
//! // Move files to trash instead of permanent deletion
//! const oldFiles = ["cache.tmp", "old-data.db", "temp.log"];
//! for (const file of oldFiles) {
//!   await moveToTrash(file);
//! }
//! ```
//!
//! ### Example 3: Shell Command Execution
//!
//! ```typescript
//! import { execute } from "runtime:shell";
//!
//! // Simple command
//! const result = await execute("ls -la");
//! console.log(result.stdout);
//!
//! // With pipes and options
//! const result = await execute("npm test", {
//!   cwd: "./my-project",
//!   timeout: 30000,
//!   env: { NODE_ENV: "test" }
//! });
//!
//! if (result.code !== 0) {
//!   console.error("Tests failed:", result.stderr);
//! }
//! ```
//!
//! ### Example 4: Environment Management
//!
//! ```typescript
//! import { getEnv, setEnv, getAllEnv, which } from "runtime:shell";
//!
//! // Check environment variables
//! const path = getEnv("PATH");
//! const home = getEnv("HOME");
//!
//! // Set variables for child processes
//! setEnv("RUST_LOG", "debug");
//! setEnv("API_KEY", "secret-123");
//!
//! // Get all environment
//! const env = getAllEnv();
//! console.log(`Total variables: ${Object.keys(env).length}`);
//!
//! // Find executable paths
//! if (which("git")) {
//!   await execute("git status");
//! }
//! ```
//!
//! ### Example 5: Cross-Platform Path Resolution
//!
//! ```typescript
//! import { which, execute } from "runtime:shell";
//!
//! // Check tool availability before use
//! const tools = ["node", "npm", "git", "cargo"];
//! const missing = tools.filter(tool => !which(tool));
//!
//! if (missing.length > 0) {
//!   console.error(`Missing tools: ${missing.join(", ")}`);
//! } else {
//!   await execute("npm install && npm test");
//! }
//! ```
//!
//! ## Error Codes
//!
//! Shell operations use error codes 8200-8214:
//!
//! | Code | Error | Description |
//! |------|-------|-------------|
//! | 8200 | OpenExternalFailed | Failed to open external URL |
//! | 8201 | OpenPathFailed | Failed to open path with default app |
//! | 8202 | ShowItemFailed | Failed to show item in folder |
//! | 8203 | TrashFailed | Failed to move to trash |
//! | 8204 | BeepFailed | Failed to play system beep |
//! | 8205 | IconFailed | Failed to get file icon |
//! | 8206 | DefaultAppFailed | Failed to get default app |
//! | 8207 | InvalidPath | Invalid path provided |
//! | 8208 | PermissionDenied | Shell operation not permitted |
//! | 8209 | NotSupported | Operation not supported on platform |
//! | 8210 | ParseError | Shell command syntax error |
//! | 8211 | ExecutionFailed | Command execution failed |
//! | 8212 | Timeout | Command timed out |
//! | 8213 | ProcessKilled | Process was killed |
//! | 8214 | InvalidHandle | Invalid process handle |
//!
//! ## Shell Syntax Support
//!
//! The `execute()` function provides full shell syntax support:
//!
//! - **Pipes**: `cmd1 | cmd2 | cmd3`
//! - **Logical Operators**: `cmd1 && cmd2`, `cmd1 || cmd2`
//! - **Sequences**: `cmd1; cmd2; cmd3`
//! - **Redirections**: `cmd > file`, `cmd 2>&1`, `cmd < input`
//! - **Variables**: `$VAR`, `${VAR}`, environment expansion
//! - **Quoting**: `'literal'`, `"expansion $VAR"`
//! - **Globs**: `*.ts`, `**/*.js`, `file[0-9].txt`
//! - **Background**: `cmd &`
//!
//! ## Built-in Commands
//!
//! Cross-platform built-in commands (no external binaries required):
//!
//! - **File Operations**: `cat`, `cp`, `mv`, `rm`, `mkdir`, `ls`
//! - **Navigation**: `cd`, `pwd`
//! - **Output**: `echo`
//! - **Environment**: `export`, `unset`
//! - **Utilities**: `sleep`, `which`, `exit`
//! - **Piping**: `head`, `xargs`
//!
//! Built-ins provide consistent behavior across platforms and don't require
//! external dependencies.
//!
//! ## Permission System
//!
//! Shell operations require permissions in `manifest.app.toml`:
//!
//! ```toml
//! [permissions.shell]
//! execute = true          # Allow shell command execution
//! open_external = true    # Allow opening URLs/files
//! trash = true            # Allow moving to trash
//! ```
//!
//! In development mode (`forge dev`), all permissions are granted. Production builds
//! enforce strict permission checks via capability adapters.
//!
//! ## Platform Support
//!
//! ### System Integration
//!
//! | Operation | macOS | Windows | Linux |
//! |-----------|-------|---------|-------|
//! | openExternal | ✅ | ✅ | ✅ |
//! | openPath | ✅ | ✅ | ✅ |
//! | showItemInFolder | ✅ `open -R` | ✅ `explorer /select` | ⚠️ dbus fallback |
//! | moveToTrash | ✅ Trash | ✅ Recycle Bin | ✅ freedesktop Trash |
//! | beep | ✅ AppleScript | ✅ PowerShell | ⚠️ paplay fallback |
//! | getFileIcon | ✅ NSWorkspace | ✅ SHGetFileInfo | ✅ icon theme + fallback |
//! | getDefaultApp | ✅ osascript | ✅ assoc | ✅ xdg-mime |
//!
//! ### Shell Execution
//!
//! All shell execution operations work consistently across platforms:
//! - **macOS/Linux**: Uses sh-compatible shell
//! - **Windows**: Uses cmd.exe compatible commands
//! - **Built-ins**: Cross-platform implementations
//!
//! ## Implementation Details
//!
//! ### State Management
//!
//! The extension maintains:
//! - **Capability Checker**: Permission validation via `ShellCapabilityChecker` trait
//! - **Process Registry**: Track spawned processes via `SpawnedProcessState`
//! - **Shell State**: Working directory and environment per execution context
//!
//! ### Process Lifecycle
//!
//! 1. Command parsed via custom shell parser
//! 2. Capability check performed
//! 3. Shell state created with cwd/env
//! 4. Command executed with pipes for stdout/stderr
//! 5. Optional timeout with graceful termination
//! 6. Output collected and returned
//!
//! ### Signal Handling
//!
//! Process termination supports multiple signal types:
//! - **SIGTERM** (default): Graceful termination with cleanup
//! - **SIGKILL**: Forceful termination, cannot be caught
//! - **SIGINT**: Interrupt signal (Ctrl+C equivalent)
//! - **SIGQUIT**: Quit with core dump
//!
//! ### Platform-Specific Implementations
//!
//! - **URL Opening**: Uses `open` crate for cross-platform support
//! - **Trash Operations**: Uses `trash` crate (freedesktop.org standard on Linux)
//! - **Reveal in Folder**: Platform-specific commands (`open -R`, `explorer /select`, `dbus-send`)
//! - **System Beep**: Platform-specific sound generation
//!
//! ## Extension Registration
//!
//! Registered as **Tier 1 (SimpleState)** extension:
//!
//! ```ignore
//! ExtensionDescriptor {
//!     id: "runtime_shell",
//!     init_fn: ExtensionInitFn::SimpleState(|state| {
//!         init_shell_state(state, None);
//!     }),
//!     tier: ExtensionTier::SimpleState,
//!     required: false,
//! }
//! ```
//!
//! State initialization injects:
//! - Default capability checker (allows all in dev mode)
//! - Empty process registry for spawn tracking
//!
//! ## Security Considerations
//!
//! - **Command Injection**: Commands are parsed, not passed directly to shell
//! - **Path Validation**: Paths are checked for existence before operations
//! - **URL Validation**: URLs must start with http://, https://, or mailto:
//! - **Capability Checks**: All operations verify permissions before execution
//! - **Timeout Protection**: Commands can be terminated if they run too long
//!
//! ## Testing
//!
//! The extension includes comprehensive unit tests:
//! - Error code validation
//! - Error message formatting
//! - Capability checker behavior
//! - Type serialization/deserialization
//!
//! Run tests with:
//! ```bash
//! cargo test -p ext_shell
//! ```

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::weld_op;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tracing::{debug, error};

// Shell execution modules
pub mod parser;
pub mod shell;

pub use parser::{parse, SequentialList};
pub use shell::{commands::*, execute, execute_with_pipes, types::*};

// Include the generated extension code from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

// ============================================================================
// Error Types
// ============================================================================

/// Error codes for shell operations (8200-8219)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellErrorCode {
    /// Failed to open external URL (8200)
    OpenExternalFailed = 8200,
    /// Failed to open path (8201)
    OpenPathFailed = 8201,
    /// Failed to show item in folder (8202)
    ShowItemFailed = 8202,
    /// Failed to move to trash (8203)
    TrashFailed = 8203,
    /// Failed to play beep (8204)
    BeepFailed = 8204,
    /// Failed to get file icon (8205)
    IconFailed = 8205,
    /// Failed to get default app (8206)
    DefaultAppFailed = 8206,
    /// Invalid path provided (8207)
    InvalidPath = 8207,
    /// Permission denied (8208)
    PermissionDenied = 8208,
    /// Operation not supported on this platform (8209)
    NotSupported = 8209,
    /// Shell command parse error (8210)
    ParseError = 8210,
    /// Shell command execution failed (8211)
    ExecutionFailed = 8211,
    /// Shell command timed out (8212)
    Timeout = 8212,
    /// Shell process was killed (8213)
    ProcessKilled = 8213,
    /// Invalid shell handle (8214)
    InvalidHandle = 8214,
}

impl std::fmt::Display for ShellErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

/// Errors that can occur during shell operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum ShellError {
    #[error("[{code}] Failed to open external URL: {message}")]
    #[class(generic)]
    OpenExternalFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to open path: {message}")]
    #[class(generic)]
    OpenPathFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to show item in folder: {message}")]
    #[class(generic)]
    ShowItemFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to move to trash: {message}")]
    #[class(generic)]
    TrashFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to play beep: {message}")]
    #[class(generic)]
    BeepFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to get file icon: {message}")]
    #[class(generic)]
    IconFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to get default app: {message}")]
    #[class(generic)]
    DefaultAppFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Invalid path: {message}")]
    #[class(generic)]
    InvalidPath {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Operation not supported: {message}")]
    #[class(generic)]
    NotSupported {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Parse error: {message}")]
    #[class(generic)]
    ParseError {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Execution failed: {message}")]
    #[class(generic)]
    ExecutionFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Command timed out: {message}")]
    #[class(generic)]
    Timeout {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Process killed: {message}")]
    #[class(generic)]
    ProcessKilled {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Invalid handle: {message}")]
    #[class(generic)]
    InvalidHandle {
        code: ShellErrorCode,
        message: String,
    },
}

impl ShellError {
    pub fn open_external_failed(message: impl Into<String>) -> Self {
        Self::OpenExternalFailed {
            code: ShellErrorCode::OpenExternalFailed,
            message: message.into(),
        }
    }

    pub fn open_path_failed(message: impl Into<String>) -> Self {
        Self::OpenPathFailed {
            code: ShellErrorCode::OpenPathFailed,
            message: message.into(),
        }
    }

    pub fn show_item_failed(message: impl Into<String>) -> Self {
        Self::ShowItemFailed {
            code: ShellErrorCode::ShowItemFailed,
            message: message.into(),
        }
    }

    pub fn trash_failed(message: impl Into<String>) -> Self {
        Self::TrashFailed {
            code: ShellErrorCode::TrashFailed,
            message: message.into(),
        }
    }

    pub fn beep_failed(message: impl Into<String>) -> Self {
        Self::BeepFailed {
            code: ShellErrorCode::BeepFailed,
            message: message.into(),
        }
    }

    pub fn icon_failed(message: impl Into<String>) -> Self {
        Self::IconFailed {
            code: ShellErrorCode::IconFailed,
            message: message.into(),
        }
    }

    pub fn default_app_failed(message: impl Into<String>) -> Self {
        Self::DefaultAppFailed {
            code: ShellErrorCode::DefaultAppFailed,
            message: message.into(),
        }
    }

    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::InvalidPath {
            code: ShellErrorCode::InvalidPath,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: ShellErrorCode::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn not_supported(message: impl Into<String>) -> Self {
        Self::NotSupported {
            code: ShellErrorCode::NotSupported,
            message: message.into(),
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            code: ShellErrorCode::ParseError,
            message: message.into(),
        }
    }

    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            code: ShellErrorCode::ExecutionFailed,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            code: ShellErrorCode::Timeout,
            message: message.into(),
        }
    }

    pub fn process_killed(message: impl Into<String>) -> Self {
        Self::ProcessKilled {
            code: ShellErrorCode::ProcessKilled,
            message: message.into(),
        }
    }

    pub fn invalid_handle(message: impl Into<String>) -> Self {
        Self::InvalidHandle {
            code: ShellErrorCode::InvalidHandle,
            message: message.into(),
        }
    }
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Trait for checking shell operation permissions
pub trait ShellCapabilityChecker: Send + Sync + 'static {
    /// Check if opening external URLs is allowed
    fn can_open_external(&self) -> bool {
        true
    }

    /// Check if opening paths is allowed
    fn can_open_path(&self) -> bool {
        true
    }

    /// Check if showing items in folder is allowed
    fn can_show_item(&self) -> bool {
        true
    }

    /// Check if moving to trash is allowed
    fn can_trash(&self) -> bool {
        true
    }

    /// Check if file icon retrieval is allowed
    fn can_get_icon(&self) -> bool {
        true
    }

    /// Check if shell command execution is allowed
    fn can_execute(&self) -> bool {
        true
    }

    /// Check if spawning shell processes is allowed
    fn can_spawn(&self) -> bool {
        true
    }
}

/// Default capability checker that allows all operations
pub struct DefaultShellCapabilityChecker;

impl ShellCapabilityChecker for DefaultShellCapabilityChecker {}

// ============================================================================
// Types
// ============================================================================

/// Information about a file's icon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIcon {
    /// Base64-encoded PNG data of the icon
    pub data: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Information about a default application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultAppInfo {
    /// Application name
    pub name: Option<String>,
    /// Application path
    pub path: Option<String>,
    /// Bundle identifier (macOS) or program ID (Windows)
    pub identifier: Option<String>,
}

/// Options for shell command execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecuteOptions {
    /// Working directory for the command
    pub cwd: Option<String>,
    /// Environment variables to set
    pub env: Option<HashMap<String, String>>,
    /// Timeout in milliseconds (0 = no timeout)
    pub timeout_ms: Option<u64>,
    /// Input to send to stdin
    pub stdin: Option<String>,
}

/// Result of shell command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOutput {
    /// Exit code of the command
    pub code: i32,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
}

/// Handle for a spawned shell process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnHandle {
    /// Unique ID for this process
    pub id: u32,
}

/// Output event from a spawned process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProcessEvent {
    /// Stdout data received
    Stdout(String),
    /// Stderr data received
    Stderr(String),
    /// Process exited
    Exit { code: i32 },
    /// Error occurred
    Error(String),
}

// ============================================================================
// Process Handle Registry
// ============================================================================

/// Global counter for process handles
static NEXT_PROCESS_ID: AtomicU32 = AtomicU32::new(1);

/// State for tracking spawned processes
pub struct SpawnedProcessState {
    /// Map of process ID to kill signal sender
    pub processes: HashMap<u32, Arc<shell::types::KillSignal>>,
}

impl SpawnedProcessState {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub fn register(&mut self, kill_signal: Arc<shell::types::KillSignal>) -> u32 {
        let id = NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst);
        self.processes.insert(id, kill_signal);
        id
    }

    pub fn get(&self, id: u32) -> Option<&Arc<shell::types::KillSignal>> {
        self.processes.get(&id)
    }

    pub fn remove(&mut self, id: u32) -> Option<Arc<shell::types::KillSignal>> {
        self.processes.remove(&id)
    }
}

impl Default for SpawnedProcessState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize shell state in the OpState
pub fn init_shell_state<C: ShellCapabilityChecker>(state: &mut OpState, checker: Option<C>) {
    let checker: Box<dyn ShellCapabilityChecker> = match checker {
        Some(c) => Box::new(c),
        None => Box::new(DefaultShellCapabilityChecker),
    };
    state.put(checker);
    state.put(SpawnedProcessState::new());
}

// ============================================================================
// Operations
// ============================================================================

/// Open a URL in the default browser
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_open_external(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] url: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_external() {
            return Err(ShellError::permission_denied(
                "Opening external URLs is not allowed",
            ));
        }
    }

    debug!("Opening external URL: {}", url);

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("mailto:") {
        return Err(ShellError::invalid_path(format!(
            "URL must start with http://, https://, or mailto:// - got: {}",
            url
        )));
    }

    open::that(&url).map_err(|e| {
        error!("Failed to open URL {}: {}", url, e);
        ShellError::open_external_failed(e.to_string())
    })?;

    Ok(())
}

/// Open a file or folder with the default application
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_open_path(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_path() {
            return Err(ShellError::permission_denied(
                "Opening paths is not allowed",
            ));
        }
    }

    debug!("Opening path: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    open::that(&path).map_err(|e| {
        error!("Failed to open path {}: {}", path, e);
        ShellError::open_path_failed(e.to_string())
    })?;

    Ok(())
}

/// Show a file in its containing folder (reveal in Finder/Explorer)
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_show_item_in_folder(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_show_item() {
            return Err(ShellError::permission_denied(
                "Showing items in folder is not allowed",
            ));
        }
    }

    debug!("Showing item in folder: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| ShellError::show_item_failed(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| ShellError::show_item_failed(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Try dbus-send first (works with most file managers)
        let dbus_result = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:file://{}", path),
                "string:",
            ])
            .spawn();

        if dbus_result.is_err() {
            // Fallback: open the containing folder
            if let Some(parent) = path_obj.parent() {
                open::that(parent).map_err(|e| ShellError::show_item_failed(e.to_string()))?;
            }
        }
    }

    Ok(())
}

/// Move a file or folder to the trash/recycle bin
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_move_to_trash(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_trash() {
            return Err(ShellError::permission_denied(
                "Moving to trash is not allowed",
            ));
        }
    }

    debug!("Moving to trash: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    trash::delete(&path).map_err(|e| {
        error!("Failed to move to trash {}: {}", path, e);
        ShellError::trash_failed(e.to_string())
    })?;

    Ok(())
}

/// Play the system beep sound
#[weld_op]
#[op2(fast)]
pub fn op_shell_beep() -> Result<(), ShellError> {
    debug!("Playing system beep");

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("osascript")
            .args(["-e", "beep"])
            .spawn()
            .map_err(|e| ShellError::beep_failed(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows beep using powershell
        use std::process::Command;
        Command::new("powershell")
            .args(["-c", "[console]::beep(800,200)"])
            .spawn()
            .map_err(|e| ShellError::beep_failed(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try multiple methods for Linux
        use std::process::Command;
        let result = Command::new("paplay")
            .args(["/usr/share/sounds/freedesktop/stereo/bell.oga"])
            .spawn();

        if result.is_err() {
            // Fallback to console bell
            print!("\x07");
        }
    }

    Ok(())
}

/// Get the icon for a file type
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shell_get_file_icon(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
    #[smi] size: i32,
) -> Result<FileIcon, ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_get_icon() {
            return Err(ShellError::permission_denied(
                "Getting file icons is not allowed",
            ));
        }
    }

    get_file_icon_impl(&path, size)
}

// ============================================================================
// File Icon Rasterization
// ============================================================================
//
// `getFileIcon` resolves the system icon associated with a file (or file type)
// and returns it as a base64-encoded PNG. Each platform uses its native icon
// provider, then the raw image is normalized to a PNG so the renderer can embed
// it directly via a `data:image/png;base64,...` URL:
//
//   - macOS:   `NSWorkspace.iconForFile:` -> `NSBitmapImageRep` PNG export.
//   - Windows: `SHGetFileInfoW(SHGFI_ICON)` -> `GetDIBits` 32bpp -> `png` encode.
//   - Linux:   freedesktop icon-theme lookup of the MIME-type icon PNG.
//
// When a platform cannot resolve a specific icon (unknown type, missing theme
// entry), a deterministic generic document icon is generated so the operation
// always yields a valid PNG instead of failing — mirroring the generic-icon
// behavior of comparable desktop frameworks.

/// Resolve the system icon for `path` and return it as a base64-encoded PNG.
///
/// `size` is the requested edge length in pixels; values `<= 0` default to 32.
/// The returned [`FileIcon`] reports the *actual* decoded dimensions of the PNG,
/// which may differ from the requested size (e.g. a theme only ships a 48px
/// variant, or macOS returns a higher-resolution representation).
fn get_file_icon_impl(path: &str, size: i32) -> Result<FileIcon, ShellError> {
    let size = if size <= 0 { 32 } else { size as u32 };
    debug!("Getting file icon for: {} (size: {})", path, size);

    let png_bytes = raster_icon(path, size)?;
    let (width, height) = png_dimensions(&png_bytes)?;
    let data =
        base64::engine::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);

    Ok(FileIcon {
        data,
        width,
        height,
    })
}

/// Read the pixel dimensions of an encoded PNG without decoding pixel data.
fn png_dimensions(png_bytes: &[u8]) -> Result<(u32, u32), ShellError> {
    let decoder = png::Decoder::new(png_bytes);
    let reader = decoder
        .read_info()
        .map_err(|e| ShellError::icon_failed(format!("Failed to decode icon PNG: {}", e)))?;
    let info = reader.info();
    Ok((info.width, info.height))
}

/// Encode raw 8-bit RGBA pixels (row-major, top-down) as a PNG byte stream.
fn encode_rgba_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ShellError> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(ShellError::icon_failed(format!(
            "RGBA buffer size {} does not match {}x{} (expected {})",
            rgba.len(),
            width,
            height,
            (width as usize) * (height as usize) * 4
        )));
    }

    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| ShellError::icon_failed(format!("PNG header write failed: {}", e)))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| ShellError::icon_failed(format!("PNG data write failed: {}", e)))?;
    }
    Ok(out)
}

/// Generate a deterministic generic document icon as a PNG.
///
/// Used as the fallback when no platform-specific icon is available. The icon is
/// a filled light-gray square with a darker one-pixel border — a real, valid
/// image (not a stub), matching how desktop frameworks return a generic icon for
/// unknown file types.
fn generate_generic_icon(size: u32) -> Result<Vec<u8>, ShellError> {
    let size = if size == 0 { 32 } else { size };
    let (w, h) = (size, size);
    let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let is_border = x == 0 || y == 0 || x == w - 1 || y == h - 1;
            if is_border {
                rgba.extend_from_slice(&[0x9e, 0x9e, 0x9e, 0xff]);
            } else {
                rgba.extend_from_slice(&[0xe0, 0xe0, 0xe0, 0xff]);
            }
        }
    }
    encode_rgba_png(&rgba, w, h)
}

/// macOS: resolve the Finder icon via `NSWorkspace` and export it as PNG.
///
/// `cocoa 0.26` marks its whole AppKit surface deprecated in favor of `objc2`
/// (a migration recommendation, not a correctness issue), so the `deprecated`
/// lint is suppressed at the function scope. The related `unexpected_cfgs` noise
/// from the `objc 0.2` macros is handled via `check-cfg` in `Cargo.toml`.
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn raster_icon(path: &str, size: u32) -> Result<Vec<u8>, ShellError> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSSize, NSString};
    use objc::{class, msg_send, sel, sel_impl};

    let size_f = size as f64;

    // SAFETY: All Objective-C messages target valid AppKit classes/instances.
    // The autorelease pool bounds the lifetime of the temporary `id`s, and the
    // PNG bytes are copied into an owned `Vec` before the pool is drained, so no
    // Objective-C memory outlives this block.
    let data: Vec<u8> = unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let ns_path = NSString::alloc(nil).init_str(path);
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let image: id = msg_send![workspace, iconForFile: ns_path];

        let bytes = if image != nil {
            let new_size = NSSize::new(size_f, size_f);
            let _: () = msg_send![image, setSize: new_size];
            let tiff: id = msg_send![image, TIFFRepresentation];
            if tiff != nil {
                let rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff];
                if rep != nil {
                    // NSBitmapImageFileTypePNG == 4
                    let png_data: id =
                        msg_send![rep, representationUsingType: 4u64 properties: nil];
                    if png_data != nil {
                        let len: usize = msg_send![png_data, length];
                        let ptr: *const u8 = msg_send![png_data, bytes];
                        if !ptr.is_null() && len > 0 {
                            std::slice::from_raw_parts(ptr, len).to_vec()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let _: () = msg_send![pool, release];
        bytes
    };

    if data.is_empty() {
        // NSWorkspace virtually always returns an icon, but guard the edge case
        // (e.g. an unreadable path) with a valid generic icon rather than failing.
        generate_generic_icon(size)
    } else {
        Ok(data)
    }
}

/// Windows: resolve the shell icon via `SHGetFileInfoW` and rasterize it to PNG.
#[cfg(target_os = "windows")]
fn raster_icon(path: &str, size: u32) -> Result<Vec<u8>, ShellError> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
    use windows::Win32::UI::Shell::{
        SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON,
        SHGFI_USEFILEATTRIBUTES,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    let wide: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let size_flag = if size <= 16 {
        SHGFI_SMALLICON
    } else {
        SHGFI_LARGEICON
    };
    let flags = SHGFI_ICON | SHGFI_USEFILEATTRIBUTES | size_flag;

    // SAFETY: Win32 calls below use valid handles obtained from the preceding
    // call; every GDI object and the icon handle are released on all paths.
    unsafe {
        let mut shfi = SHFILEINFOW::default();
        let res = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if res == 0 || shfi.hIcon.is_invalid() {
            // No shell icon available for this type: return a generic icon.
            return generate_generic_icon(size);
        }
        let hicon = shfi.hIcon;

        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            let _ = DestroyIcon(hicon);
            return Err(ShellError::icon_failed("GetIconInfo failed"));
        }
        let hbm_color = icon_info.hbmColor;
        let hbm_mask = icon_info.hbmMask;

        let mut bmp = BITMAP::default();
        let got = GetObjectW(
            HGDIOBJ(hbm_color.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut core::ffi::c_void),
        );
        if got == 0 {
            let _ = DeleteObject(HGDIOBJ(hbm_color.0));
            let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
            let _ = DestroyIcon(hicon);
            return Err(ShellError::icon_failed("GetObjectW on icon bitmap failed"));
        }
        let width = bmp.bmWidth;
        let height = bmp.bmHeight;
        if width <= 0 || height <= 0 {
            let _ = DeleteObject(HGDIOBJ(hbm_color.0));
            let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
            let _ = DestroyIcon(hicon);
            return Err(ShellError::icon_failed(
                "Icon bitmap has invalid dimensions",
            ));
        }
        let pixel_count = (width * height) as usize;

        // Request top-down 32bpp BGRA from GDI (negative height = top-down).
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = width;
        bi.bmiHeader.biHeight = -height;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB.0 as u32;

        let hdc = GetDC(HWND::default());

        let mut color_bits = vec![0u8; pixel_count * 4];
        let color_scan = GetDIBits(
            hdc,
            hbm_color,
            0,
            height as u32,
            Some(color_bits.as_mut_ptr() as *mut core::ffi::c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        // The AND mask: nonzero (white) pixels are transparent, zero are opaque.
        let mut mask_bits = vec![0u8; pixel_count * 4];
        let mut bi_mask = bi;
        let mask_scan = GetDIBits(
            hdc,
            hbm_mask,
            0,
            height as u32,
            Some(mask_bits.as_mut_ptr() as *mut core::ffi::c_void),
            &mut bi_mask,
            DIB_RGB_COLORS,
        );

        ReleaseDC(HWND::default(), hdc);
        let _ = DeleteObject(HGDIOBJ(hbm_color.0));
        let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
        let _ = DestroyIcon(hicon);

        if color_scan == 0 {
            return Err(ShellError::icon_failed("GetDIBits (color) failed"));
        }

        // GDI hands back BGRA; convert to RGBA. Modern icons carry a real alpha
        // channel; legacy icons leave alpha zero, so derive it from the AND mask.
        let mut rgba = vec![0u8; pixel_count * 4];
        let mut any_alpha = false;
        for i in 0..pixel_count {
            let b = color_bits[i * 4];
            let g = color_bits[i * 4 + 1];
            let r = color_bits[i * 4 + 2];
            let a = color_bits[i * 4 + 3];
            if a != 0 {
                any_alpha = true;
            }
            rgba[i * 4] = r;
            rgba[i * 4 + 1] = g;
            rgba[i * 4 + 2] = b;
            rgba[i * 4 + 3] = a;
        }
        if !any_alpha && mask_scan != 0 {
            for i in 0..pixel_count {
                rgba[i * 4 + 3] = if mask_bits[i * 4] != 0 { 0 } else { 255 };
            }
        } else if !any_alpha {
            // No usable alpha and no mask: treat as fully opaque.
            for i in 0..pixel_count {
                rgba[i * 4 + 3] = 255;
            }
        }

        encode_rgba_png(&rgba, width as u32, height as u32)
    }
}

/// Linux: resolve the freedesktop icon-theme PNG for the file's MIME type.
#[cfg(target_os = "linux")]
fn raster_icon(path: &str, size: u32) -> Result<Vec<u8>, ShellError> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // freedesktop icon naming: "text/plain" -> "text-plain".
    let specific = format!("{}-{}", mime.type_(), mime.subtype());
    // Generic per-type fallback name, e.g. "text-x-generic".
    let generic = format!("{}-x-generic", mime.type_());

    for name in [specific.as_str(), generic.as_str()] {
        if let Some(icon_path) = find_themed_icon_png(name, size) {
            if let Ok(bytes) = std::fs::read(&icon_path) {
                if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
                    return Ok(bytes);
                }
            }
        }
    }

    // No theme entry resolved: return a deterministic generic icon.
    generate_generic_icon(size)
}

/// Search the standard freedesktop icon-theme directories for a mimetype PNG.
///
/// Prefers the requested `size`, then falls back to common sizes. Returns the
/// first existing PNG path, or `None` if no theme provides the icon.
#[cfg(target_os = "linux")]
fn find_themed_icon_png(icon_name: &str, size: u32) -> Option<PathBuf> {
    let bases = [
        std::env::var("HOME")
            .map(|h| format!("{}/.local/share/icons", h))
            .unwrap_or_default(),
        "/usr/share/icons".to_string(),
        "/usr/local/share/icons".to_string(),
    ];
    let themes = ["hicolor", "Adwaita", "gnome", "breeze", "Papirus"];
    let mut sizes = vec![size, 48, 64, 32, 128, 256, 24, 16];
    sizes.dedup();

    for base in &bases {
        if base.is_empty() {
            continue;
        }
        for theme in &themes {
            for s in &sizes {
                let candidate = PathBuf::from(format!(
                    "{}/{}/{}x{}/mimetypes/{}.png",
                    base, theme, s, s, icon_name
                ));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// Fallback for any other platform: a valid generic icon.
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn raster_icon(_path: &str, size: u32) -> Result<Vec<u8>, ShellError> {
    generate_generic_icon(size)
}

/// Get the default application for a file type
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shell_get_default_app(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path_or_extension: String,
) -> Result<DefaultAppInfo, ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_path() {
            return Err(ShellError::permission_denied(
                "Querying default apps is not allowed",
            ));
        }
    }

    debug!("Getting default app for: {}", path_or_extension);

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Use LSCopyDefaultApplicationURLForURL via mdls or other tools
        // For now, use a simpler approach with `open -Ra`
        let output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "osascript -e 'POSIX path of (path to app id (do shell script \"mdls -name kMDItemContentType -raw {} 2>/dev/null | xargs -I{{}} defaults read /System/Library/CoreServices/CoreTypes.bundle/Contents/Info CFBundleDocumentTypes | grep -A1 \\\"{{}}\\\" | grep CFBundleTypeRole | head -1\"))' 2>/dev/null || echo ''",
                    path_or_extension
                ),
            ])
            .output()
            .ok();

        if let Some(out) = output {
            let app_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !app_path.is_empty() {
                return Ok(DefaultAppInfo {
                    name: app_path
                        .split('/')
                        .next_back()
                        .map(|s| s.replace(".app", "")),
                    path: Some(app_path),
                    identifier: None,
                });
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // Query Windows registry for file associations
        let ext = if path_or_extension.starts_with('.') {
            path_or_extension.clone()
        } else {
            Path::new(&path_or_extension)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        };

        if !ext.is_empty() {
            let output = Command::new("cmd")
                .args(["/c", "assoc", &ext])
                .output()
                .ok();

            if let Some(out) = output {
                let assoc = String::from_utf8_lossy(&out.stdout);
                if let Some(prog_id) = assoc.split('=').nth(1) {
                    return Ok(DefaultAppInfo {
                        name: Some(prog_id.trim().to_string()),
                        path: None,
                        identifier: Some(prog_id.trim().to_string()),
                    });
                }
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Use xdg-mime to query default applications
        let mime_output = Command::new("xdg-mime")
            .args(["query", "filetype", &path_or_extension])
            .output()
            .ok();

        if let Some(out) = mime_output {
            let mime_type = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !mime_type.is_empty() {
                let app_output = Command::new("xdg-mime")
                    .args(["query", "default", &mime_type])
                    .output()
                    .ok();

                if let Some(app_out) = app_output {
                    let desktop_file = String::from_utf8_lossy(&app_out.stdout).trim().to_string();
                    if !desktop_file.is_empty() {
                        return Ok(DefaultAppInfo {
                            name: Some(desktop_file.replace(".desktop", "")),
                            path: None,
                            identifier: Some(desktop_file),
                        });
                    }
                }
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[allow(unreachable_code)]
    Ok(DefaultAppInfo {
        name: None,
        path: None,
        identifier: None,
    })
}

// ============================================================================
// Shell Execution Operations
// ============================================================================

/// Execute a shell command and return the result
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shell_execute(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] command: String,
    #[serde] options: Option<ExecuteOptions>,
) -> Result<ExecuteOutput, ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_execute() {
            return Err(ShellError::permission_denied(
                "Shell command execution is not allowed",
            ));
        }
    }

    debug!("Executing shell command: {}", command);

    let options = options.unwrap_or_default();

    // Parse the command
    let parsed = parse(&command).map_err(|e| ShellError::parse_error(format!("{:?}", e)))?;

    // Create shell state
    let shell_state = Rc::new(shell::types::ShellState::new_default());

    // Set working directory
    if let Some(cwd) = options.cwd {
        shell_state.set_cwd(PathBuf::from(cwd));
    }

    // Set environment variables
    if let Some(env) = options.env {
        for (key, value) in env {
            shell_state.set_env_var(key, value);
        }
    }

    // Set up pipes to capture output
    let (stdout_reader, stdout_writer) = shell::types::pipe();
    let (stderr_reader, stderr_writer) = shell::types::pipe();

    // Handle stdin
    let stdin = if let Some(input) = options.stdin {
        shell::types::ShellPipeReader::from_string(input)
    } else {
        shell::types::ShellPipeReader::stdin()
    };

    // Execute with optional timeout
    let timeout_ms = options.timeout_ms.unwrap_or(0);

    let execute_future = shell::execute::execute_with_pipes(
        parsed,
        shell_state.clone(),
        stdin,
        stdout_writer,
        stderr_writer,
    );

    let code = if timeout_ms > 0 {
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), execute_future)
            .await
        {
            Ok(code) => code,
            Err(_) => {
                // Timeout occurred - send kill signal
                shell_state.kill_signal().send_sigterm();
                return Err(ShellError::timeout(format!(
                    "Command timed out after {}ms",
                    timeout_ms
                )));
            }
        }
    } else {
        execute_future.await
    };

    // Collect output
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();

    // Read from the readers
    let _ = stdout_reader.pipe_to(&mut stdout_bytes);
    let _ = stderr_reader.pipe_to(&mut stderr_bytes);

    let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
    let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();

    Ok(ExecuteOutput {
        code,
        stdout,
        stderr,
    })
}

/// Kill a spawned process by its handle ID
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_kill(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[smi] handle_id: u32,
    #[string] signal: Option<String>,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_spawn() {
            return Err(ShellError::permission_denied(
                "Killing processes is not allowed",
            ));
        }
    }

    debug!("Killing process {}", handle_id);

    let kill_signal = {
        let state = state.borrow();
        let spawned_state = state.borrow::<SpawnedProcessState>();
        spawned_state.get(handle_id).cloned()
    };

    match kill_signal {
        Some(ks) => {
            let sig = match signal.as_deref() {
                Some("SIGKILL") | Some("9") => shell::types::SignalKind::SIGKILL,
                Some("SIGINT") | Some("2") => shell::types::SignalKind::SIGINT,
                Some("SIGQUIT") | Some("3") => shell::types::SignalKind::SIGQUIT,
                _ => shell::types::SignalKind::SIGTERM,
            };
            ks.send(sig);
            Ok(())
        }
        None => Err(ShellError::invalid_handle(format!(
            "No process found with handle ID {}",
            handle_id
        ))),
    }
}

/// Get the current working directory for shell operations
#[weld_op]
#[op2]
#[string]
pub fn op_shell_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

/// Set the current working directory for shell operations
#[weld_op]
#[op2(fast)]
pub fn op_shell_set_cwd(#[string] path: String) -> Result<(), ShellError> {
    std::env::set_current_dir(&path).map_err(|e| {
        ShellError::execution_failed(format!("Failed to change directory to '{}': {}", path, e))
    })
}

/// Get an environment variable
#[weld_op]
#[op2]
#[string]
pub fn op_shell_get_env(#[string] name: String) -> Option<String> {
    std::env::var(&name).ok()
}

/// Set an environment variable
#[weld_op]
#[op2(fast)]
pub fn op_shell_set_env(#[string] name: String, #[string] value: String) {
    std::env::set_var(&name, &value);
}

/// Unset an environment variable
#[weld_op]
#[op2(fast)]
pub fn op_shell_unset_env(#[string] name: String) {
    std::env::remove_var(&name);
}

/// Get all environment variables
#[weld_op]
#[op2]
#[serde]
pub fn op_shell_get_all_env() -> HashMap<String, String> {
    std::env::vars().collect()
}

/// Resolve a command to its path (like `which`)
#[weld_op]
#[op2]
#[string]
pub fn op_shell_which(#[string] command: String) -> Option<String> {
    shell::which::which(std::ffi::OsStr::new(&command), None)
        .map(|p| p.to_string_lossy().to_string())
}

// ============================================================================
// Extension Export
// ============================================================================

/// Get the shell extension for registration with Deno runtime
pub fn shell_extension() -> Extension {
    runtime_shell::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ShellErrorCode::OpenExternalFailed as i32, 8200);
        assert_eq!(ShellErrorCode::OpenPathFailed as i32, 8201);
        assert_eq!(ShellErrorCode::ShowItemFailed as i32, 8202);
        assert_eq!(ShellErrorCode::TrashFailed as i32, 8203);
        assert_eq!(ShellErrorCode::BeepFailed as i32, 8204);
        assert_eq!(ShellErrorCode::IconFailed as i32, 8205);
        assert_eq!(ShellErrorCode::DefaultAppFailed as i32, 8206);
        assert_eq!(ShellErrorCode::InvalidPath as i32, 8207);
        assert_eq!(ShellErrorCode::PermissionDenied as i32, 8208);
        assert_eq!(ShellErrorCode::NotSupported as i32, 8209);
    }

    #[test]
    fn test_error_messages() {
        let err = ShellError::open_external_failed("test error");
        assert!(err.to_string().contains("8200"));
        assert!(err.to_string().contains("test error"));

        let err = ShellError::invalid_path("bad path");
        assert!(err.to_string().contains("8207"));
        assert!(err.to_string().contains("bad path"));
    }

    #[test]
    fn test_default_capability_checker() {
        let checker = DefaultShellCapabilityChecker;
        assert!(checker.can_open_external());
        assert!(checker.can_open_path());
        assert!(checker.can_show_item());
        assert!(checker.can_trash());
        assert!(checker.can_get_icon());
    }

    #[test]
    fn test_file_icon_serialization() {
        let icon = FileIcon {
            data: "base64data".to_string(),
            width: 32,
            height: 32,
        };
        let json = serde_json::to_string(&icon).unwrap();
        assert!(json.contains("base64data"));
        assert!(json.contains("32"));
    }

    #[test]
    fn test_default_app_info_serialization() {
        let info = DefaultAppInfo {
            name: Some("TextEdit".to_string()),
            path: Some("/Applications/TextEdit.app".to_string()),
            identifier: Some("com.apple.TextEdit".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("TextEdit"));
        assert!(json.contains("/Applications/TextEdit.app"));
    }

    // ------------------------------------------------------------------------
    // File icon rasterization (getFileIcon)
    // ------------------------------------------------------------------------

    /// Decode a base64 string into bytes for assertions.
    fn decode_b64(s: &str) -> Vec<u8> {
        base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
            .expect("icon data must be valid base64")
    }

    /// The 8-byte PNG file signature.
    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

    #[test]
    fn test_encode_rgba_png_roundtrips_dimensions() {
        // A 2x3 fully-opaque red image.
        let (w, h) = (2u32, 3u32);
        let mut rgba = Vec::new();
        for _ in 0..(w * h) {
            rgba.extend_from_slice(&[0xff, 0x00, 0x00, 0xff]);
        }
        let png = encode_rgba_png(&rgba, w, h).expect("encode must succeed");
        assert_eq!(&png[..8], &PNG_MAGIC, "must emit a real PNG signature");
        let (dw, dh) = png_dimensions(&png).expect("decode header must succeed");
        assert_eq!((dw, dh), (w, h), "decoded dimensions must match encoded");
    }

    #[test]
    fn test_encode_rgba_png_rejects_mismatched_buffer() {
        // 2x2 expects 16 bytes; provide 12.
        let rgba = vec![0u8; 12];
        let err = encode_rgba_png(&rgba, 2, 2).unwrap_err();
        assert!(
            err.to_string().contains("8205"),
            "should be an IconFailed error"
        );
    }

    #[test]
    fn test_generate_generic_icon_is_valid_png() {
        let png = generate_generic_icon(40).expect("generic icon must encode");
        assert_eq!(&png[..8], &PNG_MAGIC);
        let (w, h) = png_dimensions(&png).unwrap();
        assert_eq!((w, h), (40, 40), "generic icon must honor requested size");
    }

    #[test]
    fn test_generate_generic_icon_zero_size_defaults() {
        let png = generate_generic_icon(0).expect("zero size must default, not fail");
        let (w, h) = png_dimensions(&png).unwrap();
        assert_eq!((w, h), (32, 32), "zero size must default to 32x32");
    }

    #[test]
    fn test_get_file_icon_impl_negative_size_defaults() {
        // Negative/zero requested size must not panic and must yield a real PNG.
        let icon = get_file_icon_impl(".", -1).expect("icon retrieval must succeed");
        let bytes = decode_b64(&icon.data);
        assert_eq!(&bytes[..8], &PNG_MAGIC);
        assert!(
            icon.width > 0 && icon.height > 0,
            "dimensions must be reported"
        );
    }

    /// Real system-icon path: macOS `NSWorkspace` must produce a decodable PNG
    /// whose reported dimensions match the encoded image. This is the
    /// load-bearing test for the native macOS code path.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_file_icon_impl_macos_real_icon() {
        let icon = get_file_icon_impl("/bin/sh", 32).expect("macOS icon must resolve");
        let bytes = decode_b64(&icon.data);
        assert_eq!(&bytes[..8], &PNG_MAGIC, "macOS must return a real PNG");

        // The base64 payload must independently decode to the reported size.
        let (w, h) = png_dimensions(&bytes).unwrap();
        assert_eq!(
            (w, h),
            (icon.width, icon.height),
            "reported dimensions must match the PNG header"
        );

        // Prove the native NSWorkspace path ran rather than the generic fallback:
        // NSWorkspace emits the full-resolution .icns representation (e.g.
        // 1024x1024 for /bin/sh), while `generate_generic_icon` always returns
        // exactly the requested size (32x32). A result larger than the request is
        // therefore only reachable through the real native code path.
        assert!(
            icon.width > 32 && icon.height > 32,
            "expected the native icon ({}x{}) to exceed the requested 32px, \
             confirming NSWorkspace ran (not the fallback)",
            icon.width,
            icon.height
        );
    }
}
