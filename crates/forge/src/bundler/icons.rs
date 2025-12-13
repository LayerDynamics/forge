//! Icon processing utilities for platform packaging
//!
//! Handles icon loading, resizing, and format conversion for:
//! - Windows: Multiple scaled PNG icons for MSIX
//! - macOS: .icns format via sips/iconutil
//! - Linux: PNG in various sizes for hicolor theme
//!
//! ## Icon Requirements
//!
//! Apps MUST provide an icon. The recommended format is:
//! - **Format**: PNG with transparency (RGBA)
//! - **Size**: 1024x1024 pixels (minimum 512x512)
//! - **Shape**: Square (1:1 aspect ratio)
//!
//! Place your icon at one of these locations:
//! - `assets/icon.png` (recommended)
//! - `icon.png` (root of app directory)
//! - Or specify path in manifest: `[bundle] icon = "path/to/icon"`

use anyhow::{bail, Context, Result};
use image::{imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Minimum recommended icon size
pub const MIN_ICON_SIZE: u32 = 512;
/// Optimal icon size for best quality across all platforms
pub const RECOMMENDED_ICON_SIZE: u32 = 1024;

/// Icon validation result
#[derive(Debug)]
pub struct IconValidation {
    #[allow(dead_code)] // Part of public API
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub is_square: bool,
    pub meets_minimum: bool,
    pub meets_recommended: bool,
    pub has_transparency: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Icon processor for multi-platform icon generation
pub struct IconProcessor {
    source_image: DynamicImage,
}

/// Required MSIX icon sizes (base size, scale factors)
#[cfg(target_os = "windows")]
pub struct MsixIconSet {
    pub square44: Vec<(u32, u32, String)>, // (width, height, filename)
    pub square150: Vec<(u32, u32, String)>,
    pub wide310x150: Vec<(u32, u32, String)>,
    pub store_logo: Vec<(u32, u32, String)>,
    pub splash_screen: Vec<(u32, u32, String)>,
}

#[cfg(target_os = "windows")]
impl MsixIconSet {
    pub fn new() -> Self {
        Self {
            square44: vec![
                (44, 44, "Square44x44Logo.scale-100.png".into()),
                (55, 55, "Square44x44Logo.scale-125.png".into()),
                (66, 66, "Square44x44Logo.scale-150.png".into()),
                (88, 88, "Square44x44Logo.scale-200.png".into()),
                (176, 176, "Square44x44Logo.scale-400.png".into()),
            ],
            square150: vec![
                (150, 150, "Square150x150Logo.scale-100.png".into()),
                (188, 188, "Square150x150Logo.scale-125.png".into()),
                (225, 225, "Square150x150Logo.scale-150.png".into()),
                (300, 300, "Square150x150Logo.scale-200.png".into()),
                (600, 600, "Square150x150Logo.scale-400.png".into()),
            ],
            wide310x150: vec![
                (310, 150, "Wide310x150Logo.scale-100.png".into()),
                (388, 188, "Wide310x150Logo.scale-125.png".into()),
                (465, 225, "Wide310x150Logo.scale-150.png".into()),
                (620, 300, "Wide310x150Logo.scale-200.png".into()),
            ],
            store_logo: vec![
                (50, 50, "StoreLogo.scale-100.png".into()),
                (63, 63, "StoreLogo.scale-125.png".into()),
                (75, 75, "StoreLogo.scale-150.png".into()),
                (100, 100, "StoreLogo.scale-200.png".into()),
            ],
            splash_screen: vec![
                (620, 300, "SplashScreen.scale-100.png".into()),
                (775, 375, "SplashScreen.scale-125.png".into()),
                (930, 450, "SplashScreen.scale-150.png".into()),
                (1240, 600, "SplashScreen.scale-200.png".into()),
            ],
        }
    }
}

#[cfg(target_os = "windows")]
impl Default for MsixIconSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IconProcessor {
    /// Load icon from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        let img = image::open(path)
            .with_context(|| format!("Failed to open icon: {}", path.display()))?;
        Ok(Self { source_image: img })
    }

    /// Get search paths for icon discovery
    pub fn get_search_paths(app_dir: &Path, icon_base: Option<&str>) -> Vec<PathBuf> {
        if let Some(base) = icon_base {
            vec![
                app_dir.join(format!("{}.png", base)),
                app_dir.join(format!("{}.icns", base)),
                app_dir.join(format!("{}.ico", base)),
                app_dir.join(base).join("icon.png"),
                app_dir.join(base).join("1024x1024.png"),
                app_dir.join(base),
            ]
        } else {
            vec![
                app_dir.join("assets/icon.png"),
                app_dir.join("assets/icon.icns"),
                app_dir.join("assets/icon.ico"),
                app_dir.join("icon.png"),
            ]
        }
    }

    /// Find and load icon from app directory
    ///
    /// Returns an error if no icon is found - apps MUST provide an icon.
    ///
    /// Search order:
    /// 1. bundle.icon path from manifest (e.g., "assets/icon" -> assets/icon.png)
    /// 2. assets/icon.png
    /// 3. assets/icon.icns
    /// 4. assets/icon.ico
    /// 5. icon.png
    pub fn find_icon(app_dir: &Path, icon_base: Option<&str>) -> Result<Self> {
        let candidates = Self::get_search_paths(app_dir, icon_base);
        let mut tried_paths = Vec::new();

        for candidate in &candidates {
            if candidate.exists() && candidate.is_file() {
                match Self::from_path(candidate) {
                    Ok(processor) => {
                        // Validate the icon
                        let validation = processor.validate(candidate);

                        // Print warnings
                        for warning in &validation.warnings {
                            eprintln!("  Warning: {}", warning);
                        }

                        // Check for errors
                        if !validation.errors.is_empty() {
                            for error in &validation.errors {
                                eprintln!("  Error: {}", error);
                            }
                            bail!(
                                "Icon validation failed for {}. Run 'forge icon validate {}' for details.",
                                candidate.display(),
                                app_dir.display()
                            );
                        }

                        println!("  Using icon: {}", candidate.display());
                        return Ok(processor);
                    }
                    Err(e) => {
                        tried_paths.push(format!(
                            "  - {} (failed to load: {})",
                            candidate.display(),
                            e
                        ));
                    }
                }
            } else {
                tried_paths.push(format!("  - {} (not found)", candidate.display()));
            }
        }

        // No icon found - return detailed error
        bail!(
            "No app icon found!\n\n\
            Forge requires an app icon for bundling. Please add one.\n\n\
            ICON REQUIREMENTS:\n\
            • Format: PNG with transparency (RGBA)\n\
            • Size: 1024x1024 pixels (minimum 512x512)\n\
            • Shape: Square (1:1 aspect ratio)\n\n\
            RECOMMENDED LOCATION:\n\
            • {}/assets/icon.png\n\n\
            Or specify in manifest.app.toml:\n\
            [bundle]\n\
            icon = \"path/to/icon\"\n\n\
            CREATE A PLACEHOLDER:\n\
            Run: forge icon create {}/assets/icon.png\n\n\
            Searched locations:\n{}",
            app_dir.display(),
            app_dir.display(),
            tried_paths.join("\n")
        )
    }

    /// Validate an icon and return detailed results
    pub fn validate(&self, path: &Path) -> IconValidation {
        let width = self.source_image.width();
        let height = self.source_image.height();
        let is_square = width == height;
        let meets_minimum = width >= MIN_ICON_SIZE && height >= MIN_ICON_SIZE;
        let meets_recommended = width >= RECOMMENDED_ICON_SIZE && height >= RECOMMENDED_ICON_SIZE;

        // Check for alpha channel
        let has_transparency = matches!(
            self.source_image,
            DynamicImage::ImageRgba8(_)
                | DynamicImage::ImageRgba16(_)
                | DynamicImage::ImageLumaA8(_)
                | DynamicImage::ImageLumaA16(_)
        );

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check size
        if !meets_minimum {
            errors.push(format!(
                "Icon is too small ({}x{}). Minimum size is {}x{} pixels.",
                width, height, MIN_ICON_SIZE, MIN_ICON_SIZE
            ));
        } else if !meets_recommended {
            warnings.push(format!(
                "Icon is {}x{}. Recommended size is {}x{} for best quality.",
                width, height, RECOMMENDED_ICON_SIZE, RECOMMENDED_ICON_SIZE
            ));
        }

        // Check aspect ratio
        if !is_square {
            errors.push(format!(
                "Icon must be square. Current size: {}x{} (aspect ratio: {:.2}:1)",
                width,
                height,
                width as f32 / height as f32
            ));
        }

        // Check transparency
        if !has_transparency {
            warnings.push(
                "Icon does not have an alpha channel. Consider using PNG with transparency."
                    .to_string(),
            );
        }

        IconValidation {
            path: path.to_path_buf(),
            width,
            height,
            is_square,
            meets_minimum,
            meets_recommended,
            has_transparency,
            warnings,
            errors,
        }
    }

    /// Create a placeholder icon (for `forge icon create` command)
    pub fn create_placeholder(size: u32) -> Self {
        let mut img = RgbaImage::new(size, size);

        // Create a blue-purple gradient
        for y in 0..size {
            for x in 0..size {
                let gradient_x = x as f32 / size as f32;
                let gradient_y = y as f32 / size as f32;
                let gradient = (gradient_x + gradient_y) / 2.0;

                // Blue to purple gradient
                let r = (60.0 + gradient * 100.0) as u8;
                let g = (80.0 + gradient * 60.0) as u8;
                let b = (180.0 + gradient * 40.0) as u8;

                // Simple circular mask for rounded appearance
                let cx = size as f32 / 2.0;
                let cy = size as f32 / 2.0;
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = size as f32 * 0.45;

                let alpha = if dist < radius {
                    255
                } else if dist < radius + 20.0 {
                    // Smooth edge
                    ((radius + 20.0 - dist) / 20.0 * 255.0) as u8
                } else {
                    0
                };

                img.put_pixel(x, y, Rgba([r, g, b, alpha]));
            }
        }

        Self {
            source_image: DynamicImage::ImageRgba8(img),
        }
    }

    /// Get the source image dimensions
    #[allow(dead_code)] // Part of public API
    pub fn dimensions(&self) -> (u32, u32) {
        (self.source_image.width(), self.source_image.height())
    }

    /// Save the icon to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.source_image
            .save_with_format(path, ImageFormat::Png)
            .with_context(|| format!("Failed to save icon to {}", path.display()))?;
        Ok(())
    }

    /// Resize and save to a specific size
    pub fn save_resized(&self, path: &Path, width: u32, height: u32) -> Result<()> {
        let resized = self
            .source_image
            .resize_exact(width, height, FilterType::Lanczos3);
        resized
            .save_with_format(path, ImageFormat::Png)
            .with_context(|| format!("Failed to save icon to {}", path.display()))?;
        Ok(())
    }

    /// Create a wide icon by centering the source on a transparent background
    #[cfg(target_os = "windows")]
    pub fn save_wide(&self, path: &Path, width: u32, height: u32) -> Result<()> {
        let mut wide_img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Resize source to fit within bounds
        let icon_size = height.min(width);
        let resized = self
            .source_image
            .resize_exact(icon_size, icon_size, FilterType::Lanczos3);

        // Center the icon
        let x_offset = (width - icon_size) / 2;
        let y_offset = (height - icon_size) / 2;

        image::imageops::overlay(
            &mut wide_img,
            &resized.to_rgba8(),
            x_offset as i64,
            y_offset as i64,
        );

        DynamicImage::ImageRgba8(wide_img)
            .save_with_format(path, ImageFormat::Png)
            .with_context(|| format!("Failed to save wide icon to {}", path.display()))?;
        Ok(())
    }

    /// Generate all required Windows MSIX icons
    #[cfg(target_os = "windows")]
    pub fn generate_msix_icons(&self, assets_dir: &Path) -> Result<()> {
        fs::create_dir_all(assets_dir)?;
        let icon_set = MsixIconSet::new();

        // Square icons
        for (width, height, filename) in &icon_set.square44 {
            self.save_resized(&assets_dir.join(filename), *width, *height)?;
        }
        for (width, height, filename) in &icon_set.square150 {
            self.save_resized(&assets_dir.join(filename), *width, *height)?;
        }
        for (width, height, filename) in &icon_set.store_logo {
            self.save_resized(&assets_dir.join(filename), *width, *height)?;
        }

        // Wide icons
        for (width, height, filename) in &icon_set.wide310x150 {
            self.save_wide(&assets_dir.join(filename), *width, *height)?;
        }
        for (width, height, filename) in &icon_set.splash_screen {
            self.save_wide(&assets_dir.join(filename), *width, *height)?;
        }

        // Non-scaled versions for manifest references
        self.save_resized(&assets_dir.join("Square44x44Logo.png"), 44, 44)?;
        self.save_resized(&assets_dir.join("Square150x150Logo.png"), 150, 150)?;
        self.save_wide(&assets_dir.join("Wide310x150Logo.png"), 310, 150)?;
        self.save_resized(&assets_dir.join("StoreLogo.png"), 50, 50)?;
        self.save_wide(&assets_dir.join("SplashScreen.png"), 620, 300)?;

        Ok(())
    }

    /// Convert icon to macOS .icns format using system tools
    ///
    /// Requires: sips, iconutil (built-in on macOS)
    #[allow(dead_code)]
    pub fn convert_to_icns(&self, dest: &Path) -> Result<()> {
        // Create temporary iconset directory
        let iconset_dir = dest.with_extension("iconset");
        if iconset_dir.exists() {
            fs::remove_dir_all(&iconset_dir)?;
        }
        fs::create_dir_all(&iconset_dir)?;

        // Required icon sizes for macOS (size, scale, filename)
        let sizes: [(u32, u32, &str); 10] = [
            (16, 1, "icon_16x16.png"),
            (16, 2, "icon_16x16@2x.png"),
            (32, 1, "icon_32x32.png"),
            (32, 2, "icon_32x32@2x.png"),
            (128, 1, "icon_128x128.png"),
            (128, 2, "icon_128x128@2x.png"),
            (256, 1, "icon_256x256.png"),
            (256, 2, "icon_256x256@2x.png"),
            (512, 1, "icon_512x512.png"),
            (512, 2, "icon_512x512@2x.png"),
        ];

        for (size, scale, filename) in &sizes {
            let actual_size = size * scale;
            let output_path = iconset_dir.join(filename);
            self.save_resized(&output_path, actual_size, actual_size)?;
        }

        // Use iconutil to create .icns
        let status = Command::new("iconutil")
            .args([
                "-c",
                "icns",
                "-o",
                &dest.display().to_string(),
                &iconset_dir.display().to_string(),
            ])
            .status()
            .context("Failed to run iconutil")?;

        // Clean up iconset directory
        let _ = fs::remove_dir_all(&iconset_dir);

        if !status.success() {
            anyhow::bail!("iconutil failed to create .icns file");
        }

        Ok(())
    }

    /// Place Linux icons in hicolor theme directories
    #[cfg(target_os = "linux")]
    pub fn place_linux_icons(&self, appdir: &Path, exec_name: &str) -> Result<()> {
        // Root icon
        let root_icon = appdir.join(format!("{}.png", exec_name));
        self.save_resized(&root_icon, 256, 256)?;

        // Hicolor theme icons
        let sizes = [
            "16x16", "32x32", "48x48", "64x64", "128x128", "256x256", "512x512",
        ];
        for size_str in &sizes {
            let size: u32 = size_str.split('x').next().unwrap().parse().unwrap();
            let icon_dir = appdir
                .join("usr/share/icons/hicolor")
                .join(size_str)
                .join("apps");
            fs::create_dir_all(&icon_dir)?;
            self.save_resized(&icon_dir.join(format!("{}.png", exec_name)), size, size)?;
        }

        Ok(())
    }

    /// Save icon as PNG at a specific path with default size
    #[allow(dead_code)]
    pub fn save_png(&self, path: &Path, size: u32) -> Result<()> {
        self.save_resized(path, size, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_create_placeholder_icon() {
        let processor = IconProcessor::create_placeholder(RECOMMENDED_ICON_SIZE);
        let (w, h) = processor.dimensions();
        assert_eq!(w, RECOMMENDED_ICON_SIZE);
        assert_eq!(h, RECOMMENDED_ICON_SIZE);
    }

    #[test]
    fn test_save_resized() {
        let processor = IconProcessor::create_placeholder(RECOMMENDED_ICON_SIZE);
        let temp_path = temp_dir().join("test_icon_resized.png");
        processor.save_resized(&temp_path, 64, 64).unwrap();
        assert!(temp_path.exists());
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_validation_valid_icon() {
        let processor = IconProcessor::create_placeholder(RECOMMENDED_ICON_SIZE);
        let validation = processor.validate(Path::new("test.png"));
        assert!(validation.is_square);
        assert!(validation.meets_minimum);
        assert!(validation.meets_recommended);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_validation_small_icon() {
        let processor = IconProcessor::create_placeholder(256);
        let validation = processor.validate(Path::new("test.png"));
        assert!(validation.is_square);
        assert!(!validation.meets_minimum);
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_search_paths_default() {
        let paths = IconProcessor::get_search_paths(Path::new("/app"), None);
        assert!(paths.iter().any(|p| p.ends_with("assets/icon.png")));
        assert!(paths.iter().any(|p| p.ends_with("icon.png")));
    }

    #[test]
    fn test_search_paths_custom() {
        let paths = IconProcessor::get_search_paths(Path::new("/app"), Some("custom/myicon"));
        assert!(paths.iter().any(|p| p.ends_with("custom/myicon.png")));
    }
}
