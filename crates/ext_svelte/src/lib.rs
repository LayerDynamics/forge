//! runtime:svelte extension - SvelteKit adapter functionality for Forge/Deno
//!
//! Provides runtime access to SvelteKit adapter capabilities:
//! - Build phase: asset collection, deploy config generation, ISR configuration
//! - Runtime phase: server preparation, ISR config matching, cache key generation

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use tracing::debug;
use walkdir::WalkDir;

// ============================================================================
// Error Types (codes 10000-10009)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SvelteErrorCode {
    /// Generic svelte error
    Generic = 10000,
    /// Build/adapt phase failed
    BuildFailed = 10001,
    /// Invalid configuration
    InvalidConfig = 10002,
    /// ISR cache error
    CacheError = 10003,
    /// Pattern matching error
    PatternError = 10004,
    /// Server preparation failed
    ServerPrepFailed = 10005,
    /// File not found
    NotFound = 10006,
    /// IO operation failed
    IoError = 10007,
}

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum SvelteError {
    #[error("[{code}] Svelte error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Build failed: {message}")]
    #[class(generic)]
    BuildFailed { code: u32, message: String },

    #[error("[{code}] Invalid config: {message}")]
    #[class(generic)]
    InvalidConfig { code: u32, message: String },

    #[error("[{code}] Cache error: {message}")]
    #[class(generic)]
    CacheError { code: u32, message: String },

    #[error("[{code}] Pattern error: {message}")]
    #[class(generic)]
    PatternError { code: u32, message: String },

    #[error("[{code}] Server prep failed: {message}")]
    #[class(generic)]
    ServerPrepFailed { code: u32, message: String },

    #[error("[{code}] Not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },
}

impl SvelteError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: SvelteErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn build_failed(message: impl Into<String>) -> Self {
        Self::BuildFailed {
            code: SvelteErrorCode::BuildFailed as u32,
            message: message.into(),
        }
    }

    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            code: SvelteErrorCode::InvalidConfig as u32,
            message: message.into(),
        }
    }

    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError {
            code: SvelteErrorCode::CacheError as u32,
            message: message.into(),
        }
    }

    pub fn pattern_error(message: impl Into<String>) -> Self {
        Self::PatternError {
            code: SvelteErrorCode::PatternError as u32,
            message: message.into(),
        }
    }

    pub fn server_prep_failed(message: impl Into<String>) -> Self {
        Self::ServerPrepFailed {
            code: SvelteErrorCode::ServerPrepFailed as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: SvelteErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: SvelteErrorCode::IoError as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Types - Shared between build and runtime
// ============================================================================

/// Serializable regex pattern (matches JS RegExp serialization)
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    /// Regex source pattern
    pub source: String,
    /// Regex flags (e.g., "i" for case-insensitive)
    pub flags: String,
}

/// Raw ISR config (pattern not compiled) - stored in svelte.json
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrConfigRaw {
    /// Regex pattern to match URLs
    pub pattern: RegexPattern,
    /// Cache expiration in seconds (default: 604800 = 7 days)
    pub expiration: u64,
    /// Token to bypass cache (optional)
    pub bypass_token: Option<String>,
    /// Query params to include in cache key
    pub allow_query: Vec<String>,
}

/// Svelte metadata stored in svelte.json
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvelteData {
    /// ISR configurations for routes
    pub isr: Vec<IsrConfigRaw>,
}

/// Static file mapping (source URL pattern -> destination file)
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFile {
    /// URL pattern to match (e.g., "/about" or "/_app/immutable/:file*")
    pub source: String,
    /// Destination file path
    pub destination: String,
}

/// Redirect rule
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redirect {
    /// Source URL pattern
    pub source: String,
    /// Destination URL
    pub destination: String,
    /// Whether this is a permanent (301) redirect
    pub permanent: bool,
}

/// HTTP header
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// Header name
    pub key: String,
    /// Header value
    pub value: String,
}

/// Header rule (applies headers to matching paths)
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRule {
    /// URL pattern to match
    pub source: String,
    /// Headers to apply
    pub headers: Vec<Header>,
}

/// URL rewrite rule
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewrite {
    /// Source URL pattern
    pub source: String,
    /// Destination to rewrite to
    pub destination: String,
}

/// Deploy configuration (deploy.json)
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployConfig {
    /// Static file mappings
    pub static_files: Vec<StaticFile>,
    /// Redirect rules
    pub redirects: Vec<Redirect>,
    /// Header rules
    pub headers: Vec<HeaderRule>,
    /// Rewrite rules
    pub rewrites: Vec<Rewrite>,
}

/// Prerendered page info
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerenderedPage {
    /// URL pathname (e.g., "/about/")
    pub pathname: String,
    /// File path relative to output dir (e.g., "about/index.html")
    pub file: String,
}

/// Route configuration from SvelteKit builder
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Regex pattern source
    pub pattern_source: String,
    /// Regex pattern flags
    pub pattern_flags: String,
    /// Whether this route is prerendered
    pub prerender: bool,
    /// ISR configuration for this route (if enabled)
    pub isr: Option<IsrRouteConfig>,
}

/// ISR configuration for a single route
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrRouteConfig {
    /// Cache expiration in seconds
    pub expiration: Option<u64>,
    /// Bypass token
    pub bypass_token: Option<String>,
    /// Query params to include in cache key
    pub allow_query: Option<Vec<String>>,
}

/// Result from walk operation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkResult {
    /// List of file paths found
    pub files: Vec<String>,
}

/// ISR config result from getIsrConfig
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrConfigResult {
    /// Whether an ISR config was found
    pub found: bool,
    /// The ISR config if found
    pub config: Option<IsrConfigRaw>,
}

/// Cache key result
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKeyResult {
    /// The generated cache key
    pub key: String,
}

/// SvelteKit project detection result
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvelteDetectionResult {
    /// Whether this is a SvelteKit project
    pub is_sveltekit: bool,
    /// Whether svelte.config.js/ts exists
    pub has_svelte_config: bool,
    /// Whether @sveltejs/kit is in package.json dependencies
    pub has_kit_dependency: bool,
    /// Whether src/routes directory exists
    pub has_routes_dir: bool,
    /// Svelte version from package.json (if found)
    pub svelte_version: Option<String>,
    /// SvelteKit version from package.json (if found)
    pub kit_version: Option<String>,
    /// Path to svelte.config.js/ts (if found)
    pub config_path: Option<String>,
    /// Detected adapter from svelte.config (if any)
    pub adapter: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Detection messages/notes
    pub messages: Vec<String>,
}

impl Default for SvelteDetectionResult {
    fn default() -> Self {
        Self {
            is_sveltekit: false,
            has_svelte_config: false,
            has_kit_dependency: false,
            has_routes_dir: false,
            svelte_version: None,
            kit_version: None,
            config_path: None,
            adapter: None,
            confidence: 0.0,
            messages: Vec::new(),
        }
    }
}

// ============================================================================
// State - Runtime server management
// ============================================================================

/// Compiled ISR configuration with regex
pub struct CompiledIsrConfig {
    /// Compiled regex pattern
    pub pattern: Regex,
    /// Cache expiration in seconds
    pub expiration: u64,
    /// Bypass token
    pub bypass_token: Option<String>,
    /// Query params to include in cache key
    pub allow_query: Vec<String>,
}

/// Server instance state
pub struct SvelteServer {
    /// Unique server ID
    pub id: String,
    /// Compiled ISR configurations
    pub isr_configs: Vec<CompiledIsrConfig>,
    /// Deploy configuration
    pub deploy_config: DeployConfig,
    /// Current working directory
    pub cwd: String,
}

/// Extension state holding all active servers
#[derive(Default)]
pub struct SvelteState {
    /// Active servers by ID
    pub servers: HashMap<String, SvelteServer>,
    /// Next server ID counter
    pub next_id: u64,
}

impl SvelteState {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_server(
        &mut self,
        isr_configs: Vec<CompiledIsrConfig>,
        deploy_config: DeployConfig,
        cwd: String,
    ) -> String {
        let id = format!("svelte-server-{}", self.next_id);
        self.next_id += 1;

        let server = SvelteServer {
            id: id.clone(),
            isr_configs,
            deploy_config,
            cwd,
        };

        self.servers.insert(id.clone(), server);
        id
    }

    pub fn get_server(&self, id: &str) -> Option<&SvelteServer> {
        self.servers.get(id)
    }

    pub fn remove_server(&mut self, id: &str) -> Option<SvelteServer> {
        self.servers.remove(id)
    }
}

// ============================================================================
// Detection Ops
// ============================================================================

/// Detect if a directory is a SvelteKit project
/// Checks for: svelte.config.js/ts, @sveltejs/kit in package.json, src/routes dir
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_svelte_detect(#[string] dir: String) -> Result<SvelteDetectionResult, SvelteError> {
    debug!(dir = %dir, "svelte.detect");

    let dir_path = Path::new(&dir);
    if !dir_path.exists() {
        return Err(SvelteError::not_found(format!(
            "Directory does not exist: {}",
            dir
        )));
    }

    let mut result = SvelteDetectionResult::default();
    let mut confidence_score: f64 = 0.0;

    // Check for svelte.config.js or svelte.config.ts
    let config_js = dir_path.join("svelte.config.js");
    let config_ts = dir_path.join("svelte.config.ts");
    let config_mjs = dir_path.join("svelte.config.mjs");

    if config_js.exists() {
        result.has_svelte_config = true;
        result.config_path = Some(config_js.display().to_string());
        confidence_score += 0.35;
        result.messages.push("Found svelte.config.js".to_string());

        // Try to detect adapter from config content
        if let Ok(content) = tokio::fs::read_to_string(&config_js).await {
            detect_adapter_from_config(&content, &mut result);
        }
    } else if config_ts.exists() {
        result.has_svelte_config = true;
        result.config_path = Some(config_ts.display().to_string());
        confidence_score += 0.35;
        result.messages.push("Found svelte.config.ts".to_string());

        if let Ok(content) = tokio::fs::read_to_string(&config_ts).await {
            detect_adapter_from_config(&content, &mut result);
        }
    } else if config_mjs.exists() {
        result.has_svelte_config = true;
        result.config_path = Some(config_mjs.display().to_string());
        confidence_score += 0.35;
        result.messages.push("Found svelte.config.mjs".to_string());

        if let Ok(content) = tokio::fs::read_to_string(&config_mjs).await {
            detect_adapter_from_config(&content, &mut result);
        }
    }

    // Check package.json for @sveltejs/kit dependency
    let package_json = dir_path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&package_json).await {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check dependencies and devDependencies
                let deps = pkg.get("dependencies");
                let dev_deps = pkg.get("devDependencies");

                // Check for @sveltejs/kit
                let kit_version = deps
                    .and_then(|d| d.get("@sveltejs/kit"))
                    .or_else(|| dev_deps.and_then(|d| d.get("@sveltejs/kit")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if kit_version.is_some() {
                    result.has_kit_dependency = true;
                    result.kit_version = kit_version;
                    confidence_score += 0.35;
                    result
                        .messages
                        .push("Found @sveltejs/kit in package.json".to_string());
                }

                // Check for svelte
                let svelte_version = deps
                    .and_then(|d| d.get("svelte"))
                    .or_else(|| dev_deps.and_then(|d| d.get("svelte")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if svelte_version.is_some() {
                    result.svelte_version = svelte_version;
                    confidence_score += 0.1;
                    result
                        .messages
                        .push("Found svelte in package.json".to_string());
                }
            }
        }
    }

    // Check for src/routes directory (SvelteKit convention)
    let routes_dir = dir_path.join("src").join("routes");
    if routes_dir.exists() && routes_dir.is_dir() {
        result.has_routes_dir = true;
        confidence_score += 0.2;
        result
            .messages
            .push("Found src/routes directory".to_string());
    }

    // Calculate final confidence and determine if it's SvelteKit
    result.confidence = confidence_score.min(1.0);

    // It's considered a SvelteKit project if:
    // - Has svelte.config AND @sveltejs/kit dependency, OR
    // - Confidence >= 0.7 (from accumulated indicators)
    result.is_sveltekit =
        (result.has_svelte_config && result.has_kit_dependency) || result.confidence >= 0.7;

    if result.is_sveltekit {
        result.messages.push(format!(
            "Detected as SvelteKit project (confidence: {:.0}%)",
            result.confidence * 100.0
        ));
    } else if result.confidence > 0.0 {
        result.messages.push(format!(
            "May be a Svelte project but not SvelteKit (confidence: {:.0}%)",
            result.confidence * 100.0
        ));
    }

    Ok(result)
}

/// Helper to detect adapter from svelte.config content
fn detect_adapter_from_config(content: &str, result: &mut SvelteDetectionResult) {
    // Common adapter patterns
    let adapters = [
        ("@sveltejs/adapter-auto", "auto"),
        ("@sveltejs/adapter-node", "node"),
        ("@sveltejs/adapter-static", "static"),
        ("@sveltejs/adapter-vercel", "vercel"),
        ("@sveltejs/adapter-netlify", "netlify"),
        ("@sveltejs/adapter-cloudflare", "cloudflare"),
        ("@deno/svelte-adapter", "deno"),
        ("svelte-adapter-deno", "deno"),
    ];

    for (package, name) in adapters {
        if content.contains(package) {
            result.adapter = Some(name.to_string());
            result.messages.push(format!("Detected adapter: {}", name));
            break;
        }
    }
}

// ============================================================================
// Build Phase Ops
// ============================================================================

/// Walk directory recursively and return all file paths
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_svelte_walk(#[string] dir: String) -> Result<WalkResult, SvelteError> {
    debug!(dir = %dir, "svelte.walk");

    let dir_path = Path::new(&dir);
    if !dir_path.exists() {
        return Err(SvelteError::not_found(format!(
            "Directory does not exist: {}",
            dir
        )));
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.path().display().to_string());
        }
    }

    Ok(WalkResult { files })
}

/// Generate deploy.json configuration from prerendered pages and assets
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_svelte_generate_deploy_config(
    #[serde] prerendered_pages: Vec<PrerenderedPage>,
    #[string] static_dir: String,
    #[string] base_path: String,
    #[serde] assets: Vec<String>,
    #[string] asset_dir: String,
) -> Result<DeployConfig, SvelteError> {
    debug!(
        pages = prerendered_pages.len(),
        assets = assets.len(),
        "svelte.generate_deploy_config"
    );

    let mut config = DeployConfig::default();

    // Map prerendered pages to static files
    for page in &prerendered_pages {
        config.static_files.push(StaticFile {
            source: page.pathname.clone(),
            destination: format!("{}/{}", static_dir, page.file),
        });

        // Add trailing slash redirects (except for root)
        if page.pathname != "/" {
            let trailing = page.pathname.ends_with('/');
            let redirect_source = if trailing {
                page.pathname.trim_end_matches('/').to_string()
            } else {
                format!("{}/", page.pathname)
            };

            config.redirects.push(Redirect {
                source: redirect_source,
                destination: page.pathname.clone(),
                permanent: true,
            });
        }
    }

    // Add immutable assets pattern
    config.static_files.push(StaticFile {
        source: format!("{}/_app/immutable/:file*", base_path),
        destination: format!("{}{}_app/immutable/:file*", static_dir, base_path),
    });

    // Add cache headers for immutable assets
    config.headers.push(HeaderRule {
        source: format!("{}/_app/immutable/:file*", base_path),
        headers: vec![Header {
            key: "Cache-Control".to_string(),
            value: "public, immutable, max-age=31536000".to_string(),
        }],
    });

    // Map asset directory files
    let asset_dir_path = Path::new(&asset_dir);
    for asset in &assets {
        let asset_path = Path::new(asset);
        if let Ok(rel) = asset_path.strip_prefix(asset_dir_path) {
            let rel_str = rel.display().to_string().replace('\\', "/");
            config.static_files.push(StaticFile {
                source: format!("{}/{}", base_path, rel_str),
                destination: format!("{}/{}", static_dir, rel_str),
            });
        }
    }

    Ok(config)
}

/// Generate svelte.json with ISR configuration from routes
#[weld_op]
#[op2]
#[serde]
pub fn op_svelte_generate_svelte_data(
    #[serde] routes: Vec<RouteConfig>,
) -> Result<SvelteData, SvelteError> {
    debug!(routes = routes.len(), "svelte.generate_svelte_data");

    let mut isr_configs = Vec::new();

    for route in routes {
        // ISR cannot be used with prerendering
        if route.prerender {
            continue;
        }

        if let Some(isr) = route.isr {
            isr_configs.push(IsrConfigRaw {
                pattern: RegexPattern {
                    source: route.pattern_source,
                    flags: route.pattern_flags,
                },
                expiration: isr.expiration.unwrap_or(604800), // Default: 7 days
                bypass_token: isr.bypass_token,
                allow_query: isr.allow_query.unwrap_or_default(),
            });
        }
    }

    Ok(SvelteData { isr: isr_configs })
}

// ============================================================================
// Runtime Phase Ops
// ============================================================================

/// Prepare server - compile ISR patterns and return server ID
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_svelte_prepare_server(
    state: Rc<RefCell<OpState>>,
    #[serde] svelte_data: SvelteData,
    #[serde] deploy_config: DeployConfig,
    #[string] cwd: String,
) -> Result<String, SvelteError> {
    debug!(
        isr_count = svelte_data.isr.len(),
        cwd = %cwd,
        "svelte.prepare_server"
    );

    // Compile ISR patterns
    let mut compiled_configs = Vec::new();

    for raw in svelte_data.isr {
        // Build pattern string with flags
        let pattern_str = format!("{}{}", raw.pattern.source, raw.pattern.flags);

        let pattern = Regex::new(&pattern_str).map_err(|e| {
            SvelteError::pattern_error(format!(
                "Failed to compile ISR pattern '{}': {}",
                pattern_str, e
            ))
        })?;

        compiled_configs.push(CompiledIsrConfig {
            pattern,
            expiration: raw.expiration,
            bypass_token: raw.bypass_token,
            allow_query: raw.allow_query,
        });
    }

    // Create server and return ID
    let mut state = state.borrow_mut();
    let svelte_state = state.borrow_mut::<SvelteState>();
    let server_id = svelte_state.create_server(compiled_configs, deploy_config, cwd);

    Ok(server_id)
}

/// Check if URL matches any ISR rule and return config if found
/// Returns None if bypass token matches (indicating cache should be skipped)
#[weld_op]
#[op2]
#[serde]
pub fn op_svelte_get_isr_config(
    state: &mut OpState,
    #[string] server_id: String,
    #[string] pathname: String,
    #[string] method: String,
    #[serde] headers: Vec<(String, String)>,
    #[serde] cookies: Vec<String>,
) -> Result<IsrConfigResult, SvelteError> {
    debug!(
        server_id = %server_id,
        pathname = %pathname,
        method = %method,
        "svelte.get_isr_config"
    );

    let svelte_state = state.borrow::<SvelteState>();
    let server = svelte_state
        .get_server(&server_id)
        .ok_or_else(|| SvelteError::not_found(format!("Server not found: {}", server_id)))?;

    // Find matching ISR config
    for config in &server.isr_configs {
        if config.pattern.is_match(&pathname) {
            // Check for bypass token in headers (GET/HEAD only)
            if method == "GET" || method == "HEAD" {
                for (key, value) in &headers {
                    if key.to_lowercase() == "x-prerender-revalidate" {
                        if let Some(ref bypass_token) = config.bypass_token {
                            if value == bypass_token {
                                // Bypass token matches - skip ISR
                                return Ok(IsrConfigResult {
                                    found: false,
                                    config: None,
                                });
                            }
                        }
                    }
                }
            }

            // Check for bypass token in cookies
            for cookie in &cookies {
                if let Some((key, value)) = cookie.split_once('=') {
                    if key == "__prerender_bypass" {
                        if let Some(ref bypass_token) = config.bypass_token {
                            if value == bypass_token {
                                // Bypass token matches - skip ISR
                                return Ok(IsrConfigResult {
                                    found: false,
                                    config: None,
                                });
                            }
                        }
                    }
                }
            }

            // Return the ISR config (pattern as string since Regex isn't serializable)
            return Ok(IsrConfigResult {
                found: true,
                config: Some(IsrConfigRaw {
                    pattern: RegexPattern {
                        source: config.pattern.as_str().to_string(),
                        flags: String::new(),
                    },
                    expiration: config.expiration,
                    bypass_token: config.bypass_token.clone(),
                    allow_query: config.allow_query.clone(),
                }),
            });
        }
    }

    // No matching ISR config
    Ok(IsrConfigResult {
        found: false,
        config: None,
    })
}

/// Generate cache key from URL pathname and allowed query params
#[weld_op]
#[op2]
#[serde]
pub fn op_svelte_to_cache_key(
    #[string] pathname: String,
    #[serde] search_params: Vec<(String, String)>,
    #[serde] allow_query: Vec<String>,
) -> CacheKeyResult {
    debug!(
        pathname = %pathname,
        params = search_params.len(),
        allow = allow_query.len(),
        "svelte.to_cache_key"
    );

    // Start with pathname
    let mut key = pathname;

    // Filter and add allowed query params
    let mut allowed_params: Vec<(String, String)> = search_params
        .into_iter()
        .filter(|(k, _)| allow_query.contains(k))
        .collect();

    // Sort for consistent cache keys
    allowed_params.sort_by(|a, b| a.0.cmp(&b.0));

    if !allowed_params.is_empty() {
        key.push('?');
        let params: Vec<String> = allowed_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        key.push_str(&params.join("&"));
    }

    CacheKeyResult { key }
}

/// Check if a path matches static file patterns
#[weld_op]
#[op2(fast)]
pub fn op_svelte_is_static(
    state: &mut OpState,
    #[string] server_id: &str,
    #[string] pathname: &str,
) -> bool {
    debug!(
        server_id = %server_id,
        pathname = %pathname,
        "svelte.is_static"
    );

    let svelte_state = state.borrow::<SvelteState>();
    let Some(server) = svelte_state.get_server(server_id) else {
        return false;
    };

    // Check if pathname matches any static file source
    for static_file in &server.deploy_config.static_files {
        if pathname_matches_pattern(pathname, &static_file.source) {
            return true;
        }
    }

    false
}

/// Get redirect destination if path matches
#[weld_op]
#[op2]
#[serde]
pub fn op_svelte_get_redirect(
    state: &mut OpState,
    #[string] server_id: String,
    #[string] pathname: String,
) -> Result<Option<Redirect>, SvelteError> {
    debug!(
        server_id = %server_id,
        pathname = %pathname,
        "svelte.get_redirect"
    );

    let svelte_state = state.borrow::<SvelteState>();
    let server = svelte_state
        .get_server(&server_id)
        .ok_or_else(|| SvelteError::not_found(format!("Server not found: {}", server_id)))?;

    // Check if pathname matches any redirect source
    for redirect in &server.deploy_config.redirects {
        if pathname_matches_pattern(&pathname, &redirect.source) {
            return Ok(Some(redirect.clone()));
        }
    }

    Ok(None)
}

/// Get headers to apply for a path
#[weld_op]
#[op2]
#[serde]
pub fn op_svelte_get_headers(
    state: &mut OpState,
    #[string] server_id: String,
    #[string] pathname: String,
) -> Result<Vec<Header>, SvelteError> {
    debug!(
        server_id = %server_id,
        pathname = %pathname,
        "svelte.get_headers"
    );

    let svelte_state = state.borrow::<SvelteState>();
    let server = svelte_state
        .get_server(&server_id)
        .ok_or_else(|| SvelteError::not_found(format!("Server not found: {}", server_id)))?;

    let mut headers = Vec::new();

    // Collect headers from all matching rules
    for rule in &server.deploy_config.headers {
        if pathname_matches_pattern(&pathname, &rule.source) {
            headers.extend(rule.headers.clone());
        }
    }

    Ok(headers)
}

/// Clean up server state
#[weld_op(async)]
#[op2(async)]
pub async fn op_svelte_close_server(
    state: Rc<RefCell<OpState>>,
    #[string] server_id: String,
) -> Result<(), SvelteError> {
    debug!(server_id = %server_id, "svelte.close_server");

    let mut state = state.borrow_mut();
    let svelte_state = state.borrow_mut::<SvelteState>();

    svelte_state
        .remove_server(&server_id)
        .ok_or_else(|| SvelteError::not_found(format!("Server not found: {}", server_id)))?;

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a pathname matches a URL pattern (supports :param and :param* wildcards)
fn pathname_matches_pattern(pathname: &str, pattern: &str) -> bool {
    // Handle wildcard patterns like "/_app/immutable/:file*"
    if pattern.contains(":") {
        // Convert pattern to regex
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace(":file", "[^/]+")
            .replace(":path", ".+");

        if let Ok(re) = Regex::new(&format!("^{}$", regex_pattern)) {
            return re.is_match(pathname);
        }
    }

    // Exact match
    pathname == pattern
}

// ============================================================================
// Extension Setup
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn svelte_extension() -> Extension {
    runtime_svelte::ext()
}

/// Initialize the svelte extension state in the op state
pub fn init_svelte_state(state: &mut OpState) {
    state.put(SvelteState::new());
}

// Re-export forge_weld for the macros
pub use forge_weld;

// ============================================================================
// Internal Helper Functions (for testing)
// ============================================================================

#[cfg(test)]
/// Internal implementation of cache key generation
fn generate_cache_key_impl(
    pathname: &str,
    search_params: Vec<(String, String)>,
    allow_query: &[String],
) -> String {
    let mut key = pathname.to_string();

    // Filter and add allowed query params
    let mut allowed_params: Vec<(String, String)> = search_params
        .into_iter()
        .filter(|(k, _)| allow_query.contains(k))
        .collect();

    // Sort for consistent cache keys
    allowed_params.sort_by(|a, b| a.0.cmp(&b.0));

    if !allowed_params.is_empty() {
        key.push('?');
        let params: Vec<String> = allowed_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        key.push_str(&params.join("&"));
    }

    key
}

#[cfg(test)]
/// Internal implementation of svelte data generation
fn generate_svelte_data_impl(routes: Vec<RouteConfig>) -> SvelteData {
    let mut isr_configs = Vec::new();

    for route in routes {
        // ISR cannot be used with prerendering
        if route.prerender {
            continue;
        }

        if let Some(isr) = route.isr {
            isr_configs.push(IsrConfigRaw {
                pattern: RegexPattern {
                    source: route.pattern_source,
                    flags: route.pattern_flags,
                },
                expiration: isr.expiration.unwrap_or(604800),
                bypass_token: isr.bypass_token,
                allow_query: isr.allow_query.unwrap_or_default(),
            });
        }
    }

    SvelteData { isr: isr_configs }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = SvelteError::build_failed("test error");
        match err {
            SvelteError::BuildFailed { code, .. } => {
                assert_eq!(code, SvelteErrorCode::BuildFailed as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let result = generate_cache_key_impl(
            "/products/123",
            vec![
                ("sort".to_string(), "asc".to_string()),
                ("filter".to_string(), "active".to_string()),
                ("page".to_string(), "1".to_string()),
            ],
            &["sort".to_string(), "page".to_string()],
        );

        // Should only include 'sort' and 'page' params (alphabetically sorted)
        assert_eq!(result, "/products/123?page=1&sort=asc");
    }

    #[test]
    fn test_cache_key_no_allowed_params() {
        let result = generate_cache_key_impl(
            "/about",
            vec![("utm_source".to_string(), "google".to_string())],
            &[],
        );

        // No query params in output
        assert_eq!(result, "/about");
    }

    #[test]
    fn test_pathname_matches_exact() {
        assert!(pathname_matches_pattern("/about", "/about"));
        assert!(!pathname_matches_pattern("/about/", "/about"));
    }

    #[test]
    fn test_pathname_matches_wildcard() {
        assert!(pathname_matches_pattern(
            "/_app/immutable/chunks/abc123.js",
            "/_app/immutable/:file*"
        ));
    }

    #[test]
    fn test_svelte_data_generation() {
        let routes = vec![
            RouteConfig {
                pattern_source: "^/products/.*$".to_string(),
                pattern_flags: String::new(),
                prerender: false,
                isr: Some(IsrRouteConfig {
                    expiration: Some(3600),
                    bypass_token: Some("secret".to_string()),
                    allow_query: Some(vec!["sort".to_string()]),
                }),
            },
            RouteConfig {
                pattern_source: "^/about$".to_string(),
                pattern_flags: String::new(),
                prerender: true, // Should be skipped
                isr: Some(IsrRouteConfig {
                    expiration: None,
                    bypass_token: None,
                    allow_query: None,
                }),
            },
        ];

        let result = generate_svelte_data_impl(routes);

        // Only non-prerendered route with ISR should be included
        assert_eq!(result.isr.len(), 1);
        assert_eq!(result.isr[0].expiration, 3600);
        assert_eq!(result.isr[0].bypass_token, Some("secret".to_string()));
    }
}
