use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("templates.rs");

    // Get workspace root (two levels up from crates/forge)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let templates_dir = workspace_root.join("templates");

    println!("cargo:rerun-if-changed={}", templates_dir.display());

    let mut code = String::new();
    code.push_str("// Auto-generated template embedding code\n\n");

    // Define template names
    let templates = ["minimal", "react", "vue", "svelte"];

    for template in &templates {
        let template_path = templates_dir.join(template);
        if !template_path.exists() {
            println!(
                "cargo:warning=Template directory not found: {}",
                template_path.display()
            );
            continue;
        }

        // Generate module for each template
        code.push_str(&format!("pub mod {} {{\n", template));

        // Read and embed each file
        let files = [
            ("MANIFEST", "manifest.app.toml"),
            ("DENO_JSON", "deno.json"),
            ("SRC_MAIN", "src/main.ts"),
        ];

        for (const_name, file_path) in &files {
            let full_path = template_path.join(file_path);
            if full_path.exists() {
                let content = fs::read_to_string(&full_path).unwrap();
                code.push_str(&format!(
                    "    pub const {}: &str = r#\"{}\"#;\n",
                    const_name,
                    content.replace("\"#", "\\\"#")
                ));
                println!("cargo:rerun-if-changed={}", full_path.display());
            }
        }

        // Embed web files dynamically
        let web_dir = template_path.join("web");
        if web_dir.exists() {
            code.push_str("    pub mod web {\n");
            embed_dir(&web_dir, &mut code, "        ");
            code.push_str("    }\n");
        }

        code.push_str("}\n\n");
    }

    fs::write(&dest_path, code).unwrap();
}

fn embed_dir(dir: &Path, code: &mut String, indent: &str) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() {
            // Convert filename to valid Rust const name
            let const_name = name.replace(['.', '-'], "_").to_uppercase();

            let content = fs::read_to_string(&path).unwrap();
            code.push_str(&format!(
                "{}pub const {}: &str = r#\"{}\"#;\n",
                indent,
                const_name,
                content.replace("\"#", "\\\"#")
            ));
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
