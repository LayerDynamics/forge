//! Mkdir command implementation
//!
//! Creates directories.

use std::fs;
use std::path::PathBuf;

use super::{ArgParser, ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `mkdir` command - creates directories.
pub struct MkdirCommand;

impl ShellCommand for MkdirCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let parents = ArgParser::has_flag(&context.args, 'p', "parents");

            let dirs: Vec<PathBuf> = context
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

            if dirs.is_empty() {
                let _ = context.stderr.write_line("mkdir: missing operand");
                return ExecuteResult::from_exit_code(1);
            }

            let mut exit_code = 0;

            for dir in dirs {
                let result = if parents {
                    fs::create_dir_all(&dir)
                } else {
                    fs::create_dir(&dir)
                };

                if let Err(e) = result {
                    let _ = context.stderr.write_line(&format!(
                        "mkdir: cannot create directory '{}': {}",
                        dir.display(),
                        e
                    ));
                    exit_code = 1;
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}
