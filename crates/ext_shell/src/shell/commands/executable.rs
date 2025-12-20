//! External executable command implementation
//!
//! Runs external commands from the PATH.

use std::ffi::OsString;
use std::path::PathBuf;
use tokio::process::Command;

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult, SignalKind};

/// Executes external commands found in PATH.
pub struct ExecutableCommand {
    /// Path to the executable
    pub path: PathBuf,
}

impl ExecutableCommand {
    /// Create a new executable command.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ShellCommand for ExecutableCommand {
    fn execute(&self, context: ShellCommandContext) -> FutureExecuteResult {
        let path = self.path.clone();

        Box::pin(async move {
            // Check if already aborted
            if let Some(code) = context.state.kill_signal().aborted_code() {
                return ExecuteResult::from_exit_code(code);
            }

            // Build the command
            let mut cmd = Command::new(&path);

            // Add arguments (skip command name)
            let args: Vec<OsString> = context.args.iter().skip(1).cloned().collect();
            cmd.args(&args);

            // Set working directory
            cmd.current_dir(context.state.cwd());

            // Set environment (clear inherited, use shell env)
            cmd.env_clear();
            for (key, value) in context.state.env_vars() {
                cmd.env(key, value);
            }

            // Set up I/O
            cmd.stdin(context.stdin.into_stdio());
            cmd.stdout(context.stdout.into_stdio());
            cmd.stderr(context.stderr.into_stdio());

            // Spawn the process
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    // Try to write error, but we've moved stdout/stderr
                    eprintln!("{}: {}", path.display(), e);
                    return ExecuteResult::from_exit_code(127);
                }
            };

            // Track the child process
            context.state.track_child_process(&child);

            // Wait for completion or signal
            let kill_signal = context.state.kill_signal().clone();

            tokio::select! {
                status = child.wait() => {
                    match status {
                        Ok(status) => {
                            let code = status.code().unwrap_or(1);
                            ExecuteResult::from_exit_code(code)
                        }
                        Err(e) => {
                            eprintln!("{}: {}", path.display(), e);
                            ExecuteResult::from_exit_code(1)
                        }
                    }
                }
                signal = kill_signal.wait_any() => {
                    // Kill the child process
                    kill_child(&mut child, signal);

                    // Wait for it to actually exit
                    let _ = child.wait().await;

                    ExecuteResult::from_exit_code(signal.aborted_code())
                }
            }
        })
    }
}

/// Kill a child process with the given signal.
fn kill_child(child: &mut tokio::process::Child, signal: SignalKind) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Some(pid) = child.id() {
            let sig = match signal {
                SignalKind::SIGTERM => Signal::SIGTERM,
                SignalKind::SIGKILL => Signal::SIGKILL,
                SignalKind::SIGINT => Signal::SIGINT,
                SignalKind::SIGQUIT => Signal::SIGQUIT,
                SignalKind::SIGABRT => Signal::SIGABRT,
                SignalKind::SIGSTOP => Signal::SIGSTOP,
                SignalKind::Other(n) => {
                    if let Ok(sig) = Signal::try_from(n) {
                        sig
                    } else {
                        Signal::SIGTERM
                    }
                }
            };
            let _ = kill(Pid::from_raw(pid as i32), sig);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, we can only terminate (no signals)
        let _ = child.start_kill();
        let _ = signal; // Suppress warning
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = signal;
        // No-op on unsupported platforms
    }
}

/// Resolve a command name to a path using which.
pub fn resolve_command(
    name: &std::ffi::OsStr,
    state: &crate::shell::types::ShellState,
) -> Option<PathBuf> {
    // First check if it's already a path
    let path = PathBuf::from(name);
    if path.is_absolute() || name.to_string_lossy().contains(std::path::MAIN_SEPARATOR) {
        if path.exists() {
            return Some(path);
        }
        // Try relative to cwd
        let cwd_path = state.cwd().join(&path);
        if cwd_path.exists() {
            return Some(cwd_path);
        }
        return None;
    }

    // Use which crate to find in PATH
    state.resolve_command_path(name).ok()
}
