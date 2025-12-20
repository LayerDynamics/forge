//! # `runtime:fs` - Filesystem Operations Extension
//!
//! Comprehensive filesystem operations for Forge applications with capability-based
//! security and cross-platform support.
//!
//! ## Overview
//!
//! This extension provides a complete filesystem API including:
//! - **File I/O**: Text and binary read/write operations
//! - **Directory Management**: Create, read, and remove directories
//! - **File Watching**: Real-time filesystem event notifications
//! - **Symbolic Links**: Create and resolve symlinks
//! - **Metadata Access**: File stats, timestamps, and permissions
//! - **Temporary Files**: Secure temporary file and directory creation
//! - **Path Resolution**: Canonical path resolution with symlink following
//!
//! All operations are subject to capability-based permissions defined in the app's
//! `manifest.app.toml` configuration file, providing granular security control.
//!
//! ## API Categories
//!
//! ### Core File I/O
//! Operations for reading and writing file contents:
//! - `readTextFile()` / `writeTextFile()` - UTF-8 text operations
//! - `readBytes()` / `writeBytes()` - Binary data operations
//! - `appendTextFile()` / `appendBytes()` - Append to existing files
//!
//! ### Directory Operations
//! Managing filesystem directory structures:
//! - `readDir()` - List directory contents
//! - `mkdir()` - Create directories (with `recursive` option)
//! - `remove()` - Delete files or directories (with `recursive` option)
//! - `copy()` - Copy files
//! - `rename()` - Move/rename files or directories
//!
//! ### Metadata & Information
//! Querying filesystem entity properties:
//! - `stat()` - Get file statistics (size, type, readonly)
//! - `metadata()` - Get extended metadata (timestamps, permissions)
//! - `exists()` - Check if path exists
//!
//! ### File Watching
//! Real-time filesystem event monitoring:
//! - `watch()` - Watch file or directory for changes
//! - Returns async iterator yielding `WatchEvent` objects
//! - Events: create, modify, remove, rename
//!
//! ### Symbolic Links
//! Cross-platform symlink support:
//! - `symlink()` - Create symbolic links
//! - `readLink()` - Read symlink target
//! - `realPath()` - Resolve to canonical absolute path
//!
//! ### Temporary Files
//! Secure temporary filesystem entities:
//! - `tempFile()` - Create temporary file with optional prefix/suffix
//! - `tempDir()` - Create temporary directory with optional prefix
//! - Files persist until explicitly deleted
//!
//! ## TypeScript Usage Examples
//!
//! ### Basic File Operations
//! ```typescript
//! import { readTextFile, writeTextFile, exists } from "runtime:fs";
//!
//! // Read a text file
//! const config = await readTextFile("./config.json");
//! const data = JSON.parse(config);
//!
//! // Write a text file
//! await writeTextFile("./output.txt", "Hello, World!");
//!
//! // Check if file exists
//! if (await exists("./data.json")) {
//!   const content = await readTextFile("./data.json");
//!   console.log("File contents:", content);
//! }
//! ```
//!
//! ### Binary File Operations
//! ```typescript
//! import { readBytes, writeBytes } from "runtime:fs";
//!
//! // Read binary data
//! const imageData = await readBytes("./image.png");
//! const buffer = new Uint8Array(imageData);
//!
//! // Write binary data
//! const data = new Uint8Array([0x89, 0x50, 0x4E, 0x47]);
//! await writeBytes("./output.bin", data);
//! ```
//!
//! ### Directory Operations
//! ```typescript
//! import { readDir, mkdir, remove, stat } from "runtime:fs";
//!
//! // List directory contents
//! const entries = await readDir("./data");
//! for (const entry of entries) {
//!   console.log(`${entry.name} (${entry.isFile ? "file" : "directory"})`);
//! }
//!
//! // Create nested directories
//! await mkdir("./data/cache/images", { recursive: true });
//!
//! // Remove directory and contents
//! await remove("./temp", { recursive: true });
//! ```
//!
//! ### File Watching
//! ```typescript
//! import { watch } from "runtime:fs";
//!
//! // Watch directory for changes
//! const watcher = await watch("./data");
//! try {
//!   for await (const event of watcher) {
//!     console.log(`${event.kind}: ${event.paths.join(", ")}`);
//!   }
//! } finally {
//!   await watcher.close(); // Always clean up
//! }
//! ```
//!
//! ### Symbolic Links
//! ```typescript
//! import { symlink, readLink, realPath } from "runtime:fs";
//!
//! // Create a symbolic link
//! await symlink("/var/log/app", "./logs");
//!
//! // Read where symlink points
//! const target = await readLink("./logs");
//! console.log(`Points to: ${target}`); // "/var/log/app"
//!
//! // Resolve to canonical path
//! const canonical = await realPath("./logs");
//! console.log(`Canonical: ${canonical}`); // "/var/log/app"
//! ```
//!
//! ### Temporary Files
//! ```typescript
//! import { tempFile, tempDir, writeTextFile, remove } from "runtime:fs";
//!
//! // Create temp file for processing
//! const temp = await tempFile("process-", ".json");
//! try {
//!   await writeTextFile(temp.path, JSON.stringify({ data: "test" }));
//!   await processFile(temp.path);
//! } finally {
//!   await remove(temp.path);
//! }
//!
//! // Create temp directory for batch work
//! const tempDirectory = await tempDir("build-");
//! try {
//!   await mkdir(`${tempDirectory.path}/output`);
//!   // ... process files ...
//! } finally {
//!   await remove(tempDirectory.path, { recursive: true });
//! }
//! ```
//!
//! ## Error Codes
//!
//! All filesystem operations use structured error codes (3000-3009 range):
//!
//! | Code | Error | Description |
//! |------|-------|-------------|
//! | 3000 | `Io` | Generic I/O error during filesystem operation |
//! | 3001 | `PermissionDenied` | Capability system denied permission |
//! | 3002 | `NotFound` | File or directory not found |
//! | 3003 | `AlreadyExists` | File or directory already exists |
//! | 3004 | `IsDirectory` | Path is a directory (expected file) |
//! | 3005 | `IsFile` | Path is a file (expected directory) |
//! | 3006 | `Watch` | File watch setup or operation error |
//! | 3007 | `InvalidWatchId` | Invalid or closed watch ID |
//! | 3008 | `Symlink` | Symbolic link creation or resolution error |
//! | 3009 | `TempError` | Temporary file/directory creation error |
//!
//! ## Permission System
//!
//! Filesystem operations require explicit permissions in `manifest.app.toml`:
//!
//! ```toml
//! [permissions.fs]
//! # Allow reading specific paths (glob patterns supported)
//! read = [
//!   "./data/**",           # All files under ./data/
//!   "./config/*.json",     # All JSON files in ./config/
//!   "/etc/hosts"           # Specific absolute path
//! ]
//!
//! # Allow writing specific paths
//! write = [
//!   "./data/**",           # All files under ./data/
//!   "./logs/*.log",        # Log files only
//!   "/tmp/**"              # System temp directory
//! ]
//! ```
//!
//! **Development Mode**: When running with `forge dev`, all filesystem permissions are
//! granted by default for rapid development. Production builds enforce the permission
//! configuration strictly.
//!
//! **Permission Checks**: Every filesystem operation checks both read and write permissions:
//! - Read ops (`readTextFile`, `readBytes`, `stat`, etc.) require `read` permission
//! - Write ops (`writeTextFile`, `mkdir`, `remove`, etc.) require `write` permission
//! - Copy/rename require both permissions (read source, write destination)
//!
//! ## Platform Support
//!
//! ### Cross-Platform Compatibility
//! - **macOS, Linux, Windows**: All core operations fully supported
//! - **Path separators**: Use forward slashes (`/`) in paths; automatically converted on Windows
//! - **Symlinks**: Fully supported on all platforms
//!   - **Windows**: Directory symlinks require admin privileges or Developer Mode
//!   - **Unix**: No special requirements
//!
//! ### Platform-Specific Behaviors
//! - **Timestamps**: `birthtime` (creation time) may be `null` on some Linux filesystems
//! - **Permissions**: Unix-style permission bits only available on Unix systems (via `metadata()`)
//! - **Case sensitivity**: macOS default filesystems are case-insensitive; Linux/Windows vary
//! - **File watching**: Uses platform-optimal backend (FSEvents on macOS, inotify on Linux, etc.)
//!
//! ## Implementation Details
//!
//! ### State Management
//! This extension maintains state in `OpState`:
//! - `FsWatchState`: Manages active file watchers and their receivers
//! - `FsCapabilities`: Holds the capability checker for permission validation
//!
//! ### Async I/O
//! All filesystem operations use `tokio::fs` for non-blocking async I/O, ensuring
//! the Deno event loop remains responsive even during heavy filesystem activity.
//!
//! ### File Watching
//! Implemented using the `notify` crate with platform-optimal backends:
//! - Watchers use async channels (`mpsc::Receiver`) for event delivery
//! - Events are buffered (64-event capacity) to handle bursts
//! - Must call `close()` to clean up resources and stop the watcher
//!
//! ### Capability Integration
//! The extension accepts an optional `Arc<dyn FsCapabilityChecker>` during initialization.
//! The runtime injects a checker that evaluates glob patterns from `manifest.app.toml`.
//! If no checker is provided, a permissive checker allows all operations (dev mode).
//!
//! ## See Also
//!
//! - [`ext_path`](../ext_path/index.html) - Path manipulation utilities
//! - [`ext_process`](../ext_process/index.html) - Child process management
//! - [`ext_storage`](../ext_storage/index.html) - Persistent key-value storage

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for filesystem operations (for machine-readable errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FsErrorCode {
    /// Generic IO error
    Io = 3000,
    /// Permission denied by capability system
    PermissionDenied = 3001,
    /// File or directory not found
    NotFound = 3002,
    /// File already exists
    AlreadyExists = 3003,
    /// Path is a directory, expected file
    IsDirectory = 3004,
    /// Path is a file, expected directory
    IsFile = 3005,
    /// Watch error
    Watch = 3006,
    /// Invalid watch ID
    InvalidWatchId = 3007,
    /// Symlink error
    Symlink = 3008,
    /// Temp file/dir error
    TempError = 3009,
}

/// Custom error type for FS operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum FsError {
    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    Io { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] Already exists: {message}")]
    #[class(generic)]
    AlreadyExists { code: u32, message: String },

    #[error("[{code}] Is directory: {message}")]
    #[class(generic)]
    IsDirectory { code: u32, message: String },

    #[error("[{code}] Is file: {message}")]
    #[class(generic)]
    IsFile { code: u32, message: String },

    #[error("[{code}] Watch error: {message}")]
    #[class(generic)]
    Watch { code: u32, message: String },

    #[error("[{code}] Invalid watch ID: {message}")]
    #[class(generic)]
    InvalidWatchId { code: u32, message: String },

    #[error("[{code}] Symlink error: {message}")]
    #[class(generic)]
    Symlink { code: u32, message: String },

    #[error("[{code}] Temp error: {message}")]
    #[class(generic)]
    TempError { code: u32, message: String },
}

impl FsError {
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            code: FsErrorCode::Io as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: FsErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: FsErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::AlreadyExists {
            code: FsErrorCode::AlreadyExists as u32,
            message: message.into(),
        }
    }

    pub fn is_directory(message: impl Into<String>) -> Self {
        Self::IsDirectory {
            code: FsErrorCode::IsDirectory as u32,
            message: message.into(),
        }
    }

    pub fn is_file(message: impl Into<String>) -> Self {
        Self::IsFile {
            code: FsErrorCode::IsFile as u32,
            message: message.into(),
        }
    }

    pub fn watch(message: impl Into<String>) -> Self {
        Self::Watch {
            code: FsErrorCode::Watch as u32,
            message: message.into(),
        }
    }

    pub fn invalid_watch_id(message: impl Into<String>) -> Self {
        Self::InvalidWatchId {
            code: FsErrorCode::InvalidWatchId as u32,
            message: message.into(),
        }
    }

    pub fn symlink(message: impl Into<String>) -> Self {
        Self::Symlink {
            code: FsErrorCode::Symlink as u32,
            message: message.into(),
        }
    }

    pub fn temp_error(message: impl Into<String>) -> Self {
        Self::TempError {
            code: FsErrorCode::TempError as u32,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for FsError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::not_found(e.to_string()),
            std::io::ErrorKind::AlreadyExists => Self::already_exists(e.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::permission_denied(e.to_string()),
            std::io::ErrorKind::IsADirectory => Self::is_directory(e.to_string()),
            _ => Self::io(e.to_string()),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// File event from watch
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub kind: String,
    pub paths: Vec<String>,
}

/// Entry for a file watcher
pub struct WatchEntry {
    pub receiver: mpsc::Receiver<FileEvent>,
    pub watcher: notify::RecommendedWatcher,
}

/// State for file watchers
pub struct FsWatchState {
    pub watchers: HashMap<String, WatchEntry>,
    pub next_id: u64,
}

impl Default for FsWatchState {
    fn default() -> Self {
        Self {
            watchers: HashMap::new(),
            next_id: 1,
        }
    }
}

/// File stat information
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct FileStat {
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub readonly: bool,
}

/// Directory entry
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct DirEntry {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Options for mkdir
#[derive(Debug, Deserialize)]
pub struct MkdirOpts {
    pub recursive: Option<bool>,
}

/// Options for remove
#[derive(Debug, Deserialize)]
pub struct RemoveOpts {
    pub recursive: Option<bool>,
}

/// Extended file metadata with timestamps and permissions
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct FileMetadata {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub readonly: bool,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
    pub accessed_at: Option<u64>,
    #[cfg(unix)]
    pub permissions: Option<u32>,
    #[cfg(not(unix))]
    pub permissions: Option<u32>,
}

/// Information about a temporary file
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct TempFileInfo {
    pub path: String,
}

/// Information about a temporary directory
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct TempDirInfo {
    pub path: String,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait - allows forge-host to inject capability checking
pub trait FsCapabilityChecker: Send + Sync {
    fn check_read(&self, path: &str) -> Result<(), String>;
    fn check_write(&self, path: &str) -> Result<(), String>;
}

/// Default permissive checker (for dev mode or when no checker is provided)
pub struct PermissiveChecker;

impl FsCapabilityChecker for PermissiveChecker {
    fn check_read(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_write(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store the capability checker in OpState
pub struct FsCapabilities {
    pub checker: Arc<dyn FsCapabilityChecker>,
}

impl Default for FsCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to check fs read capability
fn check_fs_read(state: &OpState, path: &str) -> Result<(), FsError> {
    if let Some(caps) = state.try_borrow::<FsCapabilities>() {
        caps.checker
            .check_read(path)
            .map_err(FsError::permission_denied)
    } else {
        // No capabilities configured, allow (dev mode)
        Ok(())
    }
}

/// Helper to check fs write capability
fn check_fs_write(state: &OpState, path: &str) -> Result<(), FsError> {
    if let Some(caps) = state.try_borrow::<FsCapabilities>() {
        caps.checker
            .check_write(path)
            .map_err(FsError::permission_denied)
    } else {
        // No capabilities configured, allow (dev mode)
        Ok(())
    }
}

// ============================================================================
// Operations
// ============================================================================

#[weld_op(async)]
#[op2(async)]
#[string]
async fn op_fs_read_text(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.read_text");
    let text = tokio::fs::read_to_string(&path).await?;
    debug!(path = %path, len = text.len(), "fs.read_text complete");
    Ok(text)
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_write_text(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[string] content: String,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, len = content.len(), "fs.write_text");
    tokio::fs::write(&path, content).await?;
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_read_bytes(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<Vec<u8>, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.read_bytes");
    let bytes = tokio::fs::read(&path).await?;
    debug!(path = %path, len = bytes.len(), "fs.read_bytes complete");
    Ok(bytes)
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_write_bytes(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] content: Vec<u8>,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, len = content.len(), "fs.write_bytes");
    tokio::fs::write(&path, content).await?;
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_stat(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<FileStat, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.stat");
    let metadata = tokio::fs::metadata(&path).await?;
    Ok(FileStat {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        readonly: metadata.permissions().readonly(),
    })
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_read_dir(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<Vec<DirEntry>, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.read_dir");
    let mut entries = Vec::new();
    let mut dir = tokio::fs::read_dir(&path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_file: file_type.is_file(),
            is_dir: file_type.is_dir(),
        });
    }
    debug!(path = %path, count = entries.len(), "fs.read_dir complete");
    Ok(entries)
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_mkdir(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] opts: MkdirOpts,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, recursive = ?opts.recursive, "fs.mkdir");
    if opts.recursive.unwrap_or(false) {
        tokio::fs::create_dir_all(&path).await?;
    } else {
        tokio::fs::create_dir(&path).await?;
    }
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_remove(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] opts: RemoveOpts,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, recursive = ?opts.recursive, "fs.remove");
    let metadata = tokio::fs::metadata(&path).await?;
    if metadata.is_dir() {
        if opts.recursive.unwrap_or(false) {
            tokio::fs::remove_dir_all(&path).await?;
        } else {
            tokio::fs::remove_dir(&path).await?;
        }
    } else {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_rename(
    state: Rc<RefCell<OpState>>,
    #[string] from: String,
    #[string] to: String,
) -> Result<(), FsError> {
    // Check capabilities for both paths
    {
        let s = state.borrow();
        check_fs_read(&s, &from)?;
        check_fs_write(&s, &to)?;
    }

    debug!(from = %from, to = %to, "fs.rename");
    tokio::fs::rename(&from, &to).await?;
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_copy(
    state: Rc<RefCell<OpState>>,
    #[string] from: String,
    #[string] to: String,
) -> Result<(), FsError> {
    // Check capabilities for both paths
    {
        let s = state.borrow();
        check_fs_read(&s, &from)?;
        check_fs_write(&s, &to)?;
    }

    debug!(from = %from, to = %to, "fs.copy");
    tokio::fs::copy(&from, &to).await?;
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_exists(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<bool, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.exists");
    Ok(tokio::fs::try_exists(&path).await.unwrap_or(false))
}

// File watching operations using notify crate
#[weld_op(async)]
#[op2(async)]
#[string]
async fn op_fs_watch(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, FsError> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::path::Path;

    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.watch");

    let (tx, rx) = mpsc::channel::<FileEvent>(64);

    let watcher_tx = tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let kind = format!("{:?}", event.kind);
                let paths: Vec<String> = event
                    .paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                let _ = watcher_tx.blocking_send(FileEvent { kind, paths });
            }
        },
        Config::default(),
    )
    .map_err(|e| FsError::watch(e.to_string()))?;

    watcher
        .watch(Path::new(&path), RecursiveMode::Recursive)
        .map_err(|e| FsError::watch(e.to_string()))?;

    // Generate watch ID and store both watcher and receiver
    let watch_id = {
        let mut s = state.borrow_mut();
        let watch_state = s.try_borrow_mut::<FsWatchState>();
        match watch_state {
            Some(ws) => {
                let id = format!("watch-{}", ws.next_id);
                ws.next_id += 1;
                ws.watchers.insert(
                    id.clone(),
                    WatchEntry {
                        receiver: rx,
                        watcher,
                    },
                );
                id
            }
            None => {
                // Initialize watch state
                let mut ws = FsWatchState::default();
                let id = format!("watch-{}", ws.next_id);
                ws.next_id += 1;
                ws.watchers.insert(
                    id.clone(),
                    WatchEntry {
                        receiver: rx,
                        watcher,
                    },
                );
                s.put(ws);
                id
            }
        }
    };

    debug!(path = %path, watch_id = %watch_id, "fs.watch started");
    Ok(watch_id)
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_watch_next(
    state: Rc<RefCell<OpState>>,
    #[string] watch_id: String,
) -> Result<Option<FileEvent>, FsError> {
    // Take the entry out of state temporarily
    let maybe_entry = {
        let mut s = state.borrow_mut();
        if let Some(ws) = s.try_borrow_mut::<FsWatchState>() {
            ws.watchers.remove(&watch_id)
        } else {
            None
        }
    };

    match maybe_entry {
        Some(mut entry) => {
            let result = entry.receiver.recv().await;

            // Put the entry back
            {
                let mut s = state.borrow_mut();
                if let Some(ws) = s.try_borrow_mut::<FsWatchState>() {
                    ws.watchers.insert(watch_id, entry);
                }
            }

            Ok(result)
        }
        None => Err(FsError::invalid_watch_id(watch_id)),
    }
}

#[weld_op(async)]
#[op2(async)]
async fn op_fs_watch_close(
    state: Rc<RefCell<OpState>>,
    #[string] watch_id: String,
) -> Result<(), FsError> {
    debug!(watch_id = %watch_id, "fs.watch_close");
    let mut s = state.borrow_mut();
    if let Some(ws) = s.try_borrow_mut::<FsWatchState>() {
        // Remove the entry - this will drop both the receiver and the watcher
        if let Some(entry) = ws.watchers.remove(&watch_id) {
            // Explicitly drop to make the cleanup clear
            drop(entry.watcher);
            drop(entry.receiver);
        }
    }
    Ok(())
}

// ============================================================================
// Enhanced Operations (symlink, append, metadata, temp)
// ============================================================================

/// Create a symbolic link
#[weld_op(async)]
#[op2(async)]
async fn op_fs_symlink(
    state: Rc<RefCell<OpState>>,
    #[string] target: String,
    #[string] path: String,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &target)?;
        check_fs_write(&s, &path)?;
    }

    debug!(target = %target, path = %path, "fs.symlink");

    #[cfg(unix)]
    {
        tokio::fs::symlink(&target, &path).await?;
    }

    #[cfg(windows)]
    {
        let target_path = std::path::Path::new(&target);
        if target_path.is_dir() {
            tokio::fs::symlink_dir(&target, &path).await?;
        } else {
            tokio::fs::symlink_file(&target, &path).await?;
        }
    }

    Ok(())
}

/// Read the target of a symbolic link
#[weld_op(async)]
#[op2(async)]
#[string]
async fn op_fs_read_link(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.read_link");
    let target = tokio::fs::read_link(&path).await?;
    Ok(target.to_string_lossy().to_string())
}

/// Append text to a file
#[weld_op(async)]
#[op2(async)]
async fn op_fs_append_text(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[string] content: String,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, len = content.len(), "fs.append_text");

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}

/// Append bytes to a file
#[weld_op(async)]
#[op2(async)]
async fn op_fs_append_bytes(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] content: Vec<u8>,
) -> Result<(), FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_write(&s, &path)?;
    }

    debug!(path = %path, len = content.len(), "fs.append_bytes");

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    file.write_all(&content).await?;
    file.flush().await?;

    Ok(())
}

/// Get extended file metadata including timestamps
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_metadata(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<FileMetadata, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.metadata");

    let metadata = tokio::fs::symlink_metadata(&path).await?;

    let created_at = metadata
        .created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    let accessed_at = metadata
        .accessed()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        Some(metadata.permissions().mode())
    };

    #[cfg(not(unix))]
    let permissions = None;

    Ok(FileMetadata {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.is_symlink(),
        size: metadata.len(),
        readonly: metadata.permissions().readonly(),
        created_at,
        modified_at,
        accessed_at,
        permissions,
    })
}

/// Resolve a path to its canonical, absolute form (resolving symlinks)
#[weld_op(async)]
#[op2(async)]
#[string]
async fn op_fs_real_path(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, FsError> {
    // Check capabilities
    {
        let s = state.borrow();
        check_fs_read(&s, &path)?;
    }

    debug!(path = %path, "fs.real_path");
    let canonical = tokio::fs::canonicalize(&path).await?;
    Ok(canonical.to_string_lossy().to_string())
}

/// Create a temporary file and return its path
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_temp_file(
    state: Rc<RefCell<OpState>>,
    #[string] prefix: Option<String>,
    #[string] suffix: Option<String>,
) -> Result<TempFileInfo, FsError> {
    // Check write capability for temp directory
    {
        let s = state.borrow();
        let temp_dir = std::env::temp_dir();
        check_fs_write(&s, temp_dir.to_string_lossy().as_ref())?;
    }

    debug!(prefix = ?prefix, suffix = ?suffix, "fs.temp_file");

    let mut builder = tempfile::Builder::new();
    if let Some(p) = &prefix {
        builder.prefix(p);
    }
    if let Some(s) = &suffix {
        builder.suffix(s);
    }

    // Create a named temp file that persists (won't be deleted when handle is dropped)
    let temp_file = builder
        .tempfile()
        .map_err(|e| FsError::temp_error(e.to_string()))?;

    // Keep the file by converting to a path (don't auto-delete)
    let (_, path_buf) = temp_file
        .keep()
        .map_err(|e| FsError::temp_error(e.to_string()))?;

    Ok(TempFileInfo {
        path: path_buf.to_string_lossy().to_string(),
    })
}

/// Create a temporary directory and return its path
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_fs_temp_dir(
    state: Rc<RefCell<OpState>>,
    #[string] prefix: Option<String>,
) -> Result<TempDirInfo, FsError> {
    // Check write capability for temp directory
    {
        let s = state.borrow();
        let temp_dir = std::env::temp_dir();
        check_fs_write(&s, temp_dir.to_string_lossy().as_ref())?;
    }

    debug!(prefix = ?prefix, "fs.temp_dir");

    let mut builder = tempfile::Builder::new();
    if let Some(p) = &prefix {
        builder.prefix(p);
    }

    // Create a temp directory that persists
    let temp_dir = builder
        .tempdir()
        .map_err(|e| FsError::temp_error(e.to_string()))?;

    // Persist the directory by consuming TempDir (prevents auto-delete)
    let path = temp_dir.keep();

    Ok(TempDirInfo {
        path: path.to_string_lossy().to_string(),
    })
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize FS state in OpState
pub fn init_fs_state(op_state: &mut OpState, capabilities: Option<Arc<dyn FsCapabilityChecker>>) {
    op_state.put(FsWatchState::default());
    if let Some(caps) = capabilities {
        op_state.put(FsCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs (contains transpiled TypeScript)
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn fs_extension() -> Extension {
    runtime_fs::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = FsError::permission_denied("test");
        match err {
            FsError::PermissionDenied { code, .. } => {
                assert_eq!(code, FsErrorCode::PermissionDenied as u32);
            }
            _ => panic!("Wrong error type"),
        }

        let err = FsError::not_found("test");
        match err {
            FsError::NotFound { code, .. } => {
                assert_eq!(code, FsErrorCode::NotFound as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let fs_err: FsError = io_err.into();
        match fs_err {
            FsError::NotFound { code, .. } => {
                assert_eq!(code, FsErrorCode::NotFound as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }
}
