//! Head command implementation
//!
//! Prints the first N lines of a file.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use super::{ArgParser, ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `head` command - prints the first N lines of a file.
pub struct HeadCommand;

impl ShellCommand for HeadCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            // Parse -n option (default 10 lines)
            let num_lines: usize = ArgParser::get_option(&context.args, 'n', "lines")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);

            // Get file arguments (excluding -n and its value)
            let files: Vec<PathBuf> = context
                .args
                .iter()
                .skip(1)
                .filter(|a| {
                    let s = a.to_string_lossy();
                    !s.starts_with('-') && s.parse::<usize>().is_err()
                })
                .map(|a| {
                    let path = PathBuf::from(a);
                    if path.is_absolute() {
                        path
                    } else {
                        context.state.cwd().join(path)
                    }
                })
                .collect();

            // If no files, read from stdin
            if files.is_empty() {
                let mut buf = Vec::new();
                if let Err(e) = context.stdin.pipe_to(&mut buf) {
                    let _ = context.stderr.write_line(&format!("head: {}", e));
                    return ExecuteResult::from_exit_code(1);
                }

                let content = String::from_utf8_lossy(&buf);
                for line in content.lines().take(num_lines) {
                    if context.stdout.write_line(line).is_err() {
                        return ExecuteResult::from_exit_code(1);
                    }
                }
                return ExecuteResult::from_exit_code(0);
            }

            let mut exit_code = 0;
            let show_headers = files.len() > 1;

            for (i, path) in files.iter().enumerate() {
                if show_headers {
                    if i > 0 {
                        let _ = context.stdout.write_line("");
                    }
                    let _ = context
                        .stdout
                        .write_line(&format!("==> {} <==", path.display()));
                }

                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        for line in reader.lines().take(num_lines) {
                            match line {
                                Ok(line) => {
                                    if context.stdout.write_line(&line).is_err() {
                                        exit_code = 1;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    let _ = context.stderr.write_line(&format!(
                                        "head: {}: {}",
                                        path.display(),
                                        e
                                    ));
                                    exit_code = 1;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ =
                            context
                                .stderr
                                .write_line(&format!("head: {}: {}", path.display(), e));
                        exit_code = 1;
                    }
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}
