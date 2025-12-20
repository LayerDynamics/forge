//! runtime:image_tools extension - Image manipulation and format conversion
//!
//! Provides general-purpose image operations:
//! - PNG: load, save, info, optimize
//! - SVG: load, info, render to raster
//! - WebP: encode/decode for app asset optimization (NOT for icons)
//! - Convert: SVG→PNG, PNG→ICO, favicons, PNG→WebP
//! - Transform: resize, scale, crop, rotate, flip

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use tracing::debug;

// Submodules (file-based)
pub mod convert;
pub mod png;
pub mod svg;
pub mod transform;
pub mod webp;

// Test fixtures
#[cfg(test)]
pub mod fixtures;

// ============================================================================
// Error Types
// ============================================================================

/// Error code range: 9100-9199
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ImageToolsErrorCode {
    PngError = 9100,
    SvgError = 9101,
    IcoError = 9102,
    ConvertError = 9103,
    TransformError = 9104,
    IoError = 9105,
    InvalidInput = 9106,
    WebPError = 9107,
}

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum ImageToolsError {
    #[error("[{code}] PNG error: {message}")]
    #[class(generic)]
    PngError { code: u32, message: String },

    #[error("[{code}] SVG error: {message}")]
    #[class(generic)]
    SvgError { code: u32, message: String },

    #[error("[{code}] ICO error: {message}")]
    #[class(generic)]
    IcoError { code: u32, message: String },

    #[error("[{code}] Conversion error: {message}")]
    #[class(generic)]
    ConvertError { code: u32, message: String },

    #[error("[{code}] Transform error: {message}")]
    #[class(generic)]
    TransformError { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },

    #[error("[{code}] Invalid input: {message}")]
    #[class(generic)]
    InvalidInput { code: u32, message: String },

    #[error("[{code}] WebP error: {message}")]
    #[class(generic)]
    WebPError { code: u32, message: String },
}

impl ImageToolsError {
    pub fn png_error(message: impl Into<String>) -> Self {
        Self::PngError {
            code: ImageToolsErrorCode::PngError as u32,
            message: message.into(),
        }
    }

    pub fn svg_error(message: impl Into<String>) -> Self {
        Self::SvgError {
            code: ImageToolsErrorCode::SvgError as u32,
            message: message.into(),
        }
    }

    pub fn ico_error(message: impl Into<String>) -> Self {
        Self::IcoError {
            code: ImageToolsErrorCode::IcoError as u32,
            message: message.into(),
        }
    }

    pub fn convert_error(message: impl Into<String>) -> Self {
        Self::ConvertError {
            code: ImageToolsErrorCode::ConvertError as u32,
            message: message.into(),
        }
    }

    pub fn transform_error(message: impl Into<String>) -> Self {
        Self::TransformError {
            code: ImageToolsErrorCode::TransformError as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: ImageToolsErrorCode::IoError as u32,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: ImageToolsErrorCode::InvalidInput as u32,
            message: message.into(),
        }
    }

    pub fn webp_error(message: impl Into<String>) -> Self {
        Self::WebPError {
            code: ImageToolsErrorCode::WebPError as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub has_alpha: bool,
    pub color_type: String,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SvgInfo {
    pub width: f64,
    pub height: f64,
    pub view_box: Option<ViewBox>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ViewBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct WebPInfo {
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub is_lossless: bool,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct FaviconSet {
    #[serde(with = "serde_bytes")]
    pub favicon16: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub favicon32: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub favicon48: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub apple180: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub ico: Vec<u8>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngSaveOptions {
    /// Compression level (0-9, default 6)
    pub compression: Option<u8>,
}

#[weld_enum]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FilterType {
    /// Fastest, lowest quality
    Nearest,
    /// Balanced speed/quality
    Bilinear,
    /// Best quality, slower
    #[default]
    Lanczos3,
}

#[weld_enum]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

// ============================================================================
// State
// ============================================================================

/// Image tools extension state (currently minimal)
#[derive(Default)]
pub struct ImageToolsState {
    // Reserved for future caching of parsed SVGs, etc.
}

// ============================================================================
// PNG Ops
// ============================================================================

/// Get information about a PNG image
#[weld_op]
#[op2]
#[serde]
pub fn op_image_png_info(#[buffer] data: &[u8]) -> Result<ImageInfo, ImageToolsError> {
    debug!("image_tools.png_info");
    png::get_png_info(data)
}

/// Load PNG and return image info
#[weld_op]
#[op2]
#[serde]
pub fn op_image_png_load(#[buffer] data: &[u8]) -> Result<ImageInfo, ImageToolsError> {
    debug!("image_tools.png_load");
    png::get_png_info(data)
}

/// Save/re-encode PNG with options
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_png_save(
    #[buffer] data: &[u8],
    #[serde] options: Option<PngSaveOptions>,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!("image_tools.png_save");
    let img = png::load_png(data)?;
    png::save_png(&img, options)
}

/// Optimize PNG by re-encoding
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_png_optimize(#[buffer] data: &[u8]) -> Result<Vec<u8>, ImageToolsError> {
    debug!("image_tools.png_optimize");
    png::optimize_png(data)
}

// ============================================================================
// SVG Ops
// ============================================================================

/// Get information about an SVG
#[weld_op]
#[op2]
#[serde]
pub fn op_image_svg_info(#[string] svg_data: String) -> Result<SvgInfo, ImageToolsError> {
    debug!("image_tools.svg_info");
    svg::get_svg_info(&svg_data)
}

/// Load and parse an SVG, returning its info
#[weld_op]
#[op2]
#[serde]
pub fn op_image_svg_load(#[string] svg_data: String) -> Result<SvgInfo, ImageToolsError> {
    debug!("image_tools.svg_load");
    svg::get_svg_info(&svg_data)
}

// ============================================================================
// WebP Ops (for app asset optimization only)
// ============================================================================

/// Encode image as WebP (for app asset optimization)
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_webp_encode(
    #[buffer] data: &[u8],
    quality: u8,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(quality = quality, "image_tools.webp_encode");
    webp::encode_webp(data, quality)
}

/// Decode WebP to PNG bytes
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_webp_decode(#[buffer] data: &[u8]) -> Result<Vec<u8>, ImageToolsError> {
    debug!("image_tools.webp_decode");
    webp::decode_webp(data)
}

/// Get information about a WebP image
#[weld_op]
#[op2]
#[serde]
pub fn op_image_webp_info(#[buffer] data: &[u8]) -> Result<WebPInfo, ImageToolsError> {
    debug!("image_tools.webp_info");
    webp::get_webp_info(data)
}

// ============================================================================
// Convert Ops
// ============================================================================

/// Convert SVG to PNG at specified dimensions
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_svg_to_png(
    #[string] svg_data: String,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(width = width, height = height, "image_tools.svg_to_png");
    convert::svg_to_png(&svg_data, width, height)
}

/// Convert PNG(s) to ICO format
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_png_to_ico(#[serde] png_data: Vec<Vec<u8>>) -> Result<Vec<u8>, ImageToolsError> {
    debug!(count = png_data.len(), "image_tools.png_to_ico");
    convert::png_to_ico(&png_data)
}

/// Extract images from ICO as PNG bytes
#[weld_op]
#[op2]
#[serde]
pub fn op_image_ico_extract(#[buffer] ico_data: &[u8]) -> Result<Vec<Vec<u8>>, ImageToolsError> {
    debug!("image_tools.ico_extract");
    convert::ico_extract(ico_data)
}

/// Create a complete favicon set from source PNG
#[weld_op]
#[op2]
#[serde]
pub fn op_image_favicon_create(#[buffer] png_data: &[u8]) -> Result<FaviconSet, ImageToolsError> {
    debug!("image_tools.favicon_create");
    convert::create_favicon_set(png_data)
}

/// Convert PNG to WebP (for app asset optimization)
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_png_to_webp(
    #[buffer] data: &[u8],
    quality: u8,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(quality = quality, "image_tools.png_to_webp");
    convert::png_to_webp(data, quality)
}

// ============================================================================
// Transform Ops
// ============================================================================

/// Resize image to exact dimensions
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_resize(
    #[buffer] data: &[u8],
    width: u32,
    height: u32,
    #[serde] filter: Option<FilterType>,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(width = width, height = height, "image_tools.resize");
    transform::resize_image(data, width, height, filter.unwrap_or_default())
}

/// Scale image by a factor
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_scale(#[buffer] data: &[u8], factor: f64) -> Result<Vec<u8>, ImageToolsError> {
    debug!(factor = factor, "image_tools.scale");
    transform::scale_image(data, factor)
}

/// Crop a region from an image
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_crop(
    #[buffer] data: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(
        x = x,
        y = y,
        width = width,
        height = height,
        "image_tools.crop"
    );
    transform::crop_image(data, x, y, width, height)
}

/// Rotate image by 90, 180, or 270 degrees
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_rotate(#[buffer] data: &[u8], degrees: u32) -> Result<Vec<u8>, ImageToolsError> {
    debug!(degrees = degrees, "image_tools.rotate");
    transform::rotate_image(data, degrees)
}

/// Flip image horizontally or vertically
#[weld_op]
#[op2]
#[buffer]
pub fn op_image_flip(
    #[buffer] data: &[u8],
    #[serde] direction: FlipDirection,
) -> Result<Vec<u8>, ImageToolsError> {
    debug!(?direction, "image_tools.flip");
    let horizontal = matches!(direction, FlipDirection::Horizontal);
    transform::flip_image(data, horizontal)
}

// ============================================================================
// Extension Setup
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn image_tools_extension() -> Extension {
    runtime_image_tools::ext()
}

/// Initialize the image_tools extension state
pub fn init_image_tools_state(state: &mut OpState) {
    state.put(ImageToolsState::default());
}

// Re-export forge_weld for the macros
pub use forge_weld;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use fixtures::*;

    // ========================================================================
    // Error Type Tests
    // ========================================================================

    #[test]
    fn test_error_codes() {
        let err = ImageToolsError::png_error("test error");
        match err {
            ImageToolsError::PngError { code, message } => {
                assert_eq!(code, ImageToolsErrorCode::PngError as u32);
                assert_eq!(message, "test error");
            }
            _ => panic!("Wrong error type"),
        }

        let err = ImageToolsError::svg_error("svg error");
        match err {
            ImageToolsError::SvgError { code, .. } => {
                assert_eq!(code, ImageToolsErrorCode::SvgError as u32);
            }
            _ => panic!("Wrong error type"),
        }

        let err = ImageToolsError::webp_error("webp error");
        match err {
            ImageToolsError::WebPError { code, .. } => {
                assert_eq!(code, ImageToolsErrorCode::WebPError as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    // ========================================================================
    // PNG Tests
    // ========================================================================

    #[test]
    fn test_png_load() {
        let png_data = png_rgba_4x4();
        let result = png::load_png(&png_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_png_info() {
        let png_data = png_rgba_4x4();
        let info = png::get_png_info(&png_data).unwrap();

        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
        assert_eq!(info.format, "png");
        assert!(info.has_alpha);
        assert_eq!(info.color_type, "RGBA8");
    }

    #[test]
    fn test_png_info_rgb_no_alpha() {
        let png_data = png_rgb_64x64();
        let info = png::get_png_info(&png_data).unwrap();

        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
        assert!(!info.has_alpha);
        assert_eq!(info.color_type, "RGB8");
    }

    #[test]
    fn test_png_save() {
        let png_data = png_rgba_4x4();
        let img = png::load_png(&png_data).unwrap();
        let saved = png::save_png(&img, None).unwrap();

        // Verify the saved data is valid PNG
        let info = png::get_png_info(&saved).unwrap();
        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
    }

    #[test]
    fn test_png_optimize() {
        let png_data = png_rgba_4x4();
        let optimized = png::optimize_png(&png_data).unwrap();

        // Verify the optimized data is still valid PNG
        let info = png::get_png_info(&optimized).unwrap();
        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
    }

    #[test]
    fn test_png_invalid_data() {
        let result = png::load_png(&invalid_image_data());
        assert!(result.is_err());
    }

    // ========================================================================
    // SVG Tests
    // ========================================================================

    #[test]
    fn test_svg_load() {
        let svg_data = svg_100x100();
        let result = svg::load_svg(&svg_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_svg_info() {
        let svg_data = svg_100x100();
        let info = svg::get_svg_info(&svg_data).unwrap();

        assert_eq!(info.width, 100.0);
        assert_eq!(info.height, 100.0);
    }

    #[test]
    fn test_svg_info_with_viewbox() {
        let svg_data = svg_with_offset_viewbox();
        let info = svg::get_svg_info(&svg_data).unwrap();

        assert_eq!(info.width, 200.0);
        assert_eq!(info.height, 200.0);
        // ViewBox is different from size, so it should be Some
        assert!(info.view_box.is_some());
        let vb = info.view_box.unwrap();
        assert_eq!(vb.x, 0.0);
        assert_eq!(vb.y, 50.0);
        assert_eq!(vb.width, 100.0);
        assert_eq!(vb.height, 100.0);
    }

    #[test]
    fn test_svg_complex() {
        let svg_data = svg_complex();
        let info = svg::get_svg_info(&svg_data).unwrap();

        assert_eq!(info.width, 128.0);
        assert_eq!(info.height, 128.0);
    }

    #[test]
    fn test_svg_invalid_data() {
        let result = svg::load_svg(&svg_invalid());
        assert!(result.is_err());
    }

    // ========================================================================
    // WebP Tests
    // ========================================================================

    #[test]
    fn test_webp_encode_lossless() {
        let png_data = png_rgba_4x4();
        let webp_data = webp::encode_webp(&png_data, 100).unwrap();

        // Verify we got WebP data (starts with RIFF...WEBP)
        assert!(webp_data.len() > 12);
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");
    }

    #[test]
    fn test_webp_encode_lossy() {
        let png_data = png_rgba_4x4();
        let webp_data = webp::encode_webp(&png_data, 80).unwrap();

        // Verify we got WebP data
        assert!(webp_data.len() > 12);
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");
    }

    #[test]
    fn test_webp_encode_with_transparency() {
        let png_data = png_rgba_4x4_with_transparency();
        let webp_data = webp::encode_webp(&png_data, 100).unwrap();

        // Verify WebP header
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");
    }

    #[test]
    fn test_webp_decode() {
        let png_data = png_rgba_4x4();
        let webp_data = webp::encode_webp(&png_data, 100).unwrap();
        let decoded = webp::decode_webp(&webp_data).unwrap();

        // Verify the decoded data is valid PNG
        let info = png::get_png_info(&decoded).unwrap();
        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
    }

    #[test]
    fn test_webp_info() {
        let png_data = png_rgba_4x4_with_transparency();
        let webp_data = webp::encode_webp(&png_data, 100).unwrap();
        let info = webp::get_webp_info(&webp_data).unwrap();

        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
        // Note: Alpha preservation depends on WebP encoder behavior
    }

    #[test]
    fn test_webp_encode_rgb_no_alpha() {
        let png_data = png_rgb_64x64();
        let webp_data = webp::encode_webp(&png_data, 80).unwrap();

        // Verify WebP header
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");

        let info = webp::get_webp_info(&webp_data).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    // ========================================================================
    // Convert Tests
    // ========================================================================

    #[test]
    fn test_svg_to_png() {
        let svg_data = svg_100x100();
        let png_data = convert::svg_to_png(&svg_data, 50, 50).unwrap();

        // Verify the result is valid PNG with correct dimensions
        let info = png::get_png_info(&png_data).unwrap();
        assert_eq!(info.width, 50);
        assert_eq!(info.height, 50);
    }

    #[test]
    fn test_svg_to_png_different_sizes() {
        let svg_data = svg_100x100();

        // Test various output sizes
        for (w, h) in [(16, 16), (32, 32), (128, 128), (256, 256)] {
            let png_data = convert::svg_to_png(&svg_data, w, h).unwrap();
            let info = png::get_png_info(&png_data).unwrap();
            assert_eq!(info.width, w);
            assert_eq!(info.height, h);
        }
    }

    #[test]
    fn test_svg_complex_to_png() {
        let svg_data = svg_complex();
        let png_data = convert::svg_to_png(&svg_data, 64, 64).unwrap();

        let info = png::get_png_info(&png_data).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_png_to_ico_single_image() {
        let png_data = png_rgba_64x64();
        let ico_data = convert::png_to_ico(&[png_data]).unwrap();

        // Verify ICO header (starts with 0, 0, 1, 0 for ICO type)
        assert!(ico_data.len() > 6);
        assert_eq!(&ico_data[0..4], &[0, 0, 1, 0]);
    }

    #[test]
    fn test_png_to_ico_multiple_images() {
        // Use multiple sizes from fixtures
        let png_16 = png_rgba_16x16();
        let png_64 = png_rgba_64x64();

        // Resize 64x64 to 32x32 for a middle size
        let png_32 = transform::resize_image(&png_64, 32, 32, FilterType::Lanczos3).unwrap();

        let ico_data = convert::png_to_ico(&[png_16, png_32]).unwrap();

        // Verify ICO header
        assert!(ico_data.len() > 6);
        assert_eq!(&ico_data[0..4], &[0, 0, 1, 0]);
    }

    #[test]
    fn test_ico_extract() {
        let png_data = png_rgba_64x64();
        let ico_data = convert::png_to_ico(&[png_data]).unwrap();

        let extracted = convert::ico_extract(&ico_data).unwrap();
        assert!(!extracted.is_empty());

        // Each extracted image should be valid PNG
        for png in &extracted {
            let info = png::get_png_info(png).unwrap();
            assert!(info.width > 0);
            assert!(info.height > 0);
        }
    }

    #[test]
    fn test_favicon_create() {
        let source_png = png_rgba_256x256_icon();
        let favicon_set = convert::create_favicon_set(&source_png).unwrap();

        // Verify all sizes
        let info16 = png::get_png_info(&favicon_set.favicon16).unwrap();
        assert_eq!(info16.width, 16);
        assert_eq!(info16.height, 16);

        let info32 = png::get_png_info(&favicon_set.favicon32).unwrap();
        assert_eq!(info32.width, 32);
        assert_eq!(info32.height, 32);

        let info48 = png::get_png_info(&favicon_set.favicon48).unwrap();
        assert_eq!(info48.width, 48);
        assert_eq!(info48.height, 48);

        let info180 = png::get_png_info(&favicon_set.apple180).unwrap();
        assert_eq!(info180.width, 180);
        assert_eq!(info180.height, 180);

        // Verify ICO is valid
        assert!(!favicon_set.ico.is_empty());
        assert_eq!(&favicon_set.ico[0..4], &[0, 0, 1, 0]);
    }

    #[test]
    fn test_favicon_create_too_small() {
        let small_png = png_rgba_4x4(); // 4x4, too small for favicon
        let result = convert::create_favicon_set(&small_png);
        assert!(result.is_err());
    }

    #[test]
    fn test_png_to_webp() {
        let png_data = png_rgba_4x4();
        let webp_data = convert::png_to_webp(&png_data, 80).unwrap();

        // Verify WebP header
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");
    }

    #[test]
    fn test_png_to_webp_lossless() {
        let png_data = png_rgba_64x64();
        let webp_data = convert::png_to_webp(&png_data, 100).unwrap();

        // Verify WebP header
        assert_eq!(&webp_data[0..4], b"RIFF");
        assert_eq!(&webp_data[8..12], b"WEBP");

        // Verify dimensions preserved
        let info = webp::get_webp_info(&webp_data).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    // ========================================================================
    // Transform Tests
    // ========================================================================

    #[test]
    fn test_resize_image() {
        let png_data = png_rgba_64x64(); // 64x64
        let resized = transform::resize_image(&png_data, 32, 32, FilterType::Lanczos3).unwrap();

        let info = png::get_png_info(&resized).unwrap();
        assert_eq!(info.width, 32);
        assert_eq!(info.height, 32);
    }

    #[test]
    fn test_resize_image_upscale() {
        let png_data = png_rgba_4x4(); // 4x4
        let resized = transform::resize_image(&png_data, 64, 64, FilterType::Lanczos3).unwrap();

        let info = png::get_png_info(&resized).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_resize_image_filter_types() {
        let png_data = png_rgba_64x64();

        for filter in [
            FilterType::Nearest,
            FilterType::Bilinear,
            FilterType::Lanczos3,
        ] {
            let resized = transform::resize_image(&png_data, 32, 32, filter).unwrap();
            let info = png::get_png_info(&resized).unwrap();
            assert_eq!(info.width, 32);
            assert_eq!(info.height, 32);
        }
    }

    #[test]
    fn test_scale_image() {
        let png_data = png_rgba_64x64(); // 64x64
        let scaled = transform::scale_image(&png_data, 0.5).unwrap();

        let info = png::get_png_info(&scaled).unwrap();
        assert_eq!(info.width, 32);
        assert_eq!(info.height, 32);
    }

    #[test]
    fn test_scale_image_upscale() {
        let png_data = png_rgba_4x4(); // 4x4
        let scaled = transform::scale_image(&png_data, 2.0).unwrap();

        let info = png::get_png_info(&scaled).unwrap();
        assert_eq!(info.width, 8);
        assert_eq!(info.height, 8);
    }

    #[test]
    fn test_scale_image_invalid_factor() {
        let png_data = png_rgba_4x4();
        let result = transform::scale_image(&png_data, 0.0);
        assert!(result.is_err());

        let result = transform::scale_image(&png_data, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_crop_image() {
        let png_data = png_rgba_64x64(); // 64x64
        let cropped = transform::crop_image(&png_data, 10, 10, 20, 20).unwrap();

        let info = png::get_png_info(&cropped).unwrap();
        assert_eq!(info.width, 20);
        assert_eq!(info.height, 20);
    }

    #[test]
    fn test_crop_image_at_edge() {
        let png_data = png_rgba_64x64(); // 64x64
                                         // Crop at bottom-right, should be clamped to available space
        let cropped = transform::crop_image(&png_data, 50, 50, 30, 30).unwrap();

        let info = png::get_png_info(&cropped).unwrap();
        assert_eq!(info.width, 14); // Clamped: 64 - 50 = 14
        assert_eq!(info.height, 14);
    }

    #[test]
    fn test_crop_image_invalid_position() {
        let png_data = png_rgba_64x64(); // 64x64
        let result = transform::crop_image(&png_data, 100, 100, 10, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_rotate_image_90() {
        let png_data = png_rgba_64x64(); // 64x64
        let rotated = transform::rotate_image(&png_data, 90).unwrap();

        let info = png::get_png_info(&rotated).unwrap();
        // Rotation swaps dimensions for non-square, but 64x64 stays 64x64
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_rotate_image_180() {
        let png_data = png_rgba_64x64();
        let rotated = transform::rotate_image(&png_data, 180).unwrap();

        let info = png::get_png_info(&rotated).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_rotate_image_270() {
        let png_data = png_rgba_64x64();
        let rotated = transform::rotate_image(&png_data, 270).unwrap();

        let info = png::get_png_info(&rotated).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_rotate_image_invalid_degrees() {
        let png_data = png_rgba_64x64();
        let result = transform::rotate_image(&png_data, 45);
        assert!(result.is_err());

        let result = transform::rotate_image(&png_data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_flip_horizontal() {
        let png_data = png_rgba_64x64();
        let flipped = transform::flip_image(&png_data, true).unwrap();

        let info = png::get_png_info(&flipped).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_flip_vertical() {
        let png_data = png_rgba_64x64();
        let flipped = transform::flip_image(&png_data, false).unwrap();

        let info = png::get_png_info(&flipped).unwrap();
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    // ========================================================================
    // Roundtrip Tests
    // ========================================================================

    #[test]
    fn test_png_webp_roundtrip() {
        let original_png = png_rgba_4x4();
        let original_info = png::get_png_info(&original_png).unwrap();

        // PNG -> WebP -> PNG
        let webp_data = webp::encode_webp(&original_png, 100).unwrap(); // Lossless
        let decoded_png = webp::decode_webp(&webp_data).unwrap();
        let decoded_info = png::get_png_info(&decoded_png).unwrap();

        assert_eq!(original_info.width, decoded_info.width);
        assert_eq!(original_info.height, decoded_info.height);
    }

    #[test]
    fn test_svg_png_resize_roundtrip() {
        let svg_data = svg_100x100();

        // SVG -> PNG -> Resize -> PNG info
        let png_256 = convert::svg_to_png(&svg_data, 256, 256).unwrap();
        let png_64 = transform::resize_image(&png_256, 64, 64, FilterType::Lanczos3).unwrap();
        let info = png::get_png_info(&png_64).unwrap();

        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
    }

    #[test]
    fn test_ico_roundtrip() {
        let png_data = png_rgba_64x64();

        // PNG -> ICO -> Extract -> PNG
        let ico_data = convert::png_to_ico(&[png_data]).unwrap();
        let extracted = convert::ico_extract(&ico_data).unwrap();

        assert!(!extracted.is_empty());
        for png in extracted {
            let info = png::get_png_info(&png).unwrap();
            assert!(info.width > 0);
            assert!(info.height > 0);
        }
    }

    #[test]
    fn test_rotate_non_square_image() {
        let png_data = png_rgba_32x64(); // 32x64 non-square
        let rotated = transform::rotate_image(&png_data, 90).unwrap();

        let info = png::get_png_info(&rotated).unwrap();
        // 90-degree rotation swaps dimensions
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 32);
    }

    #[test]
    fn test_transform_preserves_alpha() {
        let png_data = png_rgba_4x4_with_transparency();
        let original_info = png::get_png_info(&png_data).unwrap();
        assert!(original_info.has_alpha);

        // Test that various transforms preserve alpha
        let resized = transform::resize_image(&png_data, 8, 8, FilterType::Lanczos3).unwrap();
        let resized_info = png::get_png_info(&resized).unwrap();
        assert!(resized_info.has_alpha);

        let scaled = transform::scale_image(&png_data, 2.0).unwrap();
        let scaled_info = png::get_png_info(&scaled).unwrap();
        assert!(scaled_info.has_alpha);

        let flipped = transform::flip_image(&png_data, true).unwrap();
        let flipped_info = png::get_png_info(&flipped).unwrap();
        assert!(flipped_info.has_alpha);
    }
}
