//! Image cropping operations

use crate::ImageToolsError;
use image::{GenericImageView, ImageFormat};
use std::io::Cursor;

/// Crop a region from an image
///
/// # Arguments
/// * `data` - Source image bytes
/// * `x` - Left edge of crop region
/// * `y` - Top edge of crop region
/// * `width` - Width of crop region
/// * `height` - Height of crop region
pub fn crop_image(
    data: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageToolsError> {
    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::transform_error(format!("Failed to load image: {}", e)))?;

    let (img_width, img_height) = img.dimensions();

    // Validate crop region
    if x >= img_width || y >= img_height {
        return Err(ImageToolsError::invalid_input(format!(
            "Crop position ({}, {}) is outside image bounds ({}x{})",
            x, y, img_width, img_height
        )));
    }

    // Clamp width/height to image bounds
    let actual_width = width.min(img_width - x);
    let actual_height = height.min(img_height - y);

    if actual_width == 0 || actual_height == 0 {
        return Err(ImageToolsError::invalid_input(
            "Crop region would have zero dimensions",
        ));
    }

    let cropped = img.crop_imm(x, y, actual_width, actual_height);

    let mut buffer = Vec::new();
    cropped
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| {
            ImageToolsError::transform_error(format!("Failed to encode cropped image: {}", e))
        })?;

    Ok(buffer)
}
