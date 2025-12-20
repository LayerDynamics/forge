//! Favicon set generation for web applications

use crate::convert::ico::png_to_ico;
use crate::{FaviconSet, ImageToolsError};
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Standard favicon sizes
const FAVICON_16: u32 = 16;
const FAVICON_32: u32 = 32;
const FAVICON_48: u32 = 48;
const APPLE_TOUCH_ICON: u32 = 180;

/// Create a complete favicon set from a source PNG image
///
/// Generates:
/// - favicon-16x16.png
/// - favicon-32x32.png
/// - favicon-48x48.png (for high-DPI)
/// - apple-touch-icon.png (180x180)
/// - favicon.ico (multi-size ICO)
pub fn create_favicon_set(png_data: &[u8]) -> Result<FaviconSet, ImageToolsError> {
    let img = image::load_from_memory(png_data)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to load source PNG: {}", e)))?;

    let (width, height) = img.dimensions();
    if width != height {
        return Err(ImageToolsError::convert_error(format!(
            "Source image must be square for favicon generation, got {}x{}",
            width, height
        )));
    }

    if width < APPLE_TOUCH_ICON {
        return Err(ImageToolsError::convert_error(format!(
            "Source image must be at least {}x{} pixels, got {}x{}",
            APPLE_TOUCH_ICON, APPLE_TOUCH_ICON, width, height
        )));
    }

    // Generate all sizes
    let favicon16 = resize_to_png(&img, FAVICON_16)?;
    let favicon32 = resize_to_png(&img, FAVICON_32)?;
    let favicon48 = resize_to_png(&img, FAVICON_48)?;
    let apple180 = resize_to_png(&img, APPLE_TOUCH_ICON)?;

    // Create ICO with multiple sizes
    let ico = png_to_ico(&[favicon16.clone(), favicon32.clone(), favicon48.clone()])?;

    Ok(FaviconSet {
        favicon16,
        favicon32,
        favicon48,
        apple180,
        ico,
    })
}

/// Resize image and encode as PNG
fn resize_to_png(img: &image::DynamicImage, size: u32) -> Result<Vec<u8>, ImageToolsError> {
    let resized = img.resize_exact(size, size, FilterType::Lanczos3);

    let mut buffer = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| ImageToolsError::convert_error(format!("Failed to encode PNG: {}", e)))?;

    Ok(buffer)
}
