//! Cd command implementation
//!
//! Changes the current working directory.

use std::ffi::OsStr;
use std::path::PathBuf;

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{EnvChange, ExecuteResult, FutureExecuteResult};

/// The `cd` command - changes the current working directory.
pub struct CdCommand;

impl ShellCommand for CdCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let target = if context.args.len() > 1 {
                let arg = &context.args[1];
                let path_str = arg.to_string_lossy();

                // Handle ~ (home directory)
                if path_str.starts_with('~') {
                    if let Some(home) =
                        std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
                    {
                        let home_path = PathBuf::from(home);
                        if path_str == "~" {
                            home_path
                        } else {
                            home_path.join(&path_str[2..]) // Skip "~/"
                        }
                    } else {
                        let _ = context.stderr.write_line("cd: HOME not set");
                        return ExecuteResult::from_exit_code(1);
                    }
                } else if path_str == "-" {
                    // cd - : go to previous directory
                    if let Some(oldpwd) = context.state.get_var(OsStr::new("OLDPWD")) {
                        PathBuf::from(oldpwd)
                    } else {
                        let _ = context.stderr.write_line("cd: OLDPWD not set");
                        return ExecuteResult::from_exit_code(1);
                    }
                } else {
                    // Resolve relative to cwd
                    let path = PathBuf::from(arg);
                    if path.is_absolute() {
                        path
                    } else {
                        context.state.cwd().join(path)
                    }
                }
            } else {
                // No argument - go to home directory
                if let Some(home) =
                    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
                {
                    PathBuf::from(home)
                } else {
                    let _ = context.stderr.write_line("cd: HOME not set");
                    return ExecuteResult::from_exit_code(1);
                }
            };

            // Canonicalize the path
            match std::fs::canonicalize(&target) {
                Ok(canonical) => {
                    if canonical.is_dir() {
                        // Store old PWD
                        let old_pwd = context.state.cwd().clone();

                        ExecuteResult::Continue(
                            0,
                            vec![
                                EnvChange::SetEnvVar("OLDPWD".into(), old_pwd.into_os_string()),
                                EnvChange::Cd(canonical),
                            ],
                            Vec::new(),
                        )
                    } else {
                        let _ = context
                            .stderr
                            .write_line(&format!("cd: not a directory: {}", target.display()));
                        ExecuteResult::from_exit_code(1)
                    }
                }
                Err(e) => {
                    let _ = context
                        .stderr
                        .write_line(&format!("cd: {}: {}", target.display(), e));
                    ExecuteResult::from_exit_code(1)
                }
            }
        })
    }
}
