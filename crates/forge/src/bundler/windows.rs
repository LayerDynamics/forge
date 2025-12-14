//! Windows packaging backend
//!
//! Creates:
//! - MSIX packages (default, for Windows 10+ Store/sideload distribution)
//! - Portable EXE directory (standalone executable with resources)
//! - NSIS installer (optional, requires NSIS to be installed)
//!
//! ## Bundle Formats
//!
//! - **msix**: Windows App Package format for modern Windows apps. Best for
//!   Windows Store distribution or enterprise sideloading.
//! - **portable**: Standalone directory containing the exe and resources.
//!   Can be zipped for distribution without installation requirements.
//! - **nsis**: Traditional installer using NSIS (Nullsoft Scriptable Install System).
//!   Best for users expecting classic Windows installers.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{build_embedded_binary, copy_dir_recursive, sanitize_name, AppManifest, IconProcessor};

/// Windows bundler supporting multiple output formats
pub struct WindowsBundler<'a> {
    app_dir: &'a Path,
    dist_dir: &'a Path,
    output_dir: &'a Path,
    manifest: &'a AppManifest,
}

impl<'a> WindowsBundler<'a> {
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

    /// Execute Windows bundling pipeline based on format configuration
    pub fn bundle(&self) -> Result<PathBuf> {
        println!("Creating Windows package...");

        let windows_config = self.manifest.bundle.windows.as_ref();
        let format = windows_config
            .and_then(|c| c.format.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("msix");

        match format {
            "msix" => self.bundle_msix(),
            "portable" => self.bundle_portable(),
            "nsis" => self.bundle_nsis(),
            "zip" => self.bundle_zip(),
            _ => bail!(
                "Unknown Windows bundle format: '{}'. Supported: msix, portable, nsis, zip",
                format
            ),
        }
    }

    /// Create MSIX package (delegates to msix module)
    fn bundle_msix(&self) -> Result<PathBuf> {
        // Delegate to the existing MSIX bundler
        super::msix::bundle(self.app_dir, self.dist_dir, self.output_dir, self.manifest)
    }

    /// Create portable executable directory
    fn bundle_portable(&self) -> Result<PathBuf> {
        println!("  Creating portable package...");

        let app_name = &self.manifest.app.name;
        let version = &self.manifest.app.version;

        // Create portable directory
        let portable_name = format!("{}-{}-win-portable", sanitize_name(app_name), version);
        let portable_dir = self.output_dir.join(&portable_name);

        if portable_dir.exists() {
            fs::remove_dir_all(&portable_dir)?;
        }
        fs::create_dir_all(&portable_dir)?;

        // 1. Build forge-host with embedded assets
        let binary_path = build_embedded_binary(self.dist_dir)?;

        // 2. Copy executable
        let exe_name = format!("{}.exe", sanitize_name(app_name));
        let dest_exe = portable_dir.join(&exe_name);
        fs::copy(&binary_path, &dest_exe)
            .with_context(|| format!("Failed to copy binary to {}", dest_exe.display()))?;

        // 3. Copy app resources (manifest, src for Deno runtime)
        let resources_dir = portable_dir.join("resources");
        fs::create_dir_all(&resources_dir)?;

        // Copy manifest
        fs::copy(
            self.dist_dir.join("manifest.app.toml"),
            resources_dir.join("manifest.app.toml"),
        )?;

        // Copy src/ directory (Deno runtime code)
        let src_dir = self.dist_dir.join("src");
        if src_dir.exists() {
            copy_dir_recursive(&src_dir, &resources_dir.join("src"))?;
        }

        // 4. Generate icon (optional for portable, but nice to have)
        println!("  Generating icons...");
        let icon_base = self.manifest.bundle.icon.as_deref();
        if let Ok(icon_processor) = IconProcessor::find_icon(self.app_dir, icon_base) {
            let icon_path = portable_dir.join("icon.png");
            icon_processor.save_resized(&icon_path, 256, 256)?;

            // Also generate .ico if we can (for shortcut creation)
            if let Ok(ico_path) = self.generate_ico(&icon_processor, &portable_dir) {
                println!("    Generated icon: {}", ico_path.display());
            }
        }

        // 5. Create README for the portable package
        let readme_content = format!(
            r#"{app_name} v{version}
========================

This is a portable Windows application.

To run:
1. Double-click {exe_name}

To create a desktop shortcut:
1. Right-click {exe_name}
2. Select "Send to" > "Desktop (create shortcut)"

No installation required. To uninstall, simply delete this folder.

---
Built with Forge (https://forge-deno.com)
"#,
            app_name = app_name,
            version = version,
            exe_name = exe_name,
        );
        fs::write(portable_dir.join("README.txt"), readme_content)?;

        // 6. Optional code signing
        let should_sign = self
            .manifest
            .bundle
            .windows
            .as_ref()
            .map(|c| c.sign.unwrap_or(false))
            .unwrap_or(false);

        if should_sign {
            println!("  Signing executable...");
            self.sign_file(&dest_exe)?;
        }

        println!("\n  Portable package created: {}", portable_dir.display());
        Ok(portable_dir)
    }

    /// Create ZIP archive of portable package
    fn bundle_zip(&self) -> Result<PathBuf> {
        // First create portable package
        let portable_dir = self.bundle_portable()?;

        let app_name = &self.manifest.app.name;
        let version = &self.manifest.app.version;

        println!("  Creating ZIP archive...");

        let zip_name = format!("{}-{}-win.zip", sanitize_name(app_name), version);
        let zip_path = self.output_dir.join(&zip_name);

        // Use PowerShell Compress-Archive for ZIP creation
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Compress-Archive -Path '{}\\*' -DestinationPath '{}' -Force",
                    portable_dir.display(),
                    zip_path.display()
                ),
            ])
            .status()
            .context("Failed to run PowerShell Compress-Archive")?;

        if !status.success() {
            bail!("Failed to create ZIP archive");
        }

        // Clean up portable directory
        let _ = fs::remove_dir_all(&portable_dir);

        println!("\n  ZIP archive created: {}", zip_path.display());
        Ok(zip_path)
    }

    /// Create NSIS installer
    fn bundle_nsis(&self) -> Result<PathBuf> {
        println!("  Creating NSIS installer...");

        // First create portable package as staging
        let staging_dir = self.output_dir.join("nsis_staging");
        if staging_dir.exists() {
            fs::remove_dir_all(&staging_dir)?;
        }
        fs::create_dir_all(&staging_dir)?;

        let app_name = &self.manifest.app.name;
        let version = &self.manifest.app.version;
        let exe_name = format!("{}.exe", sanitize_name(app_name));

        // 1. Build forge-host with embedded assets
        let binary_path = build_embedded_binary(self.dist_dir)?;

        // 2. Copy executable to staging
        let staged_exe = staging_dir.join(&exe_name);
        fs::copy(&binary_path, &staged_exe)?;

        // 3. Copy resources
        let resources_dir = staging_dir.join("resources");
        fs::create_dir_all(&resources_dir)?;
        fs::copy(
            self.dist_dir.join("manifest.app.toml"),
            resources_dir.join("manifest.app.toml"),
        )?;

        let src_dir = self.dist_dir.join("src");
        if src_dir.exists() {
            copy_dir_recursive(&src_dir, &resources_dir.join("src"))?;
        }

        // 4. Generate icon
        let icon_base = self.manifest.bundle.icon.as_deref();
        let ico_path = if let Ok(icon_processor) = IconProcessor::find_icon(self.app_dir, icon_base)
        {
            self.generate_ico(&icon_processor, &staging_dir)?
        } else {
            // No icon, NSIS will use default
            staging_dir.join("icon.ico")
        };

        // 5. Generate NSIS script
        let nsi_content = self.generate_nsis_script(&exe_name, &ico_path)?;
        let nsi_path = staging_dir.join("installer.nsi");
        fs::write(&nsi_path, &nsi_content)?;

        // 6. Run NSIS
        let installer_name = format!("{}-{}-setup.exe", sanitize_name(app_name), version);
        let installer_path = self.output_dir.join(&installer_name);

        // Find makensis
        let makensis = find_makensis()?;

        let status = Command::new(&makensis)
            .arg(&nsi_path)
            .current_dir(&staging_dir)
            .status()
            .context("Failed to run makensis")?;

        if !status.success() {
            bail!("NSIS compilation failed");
        }

        // Move installer to output directory
        let staged_installer = staging_dir.join(&installer_name);
        if staged_installer.exists() {
            fs::rename(&staged_installer, &installer_path)?;
        }

        // Clean up staging
        let _ = fs::remove_dir_all(&staging_dir);

        // 7. Optional code signing
        let should_sign = self
            .manifest
            .bundle
            .windows
            .as_ref()
            .map(|c| c.sign.unwrap_or(false))
            .unwrap_or(false);

        if should_sign {
            println!("  Signing installer...");
            self.sign_file(&installer_path)?;
        }

        println!("\n  Installer created: {}", installer_path.display());
        Ok(installer_path)
    }

    /// Generate ICO file from icon processor
    fn generate_ico(&self, icon_processor: &IconProcessor, output_dir: &Path) -> Result<PathBuf> {
        let ico_path = output_dir.join("icon.ico");

        // ICO files contain multiple sizes - we'll create a multi-resolution icon
        // For simplicity, we'll generate PNGs and use an external tool if available,
        // or fall back to a single-size ICO

        // Try to use ImageMagick convert if available
        let temp_png = output_dir.join("temp_icon.png");
        icon_processor.save_resized(&temp_png, 256, 256)?;

        let convert_result = Command::new("magick")
            .args([
                "convert",
                &temp_png.display().to_string(),
                "-define",
                "icon:auto-resize=256,128,64,48,32,16",
                &ico_path.display().to_string(),
            ])
            .status();

        let _ = fs::remove_file(&temp_png);

        match convert_result {
            Ok(status) if status.success() => Ok(ico_path),
            _ => {
                // Fallback: try with png2ico if available
                let png_256 = output_dir.join("icon_256.png");
                let png_48 = output_dir.join("icon_48.png");
                let png_32 = output_dir.join("icon_32.png");
                let png_16 = output_dir.join("icon_16.png");

                icon_processor.save_resized(&png_256, 256, 256)?;
                icon_processor.save_resized(&png_48, 48, 48)?;
                icon_processor.save_resized(&png_32, 32, 32)?;
                icon_processor.save_resized(&png_16, 16, 16)?;

                // Try png2ico
                let png2ico_result = Command::new("png2ico")
                    .args([
                        &ico_path.display().to_string(),
                        &png_256.display().to_string(),
                        &png_48.display().to_string(),
                        &png_32.display().to_string(),
                        &png_16.display().to_string(),
                    ])
                    .status();

                // Clean up temp PNGs
                let _ = fs::remove_file(&png_256);
                let _ = fs::remove_file(&png_48);
                let _ = fs::remove_file(&png_32);
                let _ = fs::remove_file(&png_16);

                match png2ico_result {
                    Ok(status) if status.success() => Ok(ico_path),
                    _ => {
                        // Last resort: just use a PNG renamed to .ico (some apps accept this)
                        println!(
                            "    Warning: Could not generate proper ICO. Install ImageMagick for better icon support."
                        );
                        let fallback_png = output_dir.join("icon_fallback.png");
                        icon_processor.save_resized(&fallback_png, 256, 256)?;
                        fs::rename(&fallback_png, &ico_path)?;
                        Ok(ico_path)
                    }
                }
            }
        }
    }

    /// Generate NSIS installer script
    fn generate_nsis_script(&self, exe_name: &str, ico_path: &Path) -> Result<String> {
        let app = &self.manifest.app;
        let windows_config = self.manifest.bundle.windows.as_ref();

        let publisher = windows_config
            .and_then(|c| c.publisher.as_ref())
            .map(|p| extract_cn_from_publisher(p))
            .unwrap_or_else(|| app.name.clone());

        let installer_name = format!("{}-{}-setup.exe", sanitize_name(&app.name), app.version);

        let ico_include = if ico_path.exists() {
            format!(
                r#"!define MUI_ICON "{}"
!define MUI_UNICON "{}""#,
                ico_path.display(),
                ico_path.display()
            )
        } else {
            String::new()
        };

        Ok(format!(
            r#"; NSIS Installer Script for {app_name}
; Generated by Forge

!include "MUI2.nsh"
!include "FileFunc.nsh"

; General settings
Name "{app_name}"
OutFile "{installer_name}"
InstallDir "$PROGRAMFILES64\{app_name}"
InstallDirRegKey HKLM "Software\{app_name}" "InstallDir"
RequestExecutionLevel admin

; UI settings
{ico_include}
!define MUI_ABORTWARNING

; Installer pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language
!insertmacro MUI_LANGUAGE "English"

; Version info
VIProductVersion "{version_quad}"
VIAddVersionKey "ProductName" "{app_name}"
VIAddVersionKey "CompanyName" "{publisher}"
VIAddVersionKey "FileDescription" "{app_name} Installer"
VIAddVersionKey "FileVersion" "{version}"
VIAddVersionKey "ProductVersion" "{version}"
VIAddVersionKey "LegalCopyright" "Copyright (c) {publisher}"

; Installer section
Section "Install"
    SetOutPath "$INSTDIR"

    ; Copy files
    File "{exe_name}"
    File /r "resources"

    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Create Start Menu shortcut
    CreateDirectory "$SMPROGRAMS\{app_name}"
    CreateShortcut "$SMPROGRAMS\{app_name}\{app_name}.lnk" "$INSTDIR\{exe_name}"
    CreateShortcut "$SMPROGRAMS\{app_name}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

    ; Create Desktop shortcut
    CreateShortcut "$DESKTOP\{app_name}.lnk" "$INSTDIR\{exe_name}"

    ; Write registry keys for uninstall
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "DisplayName" "{app_name}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "Publisher" "{publisher}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "DisplayVersion" "{version}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "NoRepair" 1

    ; Calculate installed size
    ${{GetSize}} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}" "EstimatedSize" $0

    ; Store install directory
    WriteRegStr HKLM "Software\{app_name}" "InstallDir" "$INSTDIR"
SectionEnd

; Uninstaller section
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\{exe_name}"
    RMDir /r "$INSTDIR\resources"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    ; Remove shortcuts
    Delete "$SMPROGRAMS\{app_name}\{app_name}.lnk"
    Delete "$SMPROGRAMS\{app_name}\Uninstall.lnk"
    RMDir "$SMPROGRAMS\{app_name}"
    Delete "$DESKTOP\{app_name}.lnk"

    ; Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\{app_name}"
    DeleteRegKey HKLM "Software\{app_name}"
SectionEnd
"#,
            app_name = app.name,
            installer_name = installer_name,
            exe_name = exe_name,
            publisher = publisher,
            version = app.version,
            version_quad = normalize_version(&app.version),
            ico_include = ico_include,
        ))
    }

    /// Sign a file using SignTool
    fn sign_file(&self, file_path: &Path) -> Result<()> {
        let windows_config = self
            .manifest
            .bundle
            .windows
            .as_ref()
            .context("Windows bundle config required for signing")?;

        let cert_path = windows_config
            .certificate
            .as_ref()
            .context("Certificate path required for signing (bundle.windows.certificate)")?;

        let password = windows_config.resolve_password();

        // Find SignTool
        let signtool = find_signtool()?;

        let mut cmd = Command::new(&signtool);
        cmd.args([
            "sign",
            "/fd",
            "SHA256",
            "/tr",
            "http://timestamp.digicert.com",
            "/td",
            "SHA256",
        ]);
        cmd.args(["/f", cert_path]);

        if let Some(pwd) = password {
            cmd.args(["/p", &pwd]);
        }

        cmd.arg(file_path);

        let status = cmd.status().context("Failed to run SignTool")?;

        if !status.success() {
            bail!("SignTool failed with status: {}", status);
        }

        println!("    Signed: {}", file_path.display());
        Ok(())
    }
}

/// Find SignTool.exe in Windows SDK locations
fn find_signtool() -> Result<PathBuf> {
    // Common Windows SDK paths
    let sdk_bases = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];

    for sdk_base in &sdk_bases {
        if let Ok(entries) = fs::read_dir(sdk_base) {
            let mut versions: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();

            // Sort descending to prefer newer versions
            versions.sort();
            versions.reverse();

            for version_dir in versions {
                let signtool = version_dir.join("x64").join("signtool.exe");
                if signtool.exists() {
                    return Ok(signtool);
                }
            }
        }
    }

    // Try PATH
    if Command::new("signtool").arg("/?").output().is_ok() {
        return Ok(PathBuf::from("signtool"));
    }

    bail!(
        "SignTool.exe not found.\n\
        Install Windows SDK from:\n\
        https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/\n\
        Or add SignTool to PATH."
    )
}

/// Find NSIS makensis.exe
fn find_makensis() -> Result<PathBuf> {
    // Common NSIS installation paths
    let candidates = [
        r"C:\Program Files (x86)\NSIS\makensis.exe",
        r"C:\Program Files\NSIS\makensis.exe",
        r"C:\NSIS\makensis.exe",
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try PATH
    if let Ok(output) = Command::new("where").arg("makensis").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path.lines().next().unwrap_or(&path)));
            }
        }
    }

    bail!(
        "NSIS not found. Install from:\n\
        https://nsis.sourceforge.io/Download\n\
        Or add makensis.exe to PATH."
    )
}

/// Extract CN (Common Name) from a publisher DN string
fn extract_cn_from_publisher(publisher: &str) -> String {
    // Publisher format: "CN=Company Name, O=Company, C=US"
    if let Some(cn_start) = publisher.find("CN=") {
        let after_cn = &publisher[cn_start + 3..];
        if let Some(comma_pos) = after_cn.find(',') {
            return after_cn[..comma_pos].trim().to_string();
        }
        return after_cn.trim().to_string();
    }
    publisher.to_string()
}

/// Normalize version to 4-part format for Windows
fn normalize_version(version: &str) -> String {
    let clean_version = version.split('-').next().unwrap_or(version);
    let parts: Vec<&str> = clean_version.split('.').collect();

    match parts.len() {
        1 => format!("{}.0.0.0", parts[0]),
        2 => format!("{}.{}.0.0", parts[0], parts[1]),
        3 => format!("{}.{}.{}.0", parts[0], parts[1], parts[2]),
        _ => format!("{}.{}.{}.0", parts[0], parts[1], parts[2]),
    }
}

/// Main bundle entry point
pub fn bundle(
    app_dir: &Path,
    dist_dir: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    let bundler = WindowsBundler::new(app_dir, dist_dir, output_dir, manifest);
    bundler.bundle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cn_from_publisher() {
        assert_eq!(
            extract_cn_from_publisher("CN=My Company, O=Org, C=US"),
            "My Company"
        );
        assert_eq!(extract_cn_from_publisher("CN=Simple"), "Simple");
        assert_eq!(
            extract_cn_from_publisher("Something Else"),
            "Something Else"
        );
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("1"), "1.0.0.0");
        assert_eq!(normalize_version("1.0"), "1.0.0.0");
        assert_eq!(normalize_version("1.0.0"), "1.0.0.0");
        assert_eq!(normalize_version("1.2.3.4"), "1.2.3.0");
        assert_eq!(normalize_version("1.0.0-beta.1"), "1.0.0.0");
    }
}
