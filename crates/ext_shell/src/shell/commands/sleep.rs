//! Sleep command implementation
//!
//! Pauses execution for a specified duration.

use std::time::Duration;

use super::{ShellCommand, ShellCommandContext};
use crate::shell::types::{ExecuteResult, FutureExecuteResult};

/// The `sleep` command - pauses for N seconds.
pub struct SleepCommand;

impl ShellCommand for SleepCommand {
    fn execute(&self, mut context: ShellCommandContext) -> FutureExecuteResult {
        Box::pin(async move {
            if context.args.len() < 2 {
                let _ = context.stderr.write_line("sleep: missing operand");
                return ExecuteResult::from_exit_code(1);
            }

            let duration_str = context.args[1].to_string_lossy();

            // Parse duration - supports seconds (with optional decimal)
            // and suffix: s (seconds), m (minutes), h (hours)
            let duration = parse_duration(&duration_str);

            match duration {
                Some(dur) => {
                    // Use tokio::time::sleep for async-friendly sleeping
                    tokio::time::sleep(dur).await;
                    ExecuteResult::from_exit_code(0)
                }
                None => {
                    let _ = context
                        .stderr
                        .write_line(&format!("sleep: invalid time interval '{}'", duration_str));
                    ExecuteResult::from_exit_code(1)
                }
            }
        })
    }
}

/// Parse a duration string like "5", "2.5", "1m", "2h".
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();

    if s.is_empty() {
        return None;
    }

    // Check for suffix
    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('s') {
        (n, 1.0)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60.0)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600.0)
    } else if let Some(n) = s.strip_suffix('d') {
        (n, 86400.0)
    } else {
        (s, 1.0)
    };

    let seconds: f64 = num_str.parse().ok()?;

    if seconds < 0.0 {
        return None;
    }

    let total_seconds = seconds * multiplier;
    Some(Duration::from_secs_f64(total_seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("5"), Some(Duration::from_secs(5)));
        assert_eq!(parse_duration("2.5"), Some(Duration::from_secs_f64(2.5)));
        assert_eq!(parse_duration("1m"), Some(Duration::from_secs(60)));
        assert_eq!(parse_duration("2h"), Some(Duration::from_secs(7200)));
        assert_eq!(parse_duration("1d"), Some(Duration::from_secs(86400)));
        assert_eq!(parse_duration("0.5s"), Some(Duration::from_secs_f64(0.5)));
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("abc"), None);
    }
}
