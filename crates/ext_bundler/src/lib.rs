//! forge:bundler extension - App packaging, icons, and manifest utilities
//!
//! Provides runtime access to Forge bundling capabilities:
//! - Icon creation, validation, and resizing
//! - Manifest parsing and validation
//! - Platform-specific bundle information
//! - Build configuration management

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use image::{imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BundlerErrorCode {
    /// Icon processing failed
    IconError = 9000,
    /// Manifest parsing failed
    ManifestError = 9001,
    /// Invalid input
    InvalidInput = 9002,
    /// IO error
    IoError = 9003,
    /// Platform not supported
    PlatformError = 9004,
}

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum BundlerError {
    #[error("[{code}] Icon error: {message}")]
    #[class(generic)]
    IconError { code: u32, message: String },

    #[error("[{code}] Manifest error: {message}")]
    #[class(generic)]
    ManifestError { code: u32, message: String },

    #[error("[{code}] Invalid input: {message}")]
    #[class(generic)]
    InvalidInput { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },

    #[error("[{code}] Platform error: {message}")]
    #[class(generic)]
    PlatformError { code: u32, message: String },
}

impl BundlerError {
    pub fn icon_error(message: impl Into<String>) -> Self {
        Self::IconError {
            code: BundlerErrorCode::IconError as u32,
            message: message.into(),
        }
    }

    pub fn manifest_error(message: impl Into<String>) -> Self {
        Self::ManifestError {
            code: BundlerErrorCode::ManifestError as u32,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: BundlerErrorCode::InvalidInput as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: BundlerErrorCode::IoError as u32,
            message: message.into(),
        }
    }

    pub fn platform_error(message: impl Into<String>) -> Self {
        Self::PlatformError {
            code: BundlerErrorCode::PlatformError as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// Bundler extension state for managing build configurations
#[derive(Default)]
pub struct BundlerState {
    /// Current app directory being bundled
    app_dir: Option<PathBuf>,
    /// Cached manifests by path
    manifests: HashMap<PathBuf, AppManifest>,
    /// Build configuration
    build_config: Option<BuildConfig>,
}

impl BundlerState {
    pub fn new() -> Self {
        Self {
            app_dir: None,
            manifests: HashMap::new(),
            build_config: None,
        }
    }

    pub fn set_app_dir(&mut self, path: PathBuf) {
        self.app_dir = Some(path);
    }

    pub fn get_app_dir(&self) -> Option<&Path> {
        self.app_dir.as_deref()
    }

    pub fn cache_manifest(&mut self, path: PathBuf, manifest: AppManifest) {
        self.manifests.insert(path, manifest);
    }

    pub fn get_cached_manifest(&self, path: &Path) -> Option<&AppManifest> {
        self.manifests.get(path)
    }

    pub fn set_build_config(&mut self, config: BuildConfig) {
        self.build_config = Some(config);
    }

    pub fn get_build_config(&self) -> Option<&BuildConfig> {
        self.build_config.as_ref()
    }
}

// ============================================================================
// Types
// ============================================================================

/// Minimum recommended icon size
pub const MIN_ICON_SIZE: u32 = 512;
/// Optimal icon size for best quality across all platforms
pub const RECOMMENDED_ICON_SIZE: u32 = 1024;

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: Vec<&'static str>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct IconValidation {
    pub width: u32,
    pub height: u32,
    pub is_square: bool,
    pub meets_minimum: bool,
    pub meets_recommended: bool,
    pub has_transparency: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconCreateOptions {
    /// Size in pixels (default: 1024)
    pub size: Option<u32>,
    /// Primary color (hex, default: "#3C5AB8")
    pub color: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconResizeOptions {
    /// Target width
    pub width: u32,
    /// Target height
    pub height: u32,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub icon: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub bundle_format: String,
    pub supported: bool,
}

#[weld_enum]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BundleFormat {
    /// macOS .app bundle
    App,
    /// macOS .dmg disk image
    Dmg,
    /// macOS .pkg installer
    Pkg,
    /// Windows .msix package
    Msix,
    /// Linux AppImage
    AppImage,
    /// Compressed tarball
    Tarball,
    /// ZIP archive
    Zip,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// App directory path
    pub app_dir: String,
    /// Output directory for bundle
    pub output_dir: Option<String>,
    /// Target bundle format
    pub format: Option<BundleFormat>,
    /// Whether to sign the bundle
    pub sign: Option<bool>,
    /// Code signing identity (platform-specific)
    pub signing_identity: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct PathInfo {
    /// The full path
    pub path: String,
    /// Whether the path exists
    pub exists: bool,
    /// Whether it's a directory
    pub is_dir: bool,
    /// Whether it's a file
    pub is_file: bool,
    /// File extension if any
    pub extension: Option<String>,
    /// File name without path
    pub file_name: Option<String>,
    /// Parent directory
    pub parent: Option<String>,
}

// ============================================================================
// Ops
// ============================================================================

/// Get extension info
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_bundler",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: vec![
            "icon_create",
            "icon_validate",
            "icon_resize",
            "manifest_parse",
            "platform_info",
        ],
    }
}

/// Create a placeholder icon
#[weld_op]
#[op2]
#[buffer]
pub fn op_bundler_icon_create(
    #[serde] options: Option<IconCreateOptions>,
) -> Result<Vec<u8>, BundlerError> {
    let opts = options.unwrap_or(IconCreateOptions {
        size: None,
        color: None,
    });
    let size = opts.size.unwrap_or(RECOMMENDED_ICON_SIZE);

    debug!(size = size, "bundler.icon_create");

    let mut img = RgbaImage::new(size, size);

    // Create a blue-purple gradient (or use custom color)
    for y in 0..size {
        for x in 0..size {
            let gradient_x = x as f32 / size as f32;
            let gradient_y = y as f32 / size as f32;
            let gradient = (gradient_x + gradient_y) / 2.0;

            // Blue to purple gradient
            let r = (60.0 + gradient * 100.0) as u8;
            let g = (80.0 + gradient * 60.0) as u8;
            let b = (180.0 + gradient * 40.0) as u8;

            // Simple circular mask for rounded appearance
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let radius = size as f32 * 0.45;

            let alpha = if dist < radius {
                255
            } else if dist < radius + 20.0 {
                ((radius + 20.0 - dist) / 20.0 * 255.0) as u8
            } else {
                0
            };

            img.put_pixel(x, y, Rgba([r, g, b, alpha]));
        }
    }

    // Encode to PNG bytes
    let mut buffer = Vec::new();
    let dyn_img = DynamicImage::ImageRgba8(img);
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| BundlerError::icon_error(e.to_string()))?;

    Ok(buffer)
}

/// Validate an icon from bytes
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_icon_validate(#[buffer] data: &[u8]) -> Result<IconValidation, BundlerError> {
    debug!("bundler.icon_validate");

    let img = image::load_from_memory(data)
        .map_err(|e| BundlerError::icon_error(format!("Failed to load image: {}", e)))?;

    let width = img.width();
    let height = img.height();
    let is_square = width == height;
    let meets_minimum = width >= MIN_ICON_SIZE && height >= MIN_ICON_SIZE;
    let meets_recommended = width >= RECOMMENDED_ICON_SIZE && height >= RECOMMENDED_ICON_SIZE;

    let has_transparency = matches!(
        img,
        DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageLumaA8(_)
            | DynamicImage::ImageLumaA16(_)
    );

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if !meets_minimum {
        errors.push(format!(
            "Icon is too small ({}x{}). Minimum size is {}x{} pixels.",
            width, height, MIN_ICON_SIZE, MIN_ICON_SIZE
        ));
    } else if !meets_recommended {
        warnings.push(format!(
            "Icon is {}x{}. Recommended size is {}x{} for best quality.",
            width, height, RECOMMENDED_ICON_SIZE, RECOMMENDED_ICON_SIZE
        ));
    }

    if !is_square {
        errors.push(format!(
            "Icon must be square. Current size: {}x{} (aspect ratio: {:.2}:1)",
            width,
            height,
            width as f32 / height as f32
        ));
    }

    if !has_transparency {
        warnings.push(
            "Icon does not have an alpha channel. Consider using PNG with transparency."
                .to_string(),
        );
    }

    Ok(IconValidation {
        width,
        height,
        is_square,
        meets_minimum,
        meets_recommended,
        has_transparency,
        warnings,
        errors,
    })
}

/// Resize an icon
#[weld_op]
#[op2]
#[buffer]
pub fn op_bundler_icon_resize(
    #[buffer] data: &[u8],
    #[serde] options: IconResizeOptions,
) -> Result<Vec<u8>, BundlerError> {
    debug!(
        width = options.width,
        height = options.height,
        "bundler.icon_resize"
    );

    let img = image::load_from_memory(data)
        .map_err(|e| BundlerError::icon_error(format!("Failed to load image: {}", e)))?;

    let resized = img.resize_exact(options.width, options.height, FilterType::Lanczos3);

    let mut buffer = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| BundlerError::icon_error(e.to_string()))?;

    Ok(buffer)
}

/// Parse a manifest.app.toml file
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_manifest_parse(#[string] content: String) -> Result<AppManifest, BundlerError> {
    debug!("bundler.manifest_parse");

    #[derive(Deserialize)]
    struct RawManifest {
        app: RawAppConfig,
        bundle: Option<RawBundleConfig>,
    }

    #[derive(Deserialize)]
    struct RawAppConfig {
        name: String,
        identifier: String,
        version: String,
    }

    #[derive(Deserialize)]
    struct RawBundleConfig {
        icon: Option<String>,
    }

    let raw: RawManifest = toml::from_str(&content)
        .map_err(|e| BundlerError::manifest_error(format!("Failed to parse manifest: {}", e)))?;

    Ok(AppManifest {
        name: raw.app.name,
        identifier: raw.app.identifier,
        version: raw.app.version,
        icon: raw.bundle.and_then(|b| b.icon),
    })
}

/// Sanitize a name for use as executable/identifier
#[weld_op]
#[op2]
#[string]
pub fn op_bundler_sanitize_name(#[string] name: String) -> String {
    debug!(name = %name, "bundler.sanitize_name");

    let result: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    let mut collapsed = String::new();
    let mut prev_hyphen = true;
    for c in result.chars() {
        if c == '-' {
            if !prev_hyphen {
                collapsed.push(c);
            }
            prev_hyphen = true;
        } else {
            collapsed.push(c);
            prev_hyphen = false;
        }
    }

    collapsed.trim_end_matches('-').to_string()
}

/// Get platform-specific bundle information
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_platform_info() -> PlatformInfo {
    debug!("bundler.platform_info");

    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let (bundle_format, supported) = match os.as_str() {
        "macos" => ("dmg".to_string(), true),
        "windows" => ("msix".to_string(), true),
        "linux" => ("appimage".to_string(), true),
        _ => ("unknown".to_string(), false),
    };

    PlatformInfo {
        os,
        arch,
        bundle_format,
        supported,
    }
}

/// Get icon requirements for a specific platform
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_icon_requirements(
    #[string] platform: String,
) -> Result<Vec<IconResizeOptions>, BundlerError> {
    debug!(platform = %platform, "bundler.icon_requirements");

    let sizes = match platform.to_lowercase().as_str() {
        "macos" => vec![
            (16, 16),
            (32, 32),
            (64, 64),
            (128, 128),
            (256, 256),
            (512, 512),
            (1024, 1024),
        ],
        "windows" => vec![
            (44, 44),
            (50, 50),
            (55, 55),
            (66, 66),
            (88, 88),
            (100, 100),
            (150, 150),
            (176, 176),
            (188, 188),
            (225, 225),
            (300, 300),
            (600, 600),
        ],
        "linux" => vec![
            (16, 16),
            (32, 32),
            (48, 48),
            (64, 64),
            (128, 128),
            (256, 256),
            (512, 512),
        ],
        _ => {
            return Err(BundlerError::platform_error(format!(
                "Unknown platform: {}",
                platform
            )));
        }
    };

    Ok(sizes
        .into_iter()
        .map(|(w, h)| IconResizeOptions {
            width: w,
            height: h,
        })
        .collect())
}

// ============================================================================
// State-based Ops
// ============================================================================

/// Set the current app directory for bundling operations
#[weld_op]
#[op2(fast)]
pub fn op_bundler_set_app_dir(
    state: &mut OpState,
    #[string] path: String,
) -> Result<(), BundlerError> {
    debug!(path = %path, "bundler.set_app_dir");

    let path = Path::new(&path);
    if !path.exists() {
        return Err(BundlerError::io_error(format!(
            "App directory does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(BundlerError::io_error(format!(
            "Path is not a directory: {}",
            path.display()
        )));
    }

    let bundler_state = state.borrow_mut::<BundlerState>();
    bundler_state.set_app_dir(path.to_path_buf());

    Ok(())
}

/// Get the current app directory
#[weld_op]
#[op2]
#[string]
pub fn op_bundler_get_app_dir(state: &mut OpState) -> Option<String> {
    debug!("bundler.get_app_dir");

    let bundler_state = state.borrow::<BundlerState>();
    bundler_state.get_app_dir().map(|p| p.display().to_string())
}

/// Set build configuration
#[weld_op]
#[op2]
pub fn op_bundler_set_build_config(
    state: &mut OpState,
    #[serde] config: BuildConfig,
) -> Result<(), BundlerError> {
    debug!(app_dir = %config.app_dir, "bundler.set_build_config");

    // Validate the app directory exists
    let app_path = Path::new(&config.app_dir);
    if !app_path.exists() {
        return Err(BundlerError::io_error(format!(
            "App directory does not exist: {}",
            config.app_dir
        )));
    }

    let bundler_state = state.borrow_mut::<BundlerState>();
    bundler_state.set_app_dir(app_path.to_path_buf());
    bundler_state.set_build_config(config);

    Ok(())
}

/// Get current build configuration
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_get_build_config(state: &mut OpState) -> Option<BuildConfig> {
    debug!("bundler.get_build_config");

    let bundler_state = state.borrow::<BundlerState>();
    bundler_state.get_build_config().cloned()
}

/// Analyze a path and return information about it
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_path_info(#[string] path_str: String) -> PathInfo {
    debug!(path = %path_str, "bundler.path_info");

    let path = Path::new(&path_str);
    let exists = path.exists();
    let is_dir = path.is_dir();
    let is_file = path.is_file();
    let extension = path.extension().map(|e| e.to_string_lossy().to_string());
    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
    let parent = path.parent().map(|p| p.display().to_string());

    PathInfo {
        path: path_str,
        exists,
        is_dir,
        is_file,
        extension,
        file_name,
        parent,
    }
}

/// Join path components
#[weld_op]
#[op2]
#[string]
pub fn op_bundler_path_join(#[serde] components: Vec<String>) -> String {
    debug!(components = ?components, "bundler.path_join");

    let mut path = PathBuf::new();
    for component in components {
        path.push(component);
    }
    path.display().to_string()
}

/// Get the manifest path for an app directory
#[weld_op]
#[op2]
#[string]
pub fn op_bundler_manifest_path(#[string] app_dir: String) -> String {
    debug!(app_dir = %app_dir, "bundler.manifest_path");

    let path = Path::new(&app_dir).join("manifest.app.toml");
    path.display().to_string()
}

/// Cache a parsed manifest for later retrieval
#[weld_op]
#[op2]
pub fn op_bundler_cache_manifest(
    state: &mut OpState,
    #[string] path: String,
    #[serde] manifest: AppManifest,
) {
    debug!(path = %path, "bundler.cache_manifest");

    let bundler_state = state.borrow_mut::<BundlerState>();
    bundler_state.cache_manifest(PathBuf::from(path), manifest);
}

/// Get a cached manifest
#[weld_op]
#[op2]
#[serde]
pub fn op_bundler_get_cached_manifest(
    state: &mut OpState,
    #[string] path: String,
) -> Option<AppManifest> {
    debug!(path = %path, "bundler.get_cached_manifest");

    let bundler_state = state.borrow::<BundlerState>();
    bundler_state.get_cached_manifest(Path::new(&path)).cloned()
}

// ============================================================================
// Extension Setup
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn bundler_extension() -> Extension {
    forge_bundler::ext()
}

/// Initialize the bundler extension state in the op state
pub fn init_bundler_state(state: &mut OpState) {
    state.put(BundlerState::new());
}

// Re-export forge_weld for the macros
pub use forge_weld;
