use std::path::PathBuf;

/// Configuration for schema generation
///
/// Controls the behavior of JSON Schema and OpenAPI generation for Forge extensions.
/// All options have sensible defaults via the `Default` impl.
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    /// Output directory for generated schemas (relative to workspace root)
    ///
    /// Default: `sdk/schemas`
    pub output_dir: PathBuf,

    /// Schema formats to generate
    ///
    /// Default: `[SchemaFormat::JsonSchema]`
    pub formats: Vec<SchemaFormat>,

    /// Include example values in generated schemas
    ///
    /// When enabled, schemas will include example values for struct fields
    /// and operation parameters where available.
    ///
    /// Default: `true`
    pub include_examples: bool,

    /// Add version suffix to output filenames
    ///
    /// When enabled, generates filenames like `runtime.fs.v1.schema.json`
    /// instead of `runtime.fs.schema.json`.
    ///
    /// Default: `false`
    pub versioned: bool,

    /// Base URL for schema `$id` fields
    ///
    /// Used in JSON Schema `$id` field. If `None`, uses a default forge.dev URL.
    ///
    /// Default: `None`
    pub schema_base_url: Option<String>,

    /// Fail the build if schema generation fails
    ///
    /// When `true` (default), schema generation errors cause the build to fail.
    /// When `false`, errors are emitted as cargo warnings and the build continues.
    ///
    /// Default: `true` (catch schema errors during development)
    pub fail_on_error: bool,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("sdk/schemas"),
            formats: vec![SchemaFormat::JsonSchema],
            include_examples: true,
            versioned: false,
            schema_base_url: None,
            fail_on_error: true,
        }
    }
}

/// Schema output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaFormat {
    /// JSON Schema Draft 2020-12
    ///
    /// Generates a `.schema.json` file conforming to JSON Schema 2020-12.
    /// This is the current JSON Schema standard and is compatible with OpenAPI 3.1.
    JsonSchema,

    /// OpenAPI 3.1.0 specification
    ///
    /// Generates a `.openapi.json` file conforming to OpenAPI 3.1.0.
    /// OpenAPI 3.1 uses JSON Schema 2020-12 for its schema objects.
    OpenApi,

    /// TypeScript SDK class
    ///
    /// Note: This format is handled separately via `SdkClassGenerator`.
    /// Including it here allows future integration with schema generation pipeline.
    TypeScriptSdk,
}

impl SchemaFormat {
    /// Get the file extension for this format
    pub fn file_extension(&self) -> &'static str {
        match self {
            SchemaFormat::JsonSchema => "schema.json",
            SchemaFormat::OpenApi => "openapi.json",
            SchemaFormat::TypeScriptSdk => "ts",
        }
    }

    /// Get a human-readable name for this format
    pub fn display_name(&self) -> &'static str {
        match self {
            SchemaFormat::JsonSchema => "JSON Schema 2020-12",
            SchemaFormat::OpenApi => "OpenAPI 3.1.0",
            SchemaFormat::TypeScriptSdk => "TypeScript SDK",
        }
    }
}

/// Errors that can occur during schema generation
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// I/O error while writing schema files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid or unsupported type encountered
    #[error("Invalid type: {0}")]
    InvalidType(String),

    /// Unsupported feature requested
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_config_default() {
        let config = SchemaConfig::default();
        assert_eq!(config.output_dir, PathBuf::from("sdk/schemas"));
        assert_eq!(config.formats.len(), 1);
        assert!(matches!(config.formats[0], SchemaFormat::JsonSchema));
        assert!(config.include_examples);
        assert!(!config.versioned);
        assert!(config.schema_base_url.is_none());
        assert!(!config.fail_on_error);
    }

    #[test]
    fn test_schema_format_extensions() {
        assert_eq!(SchemaFormat::JsonSchema.file_extension(), "schema.json");
        assert_eq!(SchemaFormat::OpenApi.file_extension(), "openapi.json");
        assert_eq!(SchemaFormat::TypeScriptSdk.file_extension(), "ts");
    }

    #[test]
    fn test_schema_format_display() {
        assert_eq!(SchemaFormat::JsonSchema.display_name(), "JSON Schema 2020-12");
        assert_eq!(SchemaFormat::OpenApi.display_name(), "OpenAPI 3.1.0");
        assert_eq!(SchemaFormat::TypeScriptSdk.display_name(), "TypeScript SDK");
    }
}
