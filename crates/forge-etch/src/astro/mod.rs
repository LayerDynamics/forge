//! Astro markdown documentation generator
//!
//! This module generates Astro-compatible markdown files for documentation sites.
//! Astro is a modern static site generator that supports MDX and components.

pub mod check_config;
pub mod compat;
pub mod slug;
pub mod update_document;
pub mod update_site;

// Re-export commonly used types from submodules
pub use check_config::{check_config, validate_config, validate_output_dir};
pub use check_config::{AstroConfig, SidebarItem, StarlightConfig, ValidationResult};
pub use compat::{detect_version, frontmatter_for_version, supports_feature};
pub use compat::{AstroVersion, CompatConfig, FrontmatterStyle};
pub use slug::{anchor_slug, file_slug, slug, slugify_path, unique_slug};
pub use update_document::{regenerate_category_page, update_document};
pub use update_document::{DocumentUpdate, DocumentUpdateResult};
pub use update_site::{generate_site_index, regenerate_site, update_site};
pub use update_site::{SiteUpdate, SiteUpdateResult};

use crate::diagnostics::EtchResult;
use crate::docgen::{ExtensionDoc, MarkdownRenderer};
use crate::node::{EtchNode, EtchNodeKind};
use std::fs;
use std::path::PathBuf;

/// Astro documentation generator
///
/// Generates Astro-compatible markdown files with frontmatter
/// that can be used with Astro's content collections.
pub struct AstroGenerator {
    /// Output directory for generated files
    output_dir: PathBuf,
    /// Markdown renderer for content
    renderer: MarkdownRenderer,
}

impl AstroGenerator {
    /// Create a new Astro generator
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            renderer: MarkdownRenderer::new().with_signatures(true).with_toc(true),
        }
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Get the output directory
    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    /// Get the markdown renderer
    pub fn renderer(&self) -> &MarkdownRenderer {
        &self.renderer
    }

    /// Generate Astro markdown files
    pub fn generate(&self, doc: &ExtensionDoc) -> EtchResult<Vec<PathBuf>> {
        let mut generated = Vec::new();

        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Generate index page
        let index_path = self.generate_index(doc)?;
        generated.push(index_path);

        // Group nodes by kind
        let ops: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Op)
            .collect();
        let functions: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Function)
            .collect();
        let interfaces: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Interface)
            .collect();
        let classes: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Class)
            .collect();
        let enums: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Enum)
            .collect();
        let types: Vec<_> = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::TypeAlias)
            .collect();

        // Generate category pages if there are items
        if !ops.is_empty() {
            let path = self.generate_category_page("ops", "Operations", &ops, doc)?;
            generated.push(path);
        }
        if !functions.is_empty() {
            let path = self.generate_category_page("functions", "Functions", &functions, doc)?;
            generated.push(path);
        }
        if !interfaces.is_empty() {
            let path = self.generate_category_page("interfaces", "Interfaces", &interfaces, doc)?;
            generated.push(path);
        }
        if !classes.is_empty() {
            let path = self.generate_category_page("classes", "Classes", &classes, doc)?;
            generated.push(path);
        }
        if !enums.is_empty() {
            let path = self.generate_category_page("enums", "Enums", &enums, doc)?;
            generated.push(path);
        }
        if !types.is_empty() {
            let path = self.generate_category_page("types", "Type Aliases", &types, doc)?;
            generated.push(path);
        }

        Ok(generated)
    }

    /// Generate the index page
    fn generate_index(&self, doc: &ExtensionDoc) -> EtchResult<PathBuf> {
        let mut content = String::new();

        // Frontmatter
        content.push_str("---\n");
        content.push_str(&format!("title: \"{}\"\n", doc.title));
        if let Some(desc) = &doc.description {
            content.push_str(&format!("description: \"{}\"\n", escape_yaml(desc)));
        }
        content.push_str(&format!("module: \"{}\"\n", doc.specifier));
        content.push_str("---\n\n");

        // Title and description
        content.push_str(&format!("# {}\n\n", doc.title));

        if let Some(desc) = &doc.description {
            content.push_str(desc);
            content.push_str("\n\n");
        }

        // Module import
        content.push_str("## Import\n\n");
        content.push_str("```typescript\n");
        content.push_str(&format!("import {{ ... }} from \"{}\";\n", doc.specifier));
        content.push_str("```\n\n");

        // Quick summary
        content.push_str("## Overview\n\n");

        let op_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Op)
            .count();
        let fn_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Function)
            .count();
        let interface_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Interface)
            .count();
        let class_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Class)
            .count();
        let enum_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::Enum)
            .count();
        let type_count = doc
            .nodes
            .iter()
            .filter(|n| n.kind() == EtchNodeKind::TypeAlias)
            .count();

        if op_count > 0 {
            content.push_str(&format!("- **Operations**: {}\n", op_count));
        }
        if fn_count > 0 {
            content.push_str(&format!("- **Functions**: {}\n", fn_count));
        }
        if interface_count > 0 {
            content.push_str(&format!("- **Interfaces**: {}\n", interface_count));
        }
        if class_count > 0 {
            content.push_str(&format!("- **Classes**: {}\n", class_count));
        }
        if enum_count > 0 {
            content.push_str(&format!("- **Enums**: {}\n", enum_count));
        }
        if type_count > 0 {
            content.push_str(&format!("- **Type Aliases**: {}\n", type_count));
        }

        let path = self.output_dir.join("index.md");
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Generate a category page
    fn generate_category_page(
        &self,
        slug: &str,
        title: &str,
        nodes: &[&EtchNode],
        doc: &ExtensionDoc,
    ) -> EtchResult<PathBuf> {
        let mut content = String::new();

        // Frontmatter
        content.push_str("---\n");
        content.push_str(&format!("title: \"{} - {}\"\n", title, doc.title));
        content.push_str(&format!(
            "description: \"{} in the {} module\"\n",
            title, doc.specifier
        ));
        content.push_str(&format!("module: \"{}\"\n", doc.specifier));
        content.push_str(&format!("category: \"{}\"\n", slug));
        content.push_str("---\n\n");

        // Page title
        content.push_str(&format!("# {}\n\n", title));

        // Render each node
        for node in nodes {
            content.push_str(&self.renderer.render_node(node));
            content.push_str("\n---\n\n");
        }

        let path = self.output_dir.join(format!("{}.md", slug));
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Render a whole module as a SINGLE Starlight documentation page.
    ///
    /// Unlike [`generate`](Self::generate) — which emits a directory of category
    /// files with `module`/`category` frontmatter — this produces one page whose
    /// frontmatter matches the Forge site's `docs` content collection exactly:
    /// `title`, `description`, and `slug`. That makes the output drop-in for
    /// `site/src/content/docs/api/<file>.md` (Starlight). The page body inlines
    /// every documented section (operations, functions, interfaces, classes,
    /// enums, types) under stable `##` headings.
    pub fn render_single_page(&self, doc: &ExtensionDoc, slug: &str) -> String {
        let mut content = String::new();

        // Frontmatter — Starlight content-collection shape.
        content.push_str("---\n");
        content.push_str(&format!("title: \"{}\"\n", escape_yaml(&doc.title)));
        if let Some(desc) = &doc.description {
            content.push_str(&format!("description: \"{}\"\n", escape_yaml(desc)));
        }
        content.push_str(&format!("slug: {slug}\n"));
        content.push_str("---\n\n");

        // Title + description prose.
        content.push_str(&format!("# {}\n\n", doc.title));
        if let Some(desc) = &doc.description {
            content.push_str(desc);
            content.push_str("\n\n");
        }

        // Import section.
        content.push_str("## Import\n\n```typescript\n");
        content.push_str(&format!("import {{ ... }} from \"{}\";\n", doc.specifier));
        content.push_str("```\n\n");

        // One section per node kind, in a stable order, skipping empty kinds.
        const SECTIONS: &[(EtchNodeKind, &str)] = &[
            (EtchNodeKind::Op, "Operations"),
            (EtchNodeKind::Function, "Functions"),
            (EtchNodeKind::Interface, "Interfaces"),
            (EtchNodeKind::Class, "Classes"),
            (EtchNodeKind::Enum, "Enums"),
            (EtchNodeKind::TypeAlias, "Types"),
        ];
        for (kind, heading) in SECTIONS {
            let nodes: Vec<&EtchNode> = doc.nodes.iter().filter(|n| n.kind() == *kind).collect();
            if nodes.is_empty() {
                continue;
            }
            content.push_str(&format!("## {heading}\n\n"));
            for node in nodes {
                content.push_str(&self.renderer.render_node(node));
                content.push('\n');
            }
        }

        content
    }

    /// Render a single Starlight page and write it to
    /// `<output_dir>/<file_stem>.md`.
    pub fn generate_single_page(
        &self,
        doc: &ExtensionDoc,
        slug: &str,
        file_stem: &str,
    ) -> EtchResult<PathBuf> {
        fs::create_dir_all(&self.output_dir)?;
        let content = self.render_single_page(doc, slug);
        let path = self.output_dir.join(format!("{file_stem}.md"));
        fs::write(&path, content)?;
        Ok(path)
    }
}

/// Escape a string for use in YAML frontmatter
fn escape_yaml(s: &str) -> String {
    s.replace('"', "\\\"").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("hello"), "hello");
        assert_eq!(escape_yaml("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_yaml("line1\nline2"), "line1 line2");
    }

    #[test]
    fn single_page_emits_starlight_frontmatter() {
        let generator = AstroGenerator::new(PathBuf::from("/tmp/forge-etch-unused"));
        let mut doc = ExtensionDoc::new("host_fs", "runtime:fs", "runtime:fs");
        doc.description = Some("File system operations.".to_string());

        let page = generator.render_single_page(&doc, "api/runtime-fs");

        // Exactly the Starlight content-collection frontmatter the site uses.
        assert!(page.starts_with("---\n"), "page must open with frontmatter");
        assert!(page.contains("title: \"runtime:fs\"\n"));
        assert!(page.contains("description: \"File system operations.\"\n"));
        assert!(page.contains("slug: api/runtime-fs\n"));
        // No leftover `module:`/`category:` keys from the multi-file generator.
        assert!(!page.contains("module:"));
        assert!(!page.contains("category:"));
        // Body has the import block.
        assert!(page.contains("## Import"));
        assert!(page.contains("import { ... } from \"runtime:fs\""));
    }

    #[test]
    fn single_page_omits_description_when_absent() {
        let generator = AstroGenerator::new(PathBuf::from("/tmp/forge-etch-unused"));
        let doc = ExtensionDoc::new("host_x", "runtime:x", "runtime:x");
        let page = generator.render_single_page(&doc, "api/runtime-x");
        assert!(!page.contains("description:"));
        assert!(page.contains("slug: api/runtime-x\n"));
    }
}
