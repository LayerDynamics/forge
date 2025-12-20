//! WebP encoding, decoding, and analysis operations
//!
//! Note: WebP is intended for app asset optimization only,
//! NOT for icons or bundle-specific formats.

use crate::{ImageToolsError, WebPInfo};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Encode image data as WebP
///
/// # Arguments
/// * `data` - PNG image bytes to convert
/// * `quality` - Quality level (0-100), where 100+ is lossless
pub fn encode_webp(data: &[u8], quality: u8) -> Result<Vec<u8>, ImageToolsError> {
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

/// Decode WebP to PNG bytes
pub fn decode_webp(data: &[u8]) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to decode WebP: {}", e)))?;

    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to encode PNG: {}", e)))?;

    Ok(buffer)
}

/// Get information about a WebP image
pub fn get_webp_info(data: &[u8]) -> Result<WebPInfo, ImageToolsError> {
    let img = image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to load WebP: {}", e)))?;

    let (width, height) = img.dimensions();

    let has_alpha = matches!(
        &img,
        DynamicImage::ImageRgba8(_)
            | DynamicImage::ImageRgba16(_)
            | DynamicImage::ImageRgba32F(_)
            | DynamicImage::ImageLumaA8(_)
            | DynamicImage::ImageLumaA16(_)
    );

    // Note: We can't easily determine if WebP was lossless from decoded data
    // This would require parsing the WebP container directly
    Ok(WebPInfo {
        width,
        height,
        has_alpha,
        is_lossless: false, // Cannot determine from decoded image
    })
}
