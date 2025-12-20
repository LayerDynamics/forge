//! Built-in shell commands
//!
//! This module provides:
//! - `ShellCommand` trait for implementing commands
//! - `ShellCommandContext` for command execution context
//! - Built-in commands: echo, cd, pwd, exit, export, unset, cat, head, mkdir, rm, cp, mv, sleep, xargs
//! - `builtin_commands()` function to get all built-in commands

mod args;
mod cat;
mod cd;
mod cp_mv;
mod echo;
mod executable;
mod exit;
mod export;
mod head;
mod mkdir;
mod pwd;
mod rm;
mod sleep;
mod unset;
mod xargs;

use std::collections::HashMap;
use std::ffi::OsString;
use std::rc::Rc;

use super::types::{FutureExecuteResult, ShellPipeReader, ShellPipeWriter, ShellState};

// Re-export the args module for use by command implementations
pub use args::*;

// Re-export command implementations
pub use cat::CatCommand;
pub use cd::CdCommand;
pub use cp_mv::{CpCommand, MvCommand};
pub use echo::EchoCommand;
pub use executable::{resolve_command, ExecutableCommand};
pub use exit::ExitCommand;
pub use export::ExportCommand;
pub use head::HeadCommand;
pub use mkdir::MkdirCommand;
pub use pwd::PwdCommand;
pub use rm::RmCommand;
pub use sleep::SleepCommand;
pub use unset::UnsetCommand;
pub use xargs::XargsCommand;

/// Trait for implementing shell commands.
///
/// Commands receive a context with arguments, state, and I/O pipes,
/// and return a future that resolves to an execution result.
pub trait ShellCommand: Send {
    /// Execute the command with the given context.
    fn execute(&self, context: ShellCommandContext) -> FutureExecuteResult;
}

/// Context provided to shell commands during execution.
pub struct ShellCommandContext {
    /// Command arguments (including the command name as args[0])
    pub args: Vec<OsString>,
    /// Current shell state
    pub state: ShellState,
    /// Standard input pipe
    pub stdin: ShellPipeReader,
    /// Standard output pipe
    pub stdout: ShellPipeWriter,
    /// Standard error pipe
    pub stderr: ShellPipeWriter,
    /// Function to execute sub-commands (for xargs, etc.)
    pub execute_command_args: Box<dyn Fn(ShellCommandContext) -> FutureExecuteResult + 'static>,
}

impl ShellCommandContext {
    /// Get arguments as strings (lossy conversion).
    pub fn args_str(&self) -> Vec<String> {
        self.args
            .iter()
            .map(|a| a.to_string_lossy().to_string())
            .collect()
    }

    /// Write an error message to stderr.
    pub fn write_error(&mut self, msg: &str) -> anyhow::Result<()> {
        self.stderr.write_line(&format!("error: {}", msg))
    }

    /// Write to stdout.
    pub fn write_stdout(&mut self, msg: &str) -> anyhow::Result<()> {
        self.stdout.write_all(msg.as_bytes())
    }

    /// Write a line to stdout.
    pub fn write_line(&mut self, msg: &str) -> anyhow::Result<()> {
        self.stdout.write_line(msg)
    }
}

/// True command - always returns exit code 0.
struct TrueCommand;

impl ShellCommand for TrueCommand {
    fn execute(&self, _context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async { super::types::ExecuteResult::from_exit_code(0) })
    }
}

/// False command - always returns exit code 1.
struct FalseCommand;

impl ShellCommand for FalseCommand {
    fn execute(&self, _context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async { super::types::ExecuteResult::from_exit_code(1) })
    }
}

/// Get all built-in commands as a HashMap.
pub fn builtin_commands() -> HashMap<String, Rc<dyn ShellCommand>> {
    let mut commands: HashMap<String, Rc<dyn ShellCommand>> = HashMap::new();

    // Basic commands
    commands.insert("echo".to_string(), Rc::new(EchoCommand));
    commands.insert("cd".to_string(), Rc::new(CdCommand));
    commands.insert("pwd".to_string(), Rc::new(PwdCommand));
    commands.insert("exit".to_string(), Rc::new(ExitCommand));

    // Environment commands
    commands.insert("export".to_string(), Rc::new(ExportCommand));
    commands.insert("unset".to_string(), Rc::new(UnsetCommand));

    // File commands
    commands.insert("cat".to_string(), Rc::new(CatCommand));
    commands.insert("head".to_string(), Rc::new(HeadCommand));
    commands.insert("mkdir".to_string(), Rc::new(MkdirCommand));
    commands.insert("rm".to_string(), Rc::new(RmCommand));
    commands.insert("cp".to_string(), Rc::new(CpCommand));
    commands.insert("mv".to_string(), Rc::new(MvCommand));

    // Utility commands
    commands.insert("sleep".to_string(), Rc::new(SleepCommand));
    commands.insert("xargs".to_string(), Rc::new(XargsCommand));

    // Boolean commands
    commands.insert("true".to_string(), Rc::new(TrueCommand));
    commands.insert("false".to_string(), Rc::new(FalseCommand));

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_commands_exist() {
        let commands = builtin_commands();

        assert!(commands.contains_key("echo"));
        assert!(commands.contains_key("cd"));
        assert!(commands.contains_key("pwd"));
        assert!(commands.contains_key("exit"));
        assert!(commands.contains_key("export"));
        assert!(commands.contains_key("unset"));
        assert!(commands.contains_key("cat"));
        assert!(commands.contains_key("head"));
        assert!(commands.contains_key("mkdir"));
        assert!(commands.contains_key("rm"));
        assert!(commands.contains_key("cp"));
        assert!(commands.contains_key("mv"));
        assert!(commands.contains_key("sleep"));
        assert!(commands.contains_key("xargs"));
        assert!(commands.contains_key("true"));
        assert!(commands.contains_key("false"));
    }
}
