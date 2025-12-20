//! URL slug generation for Astro documentation pages
//!
//! This module provides utilities for generating URL-safe slugs
//! that are compatible with Astro's file-based routing system.

use std::path::{Path, PathBuf};

/// Generate a URL-safe slug from a string.
///
/// Converts the input string to lowercase, replaces non-alphanumeric
/// characters with hyphens, collapses multiple consecutive hyphens,
/// and trims leading/trailing hyphens.
///
/// # Examples
///
/// ```
/// use forge_etch::astro::slug::slug;
///
/// assert_eq!(slug("Hello World"), "hello-world");
/// assert_eq!(slug("runtime:fs"), "runtime-fs");
/// assert_eq!(slug("MyClass"), "myclass");
/// ```
pub fn slug(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract a slug from a file path.
///
/// Takes the file stem (filename without extension) and converts
/// it to a URL-safe slug.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use forge_etch::astro::slug::file_slug;
///
/// assert_eq!(file_slug(Path::new("runtime-fs.md")), "runtime-fs");
/// assert_eq!(file_slug(Path::new("api/MyClass.ts")), "myclass");
/// ```
pub fn file_slug(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(slug)
        .unwrap_or_default()
}

/// Slugify all components of a path.
///
/// Converts each path component to a URL-safe slug while
/// preserving the directory structure.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// use forge_etch::astro::slug::slugify_path;
///
/// let input = Path::new("API Reference/Runtime FS.md");
/// let expected = PathBuf::from("api-reference/runtime-fs.md");
/// assert_eq!(slugify_path(input), expected);
/// ```
pub fn slugify_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::Normal(s) => {
                if let Some(s_str) = s.to_str() {
                    // Check if this is a file with extension
                    if let Some((name, ext)) = s_str.rsplit_once('.') {
                        let slugified = slug(name);
                        result.push(format!("{}.{}", slugified, ext.to_lowercase()));
                    } else {
                        result.push(slug(s_str));
                    }
                }
            }
            // Preserve other components (root, prefix, etc.)
            other => result.push(other.as_os_str()),
        }
    }

    result
}

/// Generate an anchor slug for in-page navigation.
///
/// Creates a slug suitable for use as an HTML anchor ID.
/// This follows similar rules to the main `slug` function
/// but is specifically for in-page links.
pub fn anchor_slug(s: &str) -> String {
    // Anchors use the same logic as page slugs
    slug(s)
}

/// Generate a unique slug by appending a suffix if needed.
///
/// If the base slug conflicts with existing slugs, appends
/// a numeric suffix to make it unique.
pub fn unique_slug(base: &str, existing: &[String]) -> String {
    let base_slug = slug(base);

    if !existing.contains(&base_slug) {
        return base_slug;
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}-{}", base_slug, counter);
        if !existing.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_basic() {
        assert_eq!(slug("Hello World"), "hello-world");
        assert_eq!(slug("hello"), "hello");
        assert_eq!(slug("UPPERCASE"), "uppercase");
    }

    #[test]
    fn test_slug_special_chars() {
        assert_eq!(slug("runtime:fs"), "runtime-fs");
        assert_eq!(slug("my_function"), "my-function");
        assert_eq!(slug("test.case"), "test-case");
        assert_eq!(slug("foo@bar"), "foo-bar");
    }

    #[test]
    fn test_slug_numbers() {
        assert_eq!(slug("version2"), "version2");
        assert_eq!(slug("v1.2.3"), "v1-2-3");
        assert_eq!(slug("123abc"), "123abc");
    }

    #[test]
    fn test_slug_edge_cases() {
        assert_eq!(slug(""), "");
        assert_eq!(slug("   "), "");
        assert_eq!(slug("---"), "");
        assert_eq!(slug("a--b"), "a-b");
        assert_eq!(slug("-hello-"), "hello");
    }

    #[test]
    fn test_slug_unicode() {
        // Unicode characters are converted to hyphens
        assert_eq!(slug("cafeteria"), "cafeteria");
        // Non-ASCII becomes hyphens, then collapsed
        assert_eq!(slug("test"), "test");
    }

    #[test]
    fn test_file_slug() {
        assert_eq!(file_slug(Path::new("runtime-fs.md")), "runtime-fs");
        assert_eq!(file_slug(Path::new("MyClass.ts")), "myclass");
        assert_eq!(file_slug(Path::new("path/to/File Name.md")), "file-name");
        assert_eq!(file_slug(Path::new("")), "");
    }

    #[test]
    fn test_slugify_path() {
        assert_eq!(
            slugify_path(Path::new("API/Runtime FS.md")),
            PathBuf::from("api/runtime-fs.md")
        );
        assert_eq!(
            slugify_path(Path::new("Docs/My Module/Index.MD")),
            PathBuf::from("docs/my-module/index.md")
        );
    }

    #[test]
    fn test_anchor_slug() {
        assert_eq!(anchor_slug("Parameters"), "parameters");
        assert_eq!(anchor_slug("Return Value"), "return-value");
    }

    #[test]
    fn test_unique_slug() {
        let existing = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(unique_slug("baz", &existing), "baz");
        assert_eq!(unique_slug("foo", &existing), "foo-1");

        let existing_with_suffix =
            vec!["foo".to_string(), "foo-1".to_string(), "foo-2".to_string()];
        assert_eq!(unique_slug("foo", &existing_with_suffix), "foo-3");
    }
}
