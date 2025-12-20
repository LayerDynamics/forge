//! Image scaling operations

use crate::ImageToolsError;
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Scale image by a factor (e.g., 0.5 = half size, 2.0 = double size)
pub fn scale_image(data: &[u8], factor: f64) -> Result<Vec<u8>, ImageToolsError> {
    if factor <= 0.0 {
        return Err(ImageToolsError::invalid_input(
            "Scale factor must be positive",
        ));
    }

    let img = image::load_from_memory(data)
        .map_err(|e| ImageToolsError::transform_error(format!("Failed to load image: {}", e)))?;

    let (width, height) = img.dimensions();
    let new_width = ((width as f64) * factor).round() as u32;
    let new_height = ((height as f64) * factor).round() as u32;

    if new_width == 0 || new_height == 0 {
        return Err(ImageToolsError::invalid_input(
            "Scaled dimensions would be zero",
        ));
    }

    let scaled = img.resize_exact(new_width, new_height, FilterType::Lanczos3);

    let mut buffer = Vec::new();
    scaled
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| {
            ImageToolsError::transform_error(format!("Failed to encode scaled image: {}", e))
        })?;

    Ok(buffer)
}
