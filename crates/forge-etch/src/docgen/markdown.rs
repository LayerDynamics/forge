//! Markdown rendering utilities for documentation
//!
//! This module provides utilities for generating markdown documentation
//! from parsed TypeScript/forge-weld symbols.

use crate::js_doc::EtchDoc;
use crate::node::{EtchNode, EtchNodeDef, EtchNodeKind};
use crate::types::EtchType;

/// Markdown renderer for documentation
pub struct MarkdownRenderer {
    /// Whether to include code signatures
    pub include_signatures: bool,
    /// Whether to include source locations
    pub include_locations: bool,
    /// Whether to generate table of contents
    pub generate_toc: bool,
    /// Base URL for type links
    pub type_link_base: Option<String>,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self {
            include_signatures: true,
            include_locations: false,
            generate_toc: true,
            type_link_base: None,
        }
    }
}

impl MarkdownRenderer {
    /// Create a new markdown renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include signatures
    pub fn with_signatures(mut self, include: bool) -> Self {
        self.include_signatures = include;
        self
    }

    /// Set whether to include locations
    pub fn with_locations(mut self, include: bool) -> Self {
        self.include_locations = include;
        self
    }

    /// Set whether to generate TOC
    pub fn with_toc(mut self, generate: bool) -> Self {
        self.generate_toc = generate;
        self
    }

    /// Set the base URL for type links
    pub fn with_type_link_base(mut self, base: impl Into<String>) -> Self {
        self.type_link_base = Some(base.into());
        self
    }

    /// Render a single node to markdown
    pub fn render_node(&self, node: &EtchNode) -> String {
        let mut md = String::new();

        // Header with anchor
        md.push_str(&format!("### {}\n\n", node.name));

        // Signature block
        if self.include_signatures {
            let sig = node.to_typescript_signature();
            if !sig.is_empty() {
                md.push_str("```typescript\n");
                md.push_str(&sig);
                md.push_str("\n```\n\n");
            }
        }

        // Description
        if let Some(desc) = &node.doc.description {
            md.push_str(desc);
            md.push_str("\n\n");
        }

        // Parameters (for functions/ops)
        if let Some(params) = self.render_parameters(node) {
            md.push_str(&params);
        }

        // Returns (for functions/ops)
        if let Some(returns) = self.render_returns(node) {
            md.push_str(&returns);
        }

        // Properties (for interfaces/classes)
        if let Some(props) = self.render_properties(node) {
            md.push_str(&props);
        }

        // Methods (for interfaces/classes)
        if let Some(methods) = self.render_methods(node) {
            md.push_str(&methods);
        }

        // Examples
        let examples: Vec<_> = node
            .doc
            .examples()
            .filter_map(|t| {
                if let crate::js_doc::JsDocTag::Example { doc, .. } = t {
                    Some(doc.clone())
                } else {
                    None
                }
            })
            .collect();
        if !examples.is_empty() {
            md.push_str("**Example:**\n\n");
            for example in examples {
                md.push_str("```typescript\n");
                md.push_str(&example);
                md.push_str("\n```\n\n");
            }
        }

        // Deprecation warning
        if node.doc.is_deprecated() {
            let msg = node.doc.deprecated().and_then(|t| {
                if let crate::js_doc::JsDocTag::Deprecated { doc } = t {
                    doc.clone()
                } else {
                    None
                }
            });
            if let Some(msg) = msg {
                md.push_str(&format!("> **Deprecated:** {}\n\n", msg));
            } else {
                md.push_str("> **Deprecated**\n\n");
            }
        }

        // Location
        if self.include_locations && !node.location.is_unknown() {
            md.push_str(&format!(
                "_Defined in {}:{}_\n\n",
                node.location.filename, node.location.line
            ));
        }

        md.push_str("---\n\n");

        md
    }

    /// Render parameters section
    fn render_parameters(&self, node: &EtchNode) -> Option<String> {
        let params = match &node.def {
            EtchNodeDef::Function { function_def } => &function_def.params,
            EtchNodeDef::Op { op_def } => &op_def.params,
            _ => return None,
        };

        if params.is_empty() {
            return None;
        }

        let mut md = String::from("**Parameters:**\n\n");

        for param in params {
            let type_str = param
                .ts_type
                .as_ref()
                .map(|t| format!(": `{}`", t.to_typescript()))
                .unwrap_or_default();

            let optional = if param.optional { " _(optional)_" } else { "" };

            md.push_str(&format!("- `{}`{}{}", param.name, type_str, optional));

            if let Some(doc) = &param.doc {
                md.push_str(&format!(" - {}", doc));
            }

            md.push('\n');
        }

        md.push('\n');
        Some(md)
    }

    /// Render returns section
    fn render_returns(&self, node: &EtchNode) -> Option<String> {
        let return_type = match &node.def {
            EtchNodeDef::Function { function_def } => function_def.return_type.as_ref(),
            EtchNodeDef::Op { op_def } => op_def.return_type_def.as_ref(),
            _ => return None,
        };

        let return_type = return_type?;

        let mut md = String::from("**Returns:** ");
        md.push_str(&format!("`{}`", return_type.to_typescript()));

        // Get @returns doc if available
        let returns_doc = node.doc.returns().and_then(|t| {
            if let crate::js_doc::JsDocTag::Returns { doc, .. } = t {
                doc.clone()
            } else {
                None
            }
        });
        if let Some(returns_doc) = returns_doc {
            md.push_str(&format!(" - {}", returns_doc));
        }

        md.push_str("\n\n");
        Some(md)
    }

    /// Render properties section for interfaces/classes
    fn render_properties(&self, node: &EtchNode) -> Option<String> {
        let properties = match &node.def {
            EtchNodeDef::Interface { interface_def } => &interface_def.properties,
            EtchNodeDef::Class { class_def } => {
                // Convert class properties to a compatible format
                let props: Vec<_> = class_def
                    .properties
                    .iter()
                    .map(|p| crate::interface::InterfacePropertyDef {
                        name: p.name.clone(),
                        ts_type: p.ts_type.clone(),
                        optional: p.is_optional,
                        readonly: p.readonly,
                        doc: p.doc.as_ref().and_then(|d| d.description.clone()),
                        computed: false,
                    })
                    .collect();
                if props.is_empty() {
                    return None;
                }
                return Some(self.render_property_list(&props));
            }
            _ => return None,
        };

        if properties.is_empty() {
            return None;
        }

        Some(self.render_property_list(properties))
    }

    /// Render a list of properties
    fn render_property_list(
        &self,
        properties: &[crate::interface::InterfacePropertyDef],
    ) -> String {
        let mut md = String::from("**Properties:**\n\n");
        md.push_str("| Property | Type | Description |\n");
        md.push_str("|----------|------|-------------|\n");

        for prop in properties {
            let type_str = prop
                .ts_type
                .as_ref()
                .map(|t| format!("`{}`", t.to_typescript()))
                .unwrap_or_else(|| "`any`".to_string());

            let optional = if prop.optional { "?" } else { "" };
            let readonly = if prop.readonly { " _(readonly)_" } else { "" };

            let desc = prop
                .doc
                .as_ref()
                .map(|s| s.replace('\n', " "))
                .unwrap_or_default();

            md.push_str(&format!(
                "| `{}{}`{} | {} | {} |\n",
                prop.name, optional, readonly, type_str, desc
            ));
        }

        md.push('\n');
        md
    }

    /// Render methods section for interfaces/classes
    fn render_methods(&self, node: &EtchNode) -> Option<String> {
        match &node.def {
            EtchNodeDef::Interface { interface_def } => {
                if interface_def.methods.is_empty() {
                    return None;
                }
                let mut md = String::from("**Methods:**\n\n");
                for method in &interface_def.methods {
                    let params_str = method
                        .params
                        .iter()
                        .map(|p| {
                            let ty = p
                                .ts_type
                                .as_ref()
                                .map(|t| format!(": {}", t.to_typescript()))
                                .unwrap_or_default();
                            format!("{}{}", p.name, ty)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    let return_str = method
                        .return_type
                        .as_ref()
                        .map(|t| format!(": {}", t.to_typescript()))
                        .unwrap_or_default();

                    md.push_str(&format!(
                        "- `{}({}){}`",
                        method.name, params_str, return_str
                    ));

                    if let Some(desc) = &method.doc {
                        md.push_str(&format!(" - {}", desc));
                    }
                    md.push('\n');
                }
                md.push('\n');
                Some(md)
            }
            EtchNodeDef::Class { class_def } => {
                if class_def.methods.is_empty() {
                    return None;
                }
                let mut md = String::from("**Methods:**\n\n");
                for method in &class_def.methods {
                    let params_str = method
                        .params
                        .iter()
                        .map(|p| {
                            let ty = p
                                .ts_type
                                .as_ref()
                                .map(|t| format!(": {}", t.to_typescript()))
                                .unwrap_or_default();
                            format!("{}{}", p.name, ty)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    let return_str = method
                        .return_type
                        .as_ref()
                        .map(|t| format!(": {}", t.to_typescript()))
                        .unwrap_or_default();

                    let static_str = if method.is_static { "static " } else { "" };

                    md.push_str(&format!(
                        "- `{}{}({}){}`",
                        static_str, method.name, params_str, return_str
                    ));

                    if let Some(doc) = &method.doc {
                        if let Some(desc) = &doc.description {
                            md.push_str(&format!(" - {}", desc));
                        }
                    }
                    md.push('\n');
                }
                md.push('\n');
                Some(md)
            }
            _ => None,
        }
    }

    /// Render multiple nodes grouped by kind
    pub fn render_nodes(&self, nodes: &[EtchNode]) -> String {
        let mut md = String::new();

        // Generate TOC if enabled
        if self.generate_toc && nodes.len() > 3 {
            md.push_str("## Table of Contents\n\n");
            for node in nodes {
                md.push_str(&format!("- [{}](#{})\n", node.name, slug(&node.name)));
            }
            md.push_str("\n---\n\n");
        }

        // Group by kind
        let mut functions = Vec::new();
        let mut ops = Vec::new();
        let mut interfaces = Vec::new();
        let mut classes = Vec::new();
        let mut types = Vec::new();
        let mut enums = Vec::new();
        let mut variables = Vec::new();

        for node in nodes {
            match node.kind() {
                EtchNodeKind::Function => functions.push(node),
                EtchNodeKind::Op => ops.push(node),
                EtchNodeKind::Interface | EtchNodeKind::Struct => interfaces.push(node),
                EtchNodeKind::Class => classes.push(node),
                EtchNodeKind::TypeAlias => types.push(node),
                EtchNodeKind::Enum => enums.push(node),
                EtchNodeKind::Variable => variables.push(node),
                _ => {}
            }
        }

        // Render each group
        let has_ops = !ops.is_empty();
        if has_ops {
            md.push_str("## Functions\n\n");
            for node in &ops {
                md.push_str(&self.render_node(node));
            }
        }

        if !functions.is_empty() {
            if !has_ops {
                md.push_str("## Functions\n\n");
            }
            for node in &functions {
                md.push_str(&self.render_node(node));
            }
        }

        if !interfaces.is_empty() {
            md.push_str("## Interfaces\n\n");
            for node in interfaces {
                md.push_str(&self.render_node(node));
            }
        }

        if !classes.is_empty() {
            md.push_str("## Classes\n\n");
            for node in classes {
                md.push_str(&self.render_node(node));
            }
        }

        if !types.is_empty() {
            md.push_str("## Types\n\n");
            for node in types {
                md.push_str(&self.render_node(node));
            }
        }

        if !enums.is_empty() {
            md.push_str("## Enums\n\n");
            for node in enums {
                md.push_str(&self.render_node(node));
            }
        }

        if !variables.is_empty() {
            md.push_str("## Constants\n\n");
            for node in variables {
                md.push_str(&self.render_node(node));
            }
        }

        md
    }
}

/// Generate a URL-safe slug from a string
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

/// Escape markdown special characters
pub fn escape_markdown(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' | '_' | '`' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '|' | '\\' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Convert a type to a markdown-friendly string with optional links
pub fn type_to_markdown(ty: &EtchType, link_base: Option<&str>) -> String {
    let ts = ty.to_typescript();
    if let Some(base) = link_base {
        // Find type references and link them
        let refs = super::types::collect_referenced_types(ty);
        let mut result = ts.clone();
        for ref_name in refs {
            let link = format!("[`{}`]({}#{})", ref_name, base, slug(&ref_name));
            result = result.replace(&ref_name, &link);
        }
        result
    } else {
        format!("`{}`", ts)
    }
}

/// Render a JSDoc description, handling markdown formatting
pub fn render_description(doc: &EtchDoc) -> Option<String> {
    doc.description.as_ref().map(|d| {
        // Clean up the description
        let cleaned = d
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        // Convert @link tags to markdown links

        cleaned.replace("{@link ", "[").replace("}", "]")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug() {
        assert_eq!(slug("readTextFile"), "readtextfile");
        assert_eq!(slug("MyClass"), "myclass");
        assert_eq!(slug("some_function"), "some-function");
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("Hello *world*"), "Hello \\*world\\*");
        assert_eq!(escape_markdown("Test `code`"), "Test \\`code\\`");
    }

    #[test]
    fn test_markdown_renderer() {
        let renderer = MarkdownRenderer::new()
            .with_signatures(true)
            .with_toc(false);

        assert!(renderer.include_signatures);
        assert!(!renderer.generate_toc);
    }
}
