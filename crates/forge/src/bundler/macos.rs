//! macOS .app bundle and DMG packaging backend
//!
//! Creates:
//! - .app bundle with proper structure (Contents/MacOS, Resources, Info.plist)
//! - DMG disk image via hdiutil (system tool)
//! - Optional code signing via codesign
//! - Optional notarization via xcrun notarytool

use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{build_embedded_binary, copy_dir_recursive, sanitize_name, AppManifest, IconProcessor};

/// macOS bundler
pub struct MacosBundler<'a> {
    app_dir: &'a Path,
    dist_dir: &'a Path,
    output_dir: &'a Path,
    manifest: &'a AppManifest,
}

impl<'a> MacosBundler<'a> {
    pub fn new(
        app_dir: &'a Path,
        dist_dir: &'a Path,
        output_dir: &'a Path,
        manifest: &'a AppManifest,
    ) -> Self {
        Self {
            app_dir,
            dist_dir,
            output_dir,
            manifest,
        }
    }

    /// Execute full macOS bundling pipeline
    pub fn bundle(&self) -> Result<PathBuf> {
        println!("Creating macOS app bundle...");

        let app_name = &self.manifest.app.name;
        let macos_config = self.manifest.bundle.macos.as_ref();

        // 1. Build forge-host with embedded assets
        let binary_path = build_embedded_binary(self.dist_dir)?;

        // 2. Create .app bundle structure
        let bundle_path = self.output_dir.join(format!("{}.app", app_name));
        self.create_app_bundle(&bundle_path, &binary_path)?;

        // 3. Code sign if requested
        if macos_config
            .map(|c| c.sign.unwrap_or(false))
            .unwrap_or(false)
        {
            println!("  Signing app bundle...");
            self.codesign_bundle(&bundle_path)?;
        }

        // 4. Create DMG
        println!("  Creating DMG...");
        let dmg_path = self.create_dmg(&bundle_path)?;

        // 5. Notarize if requested
        if macos_config
            .map(|c| c.notarize.unwrap_or(false))
            .unwrap_or(false)
        {
            println!("  Submitting for notarization...");
            self.notarize_dmg(&dmg_path)?;
        }

        println!("\n  App bundle: {}", bundle_path.display());
        println!("  DMG: {}", dmg_path.display());

        Ok(dmg_path)
    }

    /// Create .app bundle directory structure
    fn create_app_bundle(&self, bundle_path: &Path, binary_path: &Path) -> Result<()> {
        let app_name = &self.manifest.app.name;

        // Clean up existing bundle
        if bundle_path.exists() {
            fs::remove_dir_all(bundle_path)?;
        }

        // Create directory structure
        let contents_dir = bundle_path.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");

        fs::create_dir_all(&macos_dir)?;
        fs::create_dir_all(&resources_dir)?;

        // 1. Copy binary
        let dest_binary = macos_dir.join(sanitize_name(app_name));
        fs::copy(binary_path, &dest_binary)
            .with_context(|| format!("Failed to copy binary to {}", dest_binary.display()))?;

        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_binary, perms)?;
        }

        // 2. Generate Info.plist
        println!("  Generating Info.plist...");
        let info_plist = self.generate_info_plist()?;
        fs::write(contents_dir.join("Info.plist"), info_plist)?;

        // 3. Create PkgInfo
        fs::write(contents_dir.join("PkgInfo"), "APPL????")?;

        // 4. Handle icon
        println!("  Generating icon...");
        let icon_base = self.manifest.bundle.icon.as_deref();
        let icon_processor = IconProcessor::find_icon(self.app_dir, icon_base)?;
        let icns_path = resources_dir.join("AppIcon.icns");
        icon_processor.convert_to_icns(&icns_path)?;

        // 5. Copy app resources (manifest, src for Deno runtime)
        let app_resources = resources_dir.join("app");
        fs::create_dir_all(&app_resources)?;

        // Copy manifest
        fs::copy(
            self.dist_dir.join("manifest.app.toml"),
            app_resources.join("manifest.app.toml"),
        )?;

        // Copy src/ directory (Deno runtime code)
        let src_dir = self.dist_dir.join("src");
        if src_dir.exists() {
            copy_dir_recursive(&src_dir, &app_resources.join("src"))?;
        }

        println!("  Created app bundle: {}", bundle_path.display());
        Ok(())
    }

    /// Generate Info.plist content
    fn generate_info_plist(&self) -> Result<String> {
        let app = &self.manifest.app;
        let macos_config = self.manifest.bundle.macos.as_ref();

        let category = macos_config
            .map(|c| c.category_or_default())
            .unwrap_or_else(|| "public.app-category.developer-tools".to_string());

        let min_version = macos_config
            .map(|c| c.min_version_or_default())
            .unwrap_or_else(|| "12.0".to_string());

        let copyright = format!("Copyright (c) {} {}", Utc::now().format("%Y"), &app.name);

        // Parse version for CFBundleShortVersionString
        let short_version = app.version.split('-').next().unwrap_or(&app.version);

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>{display_name}</string>
    <key>CFBundleExecutable</key>
    <string>{executable}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>{identifier}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{name}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{short_version}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>LSApplicationCategoryType</key>
    <string>{category}</string>
    <key>LSMinimumSystemVersion</key>
    <string>{min_version}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>{copyright}</string>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
"#,
            display_name = app.name,
            executable = sanitize_name(&app.name),
            identifier = app.identifier,
            name = app.name,
            short_version = short_version,
            version = app.version,
            category = category,
            min_version = min_version,
            copyright = copyright,
        ))
    }

    /// Code sign the app bundle
    fn codesign_bundle(&self, bundle_path: &Path) -> Result<()> {
        let macos_config = self
            .manifest
            .bundle
            .macos
            .as_ref()
            .context("macOS bundle config required for signing")?;

        let identity = macos_config.signing_identity.as_ref().context(
            "Code signing enabled but no signing_identity specified.\n\
                Add [bundle.macos].signing_identity = \"Developer ID Application: ...\"",
        )?;

        let entitlements = macos_config
            .entitlements
            .as_ref()
            .map(|e| self.app_dir.join(e));

        // Sign the main bundle
        let mut cmd = Command::new("codesign");
        cmd.args([
            "--sign",
            identity,
            "--force",
            "--timestamp",
            "--options",
            "runtime", // Required for notarization
        ]);

        if let Some(ref ent) = entitlements {
            cmd.args(["--entitlements", &ent.display().to_string()]);
        }

        cmd.arg(bundle_path.display().to_string());

        let status = cmd.status().context("Failed to run codesign")?;

        if !status.success() {
            bail!("codesign failed with status: {}", status);
        }

        // Verify signature
        let verify_status = Command::new("codesign")
            .args([
                "--verify",
                "--deep",
                "--strict",
                "--verbose=2",
                &bundle_path.display().to_string(),
            ])
            .status()
            .context("Failed to verify code signature")?;

        if !verify_status.success() {
            bail!("Code signature verification failed");
        }

        println!("    Code signing complete");
        Ok(())
    }

    /// Create DMG disk image containing the app bundle
    fn create_dmg(&self, bundle_path: &Path) -> Result<PathBuf> {
        let app_name = &self.manifest.app.name;
        let version = &self.manifest.app.version;

        let dmg_name = format!("{}-{}-macos.dmg", sanitize_name(app_name), version);
        let dmg_path = self.output_dir.join(&dmg_name);

        // Remove existing DMG
        if dmg_path.exists() {
            fs::remove_file(&dmg_path)?;
        }

        // Create staging directory for DMG contents
        let staging_dir = self.output_dir.join(".dmg_staging");
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
        let temp_dmg = self
            .output_dir
            .join(format!("{}.temp.dmg", sanitize_name(app_name)));

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

    /// Notarize the DMG with Apple
    fn notarize_dmg(&self, dmg_path: &Path) -> Result<()> {
        let macos_config = self
            .manifest
            .bundle
            .macos
            .as_ref()
            .context("macOS bundle config required for notarization")?;

        let team_id = macos_config.team_id.as_ref().context(
            "Notarization enabled but no team_id specified.\n\
                Add [bundle.macos].team_id = \"YOUR_TEAM_ID\"",
        )?;

        println!("    Submitting to Apple (this may take several minutes)...");

        // Submit for notarization using notarytool
        // Note: Requires keychain profile setup via:
        // xcrun notarytool store-credentials "forge-notarize"
        let submit_status = Command::new("xcrun")
            .args([
                "notarytool",
                "submit",
                &dmg_path.display().to_string(),
                "--keychain-profile",
                "forge-notarize",
                "--team-id",
                team_id,
                "--wait",
            ])
            .status();

        match submit_status {
            Ok(s) if s.success() => {
                println!("    Notarization successful");

                // Staple the notarization ticket
                let staple_status = Command::new("xcrun")
                    .args(["stapler", "staple", &dmg_path.display().to_string()])
                    .status()
                    .context("Failed to staple notarization ticket")?;

                if !staple_status.success() {
                    println!("    Warning: Failed to staple notarization ticket");
                } else {
                    println!("    Stapled notarization ticket");
                }

                Ok(())
            }
            Ok(_) => {
                bail!(
                    "Notarization failed.\n\
                    Ensure credentials are configured:\n\
                    xcrun notarytool store-credentials forge-notarize\n\
                    Then provide your Apple ID, team ID, and app-specific password."
                )
            }
            Err(e) => {
                bail!(
                    "Failed to run notarytool: {}\n\
                    Make sure Xcode Command Line Tools are installed.",
                    e
                )
            }
        }
    }
}

/// Main bundle entry point
pub fn bundle(
    app_dir: &Path,
    dist_dir: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    let bundler = MacosBundler::new(app_dir, dist_dir, output_dir, manifest);
    bundler.bundle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My App"), "my-app");
        assert_eq!(sanitize_name("Test123"), "test123");
    }
}
