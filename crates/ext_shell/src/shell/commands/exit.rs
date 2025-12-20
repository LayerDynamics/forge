//! Exit command implementation
//!
//! Exits the shell with an optional exit code.

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `exit` command - exits the shell with an exit code.
pub struct ExitCommand;

impl ShellCommand for ExitCommand {
    fn execute(&self, context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            // Default exit code is 0
            let exit_code = if context.args.len() > 1 {
                context.args[1]
                    .to_string_lossy()
                    .parse::<i32>()
                    .unwrap_or(0)
            } else {
                0
            };

            ExecuteResult::Exit(exit_code, Vec::new())
        })
    }
}
