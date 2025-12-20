//! TypeScript extraction utilities
//!
//! This module provides utilities for extracting documentation
//! from TypeScript source files. The main parsing logic is in
//! crate::parser, but this module provides additional helpers.

use crate::diagnostics::EtchResult;
use crate::node::EtchNode;
use crate::utils::swc::{parse_typescript_file, ParsedModule};
use std::path::Path;

/// TypeScript documentation extractor
///
/// Provides a higher-level interface for extracting documentation
/// from TypeScript files with additional processing.
pub struct TypeScriptExtractor {
    /// Whether to include private symbols
    include_private: bool,
    /// Whether to include internal symbols
    include_internal: bool,
}

impl Default for TypeScriptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptExtractor {
    /// Create a new extractor with default settings
    pub fn new() -> Self {
        Self {
            include_private: false,
            include_internal: false,
        }
    }

    /// Include private symbols in extraction
    pub fn include_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Include internal symbols in extraction
    pub fn include_internal(mut self, include: bool) -> Self {
        self.include_internal = include;
        self
    }

    /// Extract documentation from a TypeScript file
    pub fn extract_file(&self, path: impl AsRef<Path>) -> EtchResult<Vec<EtchNode>> {
        let mut nodes = crate::parser::parse_typescript(path)?;

        // Filter based on settings
        if !self.include_private {
            nodes.retain(|n| n.visibility.should_document());
        }
        if !self.include_internal {
            nodes.retain(|n| !n.doc.is_internal());
        }

        Ok(nodes)
    }

    /// Extract documentation from TypeScript source code
    pub fn extract_source(
        &self,
        path: impl AsRef<Path>,
        source: &str,
    ) -> EtchResult<Vec<EtchNode>> {
        let mut nodes = crate::parser::parse_typescript_str(path, source)?;

        // Filter based on settings
        if !self.include_private {
            nodes.retain(|n| n.visibility.should_document());
        }
        if !self.include_internal {
            nodes.retain(|n| !n.doc.is_internal());
        }

        Ok(nodes)
    }

    /// Get the parsed module for advanced use cases
    pub fn parse_module(&self, path: impl AsRef<Path>) -> EtchResult<ParsedModule> {
        parse_typescript_file(path)
    }
}

/// Extract exports from a TypeScript module
pub fn extract_exports(source: &str) -> Vec<String> {
    let mut exports = Vec::new();

    // Simple regex-based extraction for export names
    // This is a fallback when full parsing isn't needed
    let export_re = regex::Regex::new(
        r"export\s+(?:async\s+)?(?:function|class|interface|type|enum|const|let|var)\s+(\w+)",
    )
    .unwrap();

    for cap in export_re.captures_iter(source) {
        if let Some(name) = cap.get(1) {
            exports.push(name.as_str().to_string());
        }
    }

    // Also check for export { ... } syntax
    let named_export_re = regex::Regex::new(r"export\s*\{\s*([^}]+)\s*\}").unwrap();
    for cap in named_export_re.captures_iter(source) {
        if let Some(names) = cap.get(1) {
            for name in names.as_str().split(',') {
                let name = name.trim();
                // Handle "foo as bar" syntax
                let name = name.split(" as ").next().unwrap_or(name).trim();
                if !name.is_empty() {
                    exports.push(name.to_string());
                }
            }
        }
    }

    exports.sort();
    exports.dedup();
    exports
}

/// Check if a file is a TypeScript declaration file (.d.ts)
pub fn is_declaration_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".d.ts"))
        .unwrap_or(false)
}

/// Check if a file is a TypeScript file (.ts or .tsx)
pub fn is_typescript_file(path: impl AsRef<Path>) -> bool {
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    matches!(ext, "ts" | "tsx" | "mts" | "cts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_exports() {
        let source = r#"
export function foo() {}
export class Bar {}
export interface Baz {}
export type Qux = string;
export const VALUE = 42;
"#;

        let exports = extract_exports(source);
        assert!(exports.contains(&"foo".to_string()));
        assert!(exports.contains(&"Bar".to_string()));
        assert!(exports.contains(&"Baz".to_string()));
        assert!(exports.contains(&"Qux".to_string()));
        assert!(exports.contains(&"VALUE".to_string()));
    }

    #[test]
    fn test_is_declaration_file() {
        assert!(is_declaration_file("foo.d.ts"));
        assert!(!is_declaration_file("foo.ts"));
        assert!(!is_declaration_file("foo.js"));
    }

    #[test]
    fn test_is_typescript_file() {
        assert!(is_typescript_file("foo.ts"));
        assert!(is_typescript_file("foo.tsx"));
        assert!(is_typescript_file("foo.mts"));
        assert!(!is_typescript_file("foo.js"));
    }
}
