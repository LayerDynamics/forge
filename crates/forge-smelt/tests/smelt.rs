//! Integration tests for the smelt pipeline.
//!
//! These build a real on-disk fixture app, run [`forge_smelt::smelt`], and assert
//! the compiled output is self-consistent — in particular that relative `.ts`
//! import specifiers were rewritten to `.js` (the failure mode the crate exists
//! to prevent) while `runtime:*` specifiers were left untouched.

use std::fs;
use std::path::Path;

use forge_smelt::{smelt, SmeltError};

/// Create `app/src/<rel>` with `contents`, making parent dirs.
fn write_src(app: &Path, rel: &str, contents: &str) {
    let path = app.join("src").join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn smelts_module_graph_and_rewrites_relative_specifiers() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path();

    // Entry imports a sibling `.ts` (explicit ext), an extensionless module in a
    // subdir, and an external runtime module.
    write_src(
        app,
        "main.ts",
        r#"
import { greeting } from "./util.ts";
import { double } from "./lib/math";
import { readTextFile } from "runtime:fs";

export function run(): string {
    return greeting(String(double(21)));
}
void readTextFile;
"#,
    );
    write_src(
        app,
        "util.ts",
        r#"export function greeting(n: string): string { return `value ${n}`; }"#,
    );
    write_src(
        app,
        "lib/math.ts",
        r#"export function double(n: number): number { return n * 2; }"#,
    );

    let out = tmp.path().join("dist");
    let result = smelt(app, &out).expect("smelt should succeed");

    // Entry compiled to main.js; both dependencies compiled into the tree.
    assert!(result.entry.ends_with("main.js"), "entry must be main.js");
    assert!(result.entry.is_file(), "compiled entry must exist on disk");
    assert!(out.join("util.js").is_file(), "util.ts -> util.js");
    assert!(
        out.join("lib/math.js").is_file(),
        "lib/math.ts -> lib/math.js"
    );
    assert_eq!(result.modules.len(), 3, "main + util + math");

    let main_js = fs::read_to_string(&result.entry).unwrap();

    // The crux: relative TS specifiers rewritten to .js, external left as-is.
    assert!(
        main_js.contains("./util.js"),
        "explicit .ts specifier must be rewritten to .js; got:\n{main_js}"
    );
    assert!(
        !main_js.contains("./util.ts"),
        "no dangling .ts specifier may remain; got:\n{main_js}"
    );
    assert!(
        main_js.contains("./lib/math.js"),
        "extensionless specifier must resolve to .js; got:\n{main_js}"
    );
    assert!(
        main_js.contains("runtime:fs"),
        "external runtime:* specifier must be left untouched; got:\n{main_js}"
    );

    // No compiled output anywhere may reference a relative .ts module.
    for m in &result.modules {
        let js = fs::read_to_string(m).unwrap();
        assert!(
            !js.contains(".ts\""),
            "compiled {} still references a .ts module:\n{js}",
            m.display()
        );
    }
}

#[test]
fn embed_writes_bootstrap_that_imports_compiled_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path();
    write_src(app, "main.ts", r#"export const ready = true;"#);

    let out = tmp.path().join("embed");
    let manifest = forge_smelt::build::embed(app, &out).expect("embed should succeed");

    // The stable bootstrap (init.js) is written and imports the compiled entry.
    assert!(manifest.bootstrap.ends_with("init.js"));
    assert!(manifest.bootstrap.is_file(), "init.js must exist");
    assert!(manifest.app_entry.ends_with("main.js"));
    assert!(manifest.app_entry.is_file(), "compiled main.js must exist");

    let bootstrap = fs::read_to_string(&manifest.bootstrap).unwrap();
    assert!(
        bootstrap.contains("./main.js"),
        "bootstrap must import the compiled entry; got:\n{bootstrap}"
    );
    // Bootstrap (init.js) is first in the file list.
    assert_eq!(manifest.files.first(), Some(&manifest.bootstrap));
}

#[test]
fn missing_entry_is_a_clear_error() {
    let tmp = tempfile::tempdir().unwrap();
    // No src/main.ts created.
    let err = smelt(tmp.path(), tmp.path().join("dist")).unwrap_err();
    assert!(
        matches!(err, SmeltError::EntryNotFound { .. }),
        "expected EntryNotFound, got: {err:?}"
    );
}

#[test]
fn unresolved_relative_import_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path();
    write_src(
        app,
        "main.ts",
        r#"import { x } from "./does-not-exist.ts"; void x;"#,
    );

    let err = smelt(app, tmp.path().join("dist")).unwrap_err();
    assert!(
        matches!(err, SmeltError::UnresolvedImport { .. }),
        "expected UnresolvedImport, got: {err:?}"
    );
}
