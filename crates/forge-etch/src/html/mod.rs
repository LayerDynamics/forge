//! HTML documentation generator
//!
//! This module generates standalone HTML documentation pages.
//! Unlike Astro markdown, these are complete HTML files that can be
//! served directly without a static site generator.

pub mod types;

use crate::diagnostics::EtchResult;
use crate::docgen::{ExtensionDoc, MarkdownRenderer};
use crate::embed::EmbedConfig;
use crate::node::{EtchNode, EtchNodeDef, EtchNodeKind};
use std::fs;
use std::path::PathBuf;

/// HTML documentation generator
///
/// Generates standalone HTML documentation that can be viewed
/// directly in a browser without any build step.
pub struct HtmlGenerator {
    /// Output directory for generated files
    output_dir: PathBuf,
    /// Markdown renderer for content sections (reserved for future use)
    #[allow(dead_code)]
    renderer: MarkdownRenderer,
    /// Configuration for embedded assets
    embed_config: Option<EmbedConfig>,
}

impl HtmlGenerator {
    /// Create a new HTML generator
    ///
    /// # Arguments
    /// * `output_dir` - Directory where HTML files will be written
    ///
    /// # Returns
    /// Returns a new HtmlGenerator or an error if setup fails
    pub fn new(output_dir: PathBuf) -> EtchResult<Self> {
        Ok(Self {
            output_dir,
            renderer: MarkdownRenderer::new().with_signatures(true).with_toc(true),
            embed_config: None,
        })
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Set the embed configuration for standalone output.
    ///
    /// When set, assets can be embedded directly into HTML files
    /// for single-file documentation output.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use forge_etch::html::HtmlGenerator;
    /// use forge_etch::embed::EmbedConfig;
    /// use std::path::PathBuf;
    ///
    /// let output_dir = PathBuf::from("docs");
    /// let generator = HtmlGenerator::new(output_dir).unwrap()
    ///     .with_embed_config(EmbedConfig::standalone());
    /// ```
    pub fn with_embed_config(mut self, config: EmbedConfig) -> Self {
        self.embed_config = Some(config);
        self
    }

    /// Check if assets should be inlined.
    fn should_inline_assets(&self) -> bool {
        self.embed_config
            .as_ref()
            .map(|c| c.inline_assets)
            .unwrap_or(false)
    }

    /// Generate HTML documentation files
    ///
    /// # Arguments
    /// * `doc` - Extension documentation to render
    ///
    /// # Returns
    /// Vector of paths to generated HTML files
    pub fn generate(&self, doc: &ExtensionDoc) -> EtchResult<Vec<PathBuf>> {
        let mut generated = Vec::new();

        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Generate index page
        let index_path = self.generate_index(doc)?;
        generated.push(index_path);

        // Generate CSS file
        let css_path = self.generate_css()?;
        generated.push(css_path);

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
        let desc = doc.description.as_deref().unwrap_or("");
        let escaped_desc = escape_html(desc);

        let mut body = String::new();
        body.push_str(&format!("<h1>{}</h1>\n", escape_html(&doc.title)));

        if !desc.is_empty() {
            body.push_str(&format!("<p class=\"description\">{}</p>\n", escaped_desc));
        }

        // Import section
        body.push_str("<h2>Import</h2>\n");
        body.push_str("<pre><code class=\"language-typescript\">");
        body.push_str(&format!(
            "import {{ ... }} from \"{}\";",
            escape_html(&doc.specifier)
        ));
        body.push_str("</code></pre>\n");

        // Overview section
        body.push_str("<h2>Overview</h2>\n");
        body.push_str("<ul class=\"overview-list\">\n");

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
            body.push_str(&format!(
                "<li><a href=\"ops.html\">Operations</a>: {}</li>\n",
                op_count
            ));
        }
        if fn_count > 0 {
            body.push_str(&format!(
                "<li><a href=\"functions.html\">Functions</a>: {}</li>\n",
                fn_count
            ));
        }
        if interface_count > 0 {
            body.push_str(&format!(
                "<li><a href=\"interfaces.html\">Interfaces</a>: {}</li>\n",
                interface_count
            ));
        }
        if class_count > 0 {
            body.push_str(&format!(
                "<li><a href=\"classes.html\">Classes</a>: {}</li>\n",
                class_count
            ));
        }
        if enum_count > 0 {
            body.push_str(&format!(
                "<li><a href=\"enums.html\">Enums</a>: {}</li>\n",
                enum_count
            ));
        }
        if type_count > 0 {
            body.push_str(&format!(
                "<li><a href=\"types.html\">Type Aliases</a>: {}</li>\n",
                type_count
            ));
        }

        body.push_str("</ul>\n");

        let html = self.wrap_html(&doc.title, &body);
        let path = self.output_dir.join("index.html");
        fs::write(&path, html)?;

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
        let mut body = String::new();

        body.push_str(&format!("<h1>{}</h1>\n", escape_html(title)));
        body.push_str(&format!(
            "<p class=\"breadcrumb\"><a href=\"index.html\">{}</a> &raquo; {}</p>\n",
            escape_html(&doc.title),
            escape_html(title)
        ));

        // Table of contents
        body.push_str("<nav class=\"toc\">\n<h2>Contents</h2>\n<ul>\n");
        for node in nodes {
            let anchor = slug_from_name(&node.name);
            body.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>\n",
                anchor,
                escape_html(&node.name)
            ));
        }
        body.push_str("</ul>\n</nav>\n");

        // Render each node
        for node in nodes {
            let anchor = slug_from_name(&node.name);
            body.push_str(&format!("<section id=\"{}\" class=\"symbol\">\n", anchor));

            // Symbol header
            body.push_str(&format!("<h2>{}</h2>\n", escape_html(&node.name)));

            // Render signature
            if let Some(sig) = self.render_signature(node) {
                body.push_str("<pre class=\"signature\"><code class=\"language-typescript\">");
                body.push_str(&escape_html(&sig));
                body.push_str("</code></pre>\n");
            }

            // Render description
            if let Some(desc) = node.doc.description.as_ref() {
                body.push_str(&format!(
                    "<div class=\"doc\">{}</div>\n",
                    markdown_to_html(desc)
                ));
            }

            // Render parameters for functions/ops
            if let EtchNodeDef::Function {
                function_def: ref func,
            } = node.def
            {
                if !func.params.is_empty() {
                    body.push_str("<h3>Parameters</h3>\n");
                    body.push_str("<table class=\"params\">\n");
                    body.push_str(
                        "<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>\n",
                    );
                    body.push_str("<tbody>\n");
                    for param in &func.params {
                        body.push_str("<tr>");
                        body.push_str(&format!(
                            "<td><code>{}</code></td>",
                            escape_html(&param.name)
                        ));
                        let ts_type = param
                            .ts_type
                            .as_ref()
                            .map(|t| t.to_typescript())
                            .unwrap_or_else(|| "any".to_string());
                        body.push_str(&format!("<td><code>{}</code></td>", escape_html(&ts_type)));
                        let param_doc = param.doc.as_deref().unwrap_or("");
                        body.push_str(&format!("<td>{}</td>", escape_html(param_doc)));
                        body.push_str("</tr>\n");
                    }
                    body.push_str("</tbody>\n</table>\n");
                }
            }

            if let EtchNodeDef::Op { op_def: ref op } = node.def {
                if !op.params.is_empty() {
                    body.push_str("<h3>Parameters</h3>\n");
                    body.push_str("<table class=\"params\">\n");
                    body.push_str(
                        "<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>\n",
                    );
                    body.push_str("<tbody>\n");
                    for param in &op.params {
                        body.push_str("<tr>");
                        body.push_str(&format!(
                            "<td><code>{}</code></td>",
                            escape_html(&param.name)
                        ));
                        let ts_type = param
                            .ts_type
                            .as_ref()
                            .map(|t| t.to_typescript())
                            .unwrap_or_else(|| "any".to_string());
                        body.push_str(&format!("<td><code>{}</code></td>", escape_html(&ts_type)));
                        let param_doc = param.doc.as_deref().unwrap_or("");
                        body.push_str(&format!("<td>{}</td>", escape_html(param_doc)));
                        body.push_str("</tr>\n");
                    }
                    body.push_str("</tbody>\n</table>\n");
                }
            }

            // Render examples
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
                body.push_str("<h3>Examples</h3>\n");
                for example in &examples {
                    body.push_str("<pre class=\"example\"><code class=\"language-typescript\">");
                    body.push_str(&escape_html(example));
                    body.push_str("</code></pre>\n");
                }
            }

            body.push_str("</section>\n<hr />\n");
        }

        let page_title = format!("{} - {}", title, doc.title);
        let html = self.wrap_html(&page_title, &body);
        let path = self.output_dir.join(format!("{}.html", slug));
        fs::write(&path, html)?;

        Ok(path)
    }

    /// Render a signature for a node
    fn render_signature(&self, node: &EtchNode) -> Option<String> {
        match &node.def {
            EtchNodeDef::Function {
                function_def: ref func,
            } => {
                let mut sig = String::new();
                if func.is_async {
                    sig.push_str("async ");
                }
                sig.push_str("function ");
                sig.push_str(&node.name);

                // Type parameters
                if !func.type_params.is_empty() {
                    sig.push('<');
                    sig.push_str(
                        &func
                            .type_params
                            .iter()
                            .map(|tp| tp.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    sig.push('>');
                }

                sig.push('(');
                sig.push_str(
                    &func
                        .params
                        .iter()
                        .map(|p| {
                            let ty_str = p
                                .ts_type
                                .as_ref()
                                .map(|t| t.to_typescript())
                                .unwrap_or_else(|| "unknown".to_string());
                            format!("{}: {}", p.name, ty_str)
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                sig.push_str("): ");
                sig.push_str(
                    &func
                        .return_type
                        .as_ref()
                        .map(|t| t.to_typescript())
                        .unwrap_or_else(|| "void".to_string()),
                );
                Some(sig)
            }
            EtchNodeDef::Op { op_def: ref op } => {
                let mut sig = String::new();
                if op.is_async {
                    sig.push_str("async ");
                }
                sig.push_str("function ");
                sig.push_str(&node.name);
                sig.push('(');
                sig.push_str(
                    &op.params
                        .iter()
                        .map(|p| {
                            let ty_str = p
                                .ts_type
                                .as_ref()
                                .map(|t| t.to_typescript())
                                .unwrap_or_else(|| "unknown".to_string());
                            format!("{}: {}", p.name, ty_str)
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                sig.push_str("): ");
                // OpDef has return_type as String
                sig.push_str(&op.return_type);
                Some(sig)
            }
            EtchNodeDef::Interface {
                interface_def: ref iface,
            } => {
                let mut sig = String::new();
                sig.push_str("interface ");
                sig.push_str(&node.name);
                if !iface.type_params.is_empty() {
                    sig.push('<');
                    sig.push_str(
                        &iface
                            .type_params
                            .iter()
                            .map(|tp| tp.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    sig.push('>');
                }
                sig.push_str(" { ... }");
                Some(sig)
            }
            EtchNodeDef::Class {
                class_def: ref class,
            } => {
                let mut sig = String::new();
                if class.is_abstract {
                    sig.push_str("abstract ");
                }
                sig.push_str("class ");
                sig.push_str(&node.name);
                if !class.type_params.is_empty() {
                    sig.push('<');
                    sig.push_str(
                        &class
                            .type_params
                            .iter()
                            .map(|tp| tp.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    sig.push('>');
                }
                if let Some(ref ext) = class.extends {
                    sig.push_str(" extends ");
                    sig.push_str(&ext.to_typescript());
                }
                sig.push_str(" { ... }");
                Some(sig)
            }
            EtchNodeDef::Enum { enum_def: ref e } => {
                let mut sig = String::new();
                if e.is_const {
                    sig.push_str("const ");
                }
                sig.push_str("enum ");
                sig.push_str(&node.name);
                sig.push_str(" { ... }");
                Some(sig)
            }
            EtchNodeDef::TypeAlias {
                type_alias_def: ref ta,
            } => {
                let mut sig = String::new();
                sig.push_str("type ");
                sig.push_str(&node.name);
                if !ta.type_params.is_empty() {
                    sig.push('<');
                    sig.push_str(
                        &ta.type_params
                            .iter()
                            .map(|tp| tp.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    sig.push('>');
                }
                sig.push_str(" = ");
                sig.push_str(&ta.ts_type.to_typescript());
                Some(sig)
            }
            EtchNodeDef::Variable {
                variable_def: ref var,
            } => {
                let mut sig = String::new();
                sig.push_str(if var.kind.is_const() {
                    "const "
                } else {
                    "let "
                });
                sig.push_str(&node.name);
                sig.push_str(": ");
                sig.push_str(
                    &var.ts_type
                        .as_ref()
                        .map(|t| t.to_typescript())
                        .unwrap_or_else(|| "unknown".to_string()),
                );
                Some(sig)
            }
            _ => None,
        }
    }

    /// Format a single node to HTML
    fn format_node(&self, node: &EtchNode) -> String {
        let mut html = String::new();

        // Symbol header
        html.push_str(&format!("<h2>{}</h2>\n", escape_html(&node.name)));

        // Render signature
        if let Some(sig) = self.render_signature(node) {
            html.push_str("<pre class=\"signature\"><code class=\"language-typescript\">");
            html.push_str(&escape_html(&sig));
            html.push_str("</code></pre>\n");
        }

        // Render description
        if let Some(desc) = node.doc.description.as_ref() {
            html.push_str(&format!(
                "<div class=\"doc\">{}</div>\n",
                markdown_to_html(desc)
            ));
        }

        // Render parameters for functions/ops
        if let EtchNodeDef::Function {
            function_def: ref func,
        } = node.def
        {
            if !func.params.is_empty() {
                html.push_str("<h3>Parameters</h3>\n");
                html.push_str("<table class=\"params\">\n");
                html.push_str(
                    "<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>\n",
                );
                html.push_str("<tbody>\n");
                for param in &func.params {
                    html.push_str("<tr>");
                    html.push_str(&format!(
                        "<td><code>{}</code></td>",
                        escape_html(&param.name)
                    ));
                    let ts_type = param
                        .ts_type
                        .as_ref()
                        .map(|t| t.to_typescript())
                        .unwrap_or_else(|| "any".to_string());
                    html.push_str(&format!("<td><code>{}</code></td>", escape_html(&ts_type)));
                    let param_doc = param.doc.as_deref().unwrap_or("");
                    html.push_str(&format!("<td>{}</td>", escape_html(param_doc)));
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody>\n</table>\n");
            }
        }

        if let EtchNodeDef::Op { op_def: ref op } = node.def {
            if !op.params.is_empty() {
                html.push_str("<h3>Parameters</h3>\n");
                html.push_str("<table class=\"params\">\n");
                html.push_str(
                    "<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>\n",
                );
                html.push_str("<tbody>\n");
                for param in &op.params {
                    html.push_str("<tr>");
                    html.push_str(&format!(
                        "<td><code>{}</code></td>",
                        escape_html(&param.name)
                    ));
                    let ts_type = param
                        .ts_type
                        .as_ref()
                        .map(|t| t.to_typescript())
                        .unwrap_or_else(|| "any".to_string());
                    html.push_str(&format!("<td><code>{}</code></td>", escape_html(&ts_type)));
                    let param_doc = param.doc.as_deref().unwrap_or("");
                    html.push_str(&format!("<td>{}</td>", escape_html(param_doc)));
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody>\n</table>\n");
            }
        }

        // Render examples
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
            html.push_str("<h3>Examples</h3>\n");
            for example in &examples {
                html.push_str("<pre class=\"example\"><code class=\"language-typescript\">");
                html.push_str(&escape_html(example));
                html.push_str("</code></pre>\n");
            }
        }

        html
    }

    /// Generate CSS file
    fn generate_css(&self) -> EtchResult<PathBuf> {
        let css = r##"/* forge-etch generated documentation styles */
:root {
    --bg-color: #ffffff;
    --text-color: #24292e;
    --code-bg: #f6f8fa;
    --border-color: #e1e4e8;
    --link-color: #0366d6;
    --heading-color: #24292e;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg-color: #0d1117;
        --text-color: #c9d1d9;
        --code-bg: #161b22;
        --border-color: #30363d;
        --link-color: #58a6ff;
        --heading-color: #f0f6fc;
    }
}

* {
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--bg-color);
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
}

h1, h2, h3, h4 {
    color: var(--heading-color);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

h1 { font-size: 2rem; border-bottom: 1px solid var(--border-color); padding-bottom: 0.3em; }
h2 { font-size: 1.5rem; }
h3 { font-size: 1.25rem; }

a {
    color: var(--link-color);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

code {
    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: 0.875em;
    background-color: var(--code-bg);
    padding: 0.2em 0.4em;
    border-radius: 3px;
}

pre {
    background-color: var(--code-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
}

pre code {
    background: none;
    padding: 0;
}

.description {
    font-size: 1.1rem;
    color: var(--text-color);
    margin-bottom: 1.5rem;
}

.breadcrumb {
    font-size: 0.9rem;
    color: var(--text-color);
    margin-bottom: 1rem;
}

.toc {
    background-color: var(--code-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 1rem;
    margin-bottom: 2rem;
}

.toc h2 {
    margin-top: 0;
    font-size: 1rem;
}

.toc ul {
    margin: 0;
    padding-left: 1.5rem;
}

.toc li {
    margin: 0.25rem 0;
}

.overview-list {
    list-style: none;
    padding: 0;
}

.overview-list li {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
}

.symbol {
    margin-bottom: 2rem;
}

.signature {
    margin: 1rem 0;
}

.params {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
}

.params th,
.params td {
    border: 1px solid var(--border-color);
    padding: 0.5rem;
    text-align: left;
}

.params th {
    background-color: var(--code-bg);
}

.doc {
    margin: 1rem 0;
}

.example {
    margin: 1rem 0;
}

hr {
    border: none;
    border-top: 1px solid var(--border-color);
    margin: 2rem 0;
}

footer {
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
    font-size: 0.8rem;
    color: var(--text-color);
    opacity: 0.7;
}
"##;

        let path = self.output_dir.join("styles.css");
        fs::write(&path, css)?;

        Ok(path)
    }

    /// Wrap content in HTML boilerplate
    fn wrap_html(&self, title: &str, body: &str) -> String {
        let body_with_footer = format!(
            "{body}\n<footer>\n    <p>Generated by forge-etch v{version}</p>\n</footer>",
            body = body,
            version = crate::VERSION,
        );

        // If assets should be inlined, use the embed module
        if self.should_inline_assets() {
            if let Some(ref config) = self.embed_config {
                return crate::embed::generate_standalone_html(title, &body_with_footer, config);
            }
        }

        // Default: use external CSS file reference
        format!(
            r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
{body}
</body>
</html>
"##,
            title = escape_html(title),
            body = body_with_footer,
        )
    }

    /// Generate a single standalone HTML file with all content and assets embedded.
    ///
    /// This generates a single HTML file containing all documentation,
    /// with CSS and JavaScript embedded inline. Useful for offline viewing
    /// or single-file distribution.
    ///
    /// # Arguments
    /// * `doc` - Extension documentation to render
    ///
    /// # Returns
    /// Path to the generated standalone HTML file
    pub fn generate_standalone(&self, doc: &ExtensionDoc) -> EtchResult<PathBuf> {
        use crate::embed::EmbedConfig;

        // Create a temporary generator with standalone embed config
        let config = self
            .embed_config
            .clone()
            .unwrap_or_else(EmbedConfig::standalone);

        // Generate the full body content
        let body = self.generate_full_body(doc);

        let html = crate::embed::generate_standalone_html(&doc.title, &body, &config);

        // Write to output
        fs::create_dir_all(&self.output_dir)?;
        let path = self
            .output_dir
            .join(format!("{}-standalone.html", doc.name));
        fs::write(&path, html)?;

        Ok(path)
    }

    /// Generate the full body HTML for all documentation.
    fn generate_full_body(&self, doc: &ExtensionDoc) -> String {
        let mut body = String::new();

        // Title and description
        body.push_str(&format!("<h1>{}</h1>\n", escape_html(&doc.title)));
        if let Some(ref desc) = doc.description {
            body.push_str(&format!(
                "<p class=\"description\">{}</p>\n",
                escape_html(desc)
            ));
        }

        // Table of contents
        body.push_str("<nav class=\"toc\">\n<h2>Contents</h2>\n<ul>\n");
        for node in &doc.nodes {
            body.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>\n",
                node.name,
                escape_html(&node.name)
            ));
        }
        body.push_str("</ul>\n</nav>\n");

        // All symbols
        for node in &doc.nodes {
            body.push_str(&format!(
                "<section class=\"symbol\" id=\"{}\">\n",
                node.name
            ));
            body.push_str(&self.format_node(node));
            body.push_str("</section>\n");
        }

        body.push_str(&format!(
            "<footer>\n<p>Generated by forge-etch v{}</p>\n</footer>",
            crate::VERSION
        ));

        body
    }
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Convert markdown to HTML (basic conversion)
fn markdown_to_html(md: &str) -> String {
    // Basic markdown conversion
    let mut html = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                let code_lang = line.trim_start_matches('`');
                html.push_str(&format!(
                    "<pre><code class=\"language-{}\">",
                    if code_lang.is_empty() {
                        "text"
                    } else {
                        code_lang
                    }
                ));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        // Handle inline code
        let line = convert_inline_code(line);

        // Handle headers
        if let Some(text) = line.strip_prefix("### ") {
            html.push_str(&format!("<h4>{}</h4>\n", text));
        } else if let Some(text) = line.strip_prefix("## ") {
            html.push_str(&format!("<h3>{}</h3>\n", text));
        } else if let Some(text) = line.strip_prefix("# ") {
            html.push_str(&format!("<h2>{}</h2>\n", text));
        } else if let Some(text) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            html.push_str(&format!("<li>{}</li>\n", text));
        } else if line.is_empty() {
            html.push_str("<br />\n");
        } else {
            html.push_str(&format!("<p>{}</p>\n", line));
        }
    }

    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

/// Convert inline code markers to HTML
fn convert_inline_code(s: &str) -> String {
    let mut result = String::new();
    let mut in_code = false;
    let chars = s.chars().peekable();

    for c in chars {
        if c == '`' {
            if in_code {
                result.push_str("</code>");
                in_code = false;
            } else {
                result.push_str("<code>");
                in_code = true;
            }
        } else if in_code {
            result.push_str(&escape_html(&c.to_string()));
        } else {
            result.push(c);
        }
    }

    // Close unclosed code tag
    if in_code {
        result.push_str("</code>");
    }

    result
}

/// Create a URL-safe slug from a name
fn slug_from_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_slug_from_name() {
        assert_eq!(slug_from_name("readFile"), "readfile");
        assert_eq!(slug_from_name("read_file"), "read-file");
        assert_eq!(slug_from_name("ReadFile"), "readfile");
    }

    #[test]
    fn test_convert_inline_code() {
        assert_eq!(
            convert_inline_code("Use `foo` here"),
            "Use <code>foo</code> here"
        );
    }
}
