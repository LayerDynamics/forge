//! PNG image loading, saving, and analysis operations

use crate::{ImageInfo, ImageToolsError, PngSaveOptions};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Load PNG from bytes and return the image data
pub fn load_png(data: &[u8]) -> Result<DynamicImage, ImageToolsError> {
    image::load_from_memory_with_format(data, ImageFormat::Png)
        .map_err(|e| ImageToolsError::png_error(format!("Failed to load PNG: {}", e)))
}

/// Get information about a PNG image
pub fn get_png_info(data: &[u8]) -> Result<ImageInfo, ImageToolsError> {
    let img = load_png(data)?;
    let (width, height) = img.dimensions();

    let (has_alpha, color_type) = match &img {
        DynamicImage::ImageRgba8(_) => (true, "RGBA8"),
        DynamicImage::ImageRgba16(_) => (true, "RGBA16"),
        DynamicImage::ImageRgba32F(_) => (true, "RGBA32F"),
        DynamicImage::ImageLumaA8(_) => (true, "GrayAlpha8"),
        DynamicImage::ImageLumaA16(_) => (true, "GrayAlpha16"),
        DynamicImage::ImageRgb8(_) => (false, "RGB8"),
        DynamicImage::ImageRgb16(_) => (false, "RGB16"),
        DynamicImage::ImageRgb32F(_) => (false, "RGB32F"),
        DynamicImage::ImageLuma8(_) => (false, "Gray8"),
        DynamicImage::ImageLuma16(_) => (false, "Gray16"),
        _ => (false, "Unknown"),
    };

    Ok(ImageInfo {
        width,
        height,
        format: "png".to_string(),
        has_alpha,
        color_type: color_type.to_string(),
    })
}

/// Save image as PNG bytes
pub fn save_png(
    img: &DynamicImage,
    _options: Option<PngSaveOptions>,
) -> Result<Vec<u8>, ImageToolsError> {
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| ImageToolsError::png_error(format!("Failed to save PNG: {}", e)))?;
    Ok(buffer)
}

/// Optimize PNG by re-encoding (removes unnecessary metadata, applies basic compression)
pub fn optimize_png(data: &[u8]) -> Result<Vec<u8>, ImageToolsError> {
    // Load and re-encode to strip metadata and apply default compression
    let img = load_png(data)?;
    save_png(&img, None)
}
