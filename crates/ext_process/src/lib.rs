//! runtime:process extension - Process spawning for Forge apps
//!
//! Provides child process spawning, I/O, and management
//! with capability-based security.

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tracing::debug;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for process operations (for machine-readable errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProcessErrorCode {
    /// Generic IO error
    Io = 4000,
    /// Permission denied by capability system
    PermissionDenied = 4001,
    /// Binary not found
    NotFound = 4002,
    /// Failed to spawn process
    FailedToSpawn = 4003,
    /// Process exited with error
    ProcessExited = 4004,
    /// Operation timeout
    Timeout = 4005,
    /// Invalid process handle
    InvalidHandle = 4006,
    /// Stdin is closed or not captured
    StdinClosed = 4007,
    /// Output stream not captured
    OutputNotCaptured = 4008,
    /// Too many concurrent processes
    TooManyProcesses = 4009,
}

/// Custom error type for Process operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum ProcessError {
    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    Io { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] Failed to spawn: {message}")]
    #[class(generic)]
    FailedToSpawn { code: u32, message: String },

    #[error("[{code}] Process exited: {message}")]
    #[class(generic)]
    ProcessExited { code: u32, message: String },

    #[error("[{code}] Timeout: {message}")]
    #[class(generic)]
    Timeout { code: u32, message: String },

    #[error("[{code}] Invalid handle: {message}")]
    #[class(generic)]
    InvalidHandle { code: u32, message: String },

    #[error("[{code}] Stdin closed: {message}")]
    #[class(generic)]
    StdinClosed { code: u32, message: String },

    #[error("[{code}] Output not captured: {message}")]
    #[class(generic)]
    OutputNotCaptured { code: u32, message: String },

    #[error("[{code}] Too many processes: {message}")]
    #[class(generic)]
    TooManyProcesses { code: u32, message: String },
}

impl ProcessError {
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            code: ProcessErrorCode::Io as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: ProcessErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: ProcessErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn failed_to_spawn(message: impl Into<String>) -> Self {
        Self::FailedToSpawn {
            code: ProcessErrorCode::FailedToSpawn as u32,
            message: message.into(),
        }
    }

    pub fn process_exited(message: impl Into<String>) -> Self {
        Self::ProcessExited {
            code: ProcessErrorCode::ProcessExited as u32,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            code: ProcessErrorCode::Timeout as u32,
            message: message.into(),
        }
    }

    pub fn invalid_handle(message: impl Into<String>) -> Self {
        Self::InvalidHandle {
            code: ProcessErrorCode::InvalidHandle as u32,
            message: message.into(),
        }
    }

    pub fn stdin_closed(message: impl Into<String>) -> Self {
        Self::StdinClosed {
            code: ProcessErrorCode::StdinClosed as u32,
            message: message.into(),
        }
    }

    pub fn output_not_captured(message: impl Into<String>) -> Self {
        Self::OutputNotCaptured {
            code: ProcessErrorCode::OutputNotCaptured as u32,
            message: message.into(),
        }
    }

    pub fn too_many_processes(message: impl Into<String>) -> Self {
        Self::TooManyProcesses {
            code: ProcessErrorCode::TooManyProcesses as u32,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::not_found(e.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::permission_denied(e.to_string()),
            std::io::ErrorKind::TimedOut => Self::timeout(e.to_string()),
            _ => Self::io(e.to_string()),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Options for spawning a process
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SpawnOpts {
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub stdout: Option<String>, // "piped", "inherit", "null"
    pub stderr: Option<String>,
    pub stdin: Option<String>,
}

/// Result of spawning a process
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SpawnResult {
    pub id: String,
    pub pid: u32,
}

/// Process status information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ProcessStatus {
    pub running: bool,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
}

/// Output from reading stdout/stderr
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ProcessOutput {
    pub data: Option<String>,
    pub eof: bool,
}

// ============================================================================
// State Management
// ============================================================================

/// Handle to a spawned process with its I/O streams
pub struct ProcessHandle {
    pub child: Arc<Mutex<Child>>,
    pub pid: u32,
    pub binary: String,
    pub stdout: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    pub stderr: Option<Arc<Mutex<BufReader<ChildStderr>>>>,
    pub stdin: Option<Arc<Mutex<ChildStdin>>>,
    pub exited: bool,
    pub exit_code: Option<i32>,
}

/// State for tracking spawned processes
pub struct ProcessState {
    pub processes: HashMap<String, ProcessHandle>,
    pub next_id: u64,
    pub max_processes: usize,
}

impl ProcessState {
    pub fn new(max_processes: usize) -> Self {
        Self {
            processes: HashMap::new(),
            next_id: 1,
            max_processes,
        }
    }

    pub fn can_spawn(&self) -> bool {
        self.processes.len() < self.max_processes
    }
}

impl Default for ProcessState {
    fn default() -> Self {
        Self::new(10) // Default max 10 concurrent processes
    }
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for process operations
pub trait ProcessCapabilityChecker: Send + Sync {
    fn check_spawn(&self, binary: &str) -> Result<(), String>;
    fn check_env(&self, key: &str) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveProcessChecker;

impl ProcessCapabilityChecker for PermissiveProcessChecker {
    fn check_spawn(&self, _binary: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_env(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store in OpState
pub struct ProcessCapabilities {
    pub checker: Arc<dyn ProcessCapabilityChecker>,
}

impl Default for ProcessCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveProcessChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn check_spawn(state: &OpState, binary: &str) -> Result<(), ProcessError> {
    if let Some(caps) = state.try_borrow::<ProcessCapabilities>() {
        caps.checker
            .check_spawn(binary)
            .map_err(ProcessError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_env(state: &OpState, key: &str) -> Result<(), ProcessError> {
    if let Some(caps) = state.try_borrow::<ProcessCapabilities>() {
        caps.checker
            .check_env(key)
            .map_err(ProcessError::permission_denied)
    } else {
        Ok(())
    }
}

fn parse_stdio(s: Option<&String>) -> Stdio {
    match s.map(|s| s.as_str()) {
        Some("piped") => Stdio::piped(),
        Some("inherit") => Stdio::inherit(),
        Some("null") => Stdio::null(),
        None => Stdio::null(), // Default to null for safety
        _ => Stdio::null(),
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Spawn a new child process
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_process_spawn(
    state: Rc<RefCell<OpState>>,
    #[string] binary: String,
    #[serde] opts: Option<SpawnOpts>,
) -> Result<SpawnResult, ProcessError> {
    let opts = opts.unwrap_or_default();

    // Check capabilities
    {
        let s = state.borrow();
        check_spawn(&s, &binary)?;

        // Check env vars if any
        if let Some(env) = &opts.env {
            for key in env.keys() {
                check_env(&s, key)?;
            }
        }

        // Check if we can spawn more processes
        if let Some(ps) = s.try_borrow::<ProcessState>() {
            if !ps.can_spawn() {
                return Err(ProcessError::too_many_processes(format!(
                    "Maximum of {} concurrent processes reached",
                    ps.max_processes
                )));
            }
        }
    }

    debug!(binary = %binary, args = ?opts.args, "process.spawn");

    // Build command
    let mut cmd = tokio::process::Command::new(&binary);

    // Add arguments
    if let Some(args) = &opts.args {
        cmd.args(args);
    }

    // Set working directory
    if let Some(cwd) = &opts.cwd {
        cmd.current_dir(cwd);
    }

    // Set environment variables
    if let Some(env) = &opts.env {
        for (key, value) in env {
            cmd.env(key, value);
        }
    }

    // Configure stdio
    let stdout_piped = opts.stdout.as_deref() == Some("piped");
    let stderr_piped = opts.stderr.as_deref() == Some("piped");
    let stdin_piped = opts.stdin.as_deref() == Some("piped");

    cmd.stdout(parse_stdio(opts.stdout.as_ref()));
    cmd.stderr(parse_stdio(opts.stderr.as_ref()));
    cmd.stdin(parse_stdio(opts.stdin.as_ref()));

    // Spawn the process
    let mut child = cmd
        .spawn()
        .map_err(|e| ProcessError::failed_to_spawn(e.to_string()))?;

    // Get PID
    let pid = child
        .id()
        .ok_or_else(|| ProcessError::failed_to_spawn("Process has no PID"))?;

    // Extract I/O streams
    let stdout = if stdout_piped {
        child
            .stdout
            .take()
            .map(|s| Arc::new(Mutex::new(BufReader::new(s))))
    } else {
        None
    };

    let stderr = if stderr_piped {
        child
            .stderr
            .take()
            .map(|s| Arc::new(Mutex::new(BufReader::new(s))))
    } else {
        None
    };

    let stdin = if stdin_piped {
        child.stdin.take().map(|s| Arc::new(Mutex::new(s)))
    } else {
        None
    };

    // Generate handle ID and store process
    let handle_id = {
        let mut s = state.borrow_mut();
        let ps = s.try_borrow_mut::<ProcessState>();
        match ps {
            Some(process_state) => {
                let id = format!("proc-{}", process_state.next_id);
                process_state.next_id += 1;
                process_state.processes.insert(
                    id.clone(),
                    ProcessHandle {
                        child: Arc::new(Mutex::new(child)),
                        pid,
                        binary: binary.clone(),
                        stdout,
                        stderr,
                        stdin,
                        exited: false,
                        exit_code: None,
                    },
                );
                id
            }
            None => {
                // Initialize process state if not present
                let mut ps = ProcessState::default();
                let id = format!("proc-{}", ps.next_id);
                ps.next_id += 1;
                ps.processes.insert(
                    id.clone(),
                    ProcessHandle {
                        child: Arc::new(Mutex::new(child)),
                        pid,
                        binary: binary.clone(),
                        stdout,
                        stderr,
                        stdin,
                        exited: false,
                        exit_code: None,
                    },
                );
                s.put(ps);
                id
            }
        }
    };

    debug!(binary = %binary, pid = %pid, handle = %handle_id, "process.spawn complete");

    Ok(SpawnResult { id: handle_id, pid })
}

/// Kill a process
#[weld_op(async)]
#[op2(async)]
async fn op_process_kill(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
    #[string] signal: Option<String>,
) -> Result<(), ProcessError> {
    debug!(handle = %handle, signal = ?signal, "process.kill");

    // Get the child process
    let child_arc = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        Arc::clone(&process.child)
    };

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let child = child_arc.lock().await;
        if let Some(pid) = child.id() {
            let sig = match signal.as_deref() {
                Some("SIGTERM") | None => Signal::SIGTERM,
                Some("SIGKILL") => Signal::SIGKILL,
                Some("SIGINT") => Signal::SIGINT,
                Some("SIGHUP") => Signal::SIGHUP,
                Some("SIGUSR1") => Signal::SIGUSR1,
                Some("SIGUSR2") => Signal::SIGUSR2,
                Some(s) => return Err(ProcessError::io(format!("Unknown signal: {}", s))),
            };

            kill(Pid::from_raw(pid as i32), sig).map_err(|e| ProcessError::io(e.to_string()))?;
        }
    }

    #[cfg(not(unix))]
    {
        let _ = signal; // Ignore signal on non-Unix
        let mut child = child_arc.lock().await;
        child.kill().await.map_err(ProcessError::from)?;
    }

    // Remove from state
    {
        let mut s = state.borrow_mut();
        if let Some(ps) = s.try_borrow_mut::<ProcessState>() {
            ps.processes.remove(&handle);
        }
    }

    Ok(())
}

/// Wait for a process to exit
#[weld_op(async)]
#[op2(async)]
async fn op_process_wait(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<i32, ProcessError> {
    debug!(handle = %handle, "process.wait");

    // Get the child process
    let child_arc = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        Arc::clone(&process.child)
    };

    let mut child = child_arc.lock().await;
    let status = child.wait().await.map_err(ProcessError::from)?;
    let exit_code = status.code().unwrap_or(-1);

    // Update state
    {
        let mut s = state.borrow_mut();
        if let Some(ps) = s.try_borrow_mut::<ProcessState>() {
            if let Some(process) = ps.processes.get_mut(&handle) {
                process.exited = true;
                process.exit_code = Some(exit_code);
            }
        }
    }

    debug!(handle = %handle, exit_code = %exit_code, "process.wait complete");

    Ok(exit_code)
}

/// Get process status
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_process_status(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<ProcessStatus, ProcessError> {
    debug!(handle = %handle, "process.status");

    // First check if we already know it exited
    let (already_exited, exit_code, child_arc) = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;

        (
            process.exited,
            process.exit_code,
            Arc::clone(&process.child),
        )
    };

    if already_exited {
        return Ok(ProcessStatus {
            running: false,
            exit_code,
            signal: None,
        });
    }

    // Try to check if process has exited
    let mut child = child_arc.lock().await;
    match child.try_wait() {
        Ok(Some(status)) => {
            let code = status.code();
            let signal = {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    status.signal().map(|s| format!("signal {}", s))
                }
                #[cfg(not(unix))]
                {
                    None
                }
            };

            // Update state
            {
                let mut s = state.borrow_mut();
                if let Some(ps) = s.try_borrow_mut::<ProcessState>() {
                    if let Some(process) = ps.processes.get_mut(&handle) {
                        process.exited = true;
                        process.exit_code = code;
                    }
                }
            }

            Ok(ProcessStatus {
                running: false,
                exit_code: code,
                signal,
            })
        }
        Ok(None) => Ok(ProcessStatus {
            running: true,
            exit_code: None,
            signal: None,
        }),
        Err(e) => Err(ProcessError::io(e.to_string())),
    }
}

/// Write to process stdin
#[weld_op(async)]
#[op2(async)]
async fn op_process_write_stdin(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
    #[string] data: String,
) -> Result<(), ProcessError> {
    debug!(handle = %handle, len = data.len(), "process.write_stdin");

    // Get stdin handle
    let stdin_arc = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        process
            .stdin
            .as_ref()
            .ok_or_else(|| ProcessError::stdin_closed("stdin not captured"))?
            .clone()
    };

    let mut stdin = stdin_arc.lock().await;
    stdin
        .write_all(data.as_bytes())
        .await
        .map_err(ProcessError::from)?;
    stdin.flush().await.map_err(ProcessError::from)?;

    Ok(())
}

/// Read a line from process stdout
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_process_read_stdout(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<ProcessOutput, ProcessError> {
    debug!(handle = %handle, "process.read_stdout");

    // Get stdout handle
    let stdout_arc = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        process
            .stdout
            .as_ref()
            .ok_or_else(|| ProcessError::output_not_captured("stdout not captured"))?
            .clone()
    };

    let mut stdout = stdout_arc.lock().await;
    let mut line = String::new();
    let bytes_read = stdout
        .read_line(&mut line)
        .await
        .map_err(ProcessError::from)?;

    if bytes_read == 0 {
        Ok(ProcessOutput {
            data: None,
            eof: true,
        })
    } else {
        // Remove trailing newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(ProcessOutput {
            data: Some(line),
            eof: false,
        })
    }
}

/// Read a line from process stderr
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_process_read_stderr(
    state: Rc<RefCell<OpState>>,
    #[string] handle: String,
) -> Result<ProcessOutput, ProcessError> {
    debug!(handle = %handle, "process.read_stderr");

    // Get stderr handle
    let stderr_arc = {
        let s = state.borrow();
        let ps = s
            .try_borrow::<ProcessState>()
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        let process = ps
            .processes
            .get(&handle)
            .ok_or_else(|| ProcessError::invalid_handle(&handle))?;
        process
            .stderr
            .as_ref()
            .ok_or_else(|| ProcessError::output_not_captured("stderr not captured"))?
            .clone()
    };

    let mut stderr = stderr_arc.lock().await;
    let mut line = String::new();
    let bytes_read = stderr
        .read_line(&mut line)
        .await
        .map_err(ProcessError::from)?;

    if bytes_read == 0 {
        Ok(ProcessOutput {
            data: None,
            eof: true,
        })
    } else {
        // Remove trailing newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(ProcessOutput {
            data: Some(line),
            eof: false,
        })
    }
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize process state in OpState
pub fn init_process_state(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn ProcessCapabilityChecker>>,
    max_processes: Option<usize>,
) {
    op_state.put(ProcessState::new(max_processes.unwrap_or(10)));
    if let Some(caps) = capabilities {
        op_state.put(ProcessCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs (contains transpiled TypeScript)
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn process_extension() -> Extension {
    runtime_process::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = ProcessError::permission_denied("test");
        match err {
            ProcessError::PermissionDenied { code, .. } => {
                assert_eq!(code, ProcessErrorCode::PermissionDenied as u32);
            }
            _ => panic!("Wrong error type"),
        }

        let err = ProcessError::too_many_processes("test");
        match err {
            ProcessError::TooManyProcesses { code, .. } => {
                assert_eq!(code, ProcessErrorCode::TooManyProcesses as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_process_state_capacity() {
        // Test just the capacity tracking logic without creating actual Child objects
        let state = ProcessState::new(2);
        assert!(state.can_spawn());
        assert_eq!(state.max_processes, 2);
        assert_eq!(state.next_id, 1);
        assert!(state.processes.is_empty());

        // Test that can_spawn respects max_processes by checking the logic directly
        // (processes.len() < max_processes)
        let mut state = ProcessState::new(0);
        assert!(!state.can_spawn()); // Cannot spawn when max is 0

        state.max_processes = 1;
        assert!(state.can_spawn()); // Can spawn when under limit
    }

    #[test]
    fn test_parse_stdio() {
        // These just test the parsing logic
        let _ = parse_stdio(Some(&"piped".to_string()));
        let _ = parse_stdio(Some(&"inherit".to_string()));
        let _ = parse_stdio(Some(&"null".to_string()));
        let _ = parse_stdio(None);
    }
}
