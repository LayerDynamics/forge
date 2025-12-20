//! Test fixtures for ext_image_tools
//!
//! Provides common test images and data for unit tests.

use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use std::io::Cursor;

/// Fixture sizes for standard icon dimensions
pub mod sizes {
    pub const TINY: u32 = 4;
    pub const SMALL: u32 = 16;
    pub const MEDIUM: u32 = 64;
    pub const LARGE: u32 = 256;
    pub const FAVICON_MIN: u32 = 180;
}

/// Create a minimal 4x4 PNG with RGBA (has alpha)
pub fn png_rgba_4x4() -> Vec<u8> {
    let mut img = RgbaImage::new(sizes::TINY, sizes::TINY);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 0, 0, 255]); // Solid red
    }
    encode_png_rgba(&img)
}

/// Create a 4x4 PNG with transparency (semi-transparent red)
pub fn png_rgba_4x4_with_transparency() -> Vec<u8> {
    let mut img = RgbaImage::new(sizes::TINY, sizes::TINY);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 0, 0, 128]); // 50% transparent red
    }
    encode_png_rgba(&img)
}

/// Create a 16x16 PNG suitable for small icon
pub fn png_rgba_16x16() -> Vec<u8> {
    let mut img = RgbaImage::new(sizes::SMALL, sizes::SMALL);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Create a simple gradient
        *pixel = Rgba([(x * 16) as u8, (y * 16) as u8, 128, 255]);
    }
    encode_png_rgba(&img)
}

/// Create a 64x64 PNG for transform tests
pub fn png_rgba_64x64() -> Vec<u8> {
    let mut img = RgbaImage::new(sizes::MEDIUM, sizes::MEDIUM);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255]);
    }
    encode_png_rgba(&img)
}

/// Create a 256x256 PNG suitable for favicon source
pub fn png_rgba_256x256_icon() -> Vec<u8> {
    let mut img = RgbaImage::new(sizes::LARGE, sizes::LARGE);
    let center = (sizes::LARGE / 2) as i32;
    let radius = 100i32;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let dx = x as i32 - center;
        let dy = y as i32 - center;
        let in_circle = dx * dx + dy * dy < radius * radius;

        *pixel = if in_circle {
            Rgba([0, 120, 255, 255]) // Blue circle
        } else {
            Rgba([255, 255, 255, 0]) // Transparent background
        };
    }
    encode_png_rgba(&img)
}

/// Create a PNG without alpha channel (RGB only)
pub fn png_rgb_64x64() -> Vec<u8> {
    let mut img = RgbImage::new(sizes::MEDIUM, sizes::MEDIUM);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgb([(x * 4) as u8, (y * 4) as u8, 128]);
    }

    let mut buffer = Vec::new();
    DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .expect("Failed to encode PNG");
    buffer
}

/// Create a non-square PNG for testing aspect ratio handling
pub fn png_rgba_32x64() -> Vec<u8> {
    let mut img = RgbaImage::new(32, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgba([(x * 8) as u8, (y * 4) as u8, 200, 255]);
    }
    encode_png_rgba(&img)
}

/// Standard SVG fixture - 100x100 with a blue rectangle
pub fn svg_100x100() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <rect x="10" y="10" width="80" height="80" fill="blue"/>
    </svg>"#
        .to_string()
}

/// SVG with different viewBox than dimensions
pub fn svg_with_offset_viewbox() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 50 100 100">
        <circle cx="50" cy="100" r="40" fill="green"/>
    </svg>"#
        .to_string()
}

/// SVG with complex paths
pub fn svg_complex() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
        <defs>
            <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
                <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
            </linearGradient>
        </defs>
        <circle cx="64" cy="64" r="60" fill="url(#grad)"/>
        <rect x="44" y="44" width="40" height="40" fill="white" rx="5"/>
    </svg>"#
        .to_string()
}

/// Invalid/malformed SVG for error testing
pub fn svg_invalid() -> String {
    "not valid svg content".to_string()
}

/// Invalid PNG data for error testing
pub fn invalid_image_data() -> Vec<u8> {
    vec![0, 1, 2, 3, 4, 5]
}

// Helper to encode RGBA image to PNG bytes
fn encode_png_rgba(img: &RgbaImage) -> Vec<u8> {
    let mut buffer = Vec::new();
    DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .expect("Failed to encode PNG");
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_are_valid_png() {
        // All PNG fixtures should decode successfully
        let fixtures = [
            ("png_rgba_4x4", png_rgba_4x4()),
            (
                "png_rgba_4x4_with_transparency",
                png_rgba_4x4_with_transparency(),
            ),
            ("png_rgba_16x16", png_rgba_16x16()),
            ("png_rgba_64x64", png_rgba_64x64()),
            ("png_rgba_256x256_icon", png_rgba_256x256_icon()),
            ("png_rgb_64x64", png_rgb_64x64()),
            ("png_rgba_32x64", png_rgba_32x64()),
        ];

        for (name, data) in fixtures {
            let result = image::load_from_memory(&data);
            assert!(result.is_ok(), "Fixture {} failed to load", name);
        }
    }

    #[test]
    fn test_fixtures_have_correct_dimensions() {
        let img = image::load_from_memory(&png_rgba_4x4()).unwrap();
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);

        let img = image::load_from_memory(&png_rgba_64x64()).unwrap();
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);

        let img = image::load_from_memory(&png_rgba_256x256_icon()).unwrap();
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);

        let img = image::load_from_memory(&png_rgba_32x64()).unwrap();
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 64);
    }
}
