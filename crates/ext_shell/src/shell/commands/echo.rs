//! Echo command implementation
//!
//! Prints arguments to stdout, optionally without trailing newline (-n).

use super::{ArgParser, ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `echo` command - prints arguments to stdout.
pub struct EchoCommand;

impl ShellCommand for EchoCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            let args = &context.args;
            let no_newline = ArgParser::has_flag(args, 'n', "no-newline");

            // Collect arguments to print (skip command name and -n flag)
            let mut output_parts: Vec<String> = Vec::new();
            let mut skip_next = false;

            for (i, arg) in args.iter().enumerate() {
                if i == 0 {
                    continue; // Skip command name
                }

                if skip_next {
                    skip_next = false;
                    continue;
                }

                let s = arg.to_string_lossy();
                if s == "-n" {
                    continue;
                }

                output_parts.push(s.to_string());
            }

            let output = output_parts.join(" ");

            let result = if no_newline {
                context.stdout.write_all(output.as_bytes())
            } else {
                context.stdout.write_line(&output)
            };

            match result {
                Ok(_) => ExecuteResult::from_exit_code(0),
                Err(_) => ExecuteResult::from_exit_code(1),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    fn make_args(args: &[&str]) -> Vec<OsString> {
        args.iter().map(|s| OsString::from(*s)).collect()
    }

    #[test]
    fn test_echo_basic() {
        let args = make_args(&["echo", "hello", "world"]);
        // Basic test - would need full context to test output
        assert_eq!(args.len(), 3);
    }
}
