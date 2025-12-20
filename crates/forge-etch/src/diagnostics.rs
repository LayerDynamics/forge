//! Error types and diagnostics
//!
//! This module provides error handling and diagnostic reporting
//! for the documentation generator.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for forge-etch operations
pub type EtchResult<T> = Result<T, EtchError>;

/// Main error type for forge-etch
#[derive(Debug, Error)]
pub enum EtchError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error
    #[error("Parse error in {file}: {message}")]
    Parse {
        file: PathBuf,
        message: String,
        line: Option<usize>,
        col: Option<usize>,
    },

    /// TypeScript parse error
    #[error("TypeScript parse error: {0}")]
    TypeScriptParse(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Module not found
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    /// Symbol not found
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Build error
    #[error("Build error: {0}")]
    Build(String),

    /// Environment variable missing
    #[error("Environment variable not set: {0}")]
    EnvVarMissing(String),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl EtchError {
    /// Create a parse error
    pub fn parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        EtchError::Parse {
            file: file.into(),
            message: message.into(),
            line: None,
            col: None,
        }
    }

    /// Create a parse error with location
    pub fn parse_at(
        file: impl Into<PathBuf>,
        message: impl Into<String>,
        line: usize,
        col: usize,
    ) -> Self {
        EtchError::Parse {
            file: file.into(),
            message: message.into(),
            line: Some(line),
            col: Some(col),
        }
    }

    /// Create a config error
    pub fn config(message: impl Into<String>) -> Self {
        EtchError::Config(message.into())
    }

    /// Create a build error
    pub fn build(message: impl Into<String>) -> Self {
        EtchError::Build(message.into())
    }

    /// Create a generic error
    pub fn other(message: impl Into<String>) -> Self {
        EtchError::Other(message.into())
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// Error - prevents doc generation
    Error,
    /// Warning - doc generation continues
    Warning,
    /// Info - informational message
    Info,
    /// Hint - suggestion for improvement
    Hint,
}

impl DiagnosticSeverity {
    /// Get display string
    pub fn display(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Hint => "hint",
        }
    }

    /// Get ANSI color code
    pub fn color(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "\x1b[31m",   // Red
            DiagnosticSeverity::Warning => "\x1b[33m", // Yellow
            DiagnosticSeverity::Info => "\x1b[34m",    // Blue
            DiagnosticSeverity::Hint => "\x1b[36m",    // Cyan
        }
    }
}

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Message
    pub message: String,
    /// Source file
    pub file: Option<PathBuf>,
    /// Line number (1-indexed)
    pub line: Option<usize>,
    /// Column number (0-indexed)
    pub col: Option<usize>,
    /// Diagnostic code (for categorization)
    pub code: Option<String>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(severity: DiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            file: None,
            line: None,
            col: None,
            code: None,
        }
    }

    /// Create an error diagnostic
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Error, message)
    }

    /// Create a warning diagnostic
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Warning, message)
    }

    /// Create an info diagnostic
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Info, message)
    }

    /// Create a hint diagnostic
    pub fn hint(message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Hint, message)
    }

    /// Set the source file
    pub fn in_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the location
    pub fn at(mut self, line: usize, col: usize) -> Self {
        self.line = Some(line);
        self.col = Some(col);
        self
    }

    /// Set the diagnostic code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Format the diagnostic for display
    pub fn format(&self) -> String {
        let mut result = String::new();

        // Location prefix
        if let Some(ref file) = self.file {
            result.push_str(&file.display().to_string());
            if let Some(line) = self.line {
                result.push(':');
                result.push_str(&line.to_string());
                if let Some(col) = self.col {
                    result.push(':');
                    result.push_str(&col.to_string());
                }
            }
            result.push_str(": ");
        }

        // Severity
        result.push_str(self.severity.display());

        // Code
        if let Some(ref code) = self.code {
            result.push('[');
            result.push_str(code);
            result.push(']');
        }

        result.push_str(": ");
        result.push_str(&self.message);

        result
    }

    /// Format with ANSI colors
    pub fn format_colored(&self) -> String {
        let mut result = String::new();
        let reset = "\x1b[0m";

        // Location prefix (dim)
        if let Some(ref file) = self.file {
            result.push_str("\x1b[2m");
            result.push_str(&file.display().to_string());
            if let Some(line) = self.line {
                result.push(':');
                result.push_str(&line.to_string());
                if let Some(col) = self.col {
                    result.push(':');
                    result.push_str(&col.to_string());
                }
            }
            result.push_str(reset);
            result.push_str(": ");
        }

        // Severity (colored)
        result.push_str(self.severity.color());
        result.push_str(self.severity.display());
        result.push_str(reset);

        // Code
        if let Some(ref code) = self.code {
            result.push_str("\x1b[2m[");
            result.push_str(code);
            result.push_str("]\x1b[0m");
        }

        result.push_str(": ");
        result.push_str(&self.message);

        result
    }
}

/// Collector for diagnostics during doc generation
#[derive(Debug, Default)]
pub struct DiagnosticsCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::error(message));
    }

    /// Add a warning
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::warning(message));
    }

    /// Add an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::info(message));
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    /// Print all diagnostics to stderr
    pub fn print(&self) {
        for diagnostic in &self.diagnostics {
            eprintln!("{}", diagnostic.format_colored());
        }
    }

    /// Print summary
    pub fn print_summary(&self) {
        let errors = self.error_count();
        let warnings = self.warning_count();

        if errors > 0 || warnings > 0 {
            eprintln!("\n{} error(s), {} warning(s)", errors, warnings);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etch_error() {
        let err = EtchError::parse("test.ts", "unexpected token");
        assert!(err.to_string().contains("test.ts"));
        assert!(err.to_string().contains("unexpected token"));
    }

    #[test]
    fn test_diagnostic() {
        let diag = Diagnostic::error("missing semicolon")
            .in_file("test.ts")
            .at(10, 5)
            .with_code("E001");

        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert!(diag.format().contains("test.ts:10:5"));
        assert!(diag.format().contains("error"));
        assert!(diag.format().contains("E001"));
    }

    #[test]
    fn test_diagnostics_collector() {
        let mut collector = DiagnosticsCollector::new();
        collector.error("error 1");
        collector.warning("warning 1");
        collector.info("info 1");

        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
        assert_eq!(collector.diagnostics().len(), 3);
    }
}
