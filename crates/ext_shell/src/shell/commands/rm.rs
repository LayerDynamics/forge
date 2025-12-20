//! Rm command implementation
//!
//! Removes files and directories.

use std::fs;
use std::path::PathBuf;

use super::{ArgParser, ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `rm` command - removes files and directories.
pub struct RmCommand;

impl ShellCommand for RmCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let recursive = ArgParser::has_flag(&context.args, 'r', "recursive")
                || ArgParser::has_flag(&context.args, 'R', "recursive");
            let force = ArgParser::has_flag(&context.args, 'f', "force");

            let paths: Vec<PathBuf> = context
                .args
                .iter()
                .skip(1)
                .filter(|a| !a.to_string_lossy().starts_with('-'))
                .map(|a| {
                    let path = PathBuf::from(a);
                    if path.is_absolute() {
                        path
                    } else {
                        context.state.cwd().join(path)
                    }
                })
                .collect();

            if paths.is_empty() && !force {
                let _ = context.stderr.write_line("rm: missing operand");
                return ExecuteResult::from_exit_code(1);
            }

            let mut exit_code = 0;

            for path in paths {
                if !path.exists() {
                    if !force {
                        let _ = context.stderr.write_line(&format!(
                            "rm: cannot remove '{}': No such file or directory",
                            path.display()
                        ));
                        exit_code = 1;
                    }
                    continue;
                }

                let result = if path.is_dir() {
                    if recursive {
                        fs::remove_dir_all(&path)
                    } else {
                        let _ = context.stderr.write_line(&format!(
                            "rm: cannot remove '{}': Is a directory",
                            path.display()
                        ));
                        exit_code = 1;
                        continue;
                    }
                } else {
                    fs::remove_file(&path)
                };

                if let Err(e) = result {
                    if !force {
                        let _ = context.stderr.write_line(&format!(
                            "rm: cannot remove '{}': {}",
                            path.display(),
                            e
                        ));
                        exit_code = 1;
                    }
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}
