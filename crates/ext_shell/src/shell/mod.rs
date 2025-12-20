//! Shell execution engine
//!
//! This module provides the core shell execution functionality:
//! - `types` - Core data structures (ShellState, pipes, signals, results)
//! - `execute` - Command execution engine
//! - `commands` - Built-in shell commands
//! - `which` - Command path resolution
//! - `fs_util` - Cross-platform filesystem utilities
//! - `child_process_tracker` - Process tracking and cleanup

pub mod child_process_tracker;
pub mod commands;
pub mod execute;
pub mod fs_util;
pub mod types;
pub mod which;

// Re-export main execution functions
pub use execute::{execute, execute_with_pipes};

// Re-export types
pub use types::{
    pipe, EnvChange, ExecuteResult, FutureExecuteResult, KillSignal, KillSignalDropGuard,
    ShellPipeReader, ShellPipeWriter, ShellState, SignalKind,
};

// Re-export command types
pub use commands::{builtin_commands, ShellCommand, ShellCommandContext};
