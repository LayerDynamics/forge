//! SVG to PNG conversion

use crate::ImageToolsError;
use image::{ImageFormat, RgbaImage};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::io::Cursor;

/// Convert SVG string to PNG bytes at specified dimensions
pub fn svg_to_png(svg_data: &str, width: u32, height: u32) -> Result<Vec<u8>, ImageToolsError> {
    // Parse SVG
    let options = Options::default();
    let tree = Tree::from_str(svg_data, &options)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to parse SVG: {}", e)))?;

    // Create pixmap for rendering
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| ImageToolsError::convert_error("Failed to create pixmap"))?;

    // Calculate scale to fit SVG into target dimensions (resvg 0.39 uses fields, not methods)
    let svg_size = tree.size;
    let scale_x = width as f32 / svg_size.width();
    let scale_y = height as f32 / svg_size.height();

    // Render SVG to pixmap
    let transform = Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to PNG bytes
    let img = RgbaImage::from_raw(width, height, pixmap.take())
        .ok_or_else(|| ImageToolsError::convert_error("Failed to create image from pixmap"))?;

    let mut buffer = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to encode PNG: {}", e)))?;

    Ok(buffer)
}

/// Convert PNG bytes to WebP bytes (for app asset optimization)
///
/// # Arguments
/// * `data` - PNG image bytes to convert
/// * `quality` - Quality level (0-100), where 100+ is lossless
pub fn png_to_webp(data: &[u8], quality: u8) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to load image: {}", e)))?;

    // Use the webp crate for encoding with quality control
    let encoder = webp::Encoder::from_image(&img).map_err(|e| {
        ImageToolsError::convert_error(format!("Failed to create WebP encoder: {}", e))
    })?;

    let webp_data = if quality >= 100 {
        // Lossless encoding
        encoder.encode_lossless()
    } else {
        // Lossy encoding with specified quality
        encoder.encode(quality as f32)
    };

    Ok(webp_data.to_vec())
}
