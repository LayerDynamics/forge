//! Argument parsing utilities for shell commands
//!
//! Provides helper functions for parsing command-line arguments
//! in built-in shell commands.

use std::ffi::OsString;

/// Result of parsing a flag argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgValue {
    /// A flag was found (e.g., -n, -r)
    Flag(char),
    /// A long flag was found (e.g., --recursive)
    LongFlag(String),
    /// A positional argument
    Positional(OsString),
    /// End of flags marker (--)
    EndOfFlags,
}

/// Simple argument parser for shell commands.
pub struct ArgParser {
    args: Vec<OsString>,
    index: usize,
    flags_ended: bool,
}

impl ArgParser {
    /// Create a new argument parser.
    /// Skips the first argument (command name).
    pub fn new(args: Vec<OsString>) -> Self {
        Self {
            args,
            index: 1, // Skip command name
            flags_ended: false,
        }
    }

    /// Parse the next argument.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ArgValue> {
        if self.index >= self.args.len() {
            return None;
        }

        let arg = &self.args[self.index];
        let arg_str = arg.to_string_lossy();

        // Handle end of flags
        if arg_str == "--" && !self.flags_ended {
            self.index += 1;
            self.flags_ended = true;
            return Some(ArgValue::EndOfFlags);
        }

        // Handle flags
        if !self.flags_ended && arg_str.starts_with('-') && arg_str.len() > 1 {
            self.index += 1;

            if let Some(long_flag) = arg_str.strip_prefix("--") {
                // Long flag
                return Some(ArgValue::LongFlag(long_flag.to_string()));
            } else if let Some(short_flags) = arg_str.strip_prefix('-') {
                // Short flag(s) - return first character
                if let Some(c) = short_flags.chars().next() {
                    return Some(ArgValue::Flag(c));
                }
            }
        }

        // Positional argument
        self.index += 1;
        Some(ArgValue::Positional(arg.clone()))
    }

    /// Get remaining arguments as positional.
    pub fn remaining(&mut self) -> Vec<OsString> {
        let remaining = self.args[self.index..].to_vec();
        self.index = self.args.len();
        remaining
    }

    /// Check if a flag is present in the arguments.
    pub fn has_flag(args: &[OsString], short: char, long: &str) -> bool {
        for arg in args.iter().skip(1) {
            let s = arg.to_string_lossy();
            if s == format!("-{}", short) || s == format!("--{}", long) {
                return true;
            }
            // Check combined short flags
            if s.starts_with('-') && !s.starts_with("--") && s.contains(short) {
                return true;
            }
        }
        false
    }

    /// Get the value of an option flag (e.g., -n 10).
    pub fn get_option(args: &[OsString], short: char, long: &str) -> Option<String> {
        let mut iter = args.iter().skip(1).peekable();
        while let Some(arg) = iter.next() {
            let s = arg.to_string_lossy();
            if s == format!("-{}", short) || s == format!("--{}", long) {
                if let Some(value) = iter.next() {
                    return Some(value.to_string_lossy().to_string());
                }
            }
            // Handle -n10 style
            if s.starts_with(&format!("-{}", short)) && s.len() > 2 {
                return Some(s[2..].to_string());
            }
        }
        None
    }

    /// Get positional arguments (excluding flags).
    pub fn positional_args(args: &[OsString]) -> Vec<OsString> {
        let mut result = Vec::new();
        let mut flags_ended = false;
        let mut skip_next = false;

        for arg in args.iter().skip(1) {
            if skip_next {
                skip_next = false;
                continue;
            }

            let s = arg.to_string_lossy();

            if s == "--" {
                flags_ended = true;
                continue;
            }

            if !flags_ended && s.starts_with('-') {
                // Check if this flag takes a value
                // For simplicity, assume single-letter flags with values
                continue;
            }

            result.push(arg.clone());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_parser() {
        let args: Vec<OsString> = vec![
            "cmd".into(),
            "-n".into(),
            "--verbose".into(),
            "file.txt".into(),
        ];

        let mut parser = ArgParser::new(args);

        assert_eq!(parser.next(), Some(ArgValue::Flag('n')));
        assert_eq!(
            parser.next(),
            Some(ArgValue::LongFlag("verbose".to_string()))
        );
        assert_eq!(parser.next(), Some(ArgValue::Positional("file.txt".into())));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_has_flag() {
        let args: Vec<OsString> = vec!["cmd".into(), "-rf".into(), "dir".into()];

        assert!(ArgParser::has_flag(&args, 'r', "recursive"));
        assert!(ArgParser::has_flag(&args, 'f', "force"));
        assert!(!ArgParser::has_flag(&args, 'v', "verbose"));
    }
}
