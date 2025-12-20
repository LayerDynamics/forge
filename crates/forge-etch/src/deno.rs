//! Deno-specific utilities for forge-etch documentation generation.
//!
//! This module provides utilities for:
//! - Detecting Deno runtime environment
//! - Generating JSR (JavaScript Registry) import specifiers
//! - Converting between file URLs and filesystem paths
//! - Configuration for Deno-specific output generation

use std::path::{Path, PathBuf};

/// Configuration for Deno-specific output generation.
///
/// This configuration controls how documentation is generated for Deno targets,
/// including import specifier format and compatibility settings.
///
/// # Example
///
/// ```
/// use forge_etch::deno::DenoConfig;
///
/// let config = DenoConfig {
///     use_jsr_imports: true,
///     target_version: Some("2.0.0".to_string()),
///     jsr_scope: Some("@forge".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DenoConfig {
    /// Use JSR specifiers instead of npm/local imports.
    ///
    /// When true, imports will be generated as `jsr:@scope/package@version`
    /// instead of standard ESM imports.
    pub use_jsr_imports: bool,

    /// Target Deno version for compatibility.
    ///
    /// Used to determine which APIs and features are available.
    /// Format: "X.Y.Z" (e.g., "2.0.0")
    pub target_version: Option<String>,

    /// JSR package scope for generated imports.
    ///
    /// The scope prefix for JSR imports (e.g., "@forge" produces `jsr:@forge/...`).
    pub jsr_scope: Option<String>,
}

impl DenoConfig {
    /// Create a new DenoConfig with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a DenoConfig configured for JSR imports.
    ///
    /// # Arguments
    ///
    /// * `scope` - The JSR scope (e.g., "@forge")
    ///
    /// # Example
    ///
    /// ```
    /// use forge_etch::deno::DenoConfig;
    ///
    /// let config = DenoConfig::with_jsr("@forge");
    /// assert!(config.use_jsr_imports);
    /// assert_eq!(config.jsr_scope, Some("@forge".to_string()));
    /// ```
    pub fn with_jsr(scope: &str) -> Self {
        Self {
            use_jsr_imports: true,
            jsr_scope: Some(scope.to_string()),
            target_version: None,
        }
    }

    /// Set the target Deno version.
    pub fn with_version(mut self, version: &str) -> Self {
        self.target_version = Some(version.to_string());
        self
    }

    /// Check if this config targets a specific Deno version or newer.
    ///
    /// Returns `true` if no target version is set (assumes latest).
    pub fn supports_version(&self, min_version: &str) -> bool {
        match &self.target_version {
            Some(target) => compare_versions(target, min_version) >= 0,
            None => true, // No target means we assume latest
        }
    }
}

/// Check if the current process is running in Deno.
///
/// This checks for the `DENO_VERSION` environment variable which Deno sets
/// automatically when running.
///
/// # Example
///
/// ```
/// use forge_etch::deno::is_deno_runtime;
///
/// if is_deno_runtime() {
///     println!("Running in Deno!");
/// }
/// ```
pub fn is_deno_runtime() -> bool {
    std::env::var("DENO_VERSION").is_ok()
}

/// Get the Deno version if running in Deno.
///
/// Returns `None` if not running in Deno or if the version cannot be determined.
///
/// # Example
///
/// ```
/// use forge_etch::deno::deno_version;
///
/// if let Some(version) = deno_version() {
///     println!("Deno version: {}", version);
/// }
/// ```
pub fn deno_version() -> Option<String> {
    std::env::var("DENO_VERSION").ok()
}

/// Convert a `file://` URL to a filesystem path.
///
/// This mimics Deno's `Deno.fromFileUrl()` function.
///
/// # Arguments
///
/// * `url` - A file URL string (e.g., `"file:///home/user/file.ts"`)
///
/// # Returns
///
/// The filesystem path, or `None` if the URL is not a valid file URL.
///
/// # Example
///
/// ```
/// use forge_etch::deno::from_file_url;
///
/// let path = from_file_url("file:///home/user/file.ts");
/// assert_eq!(path, Some(std::path::PathBuf::from("/home/user/file.ts")));
/// ```
pub fn from_file_url(url: &str) -> Option<PathBuf> {
    // Handle file:// URLs
    url.strip_prefix("file://").map(PathBuf::from)
}

/// Convert a filesystem path to a `file://` URL.
///
/// This mimics Deno's `Deno.toFileUrl()` function.
///
/// # Arguments
///
/// * `path` - A filesystem path
///
/// # Returns
///
/// A file URL string.
///
/// # Example
///
/// ```
/// use forge_etch::deno::to_file_url;
/// use std::path::Path;
///
/// let url = to_file_url(Path::new("/home/user/file.ts"));
/// assert_eq!(url, "file:///home/user/file.ts");
/// ```
pub fn to_file_url(path: &Path) -> String {
    #[cfg(windows)]
    {
        // Windows: C:\path\to\file -> file:///C:/path/to/file
        let path_str = path.to_string_lossy().replace('\\', "/");
        format!("file:///{}", path_str)
    }
    #[cfg(not(windows))]
    {
        format!("file://{}", path.display())
    }
}

/// Generate a JSR import specifier.
///
/// JSR (JavaScript Registry) is Deno's native package registry.
/// This generates import specifiers in the format `jsr:@scope/package@version`.
///
/// # Arguments
///
/// * `scope` - The package scope (e.g., "@forge")
/// * `package` - The package name (e.g., "runtime")
/// * `version` - The package version (e.g., "0.1.0")
///
/// # Example
///
/// ```
/// use forge_etch::deno::jsr_import;
///
/// let specifier = jsr_import("@forge", "runtime", "0.1.0");
/// assert_eq!(specifier, "jsr:@forge/runtime@0.1.0");
/// ```
pub fn jsr_import(scope: &str, package: &str, version: &str) -> String {
    format!("jsr:{}/{}@{}", scope, package, version)
}

/// Generate a JSR import specifier without a version constraint.
///
/// # Arguments
///
/// * `scope` - The package scope (e.g., "@forge")
/// * `package` - The package name (e.g., "runtime")
///
/// # Example
///
/// ```
/// use forge_etch::deno::jsr_import_latest;
///
/// let specifier = jsr_import_latest("@forge", "runtime");
/// assert_eq!(specifier, "jsr:@forge/runtime");
/// ```
pub fn jsr_import_latest(scope: &str, package: &str) -> String {
    format!("jsr:{}/{}", scope, package)
}

/// A module import with its specifier and exported symbols.
#[derive(Debug, Clone)]
pub struct ModuleImport {
    /// The module specifier (e.g., "runtime:fs" or "jsr:@forge/fs@0.1")
    pub specifier: String,
    /// The symbols to import from the module
    pub symbols: Vec<String>,
}

impl ModuleImport {
    /// Create a new module import.
    pub fn new(specifier: &str, symbols: Vec<&str>) -> Self {
        Self {
            specifier: specifier.to_string(),
            symbols: symbols.into_iter().map(String::from).collect(),
        }
    }

    /// Format this import as an ESM import statement.
    pub fn to_esm_import(&self) -> String {
        if self.symbols.is_empty() {
            format!("import \"{}\";", self.specifier)
        } else {
            format!(
                "import {{ {} }} from \"{}\";",
                self.symbols.join(", "),
                self.specifier
            )
        }
    }
}

/// Generate Deno-compatible import statements for modules.
///
/// When `config.use_jsr_imports` is true, this converts runtime specifiers
/// to JSR specifiers. Otherwise, it returns standard ESM imports.
///
/// # Arguments
///
/// * `config` - The Deno configuration
/// * `imports` - List of module imports to generate
///
/// # Example
///
/// ```
/// use forge_etch::deno::{DenoConfig, ModuleImport, generate_deno_imports};
///
/// let config = DenoConfig::with_jsr("@forge");
/// let imports = vec![
///     ModuleImport::new("runtime:fs", vec!["readFile", "writeFile"]),
///     ModuleImport::new("runtime:window", vec!["createWindow"]),
/// ];
///
/// let code = generate_deno_imports(&config, &imports);
/// // Generates JSR imports when configured
/// ```
pub fn generate_deno_imports(config: &DenoConfig, imports: &[ModuleImport]) -> String {
    let mut lines = Vec::new();

    for import in imports {
        let specifier = if config.use_jsr_imports {
            convert_to_jsr_specifier(config, &import.specifier)
        } else {
            import.specifier.clone()
        };

        let import_stmt = if import.symbols.is_empty() {
            format!("import \"{}\";", specifier)
        } else {
            format!(
                "import {{ {} }} from \"{}\";",
                import.symbols.join(", "),
                specifier
            )
        };

        lines.push(import_stmt);
    }

    lines.join("\n")
}

/// Convert a runtime specifier to a JSR specifier.
///
/// Converts specifiers like `runtime:fs` to `jsr:@scope/fs@version`.
fn convert_to_jsr_specifier(config: &DenoConfig, specifier: &str) -> String {
    if let Some(module_name) = specifier.strip_prefix("runtime:") {
        let scope = config.jsr_scope.as_deref().unwrap_or("@forge");
        // JSR imports without version constraint
        format!("jsr:{}/{}", scope, module_name)
    } else {
        // Not a runtime specifier, return as-is
        specifier.to_string()
    }
}

/// Compare two semver version strings.
///
/// Returns:
/// - Positive if `a > b`
/// - Zero if `a == b`
/// - Negative if `a < b`
fn compare_versions(a: &str, b: &str) -> i32 {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let (a_major, a_minor, a_patch) = parse_version(a);
    let (b_major, b_minor, b_patch) = parse_version(b);

    if a_major != b_major {
        return (a_major as i32) - (b_major as i32);
    }
    if a_minor != b_minor {
        return (a_minor as i32) - (b_minor as i32);
    }
    (a_patch as i32) - (b_patch as i32)
}

/// Common Deno standard library modules and their JSR mappings.
///
/// These are frequently used in Forge applications and can be auto-imported.
pub mod std_modules {
    /// Standard library modules available via JSR
    pub const STD_SCOPE: &str = "@std";

    /// Common std modules
    pub const PATH: &str = "path";
    pub const FS: &str = "fs";
    pub const ASYNC: &str = "async";
    pub const TESTING: &str = "testing";
    pub const ASSERT: &str = "assert";
    pub const FMT: &str = "fmt";

    /// Generate a std import specifier
    pub fn std_import(module: &str) -> String {
        format!("jsr:{}/{}", STD_SCOPE, module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deno_config_default() {
        let config = DenoConfig::default();
        assert!(!config.use_jsr_imports);
        assert!(config.target_version.is_none());
        assert!(config.jsr_scope.is_none());
    }

    #[test]
    fn test_deno_config_with_jsr() {
        let config = DenoConfig::with_jsr("@forge");
        assert!(config.use_jsr_imports);
        assert_eq!(config.jsr_scope, Some("@forge".to_string()));
    }

    #[test]
    fn test_deno_config_supports_version() {
        let config = DenoConfig::new().with_version("2.0.0");
        assert!(config.supports_version("1.0.0"));
        assert!(config.supports_version("2.0.0"));
        assert!(!config.supports_version("2.1.0"));
        assert!(!config.supports_version("3.0.0"));
    }

    #[test]
    fn test_from_file_url() {
        assert_eq!(
            from_file_url("file:///home/user/file.ts"),
            Some(PathBuf::from("/home/user/file.ts"))
        );
        assert_eq!(from_file_url("https://example.com"), None);
        assert_eq!(from_file_url("not-a-url"), None);
    }

    #[test]
    fn test_to_file_url() {
        let path = Path::new("/home/user/file.ts");
        assert_eq!(to_file_url(path), "file:///home/user/file.ts");
    }

    #[test]
    fn test_jsr_import() {
        assert_eq!(
            jsr_import("@forge", "runtime", "0.1.0"),
            "jsr:@forge/runtime@0.1.0"
        );
    }

    #[test]
    fn test_jsr_import_latest() {
        assert_eq!(jsr_import_latest("@forge", "runtime"), "jsr:@forge/runtime");
    }

    #[test]
    fn test_module_import_to_esm() {
        let import = ModuleImport::new("runtime:fs", vec!["readFile", "writeFile"]);
        assert_eq!(
            import.to_esm_import(),
            "import { readFile, writeFile } from \"runtime:fs\";"
        );

        let side_effect = ModuleImport::new("./polyfill.js", vec![]);
        assert_eq!(side_effect.to_esm_import(), "import \"./polyfill.js\";");
    }

    #[test]
    fn test_generate_deno_imports_esm() {
        let config = DenoConfig::default();
        let imports = vec![
            ModuleImport::new("runtime:fs", vec!["readFile"]),
            ModuleImport::new("runtime:window", vec!["createWindow"]),
        ];

        let result = generate_deno_imports(&config, &imports);
        assert!(result.contains("import { readFile } from \"runtime:fs\";"));
        assert!(result.contains("import { createWindow } from \"runtime:window\";"));
    }

    #[test]
    fn test_generate_deno_imports_jsr() {
        let config = DenoConfig::with_jsr("@forge");
        let imports = vec![ModuleImport::new("runtime:fs", vec!["readFile"])];

        let result = generate_deno_imports(&config, &imports);
        assert!(result.contains("import { readFile } from \"jsr:@forge/fs\";"));
    }

    #[test]
    fn test_convert_to_jsr_specifier() {
        let config = DenoConfig::with_jsr("@forge");

        assert_eq!(
            convert_to_jsr_specifier(&config, "runtime:fs"),
            "jsr:@forge/fs"
        );
        assert_eq!(
            convert_to_jsr_specifier(&config, "https://example.com/mod.ts"),
            "https://example.com/mod.ts"
        );
    }

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("2.0.0", "1.0.0") > 0);
        assert!(compare_versions("1.0.0", "2.0.0") < 0);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("1.1.0", "1.0.0") > 0);
        assert!(compare_versions("1.0.1", "1.0.0") > 0);
    }

    #[test]
    fn test_std_modules() {
        use std_modules::*;
        assert_eq!(std_import(PATH), "jsr:@std/path");
        assert_eq!(std_import(FS), "jsr:@std/fs");
    }
}
