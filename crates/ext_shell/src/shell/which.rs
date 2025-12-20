//! PATH resolution utilities
//!
//! Resolves command names to executable paths using PATH environment variable.

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::types::ShellState;

/// Error when resolving a command path
#[derive(Debug, Clone)]
pub struct CommandPathResolutionError {
    pub command: String,
    pub message: String,
}

impl std::fmt::Display for CommandPathResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.command, self.message)
    }
}

impl std::error::Error for CommandPathResolutionError {}

/// Resolve a command path using the shell state's PATH.
pub fn resolve_command_path(
    command_name: &OsStr,
    cwd: &Path,
    state: &ShellState,
) -> Result<PathBuf, CommandPathResolutionError> {
    let cmd_str = command_name.to_string_lossy();

    // Get PATH from shell state
    let path_env = state.get_env_var("PATH");

    match which(command_name, path_env.as_deref()) {
        Some(path) => Ok(path),
        None => {
            // Also check relative to cwd
            let cwd_path = cwd.join(command_name);
            if is_executable(&cwd_path) {
                return Ok(cwd_path);
            }

            Err(CommandPathResolutionError {
                command: cmd_str.to_string(),
                message: "command not found".to_string(),
            })
        }
    }
}

/// Resolve a command name to its full path.
///
/// This function searches the PATH environment variable to find the executable.
/// Returns None if the command is not found.
pub fn which(cmd: &OsStr, path_env: Option<&str>) -> Option<PathBuf> {
    // If it's already an absolute path, check if it exists
    let cmd_str = cmd.to_string_lossy();
    if cmd_str.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(cmd);
        if is_executable(&path) {
            return Some(path);
        }
        return None;
    }

    // Get PATH
    let path_value = path_env
        .map(|s| s.to_string())
        .or_else(|| env::var("PATH").ok())?;

    // Search each directory in PATH
    for dir in env::split_paths(&path_value) {
        let candidate = dir.join(cmd);

        // On Windows, also try common extensions
        #[cfg(windows)]
        {
            if is_executable(&candidate) {
                return Some(candidate);
            }

            // Try with extensions
            for ext in &[".exe", ".cmd", ".bat", ".com"] {
                let with_ext = candidate.with_extension(&ext[1..]);
                if is_executable(&with_ext) {
                    return Some(with_ext);
                }
            }
        }

        #[cfg(not(windows))]
        {
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

/// Resolve a command, returning all matching executables in PATH order.
pub fn which_all(cmd: &OsStr, path_env: Option<&str>) -> Vec<PathBuf> {
    let cmd_str = cmd.to_string_lossy();

    // If it's already a path, just check if it exists
    if cmd_str.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(cmd);
        if is_executable(&path) {
            return vec![path];
        }
        return vec![];
    }

    let path_value = match path_env
        .map(|s| s.to_string())
        .or_else(|| env::var("PATH").ok())
    {
        Some(p) => p,
        None => return vec![],
    };

    let mut results = Vec::new();

    for dir in env::split_paths(&path_value) {
        let candidate = dir.join(cmd);

        #[cfg(windows)]
        {
            if is_executable(&candidate) {
                results.push(candidate.clone());
            }

            for ext in &[".exe", ".cmd", ".bat", ".com"] {
                let with_ext = candidate.with_extension(&ext[1..]);
                if is_executable(&with_ext) {
                    results.push(with_ext);
                }
            }
        }

        #[cfg(not(windows))]
        {
            if is_executable(&candidate) {
                results.push(candidate);
            }
        }
    }

    results
}

/// Check if a path is an executable file.
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let mode = metadata.permissions().mode();
            // Check if any execute bit is set
            return mode & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        // On Windows, files are executable based on extension
        // We've already added the extensions, so just check if file exists
        true
    }

    #[cfg(not(any(unix, windows)))]
    {
        true
    }
}

/// Split a PATH string into components.
pub fn split_path(path: &str) -> Vec<PathBuf> {
    env::split_paths(path).collect()
}

/// Join PATH components into a string.
pub fn join_path(paths: &[PathBuf]) -> String {
    env::join_paths(paths)
        .map(|os| os.to_string_lossy().to_string())
        .unwrap_or_default()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_absolute_path() {
        // Test with an absolute path that exists
        #[cfg(unix)]
        {
            let result = which(OsStr::new("/bin/sh"), None);
            assert!(result.is_some() || which(OsStr::new("/usr/bin/sh"), None).is_some());
        }
    }

    #[test]
    fn test_which_not_found() {
        let result = which(OsStr::new("nonexistent_command_12345"), None);
        assert!(result.is_none());
    }

    #[test]
    fn test_split_path() {
        #[cfg(unix)]
        {
            let paths = split_path("/usr/bin:/bin:/usr/local/bin");
            assert_eq!(paths.len(), 3);
            assert_eq!(paths[0], PathBuf::from("/usr/bin"));
        }
    }
}
