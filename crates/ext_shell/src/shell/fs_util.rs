//! Filesystem utilities for shell operations
//!
//! Provides common filesystem operations used by shell commands.

use std::fs::{self, DirEntry, Metadata};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Result type for filesystem operations.
pub type FsResult<T> = io::Result<T>;

/// Options for directory listing.
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    /// Include hidden files (starting with .)
    pub all: bool,
    /// Include . and .. entries
    pub all_including_dots: bool,
    /// Long format listing
    pub long: bool,
    /// Show human-readable sizes
    pub human_readable: bool,
    /// Sort by time
    pub sort_by_time: bool,
    /// Reverse sort order
    pub reverse: bool,
    /// Recursive listing
    pub recursive: bool,
}

/// Entry in a directory listing.
#[derive(Debug, Clone)]
pub struct ListEntry {
    /// File name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// File metadata
    pub metadata: Option<Metadata>,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Whether this is a symlink
    pub is_symlink: bool,
}

impl ListEntry {
    /// Create from a DirEntry.
    pub fn from_dir_entry(entry: DirEntry) -> io::Result<Self> {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let is_symlink = entry.file_type().map(|t| t.is_symlink()).unwrap_or(false);

        Ok(Self {
            name,
            path,
            metadata,
            is_dir,
            is_symlink,
        })
    }

    /// Get file size.
    pub fn size(&self) -> u64 {
        self.metadata.as_ref().map(|m| m.len()).unwrap_or(0)
    }

    /// Get file permissions as string (Unix-style).
    #[cfg(unix)]
    pub fn permissions_string(&self) -> String {
        use std::os::unix::fs::PermissionsExt;

        let mode = self
            .metadata
            .as_ref()
            .map(|m| m.permissions().mode())
            .unwrap_or(0);

        let file_type = if self.is_dir {
            'd'
        } else if self.is_symlink {
            'l'
        } else {
            '-'
        };

        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            file_type,
            if mode & 0o400 != 0 { 'r' } else { '-' },
            if mode & 0o200 != 0 { 'w' } else { '-' },
            if mode & 0o100 != 0 { 'x' } else { '-' },
            if mode & 0o040 != 0 { 'r' } else { '-' },
            if mode & 0o020 != 0 { 'w' } else { '-' },
            if mode & 0o010 != 0 { 'x' } else { '-' },
            if mode & 0o004 != 0 { 'r' } else { '-' },
            if mode & 0o002 != 0 { 'w' } else { '-' },
            if mode & 0o001 != 0 { 'x' } else { '-' },
        )
    }

    #[cfg(not(unix))]
    pub fn permissions_string(&self) -> String {
        if self.is_dir {
            "drwxr-xr-x".to_string()
        } else {
            "-rw-r--r--".to_string()
        }
    }
}

/// List directory contents.
pub fn list_directory(path: &Path, options: &ListOptions) -> FsResult<Vec<ListEntry>> {
    let mut entries = Vec::new();

    // Add . and .. if requested
    if options.all_including_dots {
        // Current directory
        if let Ok(metadata) = path.metadata() {
            entries.push(ListEntry {
                name: ".".to_string(),
                path: path.to_path_buf(),
                metadata: Some(metadata),
                is_dir: true,
                is_symlink: false,
            });
        }

        // Parent directory
        if let Some(parent) = path.parent() {
            if let Ok(metadata) = parent.metadata() {
                entries.push(ListEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    metadata: Some(metadata),
                    is_dir: true,
                    is_symlink: false,
                });
            }
        }
    }

    // Read directory
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files unless -a flag
        if !options.all && name.starts_with('.') {
            continue;
        }

        entries.push(ListEntry::from_dir_entry(entry)?);
    }

    // Sort entries
    if options.sort_by_time {
        entries.sort_by(|a, b| {
            let time_a = a
                .metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let time_b = b
                .metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            time_b.cmp(&time_a) // Newest first
        });
    } else {
        // Sort by name (case-insensitive)
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    if options.reverse {
        entries.reverse();
    }

    Ok(entries)
}

/// Format file size as human-readable.
pub fn human_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut size = size as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{}", size as u64)
    } else if size < 10.0 {
        format!("{:.1}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.0}{}", size, UNITS[unit_idx])
    }
}

/// Read lines from a file.
pub fn read_lines(path: &Path) -> FsResult<impl Iterator<Item = io::Result<String>>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}

/// Read first N lines from a file.
pub fn read_head(path: &Path, n: usize) -> FsResult<Vec<String>> {
    let mut lines = Vec::with_capacity(n);
    for (i, line) in read_lines(path)?.enumerate() {
        if i >= n {
            break;
        }
        lines.push(line?);
    }
    Ok(lines)
}

/// Read last N lines from a file.
pub fn read_tail(path: &Path, n: usize) -> FsResult<Vec<String>> {
    let lines: Vec<String> = read_lines(path)?.filter_map(|l| l.ok()).collect();
    let start = lines.len().saturating_sub(n);
    Ok(lines[start..].to_vec())
}

/// Count lines, words, and bytes in a file.
pub fn word_count(path: &Path) -> FsResult<(usize, usize, usize)> {
    let content = fs::read_to_string(path)?;
    let lines = content.lines().count();
    let words = content.split_whitespace().count();
    let bytes = content.len();
    Ok((lines, words, bytes))
}

/// Recursively copy a directory.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> FsResult<()> {
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

/// Recursively remove a directory.
pub fn remove_dir_recursive(path: &Path) -> FsResult<()> {
    fs::remove_dir_all(path)
}

/// Create a directory and all parent directories.
pub fn mkdir_p(path: &Path) -> FsResult<()> {
    fs::create_dir_all(path)
}

/// Check if a path exists and is a file.
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// Check if a path exists and is a directory.
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

/// Check if a path exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Get the canonical (absolute, resolved) path.
pub fn canonicalize(path: &Path) -> FsResult<PathBuf> {
    fs::canonicalize(path)
}

/// Expand glob pattern.
pub fn glob_expand(pattern: &str, cwd: &Path) -> Vec<PathBuf> {
    let full_pattern = if PathBuf::from(pattern).is_absolute() {
        pattern.to_string()
    } else {
        cwd.join(pattern).to_string_lossy().to_string()
    };

    match glob::glob(&full_pattern) {
        Ok(paths) => paths.filter_map(|p| p.ok()).collect(),
        Err(_) => vec![],
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_human_size() {
        assert_eq!(human_size(0), "0");
        assert_eq!(human_size(100), "100");
        assert_eq!(human_size(1024), "1.0K");
        assert_eq!(human_size(1536), "1.5K");
        assert_eq!(human_size(1048576), "1.0M");
        assert_eq!(human_size(1073741824), "1.0G");
    }

    #[test]
    fn test_list_directory() {
        let current_dir = env::current_dir().unwrap();
        let options = ListOptions::default();
        let entries = list_directory(&current_dir, &options);
        assert!(entries.is_ok());
    }

    #[test]
    fn test_list_directory_with_hidden() {
        let current_dir = env::current_dir().unwrap();
        let options = ListOptions {
            all: true,
            ..Default::default()
        };
        let entries = list_directory(&current_dir, &options);
        assert!(entries.is_ok());
    }
}
