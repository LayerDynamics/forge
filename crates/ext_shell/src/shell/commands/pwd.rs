//! Pwd command implementation
//!
//! Prints the current working directory.

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `pwd` command - prints current working directory.
pub struct PwdCommand;

impl ShellCommand for PwdCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let cwd = context.state.cwd();
            let cwd_str = cwd.to_string_lossy();

            match context.stdout.write_line(&cwd_str) {
                Ok(_) => ExecuteResult::from_exit_code(0),
                Err(_) => ExecuteResult::from_exit_code(1),
            }
        })
    }
}
