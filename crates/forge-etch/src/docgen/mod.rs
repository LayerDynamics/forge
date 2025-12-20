//! Documentation generation core
//!
//! This module provides the main documentation extraction and generation
//! pipeline. It handles parsing TypeScript sources, extracting forge-weld
//! metadata, and producing documentation in various formats.

mod ascii;
mod chart;
mod detect;
mod etcher;
pub mod extension;
mod img;
mod markdown;
mod module;
pub mod rust;
mod symbol;
mod types;
pub mod typescript;

pub use ascii::{
    draw_box, progress_bar, render_node_ascii, render_summary_table, render_tree, AsciiTable,
    BorderStyle,
};
pub use chart::{
    ascii_bar_chart, ascii_pie_chart, dependency_graph_dot, mermaid_flowchart,
    percentage_bar_chart, type_hierarchy, ApiStats,
};
pub use detect::{ProjectType, SourceDetector};
pub use etcher::{EtchConfig, Etcher};
pub use extension::ExtensionDoc;
pub use img::{
    coverage_badge, deprecated_badge, experimental_badge, generate_badge, generate_icon,
    inline_badge_html, module_diagram, shields_io_badge_markdown, shields_io_badge_url,
    version_badge, BadgeColor, BadgeStyle, IconType,
};
pub use markdown::{escape_markdown, render_description, slug, type_to_markdown, MarkdownRenderer};
pub use module::{ExportInfo, ImportInfo, ModuleDoc};
pub use rust::{TypeExport, TypeExportKind, WeldExtractor};
pub use symbol::{BadgeKind, SymbolBadge, SymbolDoc, SymbolSummary};
pub use types::{type_complexity, RenderedType};
pub use typescript::TypeScriptExtractor;
