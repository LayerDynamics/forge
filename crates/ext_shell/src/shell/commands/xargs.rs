//! Xargs command implementation
//!
//! Build and execute commands from standard input.

use std::ffi::OsString;

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult, ShellPipeReader};

/// The `xargs` command - build and execute commands from stdin.
pub struct XargsCommand;

impl ShellCommand for XargsCommand {
    fn execute(&self, context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            // Get the command to execute (everything after xargs)
            let cmd_args: Vec<OsString> = context.args.iter().skip(1).cloned().collect();

            if cmd_args.is_empty() {
                // Default to echo if no command specified
                return execute_xargs_with_command(context, vec!["echo".into()]).await;
            }

            execute_xargs_with_command(context, cmd_args).await
        })
    }
}

async fn execute_xargs_with_command(
    mut context: ShellCommandContext,
    base_cmd: Vec<OsString>,
) -> ExecuteResult {
    // Read all input from stdin
    let mut input_bytes = Vec::new();
    if let Err(e) = context.stdin.pipe_to(&mut input_bytes) {
        let _ = context.stderr.write_line(&format!("xargs: {}", e));
        return ExecuteResult::from_exit_code(1);
    }

    let input = String::from_utf8_lossy(&input_bytes);

    // Split input into arguments (by whitespace and newlines)
    let additional_args: Vec<OsString> = input.split_whitespace().map(OsString::from).collect();

    if additional_args.is_empty() {
        // No input, nothing to do
        return ExecuteResult::from_exit_code(0);
    }

    // Build the full command
    let mut full_args = base_cmd;
    full_args.extend(additional_args);

    // Extract the executor function before creating the new context
    let executor = context.execute_command_args;

    // Create a new context for the sub-command
    let sub_context = ShellCommandContext {
        args: full_args,
        state: context.state.clone(),
        stdin: ShellPipeReader::stdin(), // Use actual stdin
        stdout: context.stdout,
        stderr: context.stderr,
        execute_command_args: Box::new(|_| Box::pin(async { ExecuteResult::from_exit_code(0) })),
    };

    // Execute the command using the extracted executor
    executor(sub_context).await
}
