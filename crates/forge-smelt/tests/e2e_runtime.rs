//! End-to-end utilization proof.
//!
//! This is the deliverable test for the `forge build -> smelt -> runtime`
//! chain: it smelts a real multi-module fixture, then loads and **executes** the
//! compiled entry in a `deno_core` JS runtime (the same engine `forge-runtime`
//! embeds) using `FsModuleLoader` (the same relative-resolution the
//! `ForgeModuleLoader` relies on).
//!
//! Correctness is asserted *inside* the compiled JavaScript: the entry throws if
//! a value imported from a sibling module is wrong. Module evaluation therefore
//! only succeeds when (a) the rewritten relative `.js` specifiers resolved on
//! disk and (b) the compiled code ran and produced the right values. A passing
//! run proves the chain end-to-end — not merely that "transpile produced JS".

use std::fs;
use std::path::Path;
use std::rc::Rc;

use deno_core::{FsModuleLoader, JsRuntime, ModuleSpecifier, PollEventLoopOptions, RuntimeOptions};

use forge_smelt::smelt;

fn write_src(app: &Path, rel: &str, contents: &str) {
    let path = app.join("src").join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn smelted_app_loads_and_executes_through_the_runtime_module_path() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path();

    // A two-dependency app: the entry imports a sibling via an extensionless
    // specifier and a subdir module via an explicit `.ts` specifier, then
    // asserts the imported values in JS — throwing on any mismatch.
    write_src(
        app,
        "main.ts",
        r#"
import { double } from "./util";
import { LABEL } from "./meta/label.ts";

const value: number = double(21);
if (value !== 42) {
    throw new Error(`smelt e2e: expected 42 from ./util, got ${value}`);
}
if (LABEL !== "smelted") {
    throw new Error(`smelt e2e: expected "smelted" from ./meta/label, got ${LABEL}`);
}
"#,
    );
    write_src(
        app,
        "util.ts",
        r#"export function double(n: number): number { return n * 2; }"#,
    );
    write_src(
        app,
        "meta/label.ts",
        r#"export const LABEL: string = "smelted";"#,
    );

    let out = tmp.path().join("dist");
    let result = smelt(app, &out).expect("smelt should succeed");
    assert!(result.entry.ends_with("main.js"));

    // Load + execute the compiled entry in a real JS runtime with on-disk
    // relative module resolution. If a rewritten `./util.js` / `./meta/label.js`
    // specifier failed to resolve, `load_main_es_module` errors; if the compiled
    // code computed the wrong value, the in-JS assertions throw and
    // `mod_evaluate` resolves to an error — either way the test fails.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, async move {
        let mut js = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            ..Default::default()
        });

        let spec = ModuleSpecifier::from_file_path(&result.entry).unwrap();
        let module_id = js
            .load_main_es_module(&spec)
            .await
            .expect("compiled entry must load (rewritten relative .js imports must resolve)");

        let eval = js.mod_evaluate(module_id);
        js.run_event_loop(PollEventLoopOptions::default())
            .await
            .expect("event loop should complete without error");
        eval.await
            .expect("module evaluation must succeed (in-JS assertions must hold)");
    });
}
