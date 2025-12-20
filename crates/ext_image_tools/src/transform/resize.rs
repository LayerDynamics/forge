//! Image resizing operations

use crate::{FilterType as ImageFilterType, ImageToolsError};
use image::{imageops::FilterType, ImageFormat};
use std::io::Cursor;

/// Resize image to exact dimensions
pub fn resize_image(
    data: &[u8],
    width: u32,
    height: u32,
    filter: ImageFilterType,
) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::transform_error(format!("Failed to load image: {}", e)))?;

    let filter_type = match filter {
        ImageFilterType::Nearest => FilterType::Nearest,
        ImageFilterType::Bilinear => FilterType::Triangle,
        ImageFilterType::Lanczos3 => FilterType::Lanczos3,
    };

    let resized = img.resize_exact(width, height, filter_type);

    let mut buffer = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| {
            ImageToolsError::transform_error(format!("Failed to encode resized image: {}", e))
        })?;

    Ok(buffer)
}

/// Rotate image by specified degrees (90, 180, 270)
pub fn rotate_image(data: &[u8], degrees: u32) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::transform_error(format!("Failed to load image: {}", e)))?;

    let rotated = match degrees {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => {
            return Err(ImageToolsError::invalid_input(format!(
                "Rotation must be 90, 180, or 270 degrees, got {}",
                degrees
            )));
        }
    };

    let mut buffer = Vec::new();
    rotated
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| {
            ImageToolsError::transform_error(format!("Failed to encode rotated image: {}", e))
        })?;

    Ok(buffer)
}

/// Flip image horizontally or vertically
pub fn flip_image(data: &[u8], horizontal: bool) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::transform_error(format!("Failed to load image: {}", e)))?;

    let flipped = if horizontal { img.fliph() } else { img.flipv() };

    let mut buffer = Vec::new();
    flipped
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| {
            ImageToolsError::transform_error(format!("Failed to encode flipped image: {}", e))
        })?;

    Ok(buffer)
}
