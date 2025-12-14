use anyhow::{anyhow, bail, Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

mod bundler;

// Include auto-generated template embedding code
include!(concat!(env!("OUT_DIR"), "/templates.rs"));

fn usage() {
    eprintln!("forge <init|dev|build|bundle|sign|icon> [options] <app-dir>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  init [--template <name>] <app-dir>  Create a new Forge app");
    eprintln!("  dev <app-dir>                       Run in development mode");
    eprintln!("  build <app-dir>                     Build for production");
    eprintln!("  bundle <app-dir>                    Package into distributable");
    eprintln!("  sign <artifact>                     Sign a package artifact");
    eprintln!("  icon <subcommand>                   Manage app icons");
    eprintln!();
    eprintln!("Icon subcommands:");
    eprintln!("  icon create <path>                  Create a placeholder icon");
    eprintln!("  icon validate <app-dir>             Validate app icon requirements");
    eprintln!();
    eprintln!("Templates for init:");
    eprintln!("  minimal (default)  Basic HTML + Deno app");
    eprintln!("  react              React with TypeScript");
    eprintln!("  vue                Vue.js with JavaScript");
    eprintln!("  svelte             Svelte with TypeScript");
    eprintln!();
    eprintln!("Bundle output formats:");
    eprintln!("  Windows: .msix package");
    eprintln!("  macOS:   .app bundle + .dmg disk image");
    eprintln!("  Linux:   .AppImage or .tar.gz");
}

/// Find the forge-host binary in standard locations
///
/// Search order:
/// 1. Same directory as forge binary (for installed binaries)
/// 2. ~/.forge/bin/ (standard install location)
/// 3. PATH (for manual installations)
/// 4. Development fallback (cargo run)
fn find_forge_host() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let binary_name = "forge-host.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "forge-host";

    // 1. Check same directory as forge binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let sibling = parent.join(binary_name);
            if sibling.exists() {
                return Ok(sibling);
            }
        }
    }

    // 2. Check ~/.forge/bin/
    if let Some(home) = dirs::home_dir() {
        let forge_bin = home.join(".forge").join("bin").join(binary_name);
        if forge_bin.exists() {
            return Ok(forge_bin);
        }
    }

    // 3. Check PATH
    if let Ok(path) = which::which(binary_name) {
        return Ok(path);
    }

    // 4. For development: try cargo run
    // Check if we're in a development environment by looking for Cargo.toml
    let cargo_toml = PathBuf::from("Cargo.toml");
    if cargo_toml.exists() {
        // Return a sentinel value that cmd_dev will recognize
        return Ok(PathBuf::from("__cargo_run__"));
    }

    bail!(
        "forge-host not found!\n\n\
        Install Forge with:\n  \
        curl -fsSL https://forge-deno.com/install.sh | sh"
    )
}

fn cmd_init(app_dir: &Path, template: &str) -> Result<()> {
    use bundler::{IconProcessor, RECOMMENDED_ICON_SIZE};

    if app_dir.exists() {
        return Err(anyhow!("path exists: {}", app_dir.display()));
    }

    match template {
        "minimal" => init_minimal(app_dir)?,
        "react" => init_react(app_dir)?,
        "vue" => init_vue(app_dir)?,
        "svelte" => init_svelte(app_dir)?,
        _ => {
            return Err(anyhow!(
                "Unknown template: {}. Use: minimal, react, vue, svelte",
                template
            ))
        }
    }

    // Create assets directory and generate placeholder icon
    let assets_dir = app_dir.join("assets");
    fs::create_dir_all(&assets_dir)?;
    let icon_path = assets_dir.join("icon.png");
    let processor = IconProcessor::create_placeholder(RECOMMENDED_ICON_SIZE);
    processor.save(&icon_path)?;

    println!("Initialized {} app at {}", template, app_dir.display());
    println!("\nCreated:");
    println!("  - App template files");
    println!("  - Placeholder icon at assets/icon.png");
    println!();
    println!("IMPORTANT: Replace assets/icon.png with your actual app icon before bundling.");
    println!("Icon requirements: 1024x1024 PNG with transparency");
    println!("\nNext steps:");
    println!("  cd {}", app_dir.display());
    println!("  forge dev .");
    Ok(())
}

fn init_minimal(app_dir: &Path) -> Result<()> {
    fs::create_dir_all(app_dir.join("web"))?;
    fs::create_dir_all(app_dir.join("src"))?;
    fs::write(app_dir.join("manifest.app.toml"), minimal::MANIFEST)?;
    fs::write(app_dir.join("deno.json"), minimal::DENO_JSON)?;
    fs::write(app_dir.join("src/main.ts"), minimal::SRC_MAIN)?;
    fs::write(app_dir.join("web/index.html"), minimal::web::INDEX_HTML)?;
    Ok(())
}

fn init_react(app_dir: &Path) -> Result<()> {
    fs::create_dir_all(app_dir.join("web"))?;
    fs::create_dir_all(app_dir.join("src"))?;
    fs::write(app_dir.join("manifest.app.toml"), react::MANIFEST)?;
    fs::write(app_dir.join("deno.json"), react::DENO_JSON)?;
    fs::write(app_dir.join("src/main.ts"), react::SRC_MAIN)?;
    fs::write(app_dir.join("web/index.html"), react::web::INDEX_HTML)?;
    fs::write(app_dir.join("web/main.tsx"), react::web::MAIN_TSX)?;
    Ok(())
}

fn init_vue(app_dir: &Path) -> Result<()> {
    fs::create_dir_all(app_dir.join("web"))?;
    fs::create_dir_all(app_dir.join("src"))?;
    fs::write(app_dir.join("manifest.app.toml"), vue::MANIFEST)?;
    fs::write(app_dir.join("deno.json"), vue::DENO_JSON)?;
    fs::write(app_dir.join("src/main.ts"), vue::SRC_MAIN)?;
    fs::write(app_dir.join("web/index.html"), vue::web::INDEX_HTML)?;
    fs::write(app_dir.join("web/main.js"), vue::web::MAIN_JS)?;
    Ok(())
}

fn init_svelte(app_dir: &Path) -> Result<()> {
    fs::create_dir_all(app_dir.join("web"))?;
    fs::create_dir_all(app_dir.join("src"))?;
    fs::write(app_dir.join("manifest.app.toml"), svelte::MANIFEST)?;
    fs::write(app_dir.join("deno.json"), svelte::DENO_JSON)?;
    fs::write(app_dir.join("src/main.ts"), svelte::SRC_MAIN)?;
    fs::write(app_dir.join("web/index.html"), svelte::web::INDEX_HTML)?;
    fs::write(app_dir.join("web/main.ts"), svelte::web::MAIN_TS)?;
    fs::write(app_dir.join("web/App.svelte"), svelte::web::APP_SVELTE)?;
    Ok(())
}

fn cmd_dev(app_dir: &Path) -> Result<()> {
    let forge_host = find_forge_host()?;

    // If we're in development mode (sentinel value), use cargo run
    if forge_host.to_string_lossy() == "__cargo_run__" {
        let status = Command::new("cargo")
            .args([
                "run",
                "-p",
                "forge-host",
                "--",
                "--app-dir",
                &app_dir.display().to_string(),
                "--dev",
            ])
            .status()
            .context("Failed to run cargo")?;
        if !status.success() {
            bail!("forge-host failed");
        }
        return Ok(());
    }

    // Run the found forge-host binary
    let status = Command::new(&forge_host)
        .args(["--app-dir", &app_dir.display().to_string(), "--dev"])
        .status()
        .context("Failed to run forge-host")?;

    if !status.success() {
        bail!("forge-host exited with error");
    }

    Ok(())
}
/// Detected framework type
#[derive(Debug, PartialEq)]
enum Framework {
    Minimal,
    React,
    Vue,
    Svelte,
}

/// Detect the framework used in the app
fn detect_framework(app_dir: &Path) -> Result<Framework> {
    let deno_json = app_dir.join("deno.json");
    if deno_json.exists() {
        let content = fs::read_to_string(&deno_json)?;
        if content.contains("react") || content.contains("React") {
            return Ok(Framework::React);
        }
        if content.contains("svelte") || content.contains("Svelte") {
            return Ok(Framework::Svelte);
        }
    }

    // Check for Vue/Svelte files
    let web_dir = app_dir.join("web");
    if web_dir.exists() {
        for entry in fs::read_dir(&web_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".vue") {
                return Ok(Framework::Vue);
            }
            if name.ends_with(".svelte") {
                return Ok(Framework::Svelte);
            }
        }
        // Check for .tsx files (React)
        for entry in fs::read_dir(&web_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".tsx") {
                return Ok(Framework::React);
            }
        }
    }

    Ok(Framework::Minimal)
}

/// Find the entry point for web bundling
fn find_entry_point(web_dir: &Path) -> Option<PathBuf> {
    // Check in order of preference
    let candidates = [
        "main.tsx",
        "main.ts",
        "main.js",
        "index.tsx",
        "index.ts",
        "index.js",
    ];
    for candidate in &candidates {
        let path = web_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Bundle web assets using esbuild via Deno
fn bundle_with_esbuild(app_dir: &Path, dist_dir: &Path, framework: &Framework) -> Result<()> {
    let web_dir = app_dir.join("web");
    let entry = match find_entry_point(&web_dir) {
        Some(e) => e,
        None => {
            println!("  No entry point found (main.tsx/ts/js), skipping bundle");
            return Ok(());
        }
    };

    let out_file = dist_dir.join("web/bundle.js");
    let deno_json = app_dir.join("deno.json");

    println!(
        "  Bundling {} with esbuild...",
        entry.file_name().unwrap().to_string_lossy()
    );

    // Build esbuild args
    let mut esbuild_args = vec![
        entry.display().to_string(),
        "--bundle".to_string(),
        "--format=esm".to_string(),
        "--target=es2020".to_string(),
        "--outfile".to_string() + &out_file.display().to_string(),
    ];

    // Add React JSX transform if needed
    if *framework == Framework::React {
        esbuild_args.push("--jsx=automatic".to_string());
    }

    // Add minification for production
    esbuild_args.push("--minify".to_string());
    esbuild_args.push("--sourcemap".to_string());

    // Create a temporary script to run esbuild
    let esbuild_script = format!(
        r#"
import * as esbuild from "npm:esbuild@0.20";

const result = await esbuild.build({{
  entryPoints: [{:?}],
  bundle: true,
  format: "esm",
  target: "es2020",
  minify: true,
  sourcemap: true,
  outfile: {:?},
  {jsx}
  external: [],
  define: {{
    "process.env.NODE_ENV": '"production"'
  }},
}});

await esbuild.stop();
console.log("Bundle complete:", result);
"#,
        entry.display().to_string(),
        out_file.display().to_string(),
        jsx = if *framework == Framework::React {
            r#"jsx: "automatic",
  jsxImportSource: "react","#
        } else {
            ""
        }
    );

    let script_path = dist_dir.join("_esbuild_bundle.ts");
    fs::write(&script_path, esbuild_script)?;

    // Run via deno
    let mut cmd = Command::new("deno");
    cmd.args(["run", "-A"]);

    // Add config if it exists
    if deno_json.exists() {
        cmd.args(["--config", &deno_json.display().to_string()]);
    }

    cmd.arg(script_path.display().to_string());

    let status = cmd.status();

    // Clean up script
    let _ = fs::remove_file(&script_path);

    match status {
        Ok(s) if s.success() => {
            println!("  Bundled to {}", out_file.display());

            // Update index.html to use bundle.js
            update_html_for_bundle(dist_dir)?;
            Ok(())
        }
        Ok(_) => {
            println!("  esbuild failed, copying files as-is");
            Ok(())
        }
        Err(e) => {
            println!("  esbuild not available ({}), copying files as-is", e);
            Ok(())
        }
    }
}

/// Transform Vue SFC files to JavaScript
fn transform_vue_files(app_dir: &Path, dist_dir: &Path) -> Result<()> {
    let web_dir = app_dir.join("web");
    let dist_web = dist_dir.join("web");

    // Check if there are any .vue files
    let mut has_vue = false;
    for entry in fs::read_dir(&web_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".vue") {
            has_vue = true;
            break;
        }
    }

    if !has_vue {
        return Ok(());
    }

    println!("  Transforming Vue SFC files...");

    // Create Vue transform script using @vue/compiler-sfc
    let transform_script = format!(
        r#"
import {{ parse, compileScript, compileTemplate, compileStyle }} from "npm:@vue/compiler-sfc@3";
import {{ walk }} from "https://deno.land/std@0.208.0/fs/walk.ts";

const webDir = {:?};
const distDir = {:?};

for await (const entry of walk(webDir, {{ exts: [".vue"] }})) {{
  const content = await Deno.readTextFile(entry.path);
  const {{ descriptor, errors }} = parse(content, {{ filename: entry.name }});

  if (errors.length > 0) {{
    console.error("Vue parse errors:", errors);
    continue;
  }}

  // Compile script
  let scriptCode = "";
  if (descriptor.script || descriptor.scriptSetup) {{
    const compiled = compileScript(descriptor, {{
      id: entry.name,
      inlineTemplate: true,
    }});
    scriptCode = compiled.content;
  }}

  // Compile template if not inlined
  let templateCode = "";
  if (descriptor.template && !descriptor.scriptSetup) {{
    const compiled = compileTemplate({{
      source: descriptor.template.content,
      filename: entry.name,
      id: entry.name,
    }});
    templateCode = compiled.code;
  }}

  // Generate output
  const outPath = entry.path.replace(webDir, distDir).replace(".vue", ".js");
  const output = `${{scriptCode}}\n${{templateCode}}`;

  await Deno.writeTextFile(outPath, output);
  console.log(`  Transformed: ${{entry.name}} -> ${{entry.name.replace(".vue", ".js")}}`);
}}
"#,
        web_dir.display().to_string(),
        dist_web.display().to_string()
    );

    let script_path = dist_dir.join("_vue_transform.ts");
    fs::write(&script_path, transform_script)?;

    let status = Command::new("deno")
        .args(["run", "-A", &script_path.display().to_string()])
        .status();

    // Clean up
    let _ = fs::remove_file(&script_path);

    match status {
        Ok(s) if s.success() => {
            println!("  Vue transform complete");
            Ok(())
        }
        _ => {
            println!("  Vue transform failed or not available, files copied as-is");
            Ok(())
        }
    }
}

/// Transform Svelte files to JavaScript
fn transform_svelte_files(app_dir: &Path, dist_dir: &Path) -> Result<()> {
    let web_dir = app_dir.join("web");
    let dist_web = dist_dir.join("web");

    // Check if there are any .svelte files
    let mut has_svelte = false;
    for entry in fs::read_dir(&web_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".svelte") {
            has_svelte = true;
            break;
        }
    }

    if !has_svelte {
        return Ok(());
    }

    println!("  Transforming Svelte files...");

    // Create Svelte transform script using svelte/compiler
    let transform_script = format!(
        r#"
import {{ compile }} from "npm:svelte@4/compiler";
import {{ walk }} from "https://deno.land/std@0.208.0/fs/walk.ts";

const webDir = {:?};
const distDir = {:?};

for await (const entry of walk(webDir, {{ exts: [".svelte"] }})) {{
  const content = await Deno.readTextFile(entry.path);

  try {{
    const result = compile(content, {{
      filename: entry.name,
      generate: "dom",
      format: "esm",
      css: "injected",
    }});

    const outPath = entry.path.replace(webDir, distDir).replace(".svelte", ".js");
    await Deno.writeTextFile(outPath, result.js.code);

    if (result.warnings.length > 0) {{
      console.log(`  Warnings for ${{entry.name}}:`, result.warnings.map(w => w.message));
    }}

    console.log(`  Transformed: ${{entry.name}} -> ${{entry.name.replace(".svelte", ".js")}}`);
  }} catch (e) {{
    console.error(`  Error transforming ${{entry.name}}:`, e.message);
  }}
}}
"#,
        web_dir.display().to_string(),
        dist_web.display().to_string()
    );

    let script_path = dist_dir.join("_svelte_transform.ts");
    fs::write(&script_path, transform_script)?;

    let status = Command::new("deno")
        .args(["run", "-A", &script_path.display().to_string()])
        .status();

    // Clean up
    let _ = fs::remove_file(&script_path);

    match status {
        Ok(s) if s.success() => {
            println!("  Svelte transform complete");
            Ok(())
        }
        _ => {
            println!("  Svelte transform failed or not available, files copied as-is");
            Ok(())
        }
    }
}

/// Update index.html to reference bundle.js instead of the original entry
fn update_html_for_bundle(dist_dir: &Path) -> Result<()> {
    let html_path = dist_dir.join("web/index.html");
    if html_path.exists() {
        let content = fs::read_to_string(&html_path)?;

        // Replace various entry point references with bundle.js
        let updated = content
            .replace("src=\"main.tsx\"", "src=\"bundle.js\"")
            .replace("src=\"main.ts\"", "src=\"bundle.js\"")
            .replace("src=\"main.js\"", "src=\"bundle.js\"")
            .replace("src=\"index.tsx\"", "src=\"bundle.js\"")
            .replace("src=\"index.ts\"", "src=\"bundle.js\"")
            .replace("src=\"./main.tsx\"", "src=\"bundle.js\"")
            .replace("src=\"./main.ts\"", "src=\"bundle.js\"")
            .replace("src=\"./main.js\"", "src=\"bundle.js\"");

        if updated != content {
            fs::write(&html_path, updated)?;
            println!("  Updated index.html to use bundle.js");
        }
    }
    Ok(())
}

fn cmd_build(app_dir: &Path) -> Result<()> {
    use bundler::{AppManifest, IconProcessor, RECOMMENDED_ICON_SIZE};

    println!("Building app at {}", app_dir.display());

    // Validate app structure
    let manifest_path = app_dir.join("manifest.app.toml");
    if !manifest_path.exists() {
        return Err(anyhow!(
            "Missing manifest.app.toml at {}",
            manifest_path.display()
        ));
    }

    let web_dir = app_dir.join("web");
    if !web_dir.exists() {
        return Err(anyhow!("Missing web/ directory at {}", web_dir.display()));
    }

    let src_dir = app_dir.join("src");
    if !src_dir.exists() {
        return Err(anyhow!("Missing src/ directory at {}", src_dir.display()));
    }

    // Load manifest for icon path
    let manifest = AppManifest::from_app_dir(app_dir)?;
    let icon_base = manifest.bundle.icon.clone();

    // Validate icon (warn if missing or invalid, but don't fail build)
    println!("  Checking app icon...");
    let search_paths = IconProcessor::get_search_paths(app_dir, icon_base.as_deref());
    let found_icon = search_paths.iter().find(|p| p.exists() && p.is_file());

    match found_icon {
        Some(path) => match IconProcessor::from_path(path) {
            Ok(processor) => {
                let validation = processor.validate(path);
                if !validation.errors.is_empty() {
                    eprintln!("\n  ⚠ Icon validation errors:");
                    for error in &validation.errors {
                        eprintln!("    ✗ {}", error);
                    }
                    eprintln!(
                        "    Run 'forge icon validate {}' for details.",
                        app_dir.display()
                    );
                    eprintln!("    Icon: {}\n", path.display());
                } else if !validation.warnings.is_empty() {
                    for warning in &validation.warnings {
                        eprintln!("  ⚠ Icon: {}", warning);
                    }
                } else {
                    println!(
                        "  Icon: {} ({}x{}) ✓",
                        path.display(),
                        validation.width,
                        validation.height
                    );
                }
            }
            Err(e) => {
                eprintln!("\n  ⚠ Failed to load icon: {}", e);
                eprintln!("    Path: {}\n", path.display());
            }
        },
        None => {
            eprintln!("\n  ⚠ No app icon found!");
            eprintln!("    Bundling will fail without an icon.");
            eprintln!(
                "    Run 'forge icon create {}/assets/icon.png' to create a placeholder.",
                app_dir.display()
            );
            eprintln!(
                "    Or add your own {}x{} PNG to {}/assets/icon.png\n",
                RECOMMENDED_ICON_SIZE,
                RECOMMENDED_ICON_SIZE,
                app_dir.display()
            );
        }
    }

    // Detect framework
    let framework = detect_framework(app_dir)?;
    println!("  Detected framework: {:?}", framework);

    // Create dist directory
    let dist_dir = app_dir.join("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)?;
    }
    fs::create_dir_all(&dist_dir)?;
    fs::create_dir_all(dist_dir.join("web"))?;

    // Copy web assets to dist
    println!("  Copying web assets...");
    copy_dir_recursive(&web_dir, &dist_dir.join("web"))?;

    // Copy manifest
    fs::copy(&manifest_path, dist_dir.join("manifest.app.toml"))?;

    // Copy src (Deno runtime code)
    fs::create_dir_all(dist_dir.join("src"))?;
    copy_dir_recursive(&src_dir, &dist_dir.join("src"))?;

    // Bundle based on framework
    match framework {
        Framework::React | Framework::Minimal => {
            bundle_with_esbuild(app_dir, &dist_dir, &framework)?;
        }
        Framework::Vue => {
            // Transform Vue SFCs first, then bundle
            transform_vue_files(app_dir, &dist_dir)?;
            bundle_with_esbuild(app_dir, &dist_dir, &framework)?;
        }
        Framework::Svelte => {
            // Transform Svelte files first, then bundle
            transform_svelte_files(app_dir, &dist_dir)?;
            bundle_with_esbuild(app_dir, &dist_dir, &framework)?;
        }
    }

    println!("\nBuild complete! Output in {}", dist_dir.display());
    println!("\nNext steps:");
    println!(
        "  forge bundle {}  # Create distributable package",
        app_dir.display()
    );

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
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
fn cmd_bundle(app_dir: &Path) -> Result<()> {
    println!("Bundling app at {}", app_dir.display());

    // 1. Verify dist/ exists, or run build first
    let dist_dir = app_dir.join("dist");
    if !dist_dir.exists() {
        println!("  dist/ not found, running build first...");
        cmd_build(app_dir)?;
    }

    // Verify build succeeded
    if !dist_dir.join("web").exists() {
        return Err(anyhow!(
            "Build failed - dist/web/ not found. Run 'forge build {}' first.",
            app_dir.display()
        ));
    }

    // 2. Parse manifest with bundle config
    let manifest = bundler::parse_manifest(app_dir)?;
    println!("  App: {} v{}", manifest.app.name, manifest.app.version);

    // 3. Create output directory
    let output_dir = app_dir.join("bundle");
    fs::create_dir_all(&output_dir)?;

    // 4. Platform-specific bundling
    let result = bundler::bundle(app_dir, &dist_dir, &output_dir, &manifest)?;

    println!("\nBundle complete!");
    println!("  Output: {}", result.display());

    Ok(())
}

fn cmd_sign(artifact_path: &Path, identity: Option<&str>) -> Result<()> {
    use bundler::codesign::{detect_signing_capabilities, sign, SigningConfig};

    println!("Signing artifact: {}", artifact_path.display());

    if !artifact_path.exists() {
        return Err(anyhow!("Artifact not found: {}", artifact_path.display()));
    }

    // Detect signing capabilities
    let caps = detect_signing_capabilities();
    println!("{}", caps);

    // Determine identity
    let identity = identity.map(String::from).unwrap_or_else(|| {
        // Try to get from environment
        env::var("FORGE_SIGNING_IDENTITY")
            .or_else(|_| env::var("CODESIGN_IDENTITY"))
            .unwrap_or_else(|_| {
                // Default macOS identity pattern
                #[cfg(target_os = "macos")]
                return "-".to_string(); // Ad-hoc signing

                #[cfg(not(target_os = "macos"))]
                return String::new();
            })
    });

    if identity.is_empty() {
        return Err(anyhow!(
            "No signing identity provided.\n\
            Usage: forge sign --identity <IDENTITY> <artifact>\n\n\
            For macOS: Developer ID Application: Your Name (TEAMID)\n\
            For Windows: Path to .pfx certificate file"
        ));
    }

    // Build signing config
    let mut config = SigningConfig::new(identity);

    // Check for password in environment (Windows)
    if let Ok(pwd) = env::var("FORGE_SIGNING_PASSWORD") {
        config = config.with_password(Some(pwd));
    }

    // Check for team ID (macOS notarization)
    if let Ok(team_id) = env::var("FORGE_TEAM_ID") {
        config = config.with_team_id(Some(team_id));
    }

    // Check for notarization flag
    if env::var("FORGE_NOTARIZE").is_ok() {
        config = config.with_notarize(true);
    }

    // Sign the artifact
    sign(artifact_path, &config)?;

    println!("\nSigning complete!");
    Ok(())
}

fn icon_usage() {
    eprintln!("forge icon <create|validate> [options]");
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!("  create <path>       Create a placeholder icon at the specified path");
    eprintln!("  validate <app-dir>  Validate icon for the specified app directory");
    eprintln!();
    eprintln!("Icon Requirements:");
    eprintln!("  • Format: PNG with transparency (RGBA)");
    eprintln!("  • Size: 1024x1024 pixels (minimum 512x512)");
    eprintln!("  • Shape: Square (1:1 aspect ratio)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  forge icon create my-app/assets/icon.png");
    eprintln!("  forge icon validate my-app");
}

fn cmd_icon_create(output_path: &Path) -> Result<()> {
    use bundler::{IconProcessor, RECOMMENDED_ICON_SIZE};

    println!("Creating placeholder icon at {}", output_path.display());

    // Check if file already exists
    if output_path.exists() {
        return Err(anyhow!(
            "File already exists: {}\n\
            Use a different path or remove the existing file first.",
            output_path.display()
        ));
    }

    // Create the placeholder icon
    let processor = IconProcessor::create_placeholder(RECOMMENDED_ICON_SIZE);
    processor.save(output_path)?;

    println!("\nPlaceholder icon created!");
    println!("  Path: {}", output_path.display());
    println!(
        "  Size: {}x{} pixels",
        RECOMMENDED_ICON_SIZE, RECOMMENDED_ICON_SIZE
    );
    println!();
    println!("IMPORTANT: Replace this placeholder with your actual app icon before release.");
    println!();
    println!("Icon Requirements:");
    println!("  • Format: PNG with transparency (RGBA)");
    println!("  • Size: 1024x1024 pixels (minimum 512x512)");
    println!("  • Shape: Square (1:1 aspect ratio)");

    Ok(())
}

fn cmd_icon_validate(app_dir: &Path) -> Result<()> {
    use bundler::{AppManifest, IconProcessor, MIN_ICON_SIZE, RECOMMENDED_ICON_SIZE};

    println!("Validating icon for app at {}", app_dir.display());

    // Check if app directory exists
    if !app_dir.exists() {
        return Err(anyhow!("App directory not found: {}", app_dir.display()));
    }

    // Try to load manifest to get icon path
    let manifest_path = app_dir.join("manifest.app.toml");
    let icon_base = if manifest_path.exists() {
        let manifest = AppManifest::from_app_dir(app_dir)?;
        manifest.bundle.icon.clone()
    } else {
        None
    };

    // Get search paths and try to find icon
    let search_paths = IconProcessor::get_search_paths(app_dir, icon_base.as_deref());

    println!("\nSearching for icon...");
    for path in &search_paths {
        let status = if path.exists() {
            "✓ found"
        } else {
            "✗ not found"
        };
        println!("  {} {}", status, path.display());
    }

    // Try to load and validate the icon
    let found_path = search_paths.iter().find(|p| p.exists() && p.is_file());

    match found_path {
        Some(path) => {
            println!("\nValidating: {}", path.display());

            let processor = IconProcessor::from_path(path)?;
            let validation = processor.validate(path);

            // Display results
            println!("\nIcon Properties:");
            println!("  Size: {}x{} pixels", validation.width, validation.height);
            println!(
                "  Square: {}",
                if validation.is_square {
                    "Yes ✓"
                } else {
                    "No ✗"
                }
            );
            println!(
                "  Meets minimum ({}x{}): {}",
                MIN_ICON_SIZE,
                MIN_ICON_SIZE,
                if validation.meets_minimum {
                    "Yes ✓"
                } else {
                    "No ✗"
                }
            );
            println!(
                "  Meets recommended ({}x{}): {}",
                RECOMMENDED_ICON_SIZE,
                RECOMMENDED_ICON_SIZE,
                if validation.meets_recommended {
                    "Yes ✓"
                } else {
                    "No ✗"
                }
            );
            println!(
                "  Has transparency: {}",
                if validation.has_transparency {
                    "Yes ✓"
                } else {
                    "No (warning)"
                }
            );

            // Display warnings
            if !validation.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &validation.warnings {
                    println!("  ⚠ {}", warning);
                }
            }

            // Display errors
            if !validation.errors.is_empty() {
                println!("\nErrors:");
                for error in &validation.errors {
                    println!("  ✗ {}", error);
                }
                return Err(anyhow!(
                    "Icon validation failed with {} error(s). Please fix the issues above.",
                    validation.errors.len()
                ));
            }

            println!("\n✓ Icon validation passed!");
            Ok(())
        }
        None => Err(anyhow!(
            "No icon found!\n\n\
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
                Run: forge icon create {}/assets/icon.png",
            app_dir.display(),
            app_dir.display()
        )),
    }
}

fn cmd_icon(args: &[String]) -> Result<()> {
    if args.is_empty() {
        icon_usage();
        return Ok(());
    }

    let subcommand = &args[0];
    let rest = &args[1..];

    match subcommand.as_str() {
        "create" => {
            if rest.is_empty() {
                return Err(anyhow!("Usage: forge icon create <path>\n\nExample: forge icon create my-app/assets/icon.png"));
            }
            let output_path = PathBuf::from(&rest[0]);
            cmd_icon_create(&output_path)
        }
        "validate" => {
            let app_dir = if rest.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(&rest[0])
            };
            cmd_icon_validate(&app_dir)
        }
        _ => {
            icon_usage();
            Err(anyhow!("Unknown icon subcommand: {}", subcommand))
        }
    }
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        usage();
        return Ok(());
    }

    let cmd = args.remove(0);

    match cmd.as_str() {
        "init" => {
            // Parse --template flag
            let mut template = "minimal".to_string();
            let mut app_dir = None;

            let mut i = 0;
            while i < args.len() {
                if args[i] == "--template" || args[i] == "-t" {
                    if i + 1 < args.len() {
                        template = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err(anyhow!("--template requires a value"));
                    }
                } else if !args[i].starts_with('-') {
                    app_dir = Some(PathBuf::from(&args[i]));
                    i += 1;
                } else {
                    return Err(anyhow!("Unknown flag: {}", args[i]));
                }
            }

            let app_dir = app_dir
                .ok_or_else(|| anyhow!("Usage: forge init [--template <name>] <app-dir>"))?;
            cmd_init(&app_dir, &template)?;
        }
        "dev" => {
            let app_dir = args
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("apps/example-deno-app"));
            cmd_dev(&app_dir)?;
        }
        "build" => {
            let app_dir = args
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("apps/example-deno-app"));
            cmd_build(&app_dir)?;
        }
        "bundle" => {
            let app_dir = args
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("apps/example-deno-app"));
            cmd_bundle(&app_dir)?;
        }
        "sign" => {
            // Parse --identity flag
            let mut identity: Option<String> = None;
            let mut artifact_path = None;

            let mut i = 0;
            while i < args.len() {
                if args[i] == "--identity" || args[i] == "-i" {
                    if i + 1 < args.len() {
                        identity = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(anyhow!("--identity requires a value"));
                    }
                } else if !args[i].starts_with('-') {
                    artifact_path = Some(PathBuf::from(&args[i]));
                    i += 1;
                } else {
                    return Err(anyhow!("Unknown flag: {}", args[i]));
                }
            }

            let artifact_path = artifact_path
                .ok_or_else(|| anyhow!("Usage: forge sign [--identity <IDENTITY>] <artifact>"))?;
            cmd_sign(&artifact_path, identity.as_deref())?;
        }
        "icon" => {
            cmd_icon(&args)?;
        }
        _ => usage(),
    }
    Ok(())
}
