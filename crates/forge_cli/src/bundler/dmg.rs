//! macOS DMG disk image creation
//!
//! Creates DMG disk images containing .app bundles for distribution.
//! Uses hdiutil (built-in macOS tool) for image creation.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{copy_dir_recursive, sanitize_name, AppManifest};

/// Create DMG disk image containing the app bundle
pub fn create_dmg(
    bundle_path: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    let app_name = &manifest.app.name;
    let version = &manifest.app.version;

    let dmg_name = format!("{}-{}-macos.dmg", sanitize_name(app_name), version);
    let dmg_path = output_dir.join(&dmg_name);

    println!("  Creating DMG...");

    // Remove existing DMG
    if dmg_path.exists() {
        fs::remove_file(&dmg_path)?;
    }

    // Create staging directory for DMG contents
    let staging_dir = output_dir.join(".dmg_staging");
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    fs::create_dir_all(&staging_dir)?;

    // Copy app bundle to staging
    let bundle_name = bundle_path.file_name().unwrap();
    let staged_app = staging_dir.join(bundle_name);
    copy_dir_recursive(bundle_path, &staged_app)?;

    // Create Applications symlink for drag-install
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let apps_link = staging_dir.join("Applications");
        symlink("/Applications", &apps_link)?;
    }

    // Create read-write DMG first
    let temp_dmg = output_dir.join(format!("{}.temp.dmg", sanitize_name(app_name)));

    let create_status = Command::new("hdiutil")
        .args([
            "create",
            "-srcfolder",
            &staging_dir.display().to_string(),
            "-volname",
            app_name,
            "-fs",
            "HFS+",
            "-fsargs",
            "-c c=64,a=16,e=16",
            "-format",
            "UDRW",
            "-size",
            "500m", // Max size, will be smaller after conversion
            &temp_dmg.display().to_string(),
        ])
        .status()
        .context("Failed to run hdiutil create")?;

    if !create_status.success() {
        // Clean up staging
        let _ = fs::remove_dir_all(&staging_dir);
        bail!("hdiutil create failed");
    }

    // Convert to compressed read-only DMG
    let convert_status = Command::new("hdiutil")
        .args([
            "convert",
            &temp_dmg.display().to_string(),
            "-format",
            "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",
            &dmg_path.display().to_string(),
        ])
        .status()
        .context("Failed to run hdiutil convert")?;

    // Clean up
    let _ = fs::remove_file(&temp_dmg);
    let _ = fs::remove_dir_all(&staging_dir);

    if !convert_status.success() {
        bail!("hdiutil convert failed");
    }

    Ok(dmg_path)
}

/// Create a sparse DMG with custom background and icon positioning
/// This creates a more polished DMG with drag-to-install UX
#[allow(dead_code)]
pub fn create_fancy_dmg(
    bundle_path: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
    background_image: Option<&Path>,
) -> Result<PathBuf> {
    let app_name = &manifest.app.name;
    let version = &manifest.app.version;

    let dmg_name = format!("{}-{}-macos.dmg", sanitize_name(app_name), version);
    let dmg_path = output_dir.join(&dmg_name);

    println!("  Creating fancy DMG with custom layout...");

    // Remove existing DMG
    if dmg_path.exists() {
        fs::remove_file(&dmg_path)?;
    }

    // Create staging directory
    let staging_dir = output_dir.join(".dmg_staging");
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    fs::create_dir_all(&staging_dir)?;

    // Copy app bundle
    let bundle_name = bundle_path.file_name().unwrap();
    let staged_app = staging_dir.join(bundle_name);
    copy_dir_recursive(bundle_path, &staged_app)?;

    // Create Applications symlink
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let apps_link = staging_dir.join("Applications");
        symlink("/Applications", &apps_link)?;
    }

    // Copy background image if provided
    if let Some(bg_path) = background_image {
        let bg_dir = staging_dir.join(".background");
        fs::create_dir_all(&bg_dir)?;
        fs::copy(bg_path, bg_dir.join("background.png"))?;
    }

    // Create temporary read-write DMG
    let temp_dmg = output_dir.join(format!("{}.temp.dmg", sanitize_name(app_name)));

    let create_status = Command::new("hdiutil")
        .args([
            "create",
            "-srcfolder",
            &staging_dir.display().to_string(),
            "-volname",
            app_name,
            "-fs",
            "HFS+",
            "-format",
            "UDRW",
            "-size",
            "500m",
            &temp_dmg.display().to_string(),
        ])
        .status()
        .context("Failed to create temporary DMG")?;

    if !create_status.success() {
        let _ = fs::remove_dir_all(&staging_dir);
        bail!("hdiutil create failed");
    }

    // Mount the DMG to customize it
    let mount_output = Command::new("hdiutil")
        .args([
            "attach",
            &temp_dmg.display().to_string(),
            "-mountpoint",
            &format!("/Volumes/{}", app_name),
            "-nobrowse",
        ])
        .output()
        .context("Failed to mount DMG")?;

    if mount_output.status.success() {
        let mount_point = format!("/Volumes/{}", app_name);

        // Apply Finder settings using AppleScript
        let applescript = format!(
            r#"
            tell application "Finder"
                tell disk "{app_name}"
                    open
                    set current view of container window to icon view
                    set toolbar visible of container window to false
                    set statusbar visible of container window to false
                    set bounds of container window to {{100, 100, 640, 480}}
                    set viewOptions to the icon view options of container window
                    set arrangement of viewOptions to not arranged
                    set icon size of viewOptions to 128
                    set position of item "{bundle_name}" of container window to {{140, 200}}
                    set position of item "Applications" of container window to {{400, 200}}
                    close
                end tell
            end tell
            "#,
            app_name = app_name,
            bundle_name = bundle_name.to_string_lossy(),
        );

        let _ = Command::new("osascript")
            .args(["-e", &applescript])
            .output();

        // Unmount
        let _ = Command::new("hdiutil")
            .args(["detach", &mount_point, "-force"])
            .status();
    }

    // Convert to final compressed DMG
    let convert_status = Command::new("hdiutil")
        .args([
            "convert",
            &temp_dmg.display().to_string(),
            "-format",
            "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",
            &dmg_path.display().to_string(),
        ])
        .status()
        .context("Failed to convert DMG")?;

    // Clean up
    let _ = fs::remove_file(&temp_dmg);
    let _ = fs::remove_dir_all(&staging_dir);

    if !convert_status.success() {
        bail!("hdiutil convert failed");
    }

    Ok(dmg_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmg_name_generation() {
        let name = format!("{}-{}-macos.dmg", sanitize_name("My App"), "1.0.0");
        assert_eq!(name, "my-app-1.0.0-macos.dmg");
    }
}
