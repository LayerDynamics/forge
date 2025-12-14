//! macOS packaging backend
//!
//! Creates:
//! - DMG disk images (default, for direct distribution)
//! - PKG installers (for enterprise/MDM deployment)
//! - .app bundles (standalone application bundle)
//! - ZIP archives (for notarization or simple distribution)
//!
//! ## Bundle Formats
//!
//! - **dmg**: DMG disk image with Applications symlink for drag-to-install.
//!   Best for direct downloads and user-friendly installation.
//! - **pkg**: PKG installer package. Best for enterprise deployment via MDM
//!   or when pre/post-install scripts are needed.
//! - **app**: Just the .app bundle without packaging. Useful for testing
//!   or when you want to handle distribution yourself.
//! - **zip**: ZIP archive of the .app bundle. Required for notarization
//!   submission and useful for simple distribution.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{build_embedded_binary, copy_dir_recursive, sanitize_name, AppManifest, IconProcessor};

/// macOS bundler supporting multiple output formats
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

    /// Execute macOS bundling pipeline based on format configuration
    pub fn bundle(&self) -> Result<PathBuf> {
        println!("Creating macOS package...");

        let macos_config = self.manifest.bundle.macos.as_ref();
        let format = macos_config
            .and_then(|c| c.format.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("dmg");

        // Always create the .app bundle first
        let app_bundle = self.create_app_bundle()?;

        // Optional code signing before packaging
        let should_sign = macos_config
            .map(|c| c.sign.unwrap_or(false))
            .unwrap_or(false);

        if should_sign {
            println!("  Signing app bundle...");
            self.codesign_bundle(&app_bundle)?;
        }

        // Create final package based on format
        let result = match format {
            "dmg" => self.bundle_dmg(&app_bundle),
            "pkg" => self.bundle_pkg(&app_bundle),
            "app" => Ok(app_bundle.clone()),
            "zip" => self.bundle_zip(&app_bundle),
            _ => bail!(
                "Unknown macOS bundle format: '{}'. Supported: dmg, pkg, app, zip",
                format
            ),
        };

        // Optional notarization
        if let Ok(ref output_path) = result {
            let should_notarize = macos_config
                .map(|c| c.notarize.unwrap_or(false))
                .unwrap_or(false);

            if should_notarize {
                println!("  Submitting for notarization...");
                self.notarize(output_path)?;
            }
        }

        // Print summary
        if let Ok(ref output_path) = result {
            println!("\n  App bundle: {}", app_bundle.display());
            if output_path != &app_bundle {
                println!("  Package: {}", output_path.display());
            }
        }

        result
    }

    /// Create .app bundle directory structure
    fn create_app_bundle(&self) -> Result<PathBuf> {
        let app_name = &self.manifest.app.name;

        println!("  Creating .app bundle...");

        // 1. Build forge-host with embedded assets
        let binary_path = build_embedded_binary(self.dist_dir)?;

        // 2. Create bundle structure
        let bundle_path = self.output_dir.join(format!("{}.app", app_name));

        // Clean up existing bundle
        if bundle_path.exists() {
            fs::remove_dir_all(&bundle_path)?;
        }

        // Create directory structure
        let contents_dir = bundle_path.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");

        fs::create_dir_all(&macos_dir)?;
        fs::create_dir_all(&resources_dir)?;

        // 3. Copy binary
        let dest_binary = macos_dir.join(sanitize_name(app_name));
        fs::copy(&binary_path, &dest_binary)
            .with_context(|| format!("Failed to copy binary to {}", dest_binary.display()))?;

        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_binary, perms)?;
        }

        // 4. Generate Info.plist
        println!("  Generating Info.plist...");
        let info_plist = self.generate_info_plist()?;
        fs::write(contents_dir.join("Info.plist"), info_plist)?;

        // 5. Create PkgInfo
        fs::write(contents_dir.join("PkgInfo"), "APPL????")?;

        // 6. Handle icon
        println!("  Generating icon...");
        let icon_base = self.manifest.bundle.icon.as_deref();
        let icon_processor = IconProcessor::find_icon(self.app_dir, icon_base)?;
        let icns_path = resources_dir.join("AppIcon.icns");
        icon_processor.convert_to_icns(&icns_path)?;

        // 7. Copy app resources (manifest, src for Deno runtime)
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
        Ok(bundle_path)
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

    /// Create DMG disk image (delegates to dmg module)
    fn bundle_dmg(&self, bundle_path: &Path) -> Result<PathBuf> {
        super::dmg::create_dmg(bundle_path, self.output_dir, self.manifest)
    }

    /// Create PKG installer (delegates to pkg module)
    fn bundle_pkg(&self, bundle_path: &Path) -> Result<PathBuf> {
        let pkg_path = super::pkg::create_pkg(bundle_path, self.output_dir, self.manifest)?;

        // Sign PKG if signing is enabled
        let macos_config = self.manifest.bundle.macos.as_ref();
        if let Some(config) = macos_config {
            if config.sign.unwrap_or(false) {
                if let Some(ref identity) = config.signing_identity {
                    // PKG signing uses "Developer ID Installer" certificate
                    let installer_identity = identity.replace("Application", "Installer");
                    super::pkg::sign_pkg(&pkg_path, &installer_identity)?;
                }
            }
        }

        Ok(pkg_path)
    }

    /// Create ZIP archive of the app bundle
    fn bundle_zip(&self, bundle_path: &Path) -> Result<PathBuf> {
        let app_name = &self.manifest.app.name;
        let version = &self.manifest.app.version;

        println!("  Creating ZIP archive...");

        let zip_name = format!("{}-{}-macos.zip", sanitize_name(app_name), version);
        let zip_path = self.output_dir.join(&zip_name);

        // Remove existing ZIP
        if zip_path.exists() {
            fs::remove_file(&zip_path)?;
        }

        // Use ditto for proper handling of macOS metadata and symlinks
        let status = Command::new("ditto")
            .args([
                "-c",
                "-k",
                "--keepParent",
                &bundle_path.display().to_string(),
                &zip_path.display().to_string(),
            ])
            .status()
            .context("Failed to run ditto")?;

        if !status.success() {
            bail!("ditto failed to create ZIP archive");
        }

        println!("  ZIP archive created: {}", zip_path.display());
        Ok(zip_path)
    }

    /// Notarize a package with Apple
    fn notarize(&self, path: &Path) -> Result<()> {
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
        let submit_status = Command::new("xcrun")
            .args([
                "notarytool",
                "submit",
                &path.display().to_string(),
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
                    .args(["stapler", "staple", &path.display().to_string()])
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
