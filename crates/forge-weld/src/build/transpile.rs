//! TypeScript transpilation utilities
//!
//! Provides functions to transpile TypeScript to JavaScript using deno_ast.

use deno_ast::{EmitOptions, MediaType, ParseParams, TranspileModuleOptions, TranspileOptions};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during transpilation
#[derive(Debug, Error)]
pub enum TranspileError {
    /// Failed to read the source file
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse TypeScript
    #[error("Failed to parse TypeScript: {0}")]
    ParseError(String),

    /// Failed to transpile TypeScript
    #[error("Failed to transpile TypeScript: {0}")]
    TranspileError(String),
}

/// Transpile TypeScript source code to JavaScript
///
/// # Arguments
/// * `ts_code` - The TypeScript source code
/// * `specifier` - A file URL specifier for error messages (e.g., "file:///init.ts")
///
/// # Returns
/// The transpiled JavaScript code
///
/// # Example
/// ```ignore
/// let js = transpile_ts(
///     "const x: string = 'hello';",
///     "file:///test.ts"
/// ).unwrap();
/// assert!(js.contains("const x = 'hello'"));
/// ```
pub fn transpile_ts(ts_code: &str, specifier: &str) -> Result<String, TranspileError> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: deno_ast::ModuleSpecifier::parse(specifier)
            .map_err(|e| TranspileError::ParseError(e.to_string()))?,
        text: ts_code.into(),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(|e| TranspileError::ParseError(e.to_string()))?;

    let transpile_result = parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &EmitOptions::default(),
        )
        .map_err(|e| TranspileError::TranspileError(e.to_string()))?;

    Ok(transpile_result.into_source().text)
}

/// Transpile a TypeScript file to JavaScript
///
/// # Arguments
/// * `path` - Path to the TypeScript file
///
/// # Returns
/// The transpiled JavaScript code
///
/// # Example
/// ```ignore
/// let js = transpile_file("ts/init.ts").unwrap();
/// ```
pub fn transpile_file(path: impl AsRef<Path>) -> Result<String, TranspileError> {
    let path = path.as_ref();
    let ts_code = fs::read_to_string(path)?;

    // Create a file:// URL specifier from the path
    let specifier = format!(
        "file:///{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("source.ts")
    );

    transpile_ts(&ts_code, &specifier)
}

/// Transpile TypeScript with custom options
pub struct TranspileBuilder {
    source: String,
    specifier: String,
    jsx: bool,
    jsx_automatic: bool,
}

impl TranspileBuilder {
    /// Create a new transpile builder with source code
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            specifier: "file:///source.ts".to_string(),
            jsx: false,
            jsx_automatic: false,
        }
    }

    /// Create a new transpile builder from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, TranspileError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)?;
        let specifier = format!(
            "file:///{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("source.ts")
        );

        Ok(Self {
            source,
            specifier,
            jsx: false,
            jsx_automatic: false,
        })
    }

    /// Set the module specifier
    pub fn specifier(mut self, specifier: impl Into<String>) -> Self {
        self.specifier = specifier.into();
        self
    }

    /// Enable JSX support (for .tsx files)
    pub fn jsx(mut self) -> Self {
        self.jsx = true;
        self
    }

    /// Use automatic JSX runtime (React 17+)
    pub fn jsx_automatic(mut self) -> Self {
        self.jsx = true;
        self.jsx_automatic = true;
        self
    }

    /// Transpile the source
    pub fn transpile(self) -> Result<String, TranspileError> {
        let media_type = if self.jsx {
            MediaType::Tsx
        } else {
            MediaType::TypeScript
        };

        let parsed = deno_ast::parse_module(ParseParams {
            specifier: deno_ast::ModuleSpecifier::parse(&self.specifier)
                .map_err(|e| TranspileError::ParseError(e.to_string()))?,
            text: self.source.into(),
            media_type,
            capture_tokens: false,
            scope_analysis: false,
            maybe_syntax: None,
        })
        .map_err(|e| TranspileError::ParseError(e.to_string()))?;

        let transpile_result = parsed
            .transpile(
                &TranspileOptions::default(),
                &TranspileModuleOptions::default(),
                &EmitOptions::default(),
            )
            .map_err(|e| TranspileError::TranspileError(e.to_string()))?;

        Ok(transpile_result.into_source().text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_simple() {
        let ts = "const x: string = 'hello';";
        let js = transpile_ts(ts, "file:///test.ts").unwrap();
        assert!(js.contains("const x = 'hello'"));
        assert!(!js.contains(": string"));
    }

    #[test]
    fn test_transpile_async() {
        let ts = "async function foo(): Promise<string> { return 'bar'; }";
        let js = transpile_ts(ts, "file:///test.ts").unwrap();
        assert!(js.contains("async function foo()"));
        assert!(!js.contains("Promise<string>"));
    }

    #[test]
    fn test_transpile_interface() {
        let ts = r#"
            interface Foo { bar: string; }
            const x: Foo = { bar: 'baz' };
        "#;
        let js = transpile_ts(ts, "file:///test.ts").unwrap();
        // Interfaces should be stripped
        assert!(!js.contains("interface"));
        assert!(js.contains("const x = { bar: 'baz' }"));
    }

    #[test]
    fn test_transpile_builder() {
        let js = TranspileBuilder::new("const x: number = 42;")
            .specifier("file:///custom.ts")
            .transpile()
            .unwrap();

        assert!(js.contains("const x = 42"));
    }
}
