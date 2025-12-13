//! Platform packaging backends for Forge apps
//!
//! This module provides packaging functionality for:
//! - Windows: MSIX packages
//! - macOS: .app bundles + DMG disk images
//! - Linux: AppImage or portable tarball
//!
//! # Usage
//!
//! ```ignore
//! use bundler::{AppManifest, bundle};
//!
//! let manifest = AppManifest::from_app_dir(&app_dir)?;
//! bundle(&app_dir, &dist_dir, &output_dir, &manifest)?;
//! ```

pub mod icons;
pub mod manifest;

#[cfg(target_os = "windows")]
pub mod msix;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

pub mod codesign;

// Re-export commonly used types
pub use icons::{IconProcessor, MIN_ICON_SIZE, RECOMMENDED_ICON_SIZE};
#[cfg(target_os = "windows")]
pub use manifest::{normalize_version, sanitize_msix_name};
pub use manifest::{sanitize_name, AppManifest};

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build forge-host binary with embedded assets
///
/// This compiles the forge-host crate in release mode with the
/// FORGE_EMBED_DIR environment variable set to embed web assets.
pub fn build_embedded_binary(dist_dir: &Path) -> Result<PathBuf> {
    let web_dir = dist_dir.join("web");
    if !web_dir.exists() {
        bail!(
            "Web assets not found at {}. Run 'forge build' first.",
            web_dir.display()
        );
    }

    println!("  Building release binary with embedded assets...");

    let status = Command::new("cargo")
        .args(["build", "-p", "forge-host", "--release"])
        .env("FORGE_EMBED_DIR", web_dir.display().to_string())
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        bail!("cargo build failed with status: {}", status);
    }

    // Find the built binary
    #[cfg(target_os = "windows")]
    let binary_name = "forge-host.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "forge-host";

    let binary_path = PathBuf::from("target/release").join(binary_name);
    if !binary_path.exists() {
        bail!(
            "Built binary not found at {}. Build may have failed.",
            binary_path.display()
        );
    }

    Ok(binary_path)
}

/// Copy directory recursively
#[allow(dead_code)]
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// Main bundle entry point - dispatches to platform-specific bundler
pub fn bundle(
    app_dir: &Path,
    dist_dir: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    // Create output directory
    fs::create_dir_all(output_dir)?;

    #[cfg(target_os = "windows")]
    {
        msix::bundle(app_dir, dist_dir, output_dir, manifest)
    }

    #[cfg(target_os = "macos")]
    {
        macos::bundle(app_dir, dist_dir, output_dir, manifest)
    }

    #[cfg(target_os = "linux")]
    {
        linux::bundle(app_dir, dist_dir, output_dir, manifest)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        bail!("Bundling is not supported on this platform")
    }
}

/// Parse manifest from app directory (convenience function)
pub fn parse_manifest(app_dir: &Path) -> Result<AppManifest> {
    AppManifest::from_app_dir(app_dir)
}
