//! TypeScript transpilation utilities
//!
//! Provides functions to transpile TypeScript to JavaScript using deno_ast.

use deno_ast::{
    EmitOptions, MediaType, ParseParams, SourceMapOption, TranspileModuleOptions, TranspileOptions,
};
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
/// ```
/// use forge_weld::transpile_ts;
///
/// let js = transpile_ts(
///     "const x: string = 'hello';",
///     "file:///test.ts"
/// ).unwrap();
/// assert!(js.contains("const x = 'hello'"));
/// ```
pub fn transpile_ts(ts_code: &str, specifier: &str) -> Result<String, TranspileError> {
    Ok(transpile_ts_with(ts_code, specifier, &TranspileSettings::default())?.code)
}

/// Options controlling [`transpile_ts_with`].
#[derive(Debug, Clone, Default)]
pub struct TranspileSettings {
    /// Emit a separate source map mapping the generated JS back to the original
    /// TypeScript. The map is returned in [`TranspileOutput::source_map`].
    pub source_map: bool,
    /// Minify the generated JavaScript (collapse whitespace, drop comments).
    pub minify: bool,
}

/// Result of [`transpile_ts_with`].
#[derive(Debug, Clone)]
pub struct TranspileOutput {
    /// The transpiled (and optionally minified) JavaScript code.
    pub code: String,
    /// The source map JSON, present only when [`TranspileSettings::source_map`]
    /// was set. When `minify` is also set, the map describes the unminified
    /// TS→JS transpile (minification is applied as a subsequent pass).
    pub source_map: Option<String>,
}

/// Transpile TypeScript to JavaScript, honoring source-map and minify settings.
///
/// `source_map` produces a real separate source map via deno_ast; `minify` runs
/// the generated JS back through the swc code generator with minification
/// enabled (the deno_ast transpile path does not expose minification directly).
///
/// # Arguments
/// * `ts_code` - The TypeScript source code
/// * `specifier` - A file URL specifier (e.g., "file:///init.ts")
/// * `settings` - Which extra outputs to produce
pub fn transpile_ts_with(
    ts_code: &str,
    specifier: &str,
    settings: &TranspileSettings,
) -> Result<TranspileOutput, TranspileError> {
    let module_specifier = to_module_specifier(specifier)?;

    let parsed = deno_ast::parse_module(ParseParams {
        specifier: module_specifier.clone(),
        text: ts_code.into(),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(|e| TranspileError::ParseError(e.to_string()))?;

    let emit_options = EmitOptions {
        source_map: if settings.source_map {
            SourceMapOption::Separate
        } else {
            SourceMapOption::None
        },
        // Dropping comments is part of minification, but only strip them in this
        // (mapped) transpile when no source map is requested. When a map is
        // produced it must describe a faithful unminified transpile, so we keep
        // the comments here; the later minify pass drops them in the final code.
        remove_comments: settings.minify && !settings.source_map,
        ..Default::default()
    };

    let emitted = parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &emit_options,
        )
        .map_err(|e| TranspileError::TranspileError(e.to_string()))?
        .into_source();

    let code = if settings.minify {
        minify_js(&emitted.text, &module_specifier)?
    } else {
        emitted.text
    };

    Ok(TranspileOutput {
        code,
        source_map: emitted.source_map,
    })
}

/// Normalize a caller-provided source name into a module specifier deno_ast
/// will accept.
///
/// Accepts a fully-qualified URL (`file:///x.ts`, `https://…`) as-is. Anything
/// else is treated as a filesystem path and normalized via
/// [`ModuleSpecifier::from_file_path`] (resolving relative paths against the
/// current directory), so platform path forms — including Windows drive paths
/// like `C:\foo.ts` — become correct `file://` URLs instead of being mangled by
/// naive string prefixing.
fn to_module_specifier(name: &str) -> Result<deno_ast::ModuleSpecifier, TranspileError> {
    // A "scheme://authority" form is a real URL. (A bare Windows path such as
    // `C:\foo.ts` has no `://`, so it correctly falls through to from_file_path
    // rather than being misread as a `c:` scheme.)
    if name.contains("://") {
        return deno_ast::ModuleSpecifier::parse(name)
            .map_err(|e| TranspileError::ParseError(e.to_string()));
    }

    let path = Path::new(name);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    deno_ast::ModuleSpecifier::from_file_path(&absolute)
        .map_err(|()| TranspileError::ParseError(format!("invalid source path: {name}")))
}

/// Re-emit already-transpiled JavaScript with the swc code generator's
/// minification enabled. deno_ast's `swc_codegen_config()` hardcodes
/// `minify = false`, so we mirror its emitter here with `minify = true`.
fn minify_js(
    js_code: &str,
    specifier: &deno_ast::ModuleSpecifier,
) -> Result<String, TranspileError> {
    use deno_ast::swc::codegen::text_writer::JsWriter;
    use deno_ast::swc::codegen::{Emitter, Node};

    let parsed = deno_ast::parse_module(ParseParams {
        specifier: specifier.clone(),
        text: js_code.into(),
        media_type: MediaType::JavaScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(|e| TranspileError::ParseError(e.to_string()))?;

    // A fresh single-file source map paired with the freshly parsed program,
    // matching how deno_ast itself sets up emit (parse → SourceMap::single → emit).
    let source_map = deno_ast::SourceMap::single(specifier.clone(), js_code.to_string());
    let cm = source_map.inner().clone();

    let mut cfg = deno_ast::swc_codegen_config();
    cfg.minify = true;

    let mut buf = Vec::new();
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg,
            comments: None,
            cm,
            wr: writer,
        };
        match parsed.program_ref() {
            deno_ast::ProgramRef::Module(module) => module.emit_with(&mut emitter),
            deno_ast::ProgramRef::Script(script) => script.emit_with(&mut emitter),
        }
        .map_err(|e| TranspileError::TranspileError(e.to_string()))?;
    }

    String::from_utf8(buf).map_err(|e| TranspileError::TranspileError(e.to_string()))
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
/// ```no_run
/// use forge_weld::transpile_file;
///
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
        // Type annotation should be stripped, quotes may be single or double
        assert!(js.contains("const x ="));
        assert!(js.contains("hello"));
        assert!(!js.contains(": string"));
    }

    #[test]
    fn test_transpile_async() {
        let ts = "async function foo(): Promise<string> { return 'bar'; }";
        let js = transpile_ts(ts, "file:///test.ts").unwrap();
        // Should have async and function name, return type stripped
        assert!(js.contains("async"));
        assert!(js.contains("foo"));
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
        assert!(!js.contains("interface Foo"));
        // Check that const x is there with the object (quotes may vary)
        assert!(js.contains("const x"));
        assert!(js.contains("baz"));
    }

    #[test]
    fn test_transpile_builder() {
        let js = TranspileBuilder::new("const x: number = 42;")
            .specifier("file:///custom.ts")
            .transpile()
            .unwrap();

        assert!(js.contains("const x"));
        assert!(js.contains("42"));
    }

    // M2 regression: source_map and minify options used to be accepted and
    // silently ignored (the op hardcoded `source_map: None`).

    #[test]
    fn transpile_with_source_map_produces_real_map() {
        let ts = "const x: string = 'hello';";
        let out = transpile_ts_with(
            ts,
            "file:///test.ts",
            &TranspileSettings {
                source_map: true,
                minify: false,
            },
        )
        .unwrap();
        let map = out.source_map.expect("a source map should be produced");
        // A real source map is JSON containing a "mappings" field.
        assert!(map.contains("\"mappings\""), "map was: {map}");
        assert!(map.contains("\"version\""));
    }

    #[test]
    fn transpile_without_source_map_omits_it() {
        let out = transpile_ts_with(
            "const x: number = 1;",
            "file:///test.ts",
            &TranspileSettings::default(),
        )
        .unwrap();
        assert!(out.source_map.is_none());
    }

    #[test]
    fn transpile_with_minify_collapses_output() {
        // Multi-line/indented source so minification visibly collapses it.
        let ts = r#"
            function greet(name: string): string {
                const greeting = "hello";
                return greeting + ", " + name;
            }
        "#;
        let plain = transpile_ts_with(ts, "file:///m.ts", &TranspileSettings::default())
            .unwrap()
            .code;
        let minified = transpile_ts_with(
            ts,
            "file:///m.ts",
            &TranspileSettings {
                source_map: false,
                minify: true,
            },
        )
        .unwrap()
        .code;

        // Minified output is shorter and collapses the original newlines/indent.
        assert!(
            minified.len() < plain.len(),
            "minified ({}) not shorter than plain ({})",
            minified.len(),
            plain.len()
        );
        assert!(minified.matches('\n').count() < plain.matches('\n').count());
        // Behavior is preserved: the identifiers/strings survive.
        assert!(minified.contains("greet"));
        assert!(minified.contains("hello"));
    }

    #[test]
    fn transpile_ts_wrapper_still_returns_plain_code() {
        // The thin wrapper must keep its original behavior (no map, no minify).
        let js = transpile_ts("const x: string = 'hi';", "file:///w.ts").unwrap();
        assert!(js.contains("const x"));
        assert!(!js.contains(": string"));
    }

    #[test]
    fn transpile_with_source_map_and_minify_together() {
        // Both options at once: a map is still produced and the code is minified.
        let ts = "// a comment\nfunction f(n: number): number {\n  return n + 1;\n}\n";
        let plain = transpile_ts_with(ts, "file:///b.ts", &TranspileSettings::default())
            .unwrap()
            .code;
        let out = transpile_ts_with(
            ts,
            "file:///b.ts",
            &TranspileSettings {
                source_map: true,
                minify: true,
            },
        )
        .unwrap();
        assert!(out.source_map.is_some(), "map should still be produced");
        assert!(out.code.len() < plain.len(), "code should be minified");
        assert!(out.code.contains("function f") || out.code.contains("f("));
    }

    #[test]
    fn specifier_passes_through_full_url() {
        let spec = to_module_specifier("file:///already/qualified.ts").unwrap();
        assert_eq!(spec.as_str(), "file:///already/qualified.ts");
    }

    #[test]
    fn specifier_resolves_bare_name_to_absolute_file_url() {
        // A bare name must become an absolute file:// URL (deno_ast rejects
        // relative specifiers), without naive string prefixing.
        let spec = to_module_specifier("input.ts").unwrap();
        assert_eq!(spec.scheme(), "file");
        assert!(spec.as_str().ends_with("input.ts"));
    }

    // The Windows drive-path normalization can only be exercised on Windows,
    // where `C:\foo.ts` is absolute; from_file_path yields a proper file URL
    // instead of the mangled `file:///C:\foo.ts` naive prefixing would produce.
    #[cfg(target_os = "windows")]
    #[test]
    fn specifier_handles_windows_drive_path() {
        let spec = to_module_specifier(r"C:\foo\bar.ts").unwrap();
        assert_eq!(spec.scheme(), "file");
        assert!(spec.as_str().contains("C:/foo/bar.ts"), "got: {spec}");
        assert!(!spec.as_str().contains('\\'));
    }
}
