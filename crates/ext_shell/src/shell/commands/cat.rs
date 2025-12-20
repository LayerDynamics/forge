//! Cat command implementation
//!
//! Concatenates and prints files to stdout.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `cat` command - concatenates and prints files.
pub struct CatCommand;

impl ShellCommand for CatCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let files: Vec<PathBuf> = context
                .args
                .iter()
                .skip(1)
                .map(|a| {
                    let path = PathBuf::from(a);
                    if path.is_absolute() {
                        path
                    } else {
                        context.state.cwd().join(path)
                    }
                })
                .collect();

            // If no files specified, read from stdin
            if files.is_empty() {
                let mut buf = Vec::new();
                if let Err(e) = context.stdin.pipe_to(&mut buf) {
                    let _ = context.stderr.write_line(&format!("cat: {}", e));
                    return ExecuteResult::from_exit_code(1);
                }
                if let Err(e) = context.stdout.write_all(&buf) {
                    let _ = context.stderr.write_line(&format!("cat: {}", e));
                    return ExecuteResult::from_exit_code(1);
                }
                return ExecuteResult::from_exit_code(0);
            }

            let mut exit_code = 0;

            for path in files {
                match File::open(&path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        let mut buffer = [0u8; 8192];

                        loop {
                            match reader.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Err(e) = context.stdout.write_all(&buffer[..n]) {
                                        let _ = context.stderr.write_line(&format!("cat: {}", e));
                                        exit_code = 1;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    let _ = context.stderr.write_line(&format!(
                                        "cat: {}: {}",
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
                                .write_line(&format!("cat: {}: {}", path.display(), e));
                        exit_code = 1;
                    }
                }
            }

            ExecuteResult::from_exit_code(exit_code)
        })
    }
}
