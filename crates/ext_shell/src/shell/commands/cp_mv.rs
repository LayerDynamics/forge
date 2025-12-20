//! Cp and Mv command implementations
//!
//! Copy and move files/directories.

use std::fs;
use std::path::PathBuf;

use super::{ArgParser, ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `cp` command - copies files and directories.
pub struct CpCommand;

impl ShellCommand for CpCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let recursive = ArgParser::has_flag(&context.args, 'r', "recursive")
                || ArgParser::has_flag(&context.args, 'R', "recursive");

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

            if paths.len() < 2 {
                let _ = context.stderr.write_line("cp: missing operand");
                return ExecuteResult::from_exit_code(1);
            }

            let dest = paths.last().unwrap().clone();
            let sources = &paths[..paths.len() - 1];

            // If multiple sources, dest must be a directory
            if sources.len() > 1 && !dest.is_dir() {
                let _ = context.stderr.write_line(&format!(
                    "cp: target '{}' is not a directory",
                    dest.display()
                ));
                return ExecuteResult::from_exit_code(1);
            }

            let mut exit_code = 0;

            for source in sources {
                let target = if dest.is_dir() {
                    dest.join(source.file_name().unwrap_or_default())
                } else {
                    dest.clone()
                };

                let result = if source.is_dir() {
                    if recursive {
                        copy_dir_recursive(source, &target)
                    } else {
                        let _ = context.stderr.write_line(&format!(
                            "cp: -r not specified; omitting directory '{}'",
                            source.display()
                        ));
                        exit_code = 1;
                        continue;
                    }
                } else {
                    fs::copy(source, &target).map(|_| ())
                };

                if let Err(e) = result {
                    let _ = context.stderr.write_line(&format!(
                        "cp: cannot copy '{}' to '{}': {}",
                        source.display(),
                        target.display(),
                        e
                    ));
                    exit_code = 1;
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}

/// The `mv` command - moves files and directories.
pub struct MvCommand;

impl ShellCommand for MvCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
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

            if paths.len() < 2 {
                let _ = context.stderr.write_line("mv: missing operand");
                return ExecuteResult::from_exit_code(1);
            }

            let dest = paths.last().unwrap().clone();
            let sources = &paths[..paths.len() - 1];

            // If multiple sources, dest must be a directory
            if sources.len() > 1 && !dest.is_dir() {
                let _ = context.stderr.write_line(&format!(
                    "mv: target '{}' is not a directory",
                    dest.display()
                ));
                return ExecuteResult::from_exit_code(1);
            }

            let mut exit_code = 0;

            for source in sources {
                let target = if dest.is_dir() {
                    dest.join(source.file_name().unwrap_or_default())
                } else {
                    dest.clone()
                };

                if let Err(e) = fs::rename(source, &target) {
                    // If rename fails (cross-device), try copy + delete
                    let copy_result = if source.is_dir() {
                        copy_dir_recursive(source, &target).and_then(|_| fs::remove_dir_all(source))
                    } else {
                        fs::copy(source, &target)
                            .map(|_| ())
                            .and_then(|_| fs::remove_file(source))
                    };

                    if let Err(e2) = copy_result {
                        let _ = context.stderr.write_line(&format!(
                            "mv: cannot move '{}' to '{}': {} (fallback: {})",
                            source.display(),
                            target.display(),
                            e,
                            e2
                        ));
                        exit_code = 1;
                    }
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
