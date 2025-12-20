//! ASCII art and table rendering for terminal output
//!
//! This module provides utilities for rendering documentation
//! in plain text format suitable for terminal display.

use crate::node::{EtchNode, EtchNodeKind};
use std::fmt::Write;

/// ASCII table renderer
pub struct AsciiTable {
    /// Column headers
    headers: Vec<String>,
    /// Column widths
    widths: Vec<usize>,
    /// Row data
    rows: Vec<Vec<String>>,
    /// Border style
    border: BorderStyle,
}

/// Border style for ASCII tables
#[derive(Debug, Clone, Copy, Default)]
pub enum BorderStyle {
    /// No borders
    None,
    /// Simple ASCII borders (+, -, |)
    #[default]
    Ascii,
    /// Unicode box drawing characters
    Unicode,
}

impl AsciiTable {
    /// Create a new ASCII table with headers
    pub fn new(headers: Vec<impl Into<String>>) -> Self {
        let headers: Vec<String> = headers.into_iter().map(Into::into).collect();
        let widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            widths,
            rows: Vec::new(),
            border: BorderStyle::default(),
        }
    }

    /// Set border style
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<impl Into<String>>) {
        let row: Vec<String> = row.into_iter().map(Into::into).collect();
        // Update widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.widths.len() {
                self.widths[i] = self.widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    /// Render the table to a string
    pub fn render(&self) -> String {
        let mut output = String::new();

        let (
            h_line,
            v_line,
            cross,
            corner_tl,
            corner_tr,
            corner_bl,
            corner_br,
            t_down,
            t_up,
            t_right,
            t_left,
        ) = match self.border {
            BorderStyle::None => (' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '),
            BorderStyle::Ascii => ('-', '|', '+', '+', '+', '+', '+', '+', '+', '+', '+'),
            BorderStyle::Unicode => ('─', '│', '┼', '┌', '┐', '└', '┘', '┬', '┴', '├', '┤'),
        };

        // Top border
        if !matches!(self.border, BorderStyle::None) {
            output.push(corner_tl);
            for (i, width) in self.widths.iter().enumerate() {
                for _ in 0..(*width + 2) {
                    output.push(h_line);
                }
                if i < self.widths.len() - 1 {
                    output.push(t_down);
                }
            }
            output.push(corner_tr);
            output.push('\n');
        }

        // Headers
        output.push(v_line);
        for (i, header) in self.headers.iter().enumerate() {
            let width = self.widths.get(i).copied().unwrap_or(header.len());
            write!(output, " {:width$} ", header, width = width).ok();
            output.push(v_line);
        }
        output.push('\n');

        // Header separator
        if !matches!(self.border, BorderStyle::None) {
            output.push(t_right);
            for (i, width) in self.widths.iter().enumerate() {
                for _ in 0..(*width + 2) {
                    output.push(h_line);
                }
                if i < self.widths.len() - 1 {
                    output.push(cross);
                }
            }
            output.push(t_left);
            output.push('\n');
        }

        // Rows
        for row in &self.rows {
            output.push(v_line);
            for (i, cell) in row.iter().enumerate() {
                let width = self.widths.get(i).copied().unwrap_or(cell.len());
                write!(output, " {:width$} ", cell, width = width).ok();
                output.push(v_line);
            }
            output.push('\n');
        }

        // Bottom border
        if !matches!(self.border, BorderStyle::None) {
            output.push(corner_bl);
            for (i, width) in self.widths.iter().enumerate() {
                for _ in 0..(*width + 2) {
                    output.push(h_line);
                }
                if i < self.widths.len() - 1 {
                    output.push(t_up);
                }
            }
            output.push(corner_br);
            output.push('\n');
        }

        output
    }
}

/// Render a node as ASCII documentation
pub fn render_node_ascii(node: &EtchNode) -> String {
    let mut output = String::new();

    // Title
    let kind_str = match node.kind() {
        EtchNodeKind::Function => "function",
        EtchNodeKind::Op => "function",
        EtchNodeKind::Class => "class",
        EtchNodeKind::Interface => "interface",
        EtchNodeKind::Struct => "interface",
        EtchNodeKind::Enum => "enum",
        EtchNodeKind::TypeAlias => "type",
        EtchNodeKind::Variable => "const",
        EtchNodeKind::Namespace => "namespace",
        EtchNodeKind::Module => todo!(),
        EtchNodeKind::Import => todo!(),
        EtchNodeKind::Reference => todo!(),
    };

    writeln!(output, "{} {}", kind_str, node.name).ok();
    writeln!(
        output,
        "{}",
        "=".repeat(kind_str.len() + 1 + node.name.len())
    )
    .ok();
    output.push('\n');

    // Signature
    if let Some(sig) = node.to_typescript_signature_opt() {
        writeln!(output, "  {}", sig).ok();
        output.push('\n');
    }

    // Description
    if let Some(desc) = &node.doc.description {
        for line in desc.lines() {
            writeln!(output, "  {}", line).ok();
        }
        output.push('\n');
    }

    // Parameters table
    let params = match &node.def {
        crate::node::EtchNodeDef::Function { function_def } => Some(&function_def.params),
        crate::node::EtchNodeDef::Op { op_def } => Some(&op_def.params),
        _ => None,
    };

    if let Some(params) = params {
        if !params.is_empty() {
            writeln!(output, "  Parameters:").ok();
            let mut table = AsciiTable::new(vec!["Name", "Type", "Optional", "Description"]);
            for param in params {
                let type_str = param
                    .ts_type
                    .as_ref()
                    .map(|t| t.to_typescript())
                    .unwrap_or_else(|| "any".to_string());
                let optional = if param.optional { "yes" } else { "no" };
                let desc = param.doc.as_deref().unwrap_or("");
                table.add_row(vec![
                    param.name.clone(),
                    type_str,
                    optional.to_string(),
                    desc.to_string(),
                ]);
            }
            for line in table.render().lines() {
                writeln!(output, "    {}", line).ok();
            }
            output.push('\n');
        }
    }

    // Return type
    let return_type = match &node.def {
        crate::node::EtchNodeDef::Function { function_def } => function_def.return_type.as_ref(),
        crate::node::EtchNodeDef::Op { op_def } => op_def.return_type_def.as_ref(),
        _ => None,
    };

    if let Some(ret) = return_type {
        writeln!(output, "  Returns: {}", ret.to_typescript()).ok();
        output.push('\n');
    }

    output
}

/// Render a summary table of nodes
pub fn render_summary_table(nodes: &[EtchNode]) -> String {
    let mut table =
        AsciiTable::new(vec!["Name", "Kind", "Description"]).with_border(BorderStyle::Unicode);

    for node in nodes {
        let kind = match node.kind() {
            EtchNodeKind::Function | EtchNodeKind::Op => "fn",
            EtchNodeKind::Class => "class",
            EtchNodeKind::Interface | EtchNodeKind::Struct => "iface",
            EtchNodeKind::Enum => "enum",
            EtchNodeKind::TypeAlias => "type",
            EtchNodeKind::Variable => "const",
            EtchNodeKind::Namespace => "ns",
            EtchNodeKind::Module => todo!(),
            EtchNodeKind::Import => todo!(),
            EtchNodeKind::Reference => todo!(),
        };

        let desc = node
            .doc
            .short_description()
            .unwrap_or_default()
            .chars()
            .take(40)
            .collect::<String>();

        table.add_row(vec![node.name.clone(), kind.to_string(), desc]);
    }

    table.render()
}

/// Create a simple progress bar
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("[{}]", " ".repeat(width));
    }

    let filled = (current * width) / total;
    let empty = width - filled;

    format!(
        "[{}{}] {}/{}",
        "=".repeat(filled),
        " ".repeat(empty),
        current,
        total
    )
}

/// Render a tree structure
pub fn render_tree(items: &[(String, Vec<String>)]) -> String {
    let mut output = String::new();

    for (i, (parent, children)) in items.iter().enumerate() {
        let is_last_parent = i == items.len() - 1;
        let prefix = if is_last_parent {
            "└── "
        } else {
            "├── "
        };
        writeln!(output, "{}{}", prefix, parent).ok();

        for (j, child) in children.iter().enumerate() {
            let is_last_child = j == children.len() - 1;
            let parent_prefix = if is_last_parent { "    " } else { "│   " };
            let child_prefix = if is_last_child {
                "└── "
            } else {
                "├── "
            };
            writeln!(output, "{}{}{}", parent_prefix, child_prefix, child).ok();
        }
    }

    output
}

/// Box drawing for sections
pub fn draw_box(title: &str, content: &str, width: usize) -> String {
    let mut output = String::new();

    // Title in box
    let title_len = title.len();
    let padding = width.saturating_sub(title_len + 4);

    writeln!(output, "┌─ {} {}┐", title, "─".repeat(padding)).ok();

    // Content
    for line in content.lines() {
        let line_len = line.len();
        let line_padding = width.saturating_sub(line_len + 2);
        writeln!(output, "│ {}{} │", line, " ".repeat(line_padding)).ok();
    }

    // Bottom
    writeln!(output, "└{}┘", "─".repeat(width)).ok();

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_table() {
        let mut table = AsciiTable::new(vec!["Name", "Type"]);
        table.add_row(vec!["path", "string"]);
        table.add_row(vec!["options", "Options"]);

        let output = table.render();
        assert!(output.contains("Name"));
        assert!(output.contains("path"));
    }

    #[test]
    fn test_progress_bar() {
        let bar = progress_bar(50, 100, 20);
        assert!(bar.contains("="));
        assert!(bar.contains("50/100"));
    }

    #[test]
    fn test_render_tree() {
        let items = vec![
            (
                "functions".to_string(),
                vec!["readFile".to_string(), "writeFile".to_string()],
            ),
            ("types".to_string(), vec!["FileStat".to_string()]),
        ];

        let tree = render_tree(&items);
        assert!(tree.contains("functions"));
        assert!(tree.contains("readFile"));
    }
}
