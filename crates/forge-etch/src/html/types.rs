//! HTML generation types
//!
//! Supporting types for HTML documentation generation.

use serde::{Deserialize, Serialize};

/// Configuration for HTML generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HtmlConfig {
    /// Whether to include syntax highlighting
    pub syntax_highlighting: bool,
    /// Whether to generate a single-page layout
    pub single_page: bool,
    /// Custom CSS to include
    pub custom_css: Option<String>,
    /// Whether to include search functionality
    pub include_search: bool,
}

/// Template context for page rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContext {
    /// Page title
    pub title: String,
    /// Page description
    pub description: Option<String>,
    /// Module specifier
    pub module: String,
    /// Whether this is the index page
    pub is_index: bool,
}
