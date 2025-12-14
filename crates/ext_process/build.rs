use deno_ast::{EmitOptions, MediaType, ParseParams, TranspileModuleOptions, TranspileOptions};
use std::fs;
use std::path::Path;

/// Transpile TypeScript to JavaScript using deno_ast
fn transpile_ts(ts_code: &str, specifier: &str) -> String {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: deno_ast::ModuleSpecifier::parse(specifier).unwrap(),
        text: ts_code.into(),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .expect("Failed to parse TypeScript");

    let transpile_result = parsed
        .transpile(
            &TranspileOptions::default(),
            &TranspileModuleOptions::default(),
            &EmitOptions::default(),
        )
        .expect("Failed to transpile TypeScript");

    transpile_result.into_source().text
}

fn main() {
    // Transpile js/init.ts to js/init.js
    println!("cargo:rerun-if-changed=js/init.ts");

    let ts_path = Path::new("js/init.ts");
    let js_path = Path::new("js/init.js");

    if ts_path.exists() {
        let ts_code = fs::read_to_string(ts_path).expect("Failed to read js/init.ts");
        let js_code = transpile_ts(&ts_code, "file:///init.ts");
        fs::write(js_path, js_code).expect("Failed to write js/init.js");
    }
}
