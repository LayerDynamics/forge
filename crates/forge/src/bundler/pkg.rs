//! macOS PKG installer creation
//!
//! Creates PKG installer packages for macOS distribution.
//! PKG installers are useful for:
//! - Enterprise deployment via MDM
//! - Apps that need to install to specific locations
//! - Apps with pre/post-install scripts
//!
//! Uses pkgbuild and productbuild (built-in macOS tools).

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{sanitize_name, AppManifest};

/// PKG installer configuration
pub struct PkgConfig {
    /// Install location (default: /Applications)
    pub install_location: String,
    /// Whether to require admin privileges
    pub require_admin: bool,
    /// Optional pre-install script path
    pub preinstall_script: Option<PathBuf>,
    /// Optional post-install script path
    pub postinstall_script: Option<PathBuf>,
    /// Optional background image for installer
    pub background: Option<PathBuf>,
    /// Optional license file (RTF or TXT)
    pub license: Option<PathBuf>,
    /// Optional welcome text file (RTF or TXT)
    pub welcome: Option<PathBuf>,
    /// Optional conclusion text file (RTF or TXT)
    pub conclusion: Option<PathBuf>,
}

impl Default for PkgConfig {
    fn default() -> Self {
        Self {
            install_location: "/Applications".to_string(),
            require_admin: true,
            preinstall_script: None,
            postinstall_script: None,
            background: None,
            license: None,
            welcome: None,
            conclusion: None,
        }
    }
}

/// Create a component PKG from an app bundle
pub fn create_component_pkg(
    bundle_path: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
    config: &PkgConfig,
) -> Result<PathBuf> {
    let app_name = &manifest.app.name;
    let identifier = &manifest.app.identifier;
    let version = &manifest.app.version;

    let pkg_name = format!("{}-{}.pkg", sanitize_name(app_name), version);
    let pkg_path = output_dir.join(&pkg_name);

    println!("  Creating component PKG...");

    // Remove existing PKG
    if pkg_path.exists() {
        fs::remove_file(&pkg_path)?;
    }

    // Create scripts directory if we have scripts
    let scripts_dir = output_dir.join(".pkg_scripts");
    let has_scripts = config.preinstall_script.is_some() || config.postinstall_script.is_some();

    if has_scripts {
        if scripts_dir.exists() {
            fs::remove_dir_all(&scripts_dir)?;
        }
        fs::create_dir_all(&scripts_dir)?;

        if let Some(ref preinstall) = config.preinstall_script {
            let dest = scripts_dir.join("preinstall");
            fs::copy(preinstall, &dest)?;
            make_executable(&dest)?;
        }

        if let Some(ref postinstall) = config.postinstall_script {
            let dest = scripts_dir.join("postinstall");
            fs::copy(postinstall, &dest)?;
            make_executable(&dest)?;
        }
    }

    // Build pkgbuild command
    let mut cmd = Command::new("pkgbuild");
    cmd.args([
        "--root",
        &bundle_path.parent().unwrap().display().to_string(),
        "--component-plist",
        "/dev/stdin", // We'll provide component plist via stdin
        "--identifier",
        identifier,
        "--version",
        version,
        "--install-location",
        &config.install_location,
    ]);

    if has_scripts {
        cmd.args(["--scripts", &scripts_dir.display().to_string()]);
    }

    cmd.arg(pkg_path.display().to_string());

    // Generate component plist
    let bundle_name = bundle_path.file_name().unwrap().to_string_lossy();
    let component_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
    <dict>
        <key>BundleHasStrictIdentifier</key>
        <true/>
        <key>BundleIsRelocatable</key>
        <false/>
        <key>BundleIsVersionChecked</key>
        <true/>
        <key>BundleOverwriteAction</key>
        <string>upgrade</string>
        <key>RootRelativeBundlePath</key>
        <string>{}</string>
    </dict>
</array>
</plist>"#,
        bundle_name
    );

    // Run pkgbuild with component plist on stdin
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start pkgbuild")?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(component_plist.as_bytes())?;
    }

    let status = child.wait().context("Failed to wait for pkgbuild")?;

    // Clean up scripts directory
    if has_scripts {
        let _ = fs::remove_dir_all(&scripts_dir);
    }

    if !status.success() {
        bail!("pkgbuild failed with status: {}", status);
    }

    Ok(pkg_path)
}

/// Create a product archive (installer PKG) with customization
pub fn create_installer_pkg(
    bundle_path: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
    config: &PkgConfig,
) -> Result<PathBuf> {
    let app_name = &manifest.app.name;
    let identifier = &manifest.app.identifier;
    let version = &manifest.app.version;

    println!("  Creating installer PKG...");

    // First create the component package
    let component_pkg = create_component_pkg(bundle_path, output_dir, manifest, config)?;

    // Create distribution XML
    let distribution_xml = generate_distribution_xml(manifest, config)?;
    let dist_path = output_dir.join("distribution.xml");
    fs::write(&dist_path, &distribution_xml)?;

    // Create resources directory for installer customization
    let resources_dir = output_dir.join(".pkg_resources");
    if resources_dir.exists() {
        fs::remove_dir_all(&resources_dir)?;
    }
    fs::create_dir_all(&resources_dir)?;

    // Copy optional resources
    if let Some(ref bg) = config.background {
        fs::copy(bg, resources_dir.join("background.png"))?;
    }
    if let Some(ref license) = config.license {
        fs::copy(license, resources_dir.join("license.rtf"))?;
    }
    if let Some(ref welcome) = config.welcome {
        fs::copy(welcome, resources_dir.join("welcome.rtf"))?;
    }
    if let Some(ref conclusion) = config.conclusion {
        fs::copy(conclusion, resources_dir.join("conclusion.rtf"))?;
    }

    // Create final installer package name
    let installer_name = format!("{}-{}-installer.pkg", sanitize_name(app_name), version);
    let installer_path = output_dir.join(&installer_name);

    // Remove existing installer
    if installer_path.exists() {
        fs::remove_file(&installer_path)?;
    }

    // Build productbuild command
    let mut cmd = Command::new("productbuild");
    cmd.args([
        "--distribution",
        &dist_path.display().to_string(),
        "--package-path",
        &output_dir.display().to_string(),
        "--resources",
        &resources_dir.display().to_string(),
        "--identifier",
        identifier,
        "--version",
        version,
    ]);

    cmd.arg(installer_path.display().to_string());

    let status = cmd.status().context("Failed to run productbuild")?;

    // Clean up
    let _ = fs::remove_file(&component_pkg);
    let _ = fs::remove_file(&dist_path);
    let _ = fs::remove_dir_all(&resources_dir);

    if !status.success() {
        bail!("productbuild failed with status: {}", status);
    }

    Ok(installer_path)
}

/// Generate distribution.xml for productbuild
fn generate_distribution_xml(manifest: &AppManifest, config: &PkgConfig) -> Result<String> {
    let app_name = &manifest.app.name;
    let identifier = &manifest.app.identifier;
    let version = &manifest.app.version;
    let pkg_ref = format!("{}-{}.pkg", sanitize_name(app_name), version);

    let auth_level = if config.require_admin {
        "root"
    } else {
        "none"
    };

    let background_element = if config.background.is_some() {
        r#"<background file="background.png" alignment="bottomleft" scaling="none"/>"#
    } else {
        ""
    };

    let license_element = if config.license.is_some() {
        r#"<license file="license.rtf"/>"#
    } else {
        ""
    };

    let welcome_element = if config.welcome.is_some() {
        r#"<welcome file="welcome.rtf"/>"#
    } else {
        ""
    };

    let conclusion_element = if config.conclusion.is_some() {
        r#"<conclusion file="conclusion.rtf"/>"#
    } else {
        ""
    };

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>{app_name}</title>
    <organization>{identifier}</organization>
    <domains enable_localSystem="true"/>
    <options customize="never" require-scripts="false" rootVolumeOnly="true"/>
    {background}
    {welcome}
    {license}
    {conclusion}
    <choices-outline>
        <line choice="default">
            <line choice="{identifier}"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="{identifier}" visible="false">
        <pkg-ref id="{identifier}"/>
    </choice>
    <pkg-ref id="{identifier}" version="{version}" onConclusion="none" auth="{auth}">{pkg_ref}</pkg-ref>
</installer-gui-script>"#,
        app_name = app_name,
        identifier = identifier,
        version = version,
        pkg_ref = pkg_ref,
        auth = auth_level,
        background = background_element,
        welcome = welcome_element,
        license = license_element,
        conclusion = conclusion_element,
    ))
}

/// Sign a PKG file with a Developer ID Installer certificate
pub fn sign_pkg(pkg_path: &Path, signing_identity: &str) -> Result<()> {
    println!("  Signing PKG...");

    let signed_path = pkg_path.with_extension("signed.pkg");

    let status = Command::new("productsign")
        .args([
            "--sign",
            signing_identity,
            &pkg_path.display().to_string(),
            &signed_path.display().to_string(),
        ])
        .status()
        .context("Failed to run productsign")?;

    if !status.success() {
        bail!("productsign failed with status: {}", status);
    }

    // Replace original with signed version
    fs::remove_file(pkg_path)?;
    fs::rename(&signed_path, pkg_path)?;

    println!("    PKG signed successfully");
    Ok(())
}

/// Notarize a PKG file with Apple
#[allow(dead_code)]
pub fn notarize_pkg(pkg_path: &Path, team_id: &str) -> Result<()> {
    println!("  Submitting PKG for notarization...");

    let submit_status = Command::new("xcrun")
        .args([
            "notarytool",
            "submit",
            &pkg_path.display().to_string(),
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
                .args(["stapler", "staple", &pkg_path.display().to_string()])
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

/// Create a simple PKG (convenience function)
pub fn create_pkg(
    bundle_path: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    let config = PkgConfig::default();
    create_installer_pkg(bundle_path, output_dir, manifest, &config)
}

/// Make a file executable
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkg_config_default() {
        let config = PkgConfig::default();
        assert_eq!(config.install_location, "/Applications");
        assert!(config.require_admin);
    }

    #[test]
    fn test_pkg_name_generation() {
        let name = format!("{}-{}.pkg", sanitize_name("My App"), "1.0.0");
        assert_eq!(name, "my-app-1.0.0.pkg");
    }
}
