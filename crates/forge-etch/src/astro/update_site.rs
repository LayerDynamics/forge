//! Full site regeneration for Astro documentation
//!
//! This module provides utilities for regenerating entire documentation
//! sites, including cleaning old files and generating fresh content.

#![allow(dead_code)] // Public API utilities - may not be used internally

use super::AstroGenerator;
use crate::diagnostics::EtchResult;
use crate::docgen::ExtensionDoc;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Request to update an entire documentation site
#[derive(Debug, Clone)]
pub struct SiteUpdate {
    /// Documentation for all extensions/modules to generate
    pub docs: Vec<ExtensionDoc>,
    /// Output directory for the site
    pub output_dir: PathBuf,
    /// Whether to clean the output directory first
    pub clean_first: bool,
}

impl SiteUpdate {
    /// Create a new site update request
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            docs: Vec::new(),
            output_dir,
            clean_first: false,
        }
    }

    /// Add documentation to generate
    pub fn with_doc(mut self, doc: ExtensionDoc) -> Self {
        self.docs.push(doc);
        self
    }

    /// Add multiple documentation items
    pub fn with_docs(mut self, docs: Vec<ExtensionDoc>) -> Self {
        self.docs.extend(docs);
        self
    }

    /// Enable cleaning the output directory before generation
    pub fn clean(mut self) -> Self {
        self.clean_first = true;
        self
    }
}

/// Result of a site update operation
#[derive(Debug, Clone, Default)]
pub struct SiteUpdateResult {
    /// Paths to generated files
    pub generated: Vec<PathBuf>,
    /// Paths to removed files (when cleaning)
    pub removed: Vec<PathBuf>,
    /// Total number of symbols documented
    pub total_symbols: usize,
    /// Number of modules processed
    pub module_count: usize,
}

impl SiteUpdateResult {
    /// Check if any files were generated
    pub fn has_changes(&self) -> bool {
        !self.generated.is_empty() || !self.removed.is_empty()
    }

    /// Get the total number of files affected
    pub fn total_files(&self) -> usize {
        self.generated.len() + self.removed.len()
    }
}

/// Update an entire documentation site.
///
/// Regenerates all documentation files for the provided extensions.
/// If `clean_first` is set, removes existing markdown files first.
pub fn update_site(
    _generator: &AstroGenerator,
    update: &SiteUpdate,
) -> EtchResult<SiteUpdateResult> {
    let mut result = SiteUpdateResult::default();

    // Clean output directory if requested
    if update.clean_first {
        result.removed = clean_output_dir(&update.output_dir)?;
    }

    // Ensure output directory exists
    fs::create_dir_all(&update.output_dir)?;

    // Generate documentation for each extension
    for doc in &update.docs {
        let module_dir = update.output_dir.join(sanitize_module_name(&doc.name));

        // Create a generator for this module's directory
        let module_generator = AstroGenerator::new(module_dir);

        // Generate all pages for this module
        let generated = module_generator.generate(doc)?;

        result.total_symbols += doc.nodes.len();
        result.generated.extend(generated);
        result.module_count += 1;
    }

    Ok(result)
}

/// Regenerate the entire site from scratch.
///
/// Convenience function that creates a SiteUpdate with clean=true
/// and processes all provided documentation.
pub fn regenerate_site(
    output_dir: PathBuf,
    docs: Vec<ExtensionDoc>,
) -> EtchResult<SiteUpdateResult> {
    let update = SiteUpdate::new(output_dir.clone()).with_docs(docs).clean();

    let generator = AstroGenerator::new(output_dir);
    update_site(&generator, &update)
}

/// Generate a site-wide index page listing all modules.
pub fn generate_site_index(output_dir: &PathBuf, docs: &[ExtensionDoc]) -> EtchResult<PathBuf> {
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str("title: \"API Reference\"\n");
    content.push_str("description: \"Complete API documentation for all modules\"\n");
    content.push_str("---\n\n");

    // Title
    content.push_str("# API Reference\n\n");
    content.push_str("Complete documentation for all available modules.\n\n");

    // Module listing
    content.push_str("## Modules\n\n");
    content.push_str("| Module | Description |\n");
    content.push_str("|--------|-------------|\n");

    for doc in docs {
        let description = doc
            .description
            .as_ref()
            .map(|d| first_line(d))
            .unwrap_or_default();

        let link = format!("./{}/", sanitize_module_name(&doc.name));
        content.push_str(&format!(
            "| [{}]({}) | {} |\n",
            doc.title, link, description
        ));
    }

    content.push('\n');

    // Statistics
    let total_symbols: usize = docs.iter().map(|d| d.nodes.len()).sum();
    content.push_str("## Statistics\n\n");
    content.push_str(&format!("- **Modules**: {}\n", docs.len()));
    content.push_str(&format!("- **Total Symbols**: {}\n", total_symbols));

    // Write the index file
    fs::create_dir_all(output_dir)?;
    let index_path = output_dir.join("index.md");
    fs::write(&index_path, content)?;

    Ok(index_path)
}

/// Clean markdown files from an output directory.
fn clean_output_dir(dir: &PathBuf) -> EtchResult<Vec<PathBuf>> {
    let mut removed = Vec::new();

    if !dir.exists() {
        return Ok(removed);
    }

    // Walk directory and collect markdown files to remove
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "md" || ext == "mdx" {
                fs::remove_file(path)?;
                removed.push(path.to_path_buf());
            }
        }
    }

    // Remove empty directories
    clean_empty_dirs(dir)?;

    Ok(removed)
}

/// Remove empty directories recursively.
fn clean_empty_dirs(dir: &PathBuf) -> EtchResult<()> {
    if !dir.exists() {
        return Ok(());
    }

    // Collect directories to potentially remove (deepest first)
    let mut dirs: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Sort by depth (deepest first) to remove leaf directories first
    dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

    for dir_path in dirs {
        // Don't remove the root output directory
        if dir_path == *dir {
            continue;
        }

        // Check if directory is empty
        if let Ok(mut entries) = fs::read_dir(&dir_path) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&dir_path);
            }
        }
    }

    Ok(())
}

/// Sanitize a module name for use as a directory name.
fn sanitize_module_name(name: &str) -> String {
    name.replace("::", "-")
        .replace([':', '/', '\\'], "-")
        .to_lowercase()
}

/// Get the first line of a string.
fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_doc(name: &str, specifier: &str) -> ExtensionDoc {
        ExtensionDoc {
            name: name.to_string(),
            specifier: specifier.to_string(),
            title: format!("{} Extension", name),
            description: Some(format!("Documentation for {}", name)),
            nodes: vec![],
            module_doc: None,
        }
    }

    #[test]
    fn test_sanitize_module_name() {
        assert_eq!(sanitize_module_name("runtime:fs"), "runtime-fs");
        assert_eq!(sanitize_module_name("ext_fs"), "ext_fs");
        assert_eq!(sanitize_module_name("My::Module"), "my-module");
    }

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("Hello\nWorld"), "Hello");
        assert_eq!(first_line("Single line"), "Single line");
        assert_eq!(first_line("  Trimmed  \nMore"), "Trimmed");
    }

    #[test]
    fn test_site_update_builder() {
        let update = SiteUpdate::new(PathBuf::from("/output"))
            .with_doc(create_test_doc("test", "runtime:test"))
            .clean();

        assert_eq!(update.docs.len(), 1);
        assert!(update.clean_first);
    }

    #[test]
    fn test_site_update_result() {
        let mut result = SiteUpdateResult::default();
        assert!(!result.has_changes());
        assert_eq!(result.total_files(), 0);

        result.generated.push(PathBuf::from("file.md"));
        assert!(result.has_changes());
        assert_eq!(result.total_files(), 1);
    }

    #[test]
    fn test_clean_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // Create some test files
        let md_file = output_dir.join("test.md");
        let mdx_file = output_dir.join("test.mdx");
        let other_file = output_dir.join("test.txt");

        fs::write(&md_file, "# Test").unwrap();
        fs::write(&mdx_file, "# Test MDX").unwrap();
        fs::write(&other_file, "Other content").unwrap();

        let removed = clean_output_dir(&output_dir).unwrap();

        // Markdown files should be removed
        assert_eq!(removed.len(), 2);
        assert!(!md_file.exists());
        assert!(!mdx_file.exists());

        // Other files should remain
        assert!(other_file.exists());
    }

    #[test]
    fn test_generate_site_index() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        let docs = vec![
            create_test_doc("fs", "runtime:fs"),
            create_test_doc("net", "runtime:net"),
        ];

        let index_path = generate_site_index(&output_dir, &docs).unwrap();

        assert!(index_path.exists());

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("# API Reference"));
        assert!(content.contains("fs Extension"));
        assert!(content.contains("net Extension"));
        assert!(content.contains("**Modules**: 2"));
    }

    #[test]
    fn test_update_site() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        let docs = vec![create_test_doc("test", "runtime:test")];

        let update = SiteUpdate::new(output_dir.clone()).with_docs(docs);

        let generator = AstroGenerator::new(output_dir.clone());
        let result = update_site(&generator, &update).unwrap();

        assert_eq!(result.module_count, 1);
        assert!(!result.generated.is_empty());
    }

    #[test]
    fn test_regenerate_site() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // Create an existing file that should be cleaned
        let old_file = output_dir.join("old.md");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(&old_file, "# Old content").unwrap();

        let docs = vec![create_test_doc("test", "runtime:test")];

        let result = regenerate_site(output_dir.clone(), docs).unwrap();

        // Old file should be removed
        assert!(!old_file.exists());
        assert!(!result.removed.is_empty());
        assert!(result.has_changes());
    }
}
