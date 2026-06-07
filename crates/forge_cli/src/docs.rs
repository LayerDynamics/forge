//! Documentation generation command for Forge CLI
//!
//! This module provides the `forge docs` command for generating API documentation
//! from extension TypeScript and Rust source files.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Run the docs command with the given arguments
pub fn run(args: &[String]) -> Result<()> {
    let cmd = DocsCommand::parse(args)?;

    if cmd.all_extensions {
        generate_all_extensions(&cmd)?;
    } else if let Some(ref ext) = cmd.extension {
        generate_single_extension(ext, &cmd)?;
    } else {
        generate_app_docs(&cmd)?;
    }

    Ok(())
}

/// Documentation command configuration
struct DocsCommand {
    /// Target app or extension directory
    target: PathBuf,
    /// Output directory for generated docs
    output: PathBuf,
    /// Output format: astro, html, or both
    format: String,
    /// Generate docs for all extensions
    all_extensions: bool,
    /// Specific extension to document
    extension: Option<String>,
}

impl DocsCommand {
    fn parse(args: &[String]) -> Result<Self> {
        let mut cmd = DocsCommand {
            target: PathBuf::from("."),
            output: PathBuf::from("docs"),
            format: "astro".to_string(),
            all_extensions: false,
            extension: None,
        };

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        cmd.output = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        bail!("--output requires a value");
                    }
                }
                "--format" | "-f" => {
                    if i + 1 < args.len() {
                        cmd.format = args[i + 1].clone();
                        i += 2;
                    } else {
                        bail!("--format requires a value (astro, html, or both)");
                    }
                }
                "--all-extensions" => {
                    cmd.all_extensions = true;
                    i += 1;
                }
                "--extension" | "-e" => {
                    if i + 1 < args.len() {
                        cmd.extension = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        bail!("--extension requires a value (e.g., fs, window)");
                    }
                }
                arg if !arg.starts_with('-') => {
                    cmd.target = PathBuf::from(arg);
                    i += 1;
                }
                _ => {
                    bail!("Unknown flag: {}", args[i]);
                }
            }
        }

        Ok(cmd)
    }
}

/// Discover every runtime extension by scanning `crates/ext_*` and reading the
/// module specifier from each crate's `build.rs`. This is the single source of
/// truth used by the binding generator (`ExtensionBuilder::new(reg, specifier)`),
/// so the list can never silently fall behind the crates on disk the way a
/// hardcoded array did.
///
/// Returns sorted `(short_name, specifier)` pairs where `short_name` is the
/// crate directory without the `ext_` prefix (e.g. `image_tools` →
/// `runtime:image_tools`, `etcher` → `forge:etcher`).
fn discover_extensions(workspace_root: &Path) -> Result<Vec<(String, String)>> {
    let crates_dir = workspace_root.join("crates");
    let mut extensions = Vec::new();

    for entry in std::fs::read_dir(&crates_dir)
        .with_context(|| format!("reading crates directory {}", crates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let short = match dir_name.strip_prefix("ext_") {
            Some(s) => s,
            None => continue,
        };

        // Prefer the specifier declared in build.rs; fall back to runtime:<name>
        // for an extension crate that has no ExtensionBuilder call.
        let specifier = std::fs::read_to_string(path.join("build.rs"))
            .ok()
            .and_then(|src| extract_specifier(&src))
            .unwrap_or_else(|| format!("runtime:{short}"));

        extensions.push((short.to_string(), specifier));
    }

    extensions.sort();
    Ok(extensions)
}

/// Extract the module specifier (second string argument) from a build.rs
/// containing `ExtensionBuilder::new("<reg_name>", "<specifier>")`.
fn extract_specifier(build_src: &str) -> Option<String> {
    let start = build_src.find("ExtensionBuilder::new")?;
    // Quoted string literals after the call site appear at odd split indices:
    // [before] " arg1 " [between] " arg2 " ...  → arg2 is the specifier.
    let mut parts = build_src[start..].split('"');
    parts.next()?; // text before the first quote
    parts.next()?; // arg1: registration name
    parts.next()?; // separator between the two string args
    let specifier = parts.next()?; // arg2: module specifier
    if specifier.is_empty() {
        None
    } else {
        Some(specifier.to_string())
    }
}

fn generate_all_extensions(cmd: &DocsCommand) -> Result<()> {
    println!("Generating documentation for all extensions...");

    // Find workspace root (where crates/ directory is)
    let workspace_root = find_workspace_root()?;
    let crates_dir = workspace_root.join("crates");

    let extensions = discover_extensions(&workspace_root)?;

    let mut generated_count = 0;
    let mut skipped_count = 0;

    for (name, specifier) in &extensions {
        let ext_path = crates_dir.join(format!("ext_{}", name));
        if ext_path.exists() {
            let output_dir = cmd.output.join(name);
            match generate_extension_docs(&ext_path, name, specifier, &output_dir, &cmd.format) {
                Ok(_) => generated_count += 1,
                Err(e) => {
                    eprintln!("  Warning: Failed to generate docs for {}: {}", name, e);
                    skipped_count += 1;
                }
            }
        } else {
            skipped_count += 1;
        }
    }

    println!(
        "\nDocumentation generation complete: {} generated, {} skipped",
        generated_count, skipped_count
    );
    println!("Output directory: {}", cmd.output.display());

    Ok(())
}

fn generate_single_extension(name: &str, cmd: &DocsCommand) -> Result<()> {
    // Find workspace root
    let workspace_root = find_workspace_root()?;
    let ext_path = workspace_root.join("crates").join(format!("ext_{}", name));

    let extensions = discover_extensions(&workspace_root)?;

    if !ext_path.exists() {
        bail!(
            "Extension not found: ext_{}\n\
            Available extensions: {}",
            name,
            extensions
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Use the specifier discovered from the crate's build.rs; fall back to the
    // conventional runtime:<name> form for a crate without an ExtensionBuilder.
    let specifier = extensions
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, s)| s.clone())
        .unwrap_or_else(|| format!("runtime:{name}"));

    generate_extension_docs(&ext_path, name, &specifier, &cmd.output, &cmd.format)
}

fn generate_extension_docs(
    crate_path: &Path,
    name: &str,
    specifier: &str,
    output: &Path,
    format: &str,
) -> Result<()> {
    use forge_etch::EtchBuilder;

    let mut builder = EtchBuilder::new(format!("host_{}", name), specifier).output_dir(output);

    // TypeScript source
    let ts_path = crate_path.join("ts/init.ts");
    if ts_path.exists() {
        builder = builder.ts_source(ts_path);
    }

    // Rust source
    let rust_path = crate_path.join("src/lib.rs");
    if rust_path.exists() {
        builder = builder.rust_source(rust_path);
    }

    // Format
    builder = match format {
        "html" => builder.generate_astro(false).generate_html(true),
        "both" => builder.generate_astro(true).generate_html(true),
        _ => builder.generate_astro(true).generate_html(false),
    };

    let result = builder.build()?;
    println!(
        "  ✓ {} -> {} (source only, no dependencies)",
        specifier,
        result.output_dir.display()
    );

    Ok(())
}

fn generate_app_docs(cmd: &DocsCommand) -> Result<()> {
    use forge_etch::EtchBuilder;

    // Document an app's TypeScript source
    let src_path = cmd.target.join("src/main.ts");
    if !src_path.exists() {
        bail!(
            "No src/main.ts found in target directory: {}\n\n\
            Usage:\n  \
            forge docs <app-dir>                 Document an app\n  \
            forge docs --extension fs            Document a specific extension\n  \
            forge docs --all-extensions          Document all extensions\n\n\
            Options:\n  \
            --output, -o <dir>                   Output directory (default: docs)\n  \
            --format, -f <astro|html|both>       Output format (default: astro)",
            cmd.target.display()
        );
    }

    println!(
        "Generating documentation for app at {}",
        cmd.target.display()
    );

    let builder = EtchBuilder::new("app", "app")
        .ts_source(&src_path)
        .output_dir(&cmd.output)
        .generate_astro(cmd.format == "astro" || cmd.format == "both")
        .generate_html(cmd.format == "html" || cmd.format == "both");

    builder.build()?;
    println!(
        "  ✓ Generated app documentation -> {}",
        cmd.output.display()
    );

    Ok(())
}

/// Find the workspace root by looking for Cargo.toml with [workspace]
fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => bail!(
                "Could not find workspace root. \
                Run this command from within the Forge workspace."
            ),
        }
    }
}

/// Print docs command usage
pub fn usage() {
    eprintln!("forge docs [options] [target]");
    eprintln!();
    eprintln!("Generate API documentation from TypeScript/Rust source files.");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  [target]                           App directory to document (default: .)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --output, -o <dir>                 Output directory (default: docs)");
    eprintln!("  --format, -f <astro|html|both>     Output format (default: astro)");
    eprintln!("  --all-extensions                   Generate docs for all runtime extensions");
    eprintln!("  --extension, -e <name>             Generate docs for specific extension");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  forge docs my-app                              Document an app");
    eprintln!("  forge docs --extension fs -o docs/api/fs       Document runtime:fs");
    eprintln!("  forge docs --all-extensions -o site/docs/api   Document all extensions");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_specifier_reads_second_string_arg() {
        assert_eq!(
            extract_specifier(r#"ExtensionBuilder::new("runtime_dock", "runtime:dock")"#),
            Some("runtime:dock".to_string())
        );
        // forge:* tooling specifier.
        assert_eq!(
            extract_specifier(
                r#"    let b = ExtensionBuilder::new("ext_etcher_runtime", "forge:etcher");"#
            ),
            Some("forge:etcher".to_string())
        );
        // No ExtensionBuilder call -> None (caller falls back to runtime:<name>).
        assert_eq!(extract_specifier("fn main() {}"), None);
    }

    #[test]
    fn discover_extensions_finds_every_ext_crate_with_correct_specifier() {
        let root = find_workspace_root().expect("workspace root");
        let exts = discover_extensions(&root).expect("discover extensions");

        // Count matches the number of crates/ext_* directories on disk.
        let on_disk = std::fs::read_dir(root.join("crates"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("ext_"))
            .count();
        assert_eq!(
            exts.len(),
            on_disk,
            "discovery must cover every ext_* crate"
        );

        let find = |name: &str| -> Option<String> {
            exts.iter().find(|(n, _)| n == name).map(|(_, s)| s.clone())
        };
        // Previously-missing extensions are now discovered.
        assert_eq!(find("console").as_deref(), Some("runtime:console"));
        assert_eq!(find("dock").as_deref(), Some("runtime:dock"));
        assert_eq!(find("image_tools").as_deref(), Some("runtime:image_tools"));
        // forge:* tooling specifiers are read correctly (not assumed runtime:*).
        assert_eq!(find("etcher").as_deref(), Some("forge:etcher"));
    }
}
