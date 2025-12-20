//! Chart and visualization utilities for documentation
//!
//! This module provides utilities for generating charts and visualizations
//! showing API structure, dependencies, and statistics.

use crate::node::{EtchNode, EtchNodeKind};
use std::collections::HashMap;
use std::fmt::Write;

/// API statistics for charting
#[derive(Debug, Default, Clone)]
pub struct ApiStats {
    /// Number of functions/ops
    pub functions: usize,
    /// Number of classes
    pub classes: usize,
    /// Number of interfaces
    pub interfaces: usize,
    /// Number of enums
    pub enums: usize,
    /// Number of type aliases
    pub types: usize,
    /// Number of variables/constants
    pub variables: usize,
    /// Number of deprecated items
    pub deprecated: usize,
    /// Number of async functions
    pub async_functions: usize,
}

impl ApiStats {
    /// Calculate stats from nodes
    pub fn from_nodes(nodes: &[EtchNode]) -> Self {
        let mut stats = Self::default();

        for node in nodes {
            match node.kind() {
                EtchNodeKind::Function | EtchNodeKind::Op => {
                    stats.functions += 1;
                    if let crate::node::EtchNodeDef::Function { function_def } = &node.def {
                        if function_def.is_async {
                            stats.async_functions += 1;
                        }
                    }
                    if let crate::node::EtchNodeDef::Op { op_def } = &node.def {
                        if op_def.is_async {
                            stats.async_functions += 1;
                        }
                    }
                }
                EtchNodeKind::Class => stats.classes += 1,
                EtchNodeKind::Interface | EtchNodeKind::Struct => stats.interfaces += 1,
                EtchNodeKind::Enum => stats.enums += 1,
                EtchNodeKind::TypeAlias => stats.types += 1,
                EtchNodeKind::Variable => stats.variables += 1,
                _ => {}
            }

            if node.doc.is_deprecated() {
                stats.deprecated += 1;
            }
        }

        stats
    }

    /// Total number of items
    pub fn total(&self) -> usize {
        self.functions + self.classes + self.interfaces + self.enums + self.types + self.variables
    }

    /// Get stats as a vector of (label, count) pairs
    pub fn as_pairs(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("Functions", self.functions),
            ("Classes", self.classes),
            ("Interfaces", self.interfaces),
            ("Enums", self.enums),
            ("Types", self.types),
            ("Variables", self.variables),
        ]
    }
}

/// Generate an ASCII bar chart
pub fn ascii_bar_chart(data: &[(&str, usize)], width: usize) -> String {
    let mut output = String::new();

    if data.is_empty() {
        return output;
    }

    // Find max value and max label length
    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
    let max_label_len = data.iter().map(|(l, _)| l.len()).max().unwrap_or(0);

    for (label, value) in data {
        let bar_width = if max_value > 0 {
            (value * width) / max_value
        } else {
            0
        };

        writeln!(
            output,
            "{:width$} | {} {}",
            label,
            "█".repeat(bar_width),
            value,
            width = max_label_len
        )
        .ok();
    }

    output
}

/// Generate an ASCII horizontal bar chart with percentages
pub fn percentage_bar_chart(data: &[(&str, usize)], width: usize) -> String {
    let mut output = String::new();

    let total: usize = data.iter().map(|(_, v)| *v).sum();
    if total == 0 {
        return output;
    }

    let max_label_len = data.iter().map(|(l, _)| l.len()).max().unwrap_or(0);

    for (label, value) in data {
        let percentage = (*value as f64 / total as f64) * 100.0;
        let bar_width = (percentage * width as f64 / 100.0) as usize;

        writeln!(
            output,
            "{:width$} | {} {:>5.1}% ({})",
            label,
            "█".repeat(bar_width),
            percentage,
            value,
            width = max_label_len
        )
        .ok();
    }

    output
}

/// Generate a simple pie chart representation (ASCII)
///
/// The `radius` parameter controls the width of the percentage bar visualization.
pub fn ascii_pie_chart(data: &[(&str, usize)], radius: usize) -> String {
    let mut output = String::new();

    let total: usize = data.iter().map(|(_, v)| *v).sum();
    if total == 0 {
        return output;
    }

    // Create a simple legend-style representation with percentage bars
    writeln!(output, "Distribution:").ok();
    writeln!(output).ok();

    let chars = ['█', '▓', '▒', '░', '▪', '▫'];
    let bar_width = radius.max(10); // Use radius as bar width, minimum 10

    for (i, (label, value)) in data.iter().enumerate() {
        if *value == 0 {
            continue;
        }

        let percentage = (*value as f64 / total as f64) * 100.0;
        let char_idx = i % chars.len();
        let bar_fill = ((percentage / 100.0) * bar_width as f64) as usize;
        let bar: String = std::iter::repeat_n(chars[char_idx], bar_fill).collect();
        let padding: String = std::iter::repeat_n(' ', bar_width - bar_fill).collect();

        writeln!(
            output,
            "  [{}{}] {:>5.1}% - {} ({})",
            bar, padding, percentage, label, value
        )
        .ok();
    }

    output
}

/// Generate a dependency graph in DOT format
pub fn dependency_graph_dot(nodes: &[EtchNode], module_name: &str) -> String {
    let mut output = String::new();

    writeln!(output, "digraph {} {{", module_name.replace(':', "_")).ok();
    writeln!(output, "  rankdir=TB;").ok();
    writeln!(output, "  node [shape=box];").ok();
    writeln!(output).ok();

    // Collect type references
    let mut edges: Vec<(String, String)> = Vec::new();

    for node in nodes {
        let node_id = sanitize_dot_id(&node.name);

        // Node styling based on kind
        let shape = match node.kind() {
            EtchNodeKind::Class => "box3d",
            EtchNodeKind::Interface | EtchNodeKind::Struct => "component",
            EtchNodeKind::Enum => "diamond",
            EtchNodeKind::Function | EtchNodeKind::Op => "ellipse",
            _ => "box",
        };

        writeln!(
            output,
            "  {} [label=\"{}\", shape={}];",
            node_id, node.name, shape
        )
        .ok();

        // Find type references
        let refs = collect_type_refs(node);
        for ref_name in refs {
            if nodes.iter().any(|n| n.name == ref_name) {
                edges.push((node.name.clone(), ref_name));
            }
        }
    }

    writeln!(output).ok();

    // Add edges
    for (from, to) in edges {
        writeln!(
            output,
            "  {} -> {};",
            sanitize_dot_id(&from),
            sanitize_dot_id(&to)
        )
        .ok();
    }

    writeln!(output, "}}").ok();

    output
}

/// Collect type references from a node
fn collect_type_refs(node: &EtchNode) -> Vec<String> {
    let mut refs = Vec::new();

    match &node.def {
        crate::node::EtchNodeDef::Function { function_def } => {
            for param in &function_def.params {
                if let Some(ty) = &param.ts_type {
                    refs.extend(ty.referenced_types());
                }
            }
            if let Some(ret) = &function_def.return_type {
                refs.extend(ret.referenced_types());
            }
        }
        crate::node::EtchNodeDef::Op { op_def } => {
            for param in &op_def.params {
                if let Some(ty) = &param.ts_type {
                    refs.extend(ty.referenced_types());
                }
            }
            if let Some(ret) = &op_def.return_type_def {
                refs.extend(ret.referenced_types());
            }
        }
        crate::node::EtchNodeDef::Interface { interface_def } => {
            for prop in &interface_def.properties {
                if let Some(ty) = &prop.ts_type {
                    refs.extend(ty.referenced_types());
                }
            }
            refs.extend(
                interface_def
                    .extends
                    .iter()
                    .flat_map(|t| t.referenced_types()),
            );
        }
        crate::node::EtchNodeDef::Class { class_def } => {
            if let Some(extends) = &class_def.extends {
                refs.push(extends.to_string());
            }
            for prop in &class_def.properties {
                if let Some(ty) = &prop.ts_type {
                    refs.extend(ty.referenced_types());
                }
            }
        }
        crate::node::EtchNodeDef::TypeAlias { type_alias_def } => {
            refs.extend(type_alias_def.ts_type.referenced_types());
        }
        _ => {}
    }

    refs
}

/// Sanitize a string for use as a DOT graph ID
fn sanitize_dot_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Generate a type hierarchy visualization
pub fn type_hierarchy(nodes: &[EtchNode]) -> String {
    let mut output = String::new();
    let mut hierarchy: HashMap<String, Vec<String>> = HashMap::new();

    // Build hierarchy from class extends and interface extends
    for node in nodes {
        match &node.def {
            crate::node::EtchNodeDef::Class { class_def } => {
                if let Some(extends) = &class_def.extends {
                    hierarchy
                        .entry(extends.to_string())
                        .or_default()
                        .push(node.name.clone());
                }
            }
            crate::node::EtchNodeDef::Interface { interface_def } => {
                for ext in &interface_def.extends {
                    if let Some(name) = ext.type_name() {
                        hierarchy.entry(name).or_default().push(node.name.clone());
                    }
                }
            }
            _ => {}
        }
    }

    // Find root types (not extending anything else)
    let all_children: Vec<&String> = hierarchy.values().flatten().collect();
    let roots: Vec<&String> = hierarchy
        .keys()
        .filter(|k| !all_children.contains(k))
        .collect();

    fn print_tree(
        output: &mut String,
        name: &str,
        hierarchy: &HashMap<String, Vec<String>>,
        prefix: &str,
        is_last: bool,
    ) {
        let connector = if is_last { "└── " } else { "├── " };
        writeln!(output, "{}{}{}", prefix, connector, name).ok();

        if let Some(children) = hierarchy.get(name) {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            for (i, child) in children.iter().enumerate() {
                print_tree(
                    output,
                    child,
                    hierarchy,
                    &new_prefix,
                    i == children.len() - 1,
                );
            }
        }
    }

    writeln!(output, "Type Hierarchy:").ok();
    for (i, root) in roots.iter().enumerate() {
        print_tree(&mut output, root, &hierarchy, "", i == roots.len() - 1);
    }

    output
}

/// Generate mermaid flowchart syntax
pub fn mermaid_flowchart(nodes: &[EtchNode], title: &str) -> String {
    let mut output = String::new();

    writeln!(output, "```mermaid").ok();
    writeln!(output, "flowchart TD").ok();
    writeln!(output, "    subgraph {}", title).ok();

    for node in nodes {
        let shape_start;
        let shape_end;

        match node.kind() {
            EtchNodeKind::Function | EtchNodeKind::Op => {
                shape_start = "([";
                shape_end = "])";
            }
            EtchNodeKind::Class => {
                shape_start = "[[";
                shape_end = "]]";
            }
            EtchNodeKind::Interface | EtchNodeKind::Struct => {
                shape_start = "[/";
                shape_end = "/]";
            }
            EtchNodeKind::Enum => {
                shape_start = "{{";
                shape_end = "}}";
            }
            _ => {
                shape_start = "[";
                shape_end = "]";
            }
        }

        let id = sanitize_dot_id(&node.name);
        writeln!(
            output,
            "        {}{}{}{}",
            id, shape_start, node.name, shape_end
        )
        .ok();
    }

    writeln!(output, "    end").ok();
    writeln!(output, "```").ok();

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_stats() {
        let stats = ApiStats {
            functions: 10,
            classes: 2,
            interfaces: 5,
            enums: 1,
            types: 3,
            variables: 2,
            deprecated: 1,
            async_functions: 5,
        };

        assert_eq!(stats.total(), 23);
    }

    #[test]
    fn test_ascii_bar_chart() {
        let data = vec![("Functions", 10), ("Classes", 5), ("Types", 3)];
        let chart = ascii_bar_chart(&data, 20);

        assert!(chart.contains("Functions"));
        assert!(chart.contains("█"));
    }

    #[test]
    fn test_percentage_bar_chart() {
        let data = vec![("A", 50), ("B", 50)];
        let chart = percentage_bar_chart(&data, 20);

        assert!(chart.contains("50.0%"));
    }

    #[test]
    fn test_dependency_graph_dot() {
        let nodes: Vec<EtchNode> = vec![];
        let dot = dependency_graph_dot(&nodes, "test");

        assert!(dot.contains("digraph"));
    }
}
