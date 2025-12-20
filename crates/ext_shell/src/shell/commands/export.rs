//! Export command implementation
//!
//! Sets environment variables that are passed to child processes.

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{EnvChange, ExecuteResult, FutureExecuteResult};

/// The `export` command - sets environment variables.
pub struct ExportCommand;

impl ShellCommand for ExportCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            if context.args.len() < 2 {
                // No arguments - list all exported variables
                for (key, value) in context.state.env_vars() {
                    let _ = context.stdout.write_line(&format!(
                        "export {}={}",
                        key.to_string_lossy(),
                        shell_escape(&value.to_string_lossy())
                    ));
                }
                return ExecuteResult::from_exit_code(0);
            }

            let mut changes = Vec::new();

            for arg in context.args.iter().skip(1) {
                let arg_str = arg.to_string_lossy();

                if let Some(eq_pos) = arg_str.find('=') {
                    // VAR=value format
                    let name = &arg_str[..eq_pos];
                    let value = &arg_str[eq_pos + 1..];
                    changes.push(EnvChange::SetEnvVar(name.into(), value.into()));
                } else {
                    // Just VAR - promote shell var to env var
                    let name = arg_str.to_string();
                    if let Some(value) = context.state.get_var(arg) {
                        changes.push(EnvChange::SetEnvVar(name.into(), value.clone()));
                    } else {
                        // Set to empty string
                        changes.push(EnvChange::SetEnvVar(name.into(), "".into()));
                    }
                }
            }

            ExecuteResult::Continue(0, changes, Vec::new())
        })
    }
}

/// Escape a string for shell output.
fn shell_escape(s: &str) -> String {
    if s.contains(|c: char| c.is_whitespace() || c == '\'' || c == '"' || c == '$') {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}
