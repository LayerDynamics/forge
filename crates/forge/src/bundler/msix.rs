//! Windows MSIX packaging backend
//!
//! Creates MSIX packages for Windows 10+ distribution.
//! MSIX is essentially a ZIP container with specific structure and manifest.

use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions, CompressionMethod};

use super::{AppManifest, IconProcessor, build_embedded_binary, sanitize_name, sanitize_msix_name};

/// Windows MSIX bundler
pub struct MsixBundler<'a> {
    app_dir: &'a Path,
    dist_dir: &'a Path,
    output_dir: &'a Path,
    manifest: &'a AppManifest,
}

impl<'a> MsixBundler<'a> {
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

    /// Execute full MSIX bundling pipeline
    pub fn bundle(&self) -> Result<PathBuf> {
        println!("Creating Windows MSIX package...");

        // 1. Create staging directory
        let stage_dir = self.output_dir.join("msix_stage");
        if stage_dir.exists() {
            fs::remove_dir_all(&stage_dir)?;
        }
        fs::create_dir_all(&stage_dir)?;

        // 2. Build forge-host with embedded assets
        let binary_path = build_embedded_binary(self.dist_dir)?;

        // 3. Copy executable to staging with proper name
        let exe_name = format!("{}.exe", sanitize_name(&self.manifest.app.name));
        let staged_exe = stage_dir.join(&exe_name);
        fs::copy(&binary_path, &staged_exe)
            .with_context(|| format!("Failed to copy {} to staging", binary_path.display()))?;

        // 4. Generate icons
        println!("  Generating icons...");
        let icon_base = self.manifest.bundle.icon.as_deref();
        let icon_processor = IconProcessor::find_icon(self.app_dir, icon_base)?;
        let assets_dir = stage_dir.join("Assets");
        icon_processor.generate_msix_icons(&assets_dir)?;

        // 5. Generate AppxManifest.xml
        println!("  Generating AppxManifest.xml...");
        let manifest_xml = self.generate_appx_manifest(&exe_name)?;
        fs::write(stage_dir.join("AppxManifest.xml"), &manifest_xml)?;

        // 6. Create MSIX package (ZIP container)
        println!("  Creating MSIX package...");
        let msix_name = format!(
            "{}-{}.msix",
            sanitize_name(&self.manifest.app.name),
            self.manifest.app.version.replace('.', "_")
        );
        let msix_path = self.output_dir.join(&msix_name);
        self.create_msix_zip(&stage_dir, &msix_path)?;

        // 7. Optional: Sign with SignTool
        let windows_config = self.manifest.bundle.windows.as_ref();
        if windows_config.map(|c| c.sign.unwrap_or(false)).unwrap_or(false) {
            println!("  Signing package...");
            self.sign_package(&msix_path)?;
        }

        // Keep staging for debugging, but could clean up:
        // fs::remove_dir_all(&stage_dir)?;

        println!("\n  MSIX package created: {}", msix_path.display());
        Ok(msix_path)
    }

    /// Generate AppxManifest.xml content
    fn generate_appx_manifest(&self, exe_name: &str) -> Result<String> {
        let app = &self.manifest.app;
        let windows_config = self.manifest.bundle.windows.as_ref();

        let msix_name = sanitize_msix_name(&app.identifier);
        let version = self.manifest.version_quad();
        let publisher = windows_config
            .map(|c| c.publisher_or_default(&app.name))
            .unwrap_or_else(|| format!("CN={}", app.name));
        let min_version = windows_config
            .map(|c| c.min_version_or_default())
            .unwrap_or_else(|| "10.0.17763.0".to_string());
        let capabilities = windows_config
            .map(|c| c.capabilities_or_default())
            .unwrap_or_else(|| vec!["internetClient".to_string()]);

        let capabilities_xml = capabilities
            .iter()
            .map(|cap| {
                if cap.contains(':') {
                    // Prefixed capability (e.g., rescap:runFullTrust)
                    format!("    <{} />", cap)
                } else {
                    format!("    <Capability Name=\"{}\" />", cap)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let description = app.name.clone();

        Ok(format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10"
  IgnorableNamespaces="uap rescap desktop">

  <Identity
    Name="{msix_name}"
    Publisher="{publisher}"
    Version="{version}"
    ProcessorArchitecture="x64" />

  <Properties>
    <DisplayName>{display_name}</DisplayName>
    <PublisherDisplayName>{publisher_display}</PublisherDisplayName>
    <Description>{description}</Description>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>

  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="{min_version}" MaxVersionTested="10.0.22621.0" />
  </Dependencies>

  <Resources>
    <Resource Language="en-us" />
  </Resources>

  <Applications>
    <Application Id="App" Executable="{exe_name}" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="{display_name}"
        Description="{description}"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\Square150x150Logo.png"
        Square44x44Logo="Assets\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" />
        <uap:SplashScreen Image="Assets\SplashScreen.png" />
      </uap:VisualElements>
    </Application>
  </Applications>

  <Capabilities>
{capabilities}
  </Capabilities>

</Package>"#,
            msix_name = msix_name,
            publisher = publisher,
            version = version,
            display_name = app.name,
            publisher_display = app.name,
            description = description,
            min_version = min_version,
            exe_name = exe_name,
            capabilities = capabilities_xml,
        ))
    }

    /// Create MSIX ZIP container
    fn create_msix_zip(&self, source_dir: &Path, output_path: &Path) -> Result<()> {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create {}", output_path.display()))?;
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Walk the staging directory and add all files
        for entry in WalkDir::new(source_dir) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(source_dir)?;

            if path.is_file() {
                let name = relative_path.to_string_lossy().replace('\\', "/");
                zip.start_file(&name, options)?;

                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            } else if path.is_dir() && path != source_dir {
                let name = format!("{}/", relative_path.to_string_lossy().replace('\\', "/"));
                zip.add_directory(&name, options)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    /// Sign MSIX with SignTool (Windows SDK required)
    fn sign_package(&self, msix_path: &Path) -> Result<()> {
        let windows_config = self.manifest.bundle.windows.as_ref()
            .context("Windows bundle config required for signing")?;

        let cert_path = windows_config.certificate.as_ref()
            .context("Certificate path required for signing (bundle.windows.certificate)")?;

        let password = windows_config.resolve_password();

        // Find SignTool
        let signtool = find_signtool()?;

        let mut cmd = Command::new(&signtool);
        cmd.args(["sign", "/fd", "SHA256"]);
        cmd.args(["/f", cert_path]);

        if let Some(pwd) = password {
            cmd.args(["/p", &pwd]);
        }

        cmd.arg(msix_path);

        let status = cmd.status()
            .context("Failed to run SignTool")?;

        if !status.success() {
            bail!("SignTool failed with status: {}", status);
        }

        println!("    Package signed successfully");
        Ok(())
    }
}

/// Find SignTool.exe in Windows SDK locations
fn find_signtool() -> Result<PathBuf> {
    // Common Windows SDK paths
    let sdk_base = r"C:\Program Files (x86)\Windows Kits\10\bin";

    // Try to find any SDK version
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

/// Main bundle entry point
pub fn bundle(
    app_dir: &Path,
    dist_dir: &Path,
    output_dir: &Path,
    manifest: &AppManifest,
) -> Result<PathBuf> {
    let bundler = MsixBundler::new(app_dir, dist_dir, output_dir, manifest);
    bundler.bundle()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_appx_manifest() {
        // This would require a full manifest setup, so just a sanity check
        let msix_name = sanitize_msix_name("com.example.app");
        assert_eq!(msix_name, "com.example.app");
    }
}
