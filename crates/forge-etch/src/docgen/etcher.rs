//! Etcher - Main documentation generation orchestrator
//!
//! This module provides the Etcher struct which coordinates all documentation
//! generation activities: parsing, extraction, merging, and output generation.

use crate::builder::BuildOutput;
use crate::diagnostics::{DiagnosticsCollector, EtchResult};
use crate::node::EtchNode;
use crate::parser::{merge_nodes, parse_typescript, weld_module_to_nodes};
use forge_weld::ir::WeldModule;
use std::path::PathBuf;

/// Configuration for the Etcher
#[derive(Debug, Clone)]
pub struct EtchConfig {
    /// Extension name (e.g., "host_fs")
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub module_specifier: String,
    /// Rust source file path
    pub rust_source: Option<PathBuf>,
    /// TypeScript source file path
    pub ts_source: Option<PathBuf>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Whether to generate Astro markdown
    pub generate_astro: bool,
    /// Whether to generate HTML
    pub generate_html: bool,
    /// Title for documentation
    pub title: Option<String>,
    /// Description for documentation
    pub description: Option<String>,
    /// Whether to include private symbols
    pub include_private: bool,
    /// Whether to include internal symbols
    pub include_internal: bool,
}

impl Default for EtchConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            module_specifier: String::new(),
            rust_source: None,
            ts_source: None,
            output_dir: PathBuf::from("docs"),
            generate_astro: true,
            generate_html: false,
            title: None,
            description: None,
            include_private: false,
            include_internal: false,
        }
    }
}

impl EtchConfig {
    /// Create a new config for an extension
    pub fn new(name: impl Into<String>, module_specifier: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            module_specifier: module_specifier.into(),
            ..Default::default()
        }
    }

    /// Get the effective title
    pub fn effective_title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| self.module_specifier.clone())
    }

    /// Get the effective description
    pub fn effective_description(&self) -> Option<String> {
        self.description.clone()
    }
}

/// The main documentation generator
///
/// Etcher coordinates the entire documentation generation pipeline:
/// 1. Parse TypeScript sources
/// 2. Extract forge-weld metadata
/// 3. Merge documentation from both sources
/// 4. Filter by visibility
/// 5. Generate output in configured formats
pub struct Etcher {
    /// Configuration
    config: EtchConfig,
    /// Diagnostics collector
    diagnostics: DiagnosticsCollector,
    /// Extracted nodes
    nodes: Vec<EtchNode>,
    /// Optional WeldModule for Rust integration
    weld_module: Option<WeldModule>,
}

impl Etcher {
    /// Create a new Etcher with the given configuration
    pub fn new(config: EtchConfig) -> Self {
        Self {
            config,
            diagnostics: DiagnosticsCollector::new(),
            nodes: Vec::new(),
            weld_module: None,
        }
    }

    /// Set a WeldModule for Rust integration
    pub fn with_weld_module(mut self, module: WeldModule) -> Self {
        self.weld_module = Some(module);
        self
    }

    /// Get the configuration
    pub fn config(&self) -> &EtchConfig {
        &self.config
    }

    /// Get the extracted nodes
    pub fn nodes(&self) -> &[EtchNode] {
        &self.nodes
    }

    /// Get the diagnostics collector
    pub fn diagnostics(&self) -> &DiagnosticsCollector {
        &self.diagnostics
    }

    /// Run the documentation generation pipeline
    pub fn run(&mut self) -> EtchResult<BuildOutput> {
        // Step 1: Parse TypeScript source
        let ts_nodes = if let Some(ref ts_path) = self.config.ts_source {
            if ts_path.exists() {
                match parse_typescript(ts_path) {
                    Ok(nodes) => {
                        self.diagnostics.info(format!(
                            "Parsed {} symbols from {}",
                            nodes.len(),
                            ts_path.display()
                        ));
                        nodes
                    }
                    Err(e) => {
                        self.diagnostics
                            .warning(format!("Failed to parse TypeScript: {}", e));
                        vec![]
                    }
                }
            } else {
                self.diagnostics.warning(format!(
                    "TypeScript source not found: {}",
                    ts_path.display()
                ));
                vec![]
            }
        } else {
            vec![]
        };

        // Step 2: Extract from WeldModule
        let weld_nodes = if let Some(ref module) = self.weld_module {
            let nodes = weld_module_to_nodes(module);
            self.diagnostics
                .info(format!("Extracted {} symbols from forge-weld", nodes.len()));
            nodes
        } else {
            vec![]
        };

        // Step 3: Merge nodes (TypeScript JSDoc takes precedence)
        self.nodes = merge_nodes(ts_nodes, weld_nodes);
        self.diagnostics
            .info(format!("Total symbols after merge: {}", self.nodes.len()));

        // Step 4: Filter by visibility
        if !self.config.include_private {
            self.nodes.retain(|n| n.visibility.should_document());
        }
        if !self.config.include_internal {
            self.nodes.retain(|n| !n.doc.is_internal())
        }

        // Step 5: Create output directory
        if !self.config.output_dir.exists() {
            std::fs::create_dir_all(&self.config.output_dir)?;
        }

        // Step 6: Generate outputs
        let mut output = BuildOutput::new(&self.config.output_dir);

        if self.config.generate_astro {
            let astro_files = self.generate_astro()?;
            output.astro_files = astro_files;
        }

        if self.config.generate_html {
            let html_files = self.generate_html()?;
            output.html_files = html_files;
        }

        output.symbol_count = self.nodes.len();

        // Print diagnostics summary
        self.diagnostics.print_summary();

        Ok(output)
    }

    /// Generate Astro markdown output
    fn generate_astro(&self) -> EtchResult<Vec<PathBuf>> {
        use crate::astro::AstroGenerator;

        let generator = AstroGenerator::new(self.config.output_dir.clone());
        let extension_doc = self.to_extension_doc();
        generator.generate(&extension_doc)
    }

    /// Generate HTML output
    fn generate_html(&self) -> EtchResult<Vec<PathBuf>> {
        use crate::html::HtmlGenerator;

        let generator = HtmlGenerator::new(self.config.output_dir.clone())?;
        let extension_doc = self.to_extension_doc();
        generator.generate(&extension_doc)
    }

    /// Convert to ExtensionDoc
    fn to_extension_doc(&self) -> super::ExtensionDoc {
        super::ExtensionDoc {
            name: self.config.name.clone(),
            specifier: self.config.module_specifier.clone(),
            title: self.config.effective_title(),
            description: self.config.effective_description(),
            nodes: self.nodes.clone(),
            module_doc: self.extract_module_doc(),
        }
    }

    /// Extract module-level documentation
    fn extract_module_doc(&self) -> Option<crate::js_doc::EtchDoc> {
        // Look for a ModuleDoc node
        for node in &self.nodes {
            if matches!(node.def, crate::node::EtchNodeDef::ModuleDoc) {
                return Some(node.doc.clone());
            }
        }
        None
    }

    /// Generate a terminal preview of the documentation.
    ///
    /// Returns a formatted string representation of all documented symbols
    /// suitable for terminal display.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use forge_etch::{Etcher, EtchConfig};
    ///
    /// let config = EtchConfig::new("fs", "runtime:fs");
    /// let mut etcher = Etcher::new(config);
    /// etcher.run().unwrap();
    /// println!("{}", etcher.preview());
    /// ```
    pub fn preview(&self) -> String {
        use crate::printer::EtchPrinter;
        let printer = EtchPrinter::new(&self.nodes, true, self.config.include_private);
        format!("{}", printer)
    }

    /// Print a terminal preview of the documentation to stdout.
    ///
    /// This prints with ANSI colors for better readability in terminals
    /// that support color output.
    pub fn print_preview(&self) {
        use crate::printer::EtchPrinter;
        let printer = EtchPrinter::new(&self.nodes, true, self.config.include_private);
        printer.print_to_stdout();
    }

    /// Generate preview without colors (for non-color terminals or piping).
    pub fn preview_plain(&self) -> String {
        use crate::printer::EtchPrinter;
        let printer = EtchPrinter::new(&self.nodes, false, self.config.include_private);
        format!("{}", printer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etch_config() {
        let config = EtchConfig::new("host_fs", "runtime:fs");
        assert_eq!(config.name, "host_fs");
        assert_eq!(config.module_specifier, "runtime:fs");
        assert_eq!(config.effective_title(), "runtime:fs");
    }

    #[test]
    fn test_etcher_creation() {
        let config = EtchConfig::new("host_fs", "runtime:fs");
        let etcher = Etcher::new(config);
        assert!(etcher.nodes().is_empty());
    }
}
