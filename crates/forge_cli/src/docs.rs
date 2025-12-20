//! Documentation generation command for Forge CLI
//!
//! This module provides the `forge docs` command for generating API documentation
//! from extension TypeScript and Rust source files.

use anyhow::{bail, Result};
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

/// All known extensions with their names and specifiers
const EXTENSIONS: &[(&str, &str)] = &[
    ("fs", "runtime:fs"),
    ("window", "runtime:window"),
    ("ipc", "runtime:ipc"),
    ("net", "runtime:net"),
    ("sys", "runtime:sys"),
    ("process", "runtime:process"),
    ("app", "runtime:app"),
    ("crypto", "runtime:crypto"),
    ("storage", "runtime:storage"),
    ("shell", "runtime:shell"),
    ("database", "runtime:database"),
    ("webview", "runtime:webview"),
    ("devtools", "runtime:devtools"),
    ("timers", "runtime:timers"),
    ("shortcuts", "runtime:shortcuts"),
    ("signals", "runtime:signals"),
    ("updater", "runtime:updater"),
    ("monitor", "runtime:monitor"),
    ("display", "runtime:display"),
    ("log", "runtime:log"),
    ("trace", "runtime:trace"),
    ("lock", "runtime:lock"),
    ("path", "runtime:path"),
    ("protocol", "runtime:protocol"),
    ("os_compat", "runtime:os_compat"),
    ("debugger", "runtime:debugger"),
    ("wasm", "runtime:wasm"),
    // Build/tooling extensions
    ("weld", "forge:weld"),
    ("etcher", "forge:etcher"),
    ("bundler", "forge:bundler"),
];

fn generate_all_extensions(cmd: &DocsCommand) -> Result<()> {
    println!("Generating documentation for all extensions...");

    // Find workspace root (where crates/ directory is)
    let workspace_root = find_workspace_root()?;
    let crates_dir = workspace_root.join("crates");

    let mut generated_count = 0;
    let mut skipped_count = 0;

    for (name, specifier) in EXTENSIONS {
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

    if !ext_path.exists() {
        bail!(
            "Extension not found: ext_{}\n\
            Available extensions: {}",
            name,
            EXTENSIONS
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let specifier = EXTENSIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| {
            // Fallback specifier format
            Box::leak(format!("runtime:{}", name).into_boxed_str())
        });

    generate_extension_docs(&ext_path, name, specifier, &cmd.output, &cmd.format)
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
