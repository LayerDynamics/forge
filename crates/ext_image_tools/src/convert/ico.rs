//! ICO format conversion

use crate::ImageToolsError;
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat, RgbaImage};
use std::io::Cursor;

/// Standard ICO sizes for Windows icons
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

/// Convert multiple PNG images to ICO format
///
/// If a single PNG is provided, it will be resized to standard ICO sizes.
/// If multiple PNGs are provided, they are used as-is (should be different sizes).
pub fn png_to_ico(png_data: &[Vec<u8>]) -> Result<Vec<u8>, ImageToolsError> {
    if png_data.is_empty() {
        return Err(ImageToolsError::ico_error("No PNG data provided"));
    }

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    if png_data.len() == 1 {
        // Single image - resize to standard sizes
        let img = image::load_from_memory(&png_data[0])
            .map_err(|e| ImageToolsError::ico_error(format!("Failed to load PNG: {}", e)))?;

        for &size in ICO_SIZES {
            let resized = img.resize_exact(size, size, FilterType::Lanczos3);
            let rgba = resized.to_rgba8();
            add_image_to_icon(&mut icon_dir, &rgba, size)?;
        }
    } else {
        // Multiple images - use as provided
        for data in png_data {
            let img = image::load_from_memory(data)
                .map_err(|e| ImageToolsError::ico_error(format!("Failed to load PNG: {}", e)))?;
            let (width, height) = img.dimensions();
            if width != height {
                return Err(ImageToolsError::ico_error(format!(
                    "ICO images must be square, got {}x{}",
                    width, height
                )));
            }
            let rgba = img.to_rgba8();
            add_image_to_icon(&mut icon_dir, &rgba, width)?;
        }
    }

    // Write ICO to buffer
    let mut buffer = Vec::new();
    icon_dir
        .write(&mut buffer)
        .map_err(|e| ImageToolsError::ico_error(format!("Failed to write ICO: {}", e)))?;

    Ok(buffer)
}

/// Add an RGBA image to the icon directory
fn add_image_to_icon(
    icon_dir: &mut ico::IconDir,
    rgba: &RgbaImage,
    size: u32,
) -> Result<(), ImageToolsError> {
    // Convert to PNG bytes first
    let mut png_buffer = Vec::new();
    DynamicImage::ImageRgba8(rgba.clone())
        .write_to(&mut Cursor::new(&mut png_buffer), ImageFormat::Png)
        .map_err(|e| ImageToolsError::ico_error(format!("Failed to encode PNG for ICO: {}", e)))?;

    // Create ICO image entry from PNG
    let image = ico::IconImage::read_png(&mut Cursor::new(&png_buffer))
        .map_err(|e| ImageToolsError::ico_error(format!("Failed to create ICO entry: {}", e)))?;

    // Add entry
    let entry = ico::IconDirEntry::encode(&image)
        .map_err(|e| ImageToolsError::ico_error(format!("Failed to encode ICO entry: {}", e)))?;

    icon_dir.add_entry(entry);

    // Size validation
    let _ = size; // Used for documentation, actual size comes from image

    Ok(())
}

/// Extract images from an ICO file as PNG bytes
pub fn ico_extract(ico_data: &[u8]) -> Result<Vec<Vec<u8>>, ImageToolsError> {
    let icon_dir = ico::IconDir::read(&mut Cursor::new(ico_data))
        .map_err(|e| ImageToolsError::ico_error(format!("Failed to read ICO: {}", e)))?;

    let mut pngs = Vec::new();

    for entry in icon_dir.entries() {
        let image = entry.decode().map_err(|e| {
            ImageToolsError::ico_error(format!("Failed to decode ICO entry: {}", e))
        })?;

        // Convert to PNG
        let mut png_buffer = Vec::new();
        image
            .write_png(&mut png_buffer)
            .map_err(|e| ImageToolsError::ico_error(format!("Failed to encode PNG: {}", e)))?;

        pngs.push(png_buffer);
    }

    Ok(pngs)
}
