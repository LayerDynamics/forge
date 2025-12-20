//! Document regeneration for Astro documentation
//!
//! This module provides utilities for regenerating individual documentation
//! files using the AstroGenerator infrastructure.

#![allow(dead_code)] // Public API utilities - may not be used internally

use super::slug::slug;
use super::AstroGenerator;
use crate::diagnostics::EtchResult;
use crate::docgen::ExtensionDoc;
use crate::node::{EtchNode, EtchNodeKind};
use std::fs;
use std::path::PathBuf;

/// Request to update a specific document
#[derive(Debug, Clone)]
pub struct DocumentUpdate {
    /// Target file path (relative to output directory)
    pub path: PathBuf,
    /// The node to document (if updating a single symbol)
    pub node: Option<EtchNode>,
    /// The full extension documentation
    pub doc: ExtensionDoc,
}

/// Result of a document update operation
#[derive(Debug, Clone)]
pub struct DocumentUpdateResult {
    /// Path to the generated file
    pub path: PathBuf,
    /// Whether the file was created (true) or updated (false)
    pub created: bool,
    /// Number of symbols documented
    pub symbol_count: usize,
}

/// Update a single documentation file.
///
/// Regenerates the specified document file using the AstroGenerator.
/// This performs a full regeneration of the file content.
pub fn update_document(
    generator: &AstroGenerator,
    update: &DocumentUpdate,
) -> EtchResult<DocumentUpdateResult> {
    let output_path = generator.output_dir().join(&update.path);
    let existed = output_path.exists();

    // Determine what type of page to generate based on the path
    let filename = update
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let symbol_count = match filename {
        "index" => {
            // Generate index page
            generate_index_page(generator, &update.doc, &output_path)?;
            update.doc.nodes.len()
        }
        "ops" | "functions" | "interfaces" | "classes" | "enums" | "types" => {
            // Generate category page

            regenerate_category_page(generator, &update.doc, filename)?
        }
        _ => {
            // Generate single symbol page if node is provided
            if let Some(node) = &update.node {
                generate_single_node_page(generator, node, &update.doc, &output_path)?;
                1
            } else {
                0
            }
        }
    };

    Ok(DocumentUpdateResult {
        path: output_path,
        created: !existed,
        symbol_count,
    })
}

/// Regenerate a category page (ops, functions, interfaces, etc.).
///
/// Filters nodes by the specified category and regenerates the page.
pub fn regenerate_category_page(
    generator: &AstroGenerator,
    doc: &ExtensionDoc,
    category: &str,
) -> EtchResult<usize> {
    let (nodes, title): (Vec<_>, &str) = match category {
        "ops" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::Op)
                .collect(),
            "Operations",
        ),
        "functions" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::Function)
                .collect(),
            "Functions",
        ),
        "interfaces" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::Interface)
                .collect(),
            "Interfaces",
        ),
        "classes" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::Class)
                .collect(),
            "Classes",
        ),
        "enums" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::Enum)
                .collect(),
            "Enums",
        ),
        "types" => (
            doc.nodes
                .iter()
                .filter(|n| n.kind() == EtchNodeKind::TypeAlias)
                .collect(),
            "Type Aliases",
        ),
        _ => return Ok(0),
    };

    if nodes.is_empty() {
        return Ok(0);
    }

    // Use the generator's internal method to create the page
    let output_path = generator.output_dir().join(format!("{}.md", category));
    let content = generate_category_content(generator, &nodes, title, doc);

    fs::write(&output_path, content)?;

    Ok(nodes.len())
}

/// Generate content for a category page
fn generate_category_content(
    generator: &AstroGenerator,
    nodes: &[&EtchNode],
    title: &str,
    doc: &ExtensionDoc,
) -> String {
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("title: \"{} - {}\"\n", title, doc.title));
    content.push_str(&format!(
        "description: \"{} in the {} module\"\n",
        title, doc.specifier
    ));
    content.push_str(&format!("module: \"{}\"\n", doc.specifier));
    content.push_str(&format!("category: \"{}\"\n", slug(title)));
    content.push_str("---\n\n");

    // Page title
    content.push_str(&format!("# {}\n\n", title));

    // Render each node
    for node in nodes {
        content.push_str(&generator.renderer().render_node(node));
        content.push_str("\n---\n\n");
    }

    content
}

/// Generate an index page for the documentation
fn generate_index_page(
    _generator: &AstroGenerator,
    doc: &ExtensionDoc,
    output_path: &PathBuf,
) -> EtchResult<()> {
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

    fs::write(output_path, content)?;

    Ok(())
}

/// Generate a page for a single node/symbol
fn generate_single_node_page(
    generator: &AstroGenerator,
    node: &EtchNode,
    doc: &ExtensionDoc,
    output_path: &PathBuf,
) -> EtchResult<()> {
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("title: \"{}\"\n", node.name));
    if let Some(desc) = node.doc.description.as_ref() {
        content.push_str(&format!(
            "description: \"{}\"\n",
            escape_yaml(&first_sentence(desc))
        ));
    }
    content.push_str(&format!("module: \"{}\"\n", doc.specifier));
    content.push_str(&format!("symbol: \"{}\"\n", node.name));
    content.push_str(&format!("kind: \"{:?}\"\n", node.kind()));
    content.push_str("---\n\n");

    // Render the node
    content.push_str(&generator.renderer().render_node(node));

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, content)?;

    Ok(())
}

/// Escape a string for use in YAML frontmatter
fn escape_yaml(s: &str) -> String {
    s.replace('"', "\\\"").replace('\n', " ")
}

/// Extract the first sentence from a description
fn first_sentence(s: &str) -> String {
    s.split(['.', '\n']).next().unwrap_or(s).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_generator(dir: &Path) -> AstroGenerator {
        AstroGenerator::new(dir.to_path_buf())
    }

    fn create_test_doc() -> ExtensionDoc {
        ExtensionDoc {
            name: "test_ext".to_string(),
            specifier: "runtime:test".to_string(),
            title: "Test Extension".to_string(),
            description: Some("A test extension".to_string()),
            nodes: vec![],
            module_doc: None,
        }
    }

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("hello"), "hello");
        assert_eq!(escape_yaml("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_yaml("line1\nline2"), "line1 line2");
    }

    #[test]
    fn test_first_sentence() {
        assert_eq!(first_sentence("Hello world. More text."), "Hello world");
        assert_eq!(first_sentence("Single sentence"), "Single sentence");
        assert_eq!(first_sentence("Line one\nLine two"), "Line one");
    }

    #[test]
    fn test_update_document_index() {
        let temp_dir = TempDir::new().unwrap();
        let generator = create_test_generator(temp_dir.path());
        let doc = create_test_doc();

        let update = DocumentUpdate {
            path: PathBuf::from("index.md"),
            node: None,
            doc,
        };

        let result = update_document(&generator, &update).unwrap();
        assert!(result.created);
        assert!(result.path.ends_with("index.md"));
    }

    #[test]
    fn test_regenerate_category_page_empty() {
        let temp_dir = TempDir::new().unwrap();
        let generator = create_test_generator(temp_dir.path());
        let doc = create_test_doc();

        // Should return 0 for empty category
        let count = regenerate_category_page(&generator, &doc, "ops").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_generate_category_content() {
        let temp_dir = TempDir::new().unwrap();
        let generator = create_test_generator(temp_dir.path());
        let doc = create_test_doc();

        let content = generate_category_content(&generator, &[], "Operations", &doc);
        assert!(content.contains("title: \"Operations - Test Extension\""));
        assert!(content.contains("# Operations"));
    }
}
