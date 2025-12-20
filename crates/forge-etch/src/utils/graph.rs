//! Dependency graph analysis
//!
//! This module provides utilities for analyzing module dependencies
//! and building a graph of imports/exports. This is useful for:
//! - Understanding module relationships
//! - Generating navigation in documentation
//! - Detecting circular dependencies

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A node in the dependency graph representing a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    /// The module specifier
    pub specifier: String,
    /// Display name for the module
    pub name: String,
    /// Modules this module imports from
    pub imports: Vec<String>,
    /// Modules that import from this module
    pub imported_by: Vec<String>,
    /// Re-exports from other modules
    pub re_exports: Vec<ReExport>,
    /// Whether this is an entry point
    pub is_entry: bool,
}

impl ModuleNode {
    /// Create a new module node
    pub fn new(specifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            specifier: specifier.into(),
            name: name.into(),
            imports: vec![],
            imported_by: vec![],
            re_exports: vec![],
            is_entry: false,
        }
    }

    /// Mark as entry point
    pub fn as_entry(mut self) -> Self {
        self.is_entry = true;
        self
    }

    /// Add an import
    pub fn add_import(&mut self, module: impl Into<String>) {
        let module = module.into();
        if !self.imports.contains(&module) {
            self.imports.push(module);
        }
    }

    /// Add a re-export
    pub fn add_re_export(&mut self, re_export: ReExport) {
        self.re_exports.push(re_export);
    }
}

/// A re-export from another module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReExport {
    /// The source module
    pub source: String,
    /// Specific symbols re-exported (None = star export)
    pub symbols: Option<Vec<String>>,
    /// Rename mapping (original -> exported)
    pub renames: HashMap<String, String>,
}

impl ReExport {
    /// Create a star re-export (export * from)
    pub fn star(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            symbols: None,
            renames: HashMap::new(),
        }
    }

    /// Create a named re-export
    pub fn named(source: impl Into<String>, symbols: Vec<String>) -> Self {
        Self {
            source: source.into(),
            symbols: Some(symbols),
            renames: HashMap::new(),
        }
    }

    /// Add a rename
    pub fn with_rename(mut self, original: impl Into<String>, exported: impl Into<String>) -> Self {
        self.renames.insert(original.into(), exported.into());
        self
    }
}

/// The module dependency graph
#[derive(Debug, Default)]
pub struct ModuleGraph {
    /// Modules indexed by specifier
    modules: IndexMap<String, ModuleNode>,
    /// Entry points
    entry_points: Vec<String>,
}

impl ModuleGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, node: ModuleNode) {
        if node.is_entry {
            self.entry_points.push(node.specifier.clone());
        }
        self.modules.insert(node.specifier.clone(), node);
    }

    /// Get a module by specifier
    pub fn get(&self, specifier: &str) -> Option<&ModuleNode> {
        self.modules.get(specifier)
    }

    /// Get a mutable reference to a module
    pub fn get_mut(&mut self, specifier: &str) -> Option<&mut ModuleNode> {
        self.modules.get_mut(specifier)
    }

    /// Record an import relationship
    pub fn record_import(&mut self, from: &str, to: &str) {
        // Add import to source module
        if let Some(from_node) = self.modules.get_mut(from) {
            from_node.add_import(to);
        }

        // Add imported_by to target module
        if let Some(to_node) = self.modules.get_mut(to) {
            if !to_node.imported_by.contains(&from.to_string()) {
                to_node.imported_by.push(from.to_string());
            }
        }
    }

    /// Get all entry points
    pub fn entry_points(&self) -> &[String] {
        &self.entry_points
    }

    /// Get all modules
    pub fn modules(&self) -> impl Iterator<Item = &ModuleNode> {
        self.modules.values()
    }

    /// Get modules in topological order (dependencies first)
    pub fn topological_order(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        fn visit<'a>(
            graph: &'a ModuleGraph,
            specifier: &'a str,
            visited: &mut HashSet<&'a str>,
            temp_visited: &mut HashSet<&'a str>,
            result: &mut Vec<&'a str>,
        ) {
            if visited.contains(specifier) {
                return;
            }
            if temp_visited.contains(specifier) {
                // Circular dependency detected, skip
                return;
            }

            temp_visited.insert(specifier);

            if let Some(node) = graph.get(specifier) {
                for import in &node.imports {
                    visit(graph, import, visited, temp_visited, result);
                }
            }

            temp_visited.remove(specifier);
            visited.insert(specifier);
            result.push(specifier);
        }

        for specifier in self.modules.keys() {
            visit(
                self,
                specifier,
                &mut visited,
                &mut temp_visited,
                &mut result,
            );
        }

        result
    }

    /// Find circular dependencies
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            graph: &ModuleGraph,
            node: &str,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
            cycles: &mut Vec<Vec<String>>,
        ) {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(module) = graph.get(node) {
                for import in &module.imports {
                    if !visited.contains(import) {
                        dfs(graph, import, visited, rec_stack, path, cycles);
                    } else if rec_stack.contains(import) {
                        // Found a cycle
                        let cycle_start = path.iter().position(|n| n == import).unwrap();
                        let cycle: Vec<String> = path[cycle_start..].to_vec();
                        if !cycles.contains(&cycle) {
                            cycles.push(cycle);
                        }
                    }
                }
            }

            path.pop();
            rec_stack.remove(node);
        }

        for specifier in self.modules.keys() {
            if !visited.contains(specifier) {
                dfs(
                    self,
                    specifier,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Get modules that depend on a given module (direct dependents)
    pub fn dependents(&self, specifier: &str) -> Vec<&str> {
        self.get(specifier)
            .map(|n| n.imported_by.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all transitive dependencies of a module
    pub fn transitive_deps(&self, specifier: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(specifier.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(node) = self.get(&current) {
                for import in &node.imports {
                    if deps.insert(import.clone()) {
                        queue.push_back(import.clone());
                    }
                }
            }
        }

        deps
    }

    /// Get the number of modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Generate a DOT graph for visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph modules {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for (specifier, node) in &self.modules {
            let style = if node.is_entry {
                " [style=filled, fillcolor=lightblue]"
            } else {
                ""
            };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\"]{};\n",
                specifier, node.name, style
            ));
        }

        dot.push('\n');

        for (specifier, node) in &self.modules {
            for import in &node.imports {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", specifier, import));
            }
        }

        dot.push_str("}\n");
        dot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_graph() {
        let mut graph = ModuleGraph::new();

        graph.add_module(ModuleNode::new("runtime:fs", "fs").as_entry());
        graph.add_module(ModuleNode::new("runtime:path", "path"));
        graph.add_module(ModuleNode::new("runtime:io", "io"));

        graph.record_import("runtime:fs", "runtime:path");
        graph.record_import("runtime:fs", "runtime:io");
        graph.record_import("runtime:path", "runtime:io");

        assert_eq!(graph.len(), 3);

        let deps = graph.transitive_deps("runtime:fs");
        assert!(deps.contains("runtime:path"));
        assert!(deps.contains("runtime:io"));

        let order = graph.topological_order();
        // io should come before path, path before fs
        let io_pos = order.iter().position(|&s| s == "runtime:io").unwrap();
        let path_pos = order.iter().position(|&s| s == "runtime:path").unwrap();
        let fs_pos = order.iter().position(|&s| s == "runtime:fs").unwrap();
        assert!(io_pos < path_pos);
        assert!(path_pos < fs_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ModuleGraph::new();

        graph.add_module(ModuleNode::new("a", "A"));
        graph.add_module(ModuleNode::new("b", "B"));
        graph.add_module(ModuleNode::new("c", "C"));

        graph.record_import("a", "b");
        graph.record_import("b", "c");
        graph.record_import("c", "a"); // Cycle!

        let cycles = graph.find_cycles();
        assert!(!cycles.is_empty());
    }
}
