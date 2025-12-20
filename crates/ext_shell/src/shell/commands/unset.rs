//! Unset command implementation
//!
//! Removes environment variables.

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{EnvChange, ExecuteResult, FutureExecuteResult};

/// The `unset` command - removes environment variables.
pub struct UnsetCommand;

impl ShellCommand for UnsetCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            if context.args.len() < 2 {
                let _ = context.stderr.write_line("unset: not enough arguments");
                return ExecuteResult::from_exit_code(1);
            }

            let mut changes = Vec::new();

            for arg in context.args.iter().skip(1) {
                changes.push(EnvChange::UnsetVar(arg.clone()));
            }

            ExecuteResult::Continue(0, changes, Vec::new())
        })
    }
}
