//! Core types for shell execution
//!
//! This module provides the fundamental data structures for shell execution:
//! - `ShellState` - Holds environment, cwd, and command registry
//! - `ExecuteResult` - Result of command execution
//! - `EnvChange` - Environment modifications from commands
//! - `ShellPipeReader`/`ShellPipeWriter` - Pipe abstractions
//! - `KillSignal` - Hierarchical signal propagation
//! - `SignalKind` - Signal types (SIGTERM, SIGKILL, etc.)

use std::borrow::Cow;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;

use anyhow::Result;
use futures::future::LocalBoxFuture;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use super::child_process_tracker::ChildProcessTracker;
use super::commands::ShellCommand;

// ============================================================================
// Tree Exit Code Cell
// ============================================================================

/// Stores the exit code for async command trees.
/// Only the first non-zero exit code is stored.
#[derive(Debug, Default, Clone)]
pub struct TreeExitCodeCell(Rc<Cell<i32>>);

impl TreeExitCodeCell {
    /// Try to set the exit code. Only sets if current value is 0.
    pub fn try_set(&self, exit_code: i32) {
        if self.0.get() == 0 {
            self.0.set(exit_code);
        }
    }

    /// Get the exit code if non-zero.
    pub fn get(&self) -> Option<i32> {
        match self.0.get() {
            0 => None,
            code => Some(code),
        }
    }
}

// ============================================================================
// Shell State
// ============================================================================

/// Central state container for shell execution.
///
/// Holds environment variables, shell-local variables, current working directory,
/// registered commands, and signal handling infrastructure.
///
/// Uses `RefCell` for interior mutability to support mutation through `Rc<ShellState>`.
#[derive(Clone)]
pub struct ShellState {
    /// Environment variables passed to child processes
    env_vars: RefCell<HashMap<OsString, OsString>>,
    /// Shell-local variables (not passed to children)
    shell_vars: RefCell<HashMap<OsString, OsString>>,
    /// Current working directory
    cwd: RefCell<PathBuf>,
    /// Registered commands (built-in + custom)
    commands: Rc<HashMap<String, Rc<dyn ShellCommand>>>,
    /// Signal for killing child processes
    kill_signal: KillSignal,
    /// Tracks spawned child processes
    process_tracker: ChildProcessTracker,
    /// Exit code for async command trees
    tree_exit_code_cell: TreeExitCodeCell,
}

impl ShellState {
    /// Create a new shell state.
    ///
    /// # Arguments
    /// * `env_vars` - Initial environment variables
    /// * `cwd` - Current working directory (must be absolute)
    /// * `custom_commands` - Custom commands to register
    /// * `kill_signal` - Signal for killing processes
    pub fn new(
        env_vars: HashMap<OsString, OsString>,
        cwd: PathBuf,
        custom_commands: HashMap<String, Rc<dyn ShellCommand>>,
        kill_signal: KillSignal,
    ) -> Self {
        assert!(cwd.is_absolute(), "cwd must be absolute path");

        let mut commands = super::commands::builtin_commands();
        commands.extend(custom_commands);

        let result = Self {
            env_vars: RefCell::new(HashMap::new()),
            shell_vars: RefCell::new(HashMap::new()),
            cwd: RefCell::new(PathBuf::new()),
            commands: Rc::new(commands),
            kill_signal,
            process_tracker: ChildProcessTracker::new(),
            tree_exit_code_cell: TreeExitCodeCell::default(),
        };

        // Normalize environment variables
        for (name, value) in env_vars {
            result.apply_env_var(&name, &value);
        }
        result.set_cwd(cwd);
        result
    }

    /// Create a default shell state for testing.
    pub fn new_default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let env_vars: HashMap<OsString, OsString> = std::env::vars_os().collect();
        Self::new(env_vars, cwd, HashMap::new(), KillSignal::default())
    }

    /// Get the current working directory.
    pub fn cwd(&self) -> PathBuf {
        self.cwd.borrow().clone()
    }

    /// Get all environment variables (cloned).
    pub fn env_vars(&self) -> HashMap<OsString, OsString> {
        self.env_vars.borrow().clone()
    }

    /// Get a variable (checks env_vars first, then shell_vars).
    pub fn get_var(&self, name: &OsStr) -> Option<OsString> {
        let name = if cfg!(windows) {
            Cow::Owned(name.to_ascii_uppercase())
        } else {
            Cow::Borrowed(name)
        };
        let name: &OsStr = &name;
        self.env_vars
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.shell_vars.borrow().get(name).cloned())
    }

    /// Set the current working directory.
    pub fn set_cwd(&self, cwd: PathBuf) {
        *self.cwd.borrow_mut() = cwd.clone();
        // Keep $PWD in sync with cwd
        self.env_vars
            .borrow_mut()
            .insert("PWD".into(), cwd.into_os_string());
    }

    /// Apply multiple environment changes.
    pub fn apply_changes(&self, changes: &[EnvChange]) {
        for change in changes {
            self.apply_change(change);
        }
    }

    /// Apply a single environment change.
    pub fn apply_change(&self, change: &EnvChange) {
        match change {
            EnvChange::SetEnvVar(name, value) => self.apply_env_var(name, value),
            EnvChange::SetShellVar(name, value) => {
                if self.env_vars.borrow().contains_key(name) {
                    self.apply_env_var(name, value);
                } else {
                    self.shell_vars
                        .borrow_mut()
                        .insert(name.to_os_string(), value.to_os_string());
                }
            }
            EnvChange::UnsetVar(name) => {
                self.shell_vars.borrow_mut().remove(name);
                self.env_vars.borrow_mut().remove(name);
            }
            EnvChange::Cd(new_dir) => {
                self.set_cwd(new_dir.clone());
            }
        }
    }

    /// Apply an environment variable (handles PWD specially).
    pub fn apply_env_var(&self, name: &OsStr, value: &OsStr) {
        let name = if cfg!(windows) {
            name.to_ascii_uppercase()
        } else {
            name.to_os_string()
        };

        if name == "PWD" {
            let cwd = Path::new(value);
            if cwd.is_absolute() {
                if let Ok(canonical) = std::fs::canonicalize(cwd) {
                    self.set_cwd(canonical);
                    return;
                }
            }
        }

        self.shell_vars.borrow_mut().remove(&name);
        self.env_vars
            .borrow_mut()
            .insert(name, value.to_os_string());
    }

    /// Set an environment variable.
    pub fn set_env_var(&self, name: impl Into<OsString>, value: impl Into<OsString>) {
        let name = name.into();
        let value = value.into();
        self.apply_env_var(&name, &value);
    }

    /// Set a shell-local variable.
    pub fn set_shell_var(&self, name: impl Into<OsString>, value: impl Into<OsString>) {
        self.shell_vars
            .borrow_mut()
            .insert(name.into(), value.into());
    }

    /// Unset a variable.
    pub fn unset_var(&self, name: &OsStr) {
        self.shell_vars.borrow_mut().remove(name);
        self.env_vars.borrow_mut().remove(name);
    }

    /// Get the kill signal.
    pub fn kill_signal(&self) -> &KillSignal {
        &self.kill_signal
    }

    /// Track a child process for cleanup.
    pub fn track_child_process(&self, child: &tokio::process::Child) {
        self.process_tracker.track(child);
    }

    /// Get the tree exit code cell.
    pub fn tree_exit_code_cell(&self) -> &TreeExitCodeCell {
        &self.tree_exit_code_cell
    }

    /// Resolve a command by name.
    pub fn resolve_custom_command(&self, name: &OsStr) -> Option<Rc<dyn ShellCommand>> {
        name.to_str()
            .and_then(|name| self.commands.get(name).cloned())
    }

    /// Resolve command path from PATH environment variable.
    pub fn resolve_command_path(
        &self,
        command_name: &OsStr,
    ) -> Result<PathBuf, super::which::CommandPathResolutionError> {
        super::which::resolve_command_path(command_name, &self.cwd(), self)
    }

    /// Create a child state with its own kill signal.
    pub fn with_child_signal(&self) -> ShellState {
        let mut state = self.clone();
        state.kill_signal = self.kill_signal.child_signal();
        state.tree_exit_code_cell = TreeExitCodeCell::default();
        state
    }

    /// Clone the state for use in a subshell.
    ///
    /// Creates a copy with a fresh tree exit code cell but same kill signal.
    /// Subshells inherit the parent's environment but have their own exit tracking.
    pub fn clone_for_subshell(&self) -> ShellState {
        let mut state = self.clone();
        state.tree_exit_code_cell = TreeExitCodeCell::default();
        state
    }

    /// Get an environment variable by name.
    pub fn get_env_var(&self, name: &str) -> Option<String> {
        let name_os: OsString = name.into();
        self.env_vars
            .borrow()
            .get(&name_os)
            .map(|v| v.to_string_lossy().to_string())
    }

    /// Get a variable by name (string version).
    pub fn get_var_str(&self, name: &str) -> Option<String> {
        self.get_var(std::ffi::OsStr::new(name))
            .map(|s| s.to_string_lossy().to_string())
    }

    /// Get the last exit code (stored as $? variable).
    pub fn last_exit_code(&self) -> i32 {
        self.shell_vars
            .borrow()
            .get(&OsString::from("?"))
            .and_then(|s| s.to_string_lossy().parse().ok())
            .unwrap_or(0)
    }

    /// Get the home directory.
    pub fn home_dir(&self) -> Option<PathBuf> {
        self.get_env_var("HOME").map(PathBuf::from).or_else(|| {
            #[cfg(unix)]
            {
                std::env::var("HOME").ok().map(PathBuf::from)
            }
            #[cfg(windows)]
            {
                std::env::var("USERPROFILE").ok().map(PathBuf::from)
            }
            #[cfg(not(any(unix, windows)))]
            {
                None
            }
        })
    }
}

// ============================================================================
// Environment Changes
// ============================================================================

/// Represents a change to the shell environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvChange {
    /// Set an environment variable (passed to children): `export VAR=value`
    SetEnvVar(OsString, OsString),
    /// Set a shell-local variable: `VAR=value`
    SetShellVar(OsString, OsString),
    /// Unset a variable: `unset VAR`
    UnsetVar(OsString),
    /// Change directory: `cd path`
    Cd(PathBuf),
}

// ============================================================================
// Execution Result
// ============================================================================

/// Future type for async command execution.
pub type FutureExecuteResult = LocalBoxFuture<'static, ExecuteResult>;

/// Result of executing a command or sequence.
#[derive(Debug)]
pub enum ExecuteResult {
    /// Exit the shell with a code: `exit <code>`
    Exit(i32, Vec<JoinHandle<i32>>),
    /// Continue execution with exit code, env changes, and async handles
    Continue(i32, Vec<EnvChange>, Vec<JoinHandle<i32>>),
}

impl ExecuteResult {
    /// Create a simple result from an exit code.
    pub fn from_exit_code(exit_code: i32) -> ExecuteResult {
        ExecuteResult::Continue(exit_code, Vec::new(), Vec::new())
    }

    /// Get the exit code without consuming the result.
    pub fn exit_code(&self) -> i32 {
        match self {
            ExecuteResult::Exit(code, _) => *code,
            ExecuteResult::Continue(code, _, _) => *code,
        }
    }

    /// Extract exit code and handles from the result.
    pub fn into_exit_code_and_handles(self) -> (i32, Vec<JoinHandle<i32>>) {
        match self {
            ExecuteResult::Exit(code, handles) => (code, handles),
            ExecuteResult::Continue(code, _, handles) => (code, handles),
        }
    }

    /// Extract just the handles.
    pub fn into_handles(self) -> Vec<JoinHandle<i32>> {
        self.into_exit_code_and_handles().1
    }
}

// ============================================================================
// Pipes
// ============================================================================

/// Reader side of a shell pipe.
#[derive(Debug)]
pub enum ShellPipeReader {
    /// OS pipe reader
    OsPipe(std::io::PipeReader),
    /// File reader
    StdFile(std::fs::File),
}

impl Clone for ShellPipeReader {
    fn clone(&self) -> Self {
        match self {
            Self::OsPipe(pipe) => Self::OsPipe(pipe.try_clone().unwrap()),
            Self::StdFile(file) => Self::StdFile(file.try_clone().unwrap()),
        }
    }
}

impl ShellPipeReader {
    /// Create a reader from stdin.
    pub fn stdin() -> ShellPipeReader {
        #[cfg(unix)]
        fn dup_stdin() -> std::io::PipeReader {
            use std::os::fd::AsFd;
            use std::os::fd::FromRawFd;
            use std::os::fd::IntoRawFd;
            let owned = std::io::stdin().as_fd().try_clone_to_owned().unwrap();
            let raw = owned.into_raw_fd();
            unsafe { std::io::PipeReader::from_raw_fd(raw) }
        }

        #[cfg(windows)]
        fn dup_stdin() -> std::io::PipeReader {
            use std::os::windows::io::AsHandle;
            use std::os::windows::io::FromRawHandle;
            use std::os::windows::io::IntoRawHandle;
            let owned = std::io::stdin().as_handle().try_clone_to_owned().unwrap();
            let raw = owned.into_raw_handle();
            unsafe { std::io::PipeReader::from_raw_handle(raw) }
        }

        ShellPipeReader::OsPipe(dup_stdin())
    }

    /// Create from a raw pipe reader.
    pub fn from_raw(reader: std::io::PipeReader) -> Self {
        Self::OsPipe(reader)
    }

    /// Create from a file.
    pub fn from_std(std_file: std::fs::File) -> Self {
        Self::StdFile(std_file)
    }

    /// Alias for from_std for compatibility.
    pub fn from_file(std_file: std::fs::File) -> Self {
        Self::from_std(std_file)
    }

    /// Create from a string (for here-strings and here-docs).
    pub fn from_string(content: String) -> Self {
        use std::io::Write;
        // Create a pipe and write the content to it
        let (reader, mut writer) = std::io::pipe().unwrap();
        // Spawn a thread to write the content
        std::thread::spawn(move || {
            let _ = writer.write_all(content.as_bytes());
            // Writer is dropped here, closing the write end
        });
        Self::OsPipe(reader)
    }

    /// Convert to process stdio.
    pub fn into_stdio(self) -> std::process::Stdio {
        match self {
            Self::OsPipe(pipe) => pipe.into(),
            Self::StdFile(file) => file.into(),
        }
    }

    /// Pipe all data to a writer.
    pub fn pipe_to(self, writer: &mut dyn Write) -> Result<()> {
        self.pipe_to_inner(writer, false)
    }

    fn pipe_to_with_flushing(self, writer: &mut dyn Write) -> Result<()> {
        self.pipe_to_inner(writer, true)
    }

    fn pipe_to_inner(mut self, writer: &mut dyn Write, flush: bool) -> Result<()> {
        loop {
            let mut buffer = [0u8; 512];
            let size = match &mut self {
                ShellPipeReader::OsPipe(pipe) => pipe.read(&mut buffer)?,
                ShellPipeReader::StdFile(file) => file.read(&mut buffer)?,
            };
            if size == 0 {
                break;
            }
            writer.write_all(&buffer[0..size])?;
            if flush {
                writer.flush()?;
            }
        }
        Ok(())
    }

    /// Pipe to another shell pipe writer.
    pub fn pipe_to_sender(self, mut sender: ShellPipeWriter) -> Result<()> {
        match &mut sender {
            ShellPipeWriter::OsPipe(pipe) => self.pipe_to(pipe),
            ShellPipeWriter::StdFile(file) => self.pipe_to(file),
            ShellPipeWriter::Stdout => self.pipe_to_with_flushing(&mut std::io::stdout()),
            ShellPipeWriter::Stderr => self.pipe_to_with_flushing(&mut std::io::stderr()),
            ShellPipeWriter::Null => Ok(()),
        }
    }

    /// Pipe to a string, returning a handle.
    pub fn pipe_to_string_handle(self) -> JoinHandle<String> {
        tokio::task::spawn_blocking(|| {
            let mut buf = Vec::new();
            self.pipe_to(&mut buf).unwrap();
            String::from_utf8_lossy(&buf).to_string()
        })
    }

    /// Read bytes into buffer.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            ShellPipeReader::OsPipe(pipe) => pipe.read(buf).map_err(|e| e.into()),
            ShellPipeReader::StdFile(file) => file.read(buf).map_err(|e| e.into()),
        }
    }
}

/// Writer side of a shell pipe.
#[derive(Debug)]
pub enum ShellPipeWriter {
    /// OS pipe writer
    OsPipe(std::io::PipeWriter),
    /// File writer
    StdFile(std::fs::File),
    /// Rust's stdout (handles encoding on Windows)
    Stdout,
    /// Rust's stderr (handles encoding on Windows)
    Stderr,
    /// Discard output (/dev/null)
    Null,
}

impl Clone for ShellPipeWriter {
    fn clone(&self) -> Self {
        match self {
            Self::OsPipe(pipe) => Self::OsPipe(pipe.try_clone().unwrap()),
            Self::StdFile(file) => Self::StdFile(file.try_clone().unwrap()),
            Self::Stdout => Self::Stdout,
            Self::Stderr => Self::Stderr,
            Self::Null => Self::Null,
        }
    }
}

impl ShellPipeWriter {
    /// Create stdout writer.
    pub fn stdout() -> Self {
        Self::Stdout
    }

    /// Create stderr writer.
    pub fn stderr() -> Self {
        Self::Stderr
    }

    /// Create null writer (discards output).
    pub fn null() -> Self {
        Self::Null
    }

    /// Create from a file.
    pub fn from_std(std_file: std::fs::File) -> Self {
        Self::StdFile(std_file)
    }

    /// Alias for from_std for compatibility.
    pub fn from_file(std_file: std::fs::File) -> Self {
        Self::from_std(std_file)
    }

    /// Convert to process stdio.
    pub fn into_stdio(self) -> std::process::Stdio {
        match self {
            Self::OsPipe(pipe) => pipe.into(),
            Self::StdFile(file) => file.into(),
            Self::Stdout => std::process::Stdio::inherit(),
            Self::Stderr => std::process::Stdio::inherit(),
            Self::Null => std::process::Stdio::null(),
        }
    }

    /// Write all bytes.
    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_all_iter(std::iter::once(bytes))
    }

    /// Write all bytes from an iterator.
    pub fn write_all_iter<'a>(&mut self, iter: impl Iterator<Item = &'a [u8]> + 'a) -> Result<()> {
        match self {
            Self::OsPipe(pipe) => {
                for bytes in iter {
                    pipe.write_all(bytes)?;
                }
            }
            Self::StdFile(file) => {
                for bytes in iter {
                    file.write_all(bytes)?;
                }
            }
            Self::Stdout => {
                let mut stdout = std::io::stdout().lock();
                for bytes in iter {
                    stdout.write_all(bytes)?;
                }
                stdout.flush()?;
            }
            Self::Stderr => {
                let mut stderr = std::io::stderr().lock();
                for bytes in iter {
                    stderr.write_all(bytes)?;
                }
                stderr.flush()?;
            }
            Self::Null => {}
        }
        Ok(())
    }

    /// Write a line (with newline).
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        let bytes = format!("{line}\n");
        self.write_all(bytes.as_bytes())
    }
}

/// Create a pipe pair.
pub fn pipe() -> (ShellPipeReader, ShellPipeWriter) {
    let (reader, writer) = std::io::pipe().unwrap();
    (
        ShellPipeReader::OsPipe(reader),
        ShellPipeWriter::OsPipe(writer),
    )
}

// ============================================================================
// Signals
// ============================================================================

#[derive(Debug)]
struct KillSignalInner {
    aborted_code: RefCell<Option<i32>>,
    sender: broadcast::Sender<SignalKind>,
    children: RefCell<Vec<Weak<KillSignalInner>>>,
}

impl KillSignalInner {
    pub fn send(&self, signal_kind: SignalKind) {
        if signal_kind.causes_abort() {
            let mut stored = self.aborted_code.borrow_mut();
            if stored.is_none() {
                *stored = Some(signal_kind.aborted_code());
            }
        }
        let _ = self.sender.send(signal_kind);

        // Notify children
        self.children.borrow_mut().retain(|weak_child| {
            if let Some(child) = weak_child.upgrade() {
                child.send(signal_kind);
                true
            } else {
                false
            }
        });
    }
}

/// Signal for killing shell commands.
///
/// Supports hierarchical propagation: signals sent to a parent
/// propagate to all children, but not vice versa.
#[derive(Debug, Clone)]
pub struct KillSignal(Rc<KillSignalInner>);

impl Default for KillSignal {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self(Rc::new(KillSignalInner {
            aborted_code: RefCell::new(None),
            sender,
            children: RefCell::new(Vec::new()),
        }))
    }
}

impl KillSignal {
    /// Get the abort code if signal caused abort.
    pub fn aborted_code(&self) -> Option<i32> {
        *self.0.aborted_code.borrow()
    }

    /// Create a child signal that receives parent signals.
    pub fn child_signal(&self) -> Self {
        let (sender, _) = broadcast::channel(100);
        let child = Rc::new(KillSignalInner {
            aborted_code: RefCell::new(self.aborted_code()),
            sender,
            children: RefCell::new(Vec::new()),
        });

        self.0.children.borrow_mut().push(Rc::downgrade(&child));

        Self(child)
    }

    /// Create a drop guard that sends SIGTERM on drop.
    pub fn drop_guard(self) -> KillSignalDropGuard {
        self.drop_guard_with_kind(SignalKind::SIGTERM)
    }

    /// Create a drop guard with a specific signal.
    pub fn drop_guard_with_kind(self, kind: SignalKind) -> KillSignalDropGuard {
        KillSignalDropGuard {
            disarmed: Cell::new(false),
            kill_signal_kind: kind,
            signal: self,
        }
    }

    /// Send a signal to this signal and all children.
    pub fn send(&self, signal: SignalKind) {
        self.0.send(signal)
    }

    /// Send SIGTERM signal.
    pub fn send_sigterm(&self) {
        self.send(SignalKind::SIGTERM)
    }

    /// Send SIGKILL signal.
    pub fn send_sigkill(&self) {
        self.send(SignalKind::SIGKILL)
    }

    /// Wait for an aborting signal.
    pub async fn wait_aborted(&self) -> SignalKind {
        let mut receiver = self.0.sender.subscribe();
        loop {
            let signal = receiver.recv().await.unwrap();
            if signal.causes_abort() {
                return signal;
            }
        }
    }

    /// Wait for any signal.
    pub async fn wait_any(&self) -> SignalKind {
        let mut receiver = self.0.sender.subscribe();
        receiver.recv().await.unwrap()
    }
}

/// Guard that sends a signal on drop.
#[derive(Debug)]
pub struct KillSignalDropGuard {
    disarmed: Cell<bool>,
    kill_signal_kind: SignalKind,
    signal: KillSignal,
}

impl Drop for KillSignalDropGuard {
    fn drop(&mut self) {
        if !self.disarmed.get() {
            self.signal.send(self.kill_signal_kind);
        }
    }
}

impl KillSignalDropGuard {
    /// Prevent the guard from sending a signal on drop.
    pub fn disarm(&self) {
        self.disarmed.set(true);
    }
}

/// Types of signals that can be sent to processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    SIGTERM,
    SIGKILL,
    SIGABRT,
    SIGQUIT,
    SIGINT,
    SIGSTOP,
    Other(i32),
}

impl SignalKind {
    /// Check if this signal causes command abortion.
    pub fn causes_abort(&self) -> bool {
        matches!(
            self,
            SignalKind::SIGTERM
                | SignalKind::SIGKILL
                | SignalKind::SIGQUIT
                | SignalKind::SIGINT
                | SignalKind::SIGSTOP
                | SignalKind::SIGABRT
        )
    }

    /// Get the exit code for abort (128 + signal number).
    pub fn aborted_code(&self) -> i32 {
        let value: i32 = (*self).into();
        128 + value
    }
}

impl From<i32> for SignalKind {
    fn from(value: i32) -> Self {
        #[cfg(unix)]
        {
            match value {
                2 => SignalKind::SIGINT,
                3 => SignalKind::SIGQUIT,
                6 => SignalKind::SIGABRT,
                9 => SignalKind::SIGKILL,
                15 => SignalKind::SIGTERM,
                19 => SignalKind::SIGSTOP,
                _ => SignalKind::Other(value),
            }
        }
        #[cfg(not(unix))]
        {
            match value {
                2 => SignalKind::SIGINT,
                3 => SignalKind::SIGQUIT,
                6 => SignalKind::SIGABRT,
                9 => SignalKind::SIGKILL,
                15 => SignalKind::SIGTERM,
                19 => SignalKind::SIGSTOP,
                _ => SignalKind::Other(value),
            }
        }
    }
}

impl From<SignalKind> for i32 {
    fn from(kind: SignalKind) -> i32 {
        match kind {
            SignalKind::SIGINT => 2,
            SignalKind::SIGQUIT => 3,
            SignalKind::SIGABRT => 6,
            SignalKind::SIGKILL => 9,
            SignalKind::SIGTERM => 15,
            SignalKind::SIGSTOP => 19,
            SignalKind::Other(value) => value,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_exit_code_cell() {
        let cell = TreeExitCodeCell::default();
        assert_eq!(cell.get(), None);

        cell.try_set(1);
        assert_eq!(cell.get(), Some(1));

        // Should not overwrite
        cell.try_set(2);
        assert_eq!(cell.get(), Some(1));
    }

    #[test]
    fn test_signal_kind_conversion() {
        assert_eq!(SignalKind::from(9), SignalKind::SIGKILL);
        assert_eq!(i32::from(SignalKind::SIGKILL), 9);
        assert_eq!(SignalKind::SIGKILL.aborted_code(), 137);
    }

    #[test]
    fn test_execute_result() {
        let result = ExecuteResult::from_exit_code(0);
        let (code, handles) = result.into_exit_code_and_handles();
        assert_eq!(code, 0);
        assert!(handles.is_empty());
    }

    #[tokio::test]
    async fn test_pipe_creation() {
        let (reader, mut writer) = pipe();
        writer.write_all(b"hello").unwrap();
        drop(writer);

        let handle = reader.pipe_to_string_handle();
        let result = handle.await.unwrap();
        assert_eq!(result, "hello");
    }
}
