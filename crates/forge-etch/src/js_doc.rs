//! JSDoc parsing and representation
//!
//! This module provides types and parsing logic for JSDoc comments.
//! It extracts structured documentation from TypeScript/JavaScript
//! doc comments including tags like @param, @returns, @example, etc.

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// Regex for parsing JSDoc tags
    static ref TAG_REGEX: Regex = Regex::new(
        r"@(\w+)(?:\s+\{([^}]*)\})?\s*(?:(\[)?(\w+)(?:=([^\]]+))?\]?)?(?:\s*-?\s*)?(.*)"
    ).unwrap();

    /// Regex for {@link ...} references
    static ref LINK_REGEX: Regex = Regex::new(
        r"\{@(link|linkcode|linkplain)\s+([^}]+)\}"
    ).unwrap();

    /// Regex to extract the main description before tags
    /// Captures everything up to (but not including) a newline followed by whitespace and @
    static ref DESC_REGEX: Regex = Regex::new(
        r"(?s)^(.*?)(?:\n\s*@|$)"
    ).unwrap();
}

/// Parsed JSDoc documentation
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtchDoc {
    /// Main description text
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Parsed JSDoc tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<JsDocTag>,
}

impl EtchDoc {
    /// Create a new empty doc
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from description only
    pub fn from_description(desc: impl Into<String>) -> Self {
        Self {
            description: Some(desc.into()),
            tags: Vec::new(),
        }
    }

    /// Parse JSDoc from a comment string
    ///
    /// Handles both single-line `/** ... */` and multi-line JSDoc comments.
    pub fn parse(comment: &str) -> Self {
        // Remove comment delimiters and normalize whitespace
        let cleaned = clean_jsdoc_comment(comment);

        // Extract description (everything before first @tag)
        let description = extract_description(&cleaned);

        // Parse tags
        let tags = parse_tags(&cleaned);

        Self { description, tags }
    }

    /// Check if this doc is empty
    pub fn is_empty(&self) -> bool {
        self.description.is_none() && self.tags.is_empty()
    }

    /// Get the main description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the first sentence of the description (for summaries)
    pub fn summary(&self) -> Option<String> {
        self.description.as_ref().map(|d| {
            // Find first sentence ending
            if let Some(idx) = d.find(". ") {
                d[..=idx].to_string()
            } else if let Some(idx) = d.find(".\n") {
                d[..=idx].to_string()
            } else if d.len() > 150 {
                format!("{}...", &d[..150])
            } else {
                d.clone()
            }
        })
    }

    /// Get a short description (first sentence, as a reference)
    pub fn short_description(&self) -> Option<&str> {
        self.description.as_ref().map(|d| {
            // Find first sentence ending
            if let Some(idx) = d.find(". ") {
                &d[..=idx]
            } else if let Some(idx) = d.find(".\n") {
                &d[..idx]
            } else if d.len() > 80 {
                &d[..80]
            } else {
                d.as_str()
            }
        })
    }

    /// Get all @param tags
    pub fn params(&self) -> impl Iterator<Item = &JsDocTag> {
        self.tags
            .iter()
            .filter(|t| matches!(t, JsDocTag::Param { .. }))
    }

    /// Get param doc by name
    pub fn param(&self, name: &str) -> Option<&JsDocTag> {
        self.params().find(|t| {
            if let JsDocTag::Param { name: n, .. } = t {
                n == name
            } else {
                false
            }
        })
    }

    /// Get @returns tag
    pub fn returns(&self) -> Option<&JsDocTag> {
        self.tags
            .iter()
            .find(|t| matches!(t, JsDocTag::Returns { .. }))
    }

    /// Get all @example tags
    pub fn examples(&self) -> impl Iterator<Item = &JsDocTag> {
        self.tags
            .iter()
            .filter(|t| matches!(t, JsDocTag::Example { .. }))
    }

    /// Get @deprecated tag
    pub fn deprecated(&self) -> Option<&JsDocTag> {
        self.tags
            .iter()
            .find(|t| matches!(t, JsDocTag::Deprecated { .. }))
    }

    /// Check if marked deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated().is_some()
    }

    /// Get @since tag
    pub fn since(&self) -> Option<&str> {
        self.tags.iter().find_map(|t| {
            if let JsDocTag::Since { version } = t {
                Some(version.as_str())
            } else {
                None
            }
        })
    }

    /// Get @category tag
    pub fn category(&self) -> Option<&str> {
        self.tags.iter().find_map(|t| {
            if let JsDocTag::Category { name } = t {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Check if marked @internal
    pub fn is_internal(&self) -> bool {
        self.tags.iter().any(|t| matches!(t, JsDocTag::Internal))
    }

    /// Check if marked @experimental
    pub fn is_experimental(&self) -> bool {
        self.tags
            .iter()
            .any(|t| matches!(t, JsDocTag::Experimental))
    }

    /// Get all @see tags
    pub fn see_also(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().filter_map(|t| {
            if let JsDocTag::See { reference } = t {
                Some(reference.as_str())
            } else {
                None
            }
        })
    }

    /// Merge documentation from multiple sources
    ///
    /// TypeScript JSDoc takes precedence over Rust doc comments.
    pub fn merge(rust_doc: Option<&str>, ts_doc: Option<&EtchDoc>) -> Self {
        match (rust_doc, ts_doc) {
            (_, Some(ts)) if !ts.is_empty() => ts.clone(),
            (Some(rust), _) => Self::from_description(rust.trim()),
            _ => Self::default(),
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: JsDocTag) {
        self.tags.push(tag);
    }

    /// Set description
    pub fn set_description(&mut self, desc: impl Into<String>) {
        self.description = Some(desc.into());
    }
}

/// JSDoc tag types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum JsDocTag {
    /// @param {type} name - description
    #[serde(rename_all = "camelCase")]
    Param {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        type_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
        #[serde(default)]
        optional: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String>,
    },

    /// @returns {type} description
    #[serde(rename_all = "camelCase")]
    Returns {
        #[serde(skip_serializing_if = "Option::is_none")]
        type_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },

    /// @example
    #[serde(rename_all = "camelCase")]
    Example {
        doc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },

    /// @deprecated message
    #[serde(rename_all = "camelCase")]
    Deprecated {
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },

    /// @see reference
    #[serde(rename_all = "camelCase")]
    See { reference: String },

    /// @since version
    #[serde(rename_all = "camelCase")]
    Since { version: String },

    /// @category name
    #[serde(rename_all = "camelCase")]
    Category { name: String },

    /// @internal
    Internal,

    /// @experimental
    Experimental,

    /// @throws {type} description
    #[serde(rename_all = "camelCase")]
    Throws {
        #[serde(skip_serializing_if = "Option::is_none")]
        type_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },

    /// @template T - description
    #[serde(rename_all = "camelCase")]
    Template {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        constraint: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        doc: Option<String>,
    },

    /// @typedef {type} name
    #[serde(rename_all = "camelCase")]
    TypeDef {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        type_ref: Option<String>,
    },

    /// @callback name
    #[serde(rename_all = "camelCase")]
    Callback { name: String },

    /// @type {type}
    #[serde(rename_all = "camelCase")]
    Type { type_ref: String },

    /// @default value
    #[serde(rename_all = "camelCase")]
    Default { value: String },

    /// @readonly
    Readonly,

    /// @override
    Override,

    /// @abstract
    Abstract,

    /// @virtual
    Virtual,

    /// @public
    Public,

    /// @private
    Private,

    /// @protected
    Protected,

    /// @module name
    #[serde(rename_all = "camelCase")]
    Module { name: String },

    /// Unknown/custom tag
    #[serde(rename_all = "camelCase")]
    Unknown { tag: String, value: String },
}

/// Clean JSDoc comment by removing delimiters and normalizing whitespace
fn clean_jsdoc_comment(comment: &str) -> String {
    let mut result = String::new();

    for line in comment.lines() {
        let trimmed = line.trim();

        // Skip opening/closing delimiters
        if trimmed == "/**" || trimmed == "*/" {
            continue;
        }

        // Start by removing leading * and /** markers
        let mut content = trimmed;

        // Remove leading /** if present
        if content.starts_with("/**") {
            content = content.trim_start_matches("/**").trim_start();
        }

        // Remove leading * and whitespace
        if content.starts_with("* ") {
            content = &content[2..];
        } else if content.starts_with('*') {
            content = &content[1..];
        }

        // Remove trailing */ if present
        if content.ends_with("*/") {
            content = content.trim_end_matches("*/").trim_end();
        }

        if !result.is_empty() && !content.is_empty() {
            result.push('\n');
        }
        result.push_str(content);
    }

    result.trim().to_string()
}

/// Extract description text before the first tag
fn extract_description(text: &str) -> Option<String> {
    // Find first @tag
    let first_tag_pos =
        text.find("\n@")
            .or_else(|| if text.starts_with('@') { Some(0) } else { None });

    let desc = match first_tag_pos {
        Some(0) => return None,
        Some(pos) => &text[..pos],
        None => text,
    };

    let trimmed = desc.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse all JSDoc tags from the comment
fn parse_tags(text: &str) -> Vec<JsDocTag> {
    let mut tags = Vec::new();
    let mut current_tag: Option<(String, String)> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if let Some(after_at) = trimmed.strip_prefix('@') {
            // Save previous tag
            if let Some((tag_name, content)) = current_tag.take() {
                if let Some(tag) = parse_single_tag(&tag_name, &content) {
                    tags.push(tag);
                }
            }

            // Start new tag
            if let Some(space_pos) = after_at.find(' ') {
                let tag_name = after_at[..space_pos].to_string();
                let content = after_at[space_pos + 1..].to_string();
                current_tag = Some((tag_name, content));
            } else {
                let tag_name = after_at.to_string();
                current_tag = Some((tag_name, String::new()));
            }
        } else if let Some((_, ref mut content)) = current_tag {
            // Continue multi-line tag content
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(trimmed);
        }
    }

    // Don't forget the last tag
    if let Some((tag_name, content)) = current_tag {
        if let Some(tag) = parse_single_tag(&tag_name, &content) {
            tags.push(tag);
        }
    }

    tags
}

/// Parse a single JSDoc tag
fn parse_single_tag(tag_name: &str, content: &str) -> Option<JsDocTag> {
    let content = content.trim();

    Some(match tag_name {
        "param" | "arg" | "argument" => parse_param_tag(content),
        "returns" | "return" => parse_returns_tag(content),
        "example" => JsDocTag::Example {
            doc: content.to_string(),
            caption: None,
        },
        "deprecated" => JsDocTag::Deprecated {
            doc: if content.is_empty() {
                None
            } else {
                Some(content.to_string())
            },
        },
        "see" => JsDocTag::See {
            reference: content.to_string(),
        },
        "since" => JsDocTag::Since {
            version: content.to_string(),
        },
        "category" => JsDocTag::Category {
            name: content.to_string(),
        },
        "internal" => JsDocTag::Internal,
        "experimental" | "beta" => JsDocTag::Experimental,
        "throws" | "exception" => parse_throws_tag(content),
        "template" | "typeparam" => parse_template_tag(content),
        "typedef" => parse_typedef_tag(content),
        "callback" => JsDocTag::Callback {
            name: content.to_string(),
        },
        "type" => JsDocTag::Type {
            type_ref: extract_type_from_braces(content)
                .unwrap_or(content)
                .to_string(),
        },
        "default" | "defaultvalue" => JsDocTag::Default {
            value: content.to_string(),
        },
        "readonly" => JsDocTag::Readonly,
        "override" => JsDocTag::Override,
        "abstract" => JsDocTag::Abstract,
        "virtual" => JsDocTag::Virtual,
        "public" => JsDocTag::Public,
        "private" => JsDocTag::Private,
        "protected" => JsDocTag::Protected,
        "module" => JsDocTag::Module {
            name: content.to_string(),
        },
        _ => JsDocTag::Unknown {
            tag: tag_name.to_string(),
            value: content.to_string(),
        },
    })
}

/// Parse @param tag content
fn parse_param_tag(content: &str) -> JsDocTag {
    let (type_ref, rest) = extract_type_and_rest(content);

    // Check for optional [name] or [name=default] syntax
    let (name, optional, default, doc) = if rest.starts_with('[') {
        if let Some(bracket_end) = rest.find(']') {
            let bracket_content = &rest[1..bracket_end];
            let after_bracket = rest[bracket_end + 1..].trim();

            let (name, default) = if let Some(eq_pos) = bracket_content.find('=') {
                (
                    bracket_content[..eq_pos].trim().to_string(),
                    Some(bracket_content[eq_pos + 1..].trim().to_string()),
                )
            } else {
                (bracket_content.trim().to_string(), None)
            };

            let doc = extract_doc_after_name(after_bracket);
            (name, true, default, doc)
        } else {
            (rest.to_string(), false, None, None)
        }
    } else {
        // Regular name followed by description
        let (name, doc) = split_name_and_doc(rest);
        (name, false, None, doc)
    };

    JsDocTag::Param {
        name,
        type_ref,
        doc,
        optional,
        default,
    }
}

/// Parse @returns tag content
fn parse_returns_tag(content: &str) -> JsDocTag {
    let (type_ref, rest) = extract_type_and_rest(content);
    let doc = if rest.is_empty() {
        None
    } else {
        Some(rest.trim_start_matches('-').trim().to_string())
    };

    JsDocTag::Returns { type_ref, doc }
}

/// Parse @throws tag content
fn parse_throws_tag(content: &str) -> JsDocTag {
    let (type_ref, rest) = extract_type_and_rest(content);
    let doc = if rest.is_empty() {
        None
    } else {
        Some(rest.trim_start_matches('-').trim().to_string())
    };

    JsDocTag::Throws { type_ref, doc }
}

/// Parse @template tag content
fn parse_template_tag(content: &str) -> JsDocTag {
    let parts: Vec<&str> = content.splitn(2, '-').collect();
    let name_part = parts[0].trim();
    let doc = parts.get(1).map(|s| s.trim().to_string());

    // Check for constraint: T extends Foo
    let (name, constraint) = if let Some(ext_pos) = name_part.find(" extends ") {
        (
            name_part[..ext_pos].trim().to_string(),
            Some(name_part[ext_pos + 9..].trim().to_string()),
        )
    } else {
        (name_part.to_string(), None)
    };

    JsDocTag::Template {
        name,
        constraint,
        doc,
    }
}

/// Parse @typedef tag content
fn parse_typedef_tag(content: &str) -> JsDocTag {
    let (type_ref, rest) = extract_type_and_rest(content);
    JsDocTag::TypeDef {
        name: rest.trim().to_string(),
        type_ref,
    }
}

/// Extract type from {type} at start of content
fn extract_type_and_rest(content: &str) -> (Option<String>, &str) {
    if content.starts_with('{') {
        if let Some(close_pos) = find_matching_brace(content) {
            let type_str = &content[1..close_pos];
            let rest = content[close_pos + 1..].trim();
            (Some(type_str.to_string()), rest)
        } else {
            (None, content)
        }
    } else {
        (None, content)
    }
}

/// Extract type from {type} anywhere in string
fn extract_type_from_braces(content: &str) -> Option<&str> {
    if let Some(start) = content.find('{') {
        if let Some(end) = content[start..].find('}') {
            return Some(&content[start + 1..start + end]);
        }
    }
    None
}

/// Find matching closing brace, handling nested braces
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split "name - description" or "name description"
fn split_name_and_doc(s: &str) -> (String, Option<String>) {
    let s = s.trim();

    // Try "name - description" first
    if let Some(dash_pos) = s.find(" - ") {
        let name = s[..dash_pos].trim().to_string();
        let doc = s[dash_pos + 3..].trim().to_string();
        return (name, Some(doc));
    }

    // Otherwise split on first whitespace
    if let Some(space_pos) = s.find(char::is_whitespace) {
        let name = s[..space_pos].trim().to_string();
        let doc = s[space_pos..].trim().to_string();
        if doc.is_empty() {
            (name, None)
        } else {
            (name, Some(doc))
        }
    } else {
        (s.to_string(), None)
    }
}

/// Extract description after parameter name
fn extract_doc_after_name(s: &str) -> Option<String> {
    let trimmed = s.trim().trim_start_matches('-').trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_jsdoc() {
        let doc = EtchDoc::parse("/** Hello world */");
        assert_eq!(doc.description(), Some("Hello world"));
        assert!(doc.tags.is_empty());
    }

    #[test]
    fn test_parse_multiline_jsdoc() {
        let doc = EtchDoc::parse(
            r#"/**
             * This is a description.
             * It spans multiple lines.
             */"#,
        );
        assert!(doc.description().unwrap().contains("This is a description"));
        assert!(doc.description().unwrap().contains("multiple lines"));
    }

    #[test]
    fn test_parse_param_tag() {
        let doc = EtchDoc::parse(
            r#"/**
             * Does something.
             * @param path - The file path
             */"#,
        );

        assert_eq!(doc.description(), Some("Does something."));
        let params: Vec<_> = doc.params().collect();
        assert_eq!(params.len(), 1);

        if let JsDocTag::Param { name, doc, .. } = &params[0] {
            assert_eq!(name, "path");
            assert_eq!(doc.as_deref(), Some("The file path"));
        } else {
            panic!("Expected Param tag");
        }
    }

    #[test]
    fn test_parse_typed_param() {
        let doc = EtchDoc::parse("/** @param {string} path - The path */");
        let params: Vec<_> = doc.params().collect();

        if let JsDocTag::Param {
            name,
            type_ref,
            doc,
            ..
        } = &params[0]
        {
            assert_eq!(name, "path");
            assert_eq!(type_ref.as_deref(), Some("string"));
            assert_eq!(doc.as_deref(), Some("The path"));
        }
    }

    #[test]
    fn test_parse_optional_param() {
        let doc = EtchDoc::parse("/** @param {string} [encoding] - Optional encoding */");
        let params: Vec<_> = doc.params().collect();

        if let JsDocTag::Param {
            name,
            optional,
            default,
            ..
        } = &params[0]
        {
            assert_eq!(name, "encoding");
            assert!(*optional);
            assert!(default.is_none());
        }
    }

    #[test]
    fn test_parse_param_with_default() {
        let doc = EtchDoc::parse("/** @param {string} [encoding=utf-8] - The encoding */");
        let params: Vec<_> = doc.params().collect();

        if let JsDocTag::Param {
            name,
            optional,
            default,
            ..
        } = &params[0]
        {
            assert_eq!(name, "encoding");
            assert!(*optional);
            assert_eq!(default.as_deref(), Some("utf-8"));
        }
    }

    #[test]
    fn test_parse_returns() {
        let doc = EtchDoc::parse("/** @returns {Promise<string>} The file contents */");

        if let Some(JsDocTag::Returns { type_ref, doc }) = doc.returns() {
            assert_eq!(type_ref.as_deref(), Some("Promise<string>"));
            assert_eq!(doc.as_deref(), Some("The file contents"));
        } else {
            panic!("Expected Returns tag");
        }
    }

    #[test]
    fn test_parse_example() {
        let doc = EtchDoc::parse(
            r#"/**
             * @example
             * const content = await readFile("test.txt");
             * console.log(content);
             */"#,
        );

        let examples: Vec<_> = doc.examples().collect();
        assert_eq!(examples.len(), 1);

        if let JsDocTag::Example { doc, .. } = &examples[0] {
            assert!(doc.contains("readFile"));
        }
    }

    #[test]
    fn test_parse_deprecated() {
        let doc = EtchDoc::parse("/** @deprecated Use newFunction instead */");
        assert!(doc.is_deprecated());

        if let Some(JsDocTag::Deprecated { doc }) = doc.deprecated() {
            assert_eq!(doc.as_deref(), Some("Use newFunction instead"));
        }
    }

    #[test]
    fn test_parse_template() {
        let doc = EtchDoc::parse("/** @template T extends object - The type parameter */");

        let templates: Vec<_> = doc
            .tags
            .iter()
            .filter(|t| matches!(t, JsDocTag::Template { .. }))
            .collect();

        if let JsDocTag::Template {
            name,
            constraint,
            doc,
        } = &templates[0]
        {
            assert_eq!(name, "T");
            assert_eq!(constraint.as_deref(), Some("object"));
            assert_eq!(doc.as_deref(), Some("The type parameter"));
        }
    }

    #[test]
    fn test_is_empty() {
        assert!(EtchDoc::new().is_empty());
        assert!(!EtchDoc::from_description("hello").is_empty());
    }

    #[test]
    fn test_summary() {
        let doc = EtchDoc::from_description("First sentence. Second sentence.");
        assert_eq!(doc.summary(), Some("First sentence.".to_string()));

        let doc = EtchDoc::from_description("Short text");
        assert_eq!(doc.summary(), Some("Short text".to_string()));
    }

    #[test]
    fn test_merge() {
        let rust_doc = Some("Rust documentation");
        let ts_doc = EtchDoc::from_description("TypeScript documentation");

        // TS takes precedence
        let merged = EtchDoc::merge(rust_doc, Some(&ts_doc));
        assert_eq!(merged.description(), Some("TypeScript documentation"));

        // Falls back to Rust if TS is empty
        let merged = EtchDoc::merge(rust_doc, None);
        assert_eq!(merged.description(), Some("Rust documentation"));
    }
}
