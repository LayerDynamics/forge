//! Internal module organization for ext_shell
//!
//! This module re-exports the shell execution components:
//! - `parser` - Shell command parser (AST types + parsing)
//! - `shell` - Shell execution engine and types

pub mod parser;
pub mod shell;

// Re-export commonly used types for convenience
pub use parser::{parse, SequentialList};
pub use shell::{
    execute, execute_with_pipes,
    types::{
        EnvChange, ExecuteResult, FutureExecuteResult,
        KillSignal, ShellPipeReader, ShellPipeWriter, ShellState, SignalKind,
    },
    commands::{ShellCommand, ShellCommandContext},
};
