//! Fixture-based unit tests for each drift rule. Each test builds a synthetic
//! workspace on disk and asserts the rule reacts correctly — these are always
//! green and guard the rule logic against regressions (independent of the real
//! repository's current drift, which is covered by `docs_sync.rs`).

use forge_docs_check::checks;
use forge_docs_check::discovery::Workspace;
use std::fs;
use std::path::PathBuf;

/// Minimal synthetic workspace builder.
struct Fixture {
    root: PathBuf,
    members: Vec<String>,
    _tmp: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        fs::create_dir_all(root.join("sdk")).unwrap();
        fs::create_dir_all(root.join("site/src/content/docs/crates")).unwrap();
        fs::create_dir_all(root.join("site/src/content/docs/api")).unwrap();
        Fixture {
            root,
            members: Vec::new(),
            _tmp: tmp,
        }
    }

    /// Add a crate directory + Cargo.toml and register it as a workspace member.
    fn add_crate(&mut self, dir_name: &str) -> &mut Self {
        let dir = self.root.join("crates").join(dir_name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{dir_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        self.members.push(format!("crates/{dir_name}"));
        self
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    /// Materialize the workspace Cargo.toml and discover it.
    fn discover(&self) -> Workspace {
        let members = self
            .members
            .iter()
            .map(|m| format!("  \"{m}\","))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            self.root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [\n{members}\n]\nresolver = \"2\"\n"),
        )
        .unwrap();
        Workspace::discover_at(&self.root).unwrap()
    }
}

fn messages(findings: &[forge_docs_check::Finding]) -> Vec<String> {
    findings.iter().map(|f| f.message.clone()).collect()
}

fn any_contains(findings: &[forge_docs_check::Finding], needle: &str) -> bool {
    messages(findings).iter().any(|m| m.contains(needle))
}

#[test]
fn crate_page_missing_is_flagged_and_present_is_not() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_console"); // no page -> flagged
    fx.add_crate("ext_fs"); // has page -> ok
    fx.write("site/src/content/docs/crates/ext-fs.md", "# fs\n");
    let ws = fx.discover();

    let findings = checks::crate_pages::check(&ws);
    assert!(
        any_contains(&findings, "ext_console"),
        "missing ext-console.md should be flagged: {:?}",
        messages(&findings)
    );
    assert!(
        !any_contains(&findings, "ext_fs"),
        "ext_fs has a page and must not be flagged"
    );
}

#[test]
fn forge_cli_maps_to_forge_md() {
    let mut fx = Fixture::new();
    fx.add_crate("forge_cli");
    fx.write("site/src/content/docs/crates/forge.md", "# forge\n");
    let ws = fx.discover();
    assert!(
        checks::crate_pages::check(&ws).is_empty(),
        "forge_cli documented as forge.md must not be flagged"
    );
}

#[test]
fn api_drift_forward_flags_undocumented_export() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_dock");
    fx.write(
        "sdk/runtime.dock.ts",
        "export function bounce() {}\nexport function onMenuItemClick() {}\n",
    );
    // Doc only mentions bounce, not onMenuItemClick.
    fx.write(
        "site/src/content/docs/api/runtime-dock.md",
        "---\ntitle: dock\n---\n### bounce()\nDoes a bounce.\n",
    );
    let ws = fx.discover();
    let findings = checks::api_drift::check(&ws);
    assert!(
        any_contains(&findings, "onMenuItemClick"),
        "undocumented export must be flagged: {:?}",
        messages(&findings)
    );
    assert!(
        !any_contains(&findings, "`bounce`"),
        "documented export must not be flagged"
    );
}

#[test]
fn api_drift_reverse_flags_renamed_method() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_shell");
    // SDK exports chdir; doc still documents the old setCwd name.
    fx.write("sdk/runtime.shell.ts", "export function chdir() {}\n");
    fx.write(
        "site/src/content/docs/api/runtime-shell.md",
        "---\ntitle: shell\n---\n### chdir(path)\nok\n### setCwd(path)\nold name\n",
    );
    let ws = fx.discover();
    let findings = checks::api_drift::check(&ws);
    assert!(
        any_contains(&findings, "setCwd"),
        "documented-but-removed method must be flagged: {:?}",
        messages(&findings)
    );
}

#[test]
fn api_drift_accepts_reexport_lists_and_aliases() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_timers");
    // setTimeout is declared then re-exported; exec is an alias of execute.
    fx.write(
        "sdk/runtime.timers.ts",
        "function setTimeout() {}\nexport async function execute() {}\nexport { setTimeout };\nexport { execute as exec };\n",
    );
    // Doc documents setTimeout() and execute() (canonical), not the alias exec.
    fx.write(
        "site/src/content/docs/api/runtime-timers.md",
        "---\ntitle: timers\n---\n### setTimeout()\nok\n### execute()\nok\n",
    );
    let ws = fx.discover();
    let findings = checks::api_drift::check(&ws);
    assert!(
        findings.is_empty(),
        "re-exports and aliases must not produce drift: {:?}",
        messages(&findings)
    );
}

#[test]
fn forge_docs_list_detects_missing_extension() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_fs");
    fx.add_crate("ext_console");
    fx.add_crate("forge_cli");
    // EXTENSIONS lists fs but not console — note the `&[` in the TYPE annotation,
    // which the extractor must skip past (the bug this rule's parser had).
    fx.write(
        "crates/forge_cli/src/docs.rs",
        "const EXTENSIONS: &[(&str, &str)] = &[\n    (\"fs\", \"runtime:fs\"),\n];\n",
    );
    let ws = fx.discover();
    let findings = checks::forge_docs::check(&ws);
    assert!(
        any_contains(&findings, "console"),
        "missing console must be flagged: {:?}",
        messages(&findings)
    );
    assert!(
        !any_contains(&findings, "`fs`"),
        "listed fs must not be flagged"
    );
}

#[test]
fn counts_freeform_flags_stale_number() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_fs");
    fx.add_crate("ext_net");
    fx.add_crate("forge_cli");
    // 2 ext crates, but the doc claims 27.
    fx.write(
        "site/src/content/docs/architecture.md",
        "Forge has 27+ extension crates today.\n",
    );
    let ws = fx.discover();
    let findings = checks::counts::check(&ws);
    assert!(
        any_contains(&findings, "27+ extension crates"),
        "stale extension count must be flagged: {:?}",
        messages(&findings)
    );
}

#[test]
fn counts_marker_is_authoritative() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_fs");
    fx.add_crate("forge_cli");
    // Marker claims 5 ext crates; reality is 1.
    fx.write(
        "site/src/content/docs/internals.md",
        "Total: <!-- forge:count:ext_crates -->5<!-- /forge:count --> extensions.\n",
    );
    let ws = fx.discover();
    assert!(
        any_contains(&checks::counts::check(&ws), "marker `ext_crates` says 5"),
        "wrong marker value must be flagged"
    );

    // Correct it to 1 and the marker passes.
    fx.write(
        "site/src/content/docs/internals.md",
        "Total: <!-- forge:count:ext_crates -->1<!-- /forge:count --> extensions.\n",
    );
    let ws = fx.discover();
    assert!(
        !any_contains(&checks::counts::check(&ws), "marker `ext_crates`"),
        "correct marker value must pass"
    );
}

#[test]
fn cli_command_flags_missing_subcommand() {
    // The subcommand list is introspected from the real `forge_cli::cli()`
    // clap model (post-P5.2), so the fixture only controls the docs page.
    // `smelt` is a real subcommand; documenting `dev` but not `smelt` must
    // flag the gap.
    let fx = Fixture::new();
    fx.write("site/src/content/docs/crates/forge.md", "### `forge dev`\n");
    let ws = fx.discover();
    let findings = checks::cli_commands::check(&ws);
    assert!(
        any_contains(&findings, "forge smelt"),
        "missing smelt must be flagged: {:?}",
        messages(&findings)
    );
    assert!(
        !any_contains(&findings, "forge dev"),
        "documented dev must not be flagged"
    );
}

#[test]
fn cli_doc_flags_stale_region_and_passes_when_current() {
    use forge_docs_check::clidoc::{self, BLOCK_CLOSE, BLOCK_OPEN};

    // A `<!-- forge:cli -->` region whose body does not match the clap model
    // must be flagged stale.
    let fx = Fixture::new();
    fx.write(
        "site/src/content/docs/crates/forge.md",
        &format!("# forge\n\n{BLOCK_OPEN}\nstale, hand-edited content\n{BLOCK_CLOSE}\n"),
    );
    let ws = fx.discover();
    assert!(
        any_contains(&clidoc::check(&ws), "stale"),
        "a region that doesn't match the clap model must be flagged: {:?}",
        messages(&clidoc::check(&ws))
    );

    // The canonical generated body (open\n{body}\n close) must pass.
    let body = clidoc::render_block_body();
    fx.write(
        "site/src/content/docs/crates/forge.md",
        &format!("# forge\n\n{BLOCK_OPEN}\n{body}\n{BLOCK_CLOSE}\n"),
    );
    let ws = fx.discover();
    assert!(
        clidoc::check(&ws).is_empty(),
        "the freshly generated region must be in sync: {:?}",
        messages(&clidoc::check(&ws))
    );

    // A page that never opts in (no marker) is silently skipped.
    fx.write(
        "site/src/content/docs/crates/forge.md",
        "# forge\n\nno markers here\n",
    );
    let ws = fx.discover();
    assert!(
        clidoc::check(&ws).is_empty(),
        "a page without the marker must not be flagged"
    );
}

#[test]
fn slug_prefix_flags_root_slug_and_passes_under_docs() {
    // Docs are served under /docs/, so a page slug must start with `docs/`.
    let fx = Fixture::new();
    fx.write(
        "site/src/content/docs/crates/forge.md",
        "---\ntitle: forge\nslug: crates/forge\n---\n# forge\n",
    );
    let ws = fx.discover();
    assert!(
        any_contains(&checks::slug_prefix::check(&ws), "crates/forge"),
        "a root-level slug must be flagged: {:?}",
        messages(&checks::slug_prefix::check(&ws))
    );

    // Same page mounted under docs/ passes.
    fx.write(
        "site/src/content/docs/crates/forge.md",
        "---\ntitle: forge\nslug: docs/crates/forge\n---\n# forge\n",
    );
    let ws = fx.discover();
    assert!(
        checks::slug_prefix::check(&ws).is_empty(),
        "a docs/-prefixed slug must pass: {:?}",
        messages(&checks::slug_prefix::check(&ws))
    );
}

/// Sanity: discovery distinguishes extension crates and maps page stems.
#[test]
fn discovery_classifies_crates() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_image_tools");
    fx.add_crate("forge_cli");
    fx.add_crate("forge-smelt");
    let ws = fx.discover();
    assert_eq!(ws.extension_crates().len(), 1);
    let stems: Vec<String> = ws.crates.iter().map(|c| c.crate_page_stem()).collect();
    assert!(stems.contains(&"ext-image-tools".to_string()));
    assert!(stems.contains(&"forge".to_string())); // forge_cli special case
    assert!(stems.contains(&"forge-smelt".to_string()));
}

// --- Cross-platform robustness ---------------------------------------------
// These guard the class of bug that escaped to the Windows CI matrix: drift
// findings must be identical regardless of file line endings (CRLF on a Windows
// checkout vs LF) and finding messages must use forward-slash paths so they
// match the checked-in baseline on every OS.

use forge_docs_check::apiblock;

#[test]
fn api_block_check_is_line_ending_agnostic() {
    let sdk_src =
        "export function foo(a: number): void {}\nexport function bar(): string { return \"\"; }\n";

    let make_page = |eol: &str| {
        let body = apiblock::render_block_body("x", &apiblock::public_signatures(sdk_src));
        let lf = format!(
            "---\ntitle: x\n---\n\n## API Reference\n\n{}\n{}\n{}\n\n### foo\nprose\n",
            apiblock::BLOCK_OPEN,
            body,
            apiblock::BLOCK_CLOSE
        );
        lf.replace('\n', eol)
    };

    for eol in ["\n", "\r\n"] {
        let mut fx = Fixture::new();
        fx.add_crate("ext_x");
        fx.write("sdk/runtime.x.ts", sdk_src);
        fx.write("site/src/content/docs/api/runtime-x.md", &make_page(eol));
        let ws = fx.discover();
        let findings = apiblock::check(&ws);
        assert!(
            findings.is_empty(),
            "a current block must not be flagged (eol={eol:?}): {:?}",
            messages(&findings)
        );
    }
}

#[test]
fn api_block_check_flags_stale_regardless_of_eol() {
    let sdk_src = "export function foo(a: number): void {}\n";
    for eol in ["\n", "\r\n"] {
        let mut fx = Fixture::new();
        fx.add_crate("ext_x");
        fx.write("sdk/runtime.x.ts", sdk_src);
        let page = format!(
            "## API Reference\n\n{}\n```typescript\nWRONG(): void\n```\n{}\n",
            apiblock::BLOCK_OPEN,
            apiblock::BLOCK_CLOSE
        )
        .replace('\n', eol);
        fx.write("site/src/content/docs/api/runtime-x.md", &page);
        let ws = fx.discover();
        assert!(
            apiblock::check(&ws).iter().any(|f| f.rule == "api-block"),
            "a stale block must be flagged (eol={eol:?})"
        );
    }
}

#[test]
fn write_counts_updates_markers_to_derived_value() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_a");
    fx.add_crate("ext_b");
    fx.add_crate("forge_cli");
    // Marker has a wrong value (9); the fixture has 2 ext crates.
    fx.write(
        "site/src/content/docs/architecture.md",
        "x <!-- forge:count:ext_crates -->9<!-- /forge:count --> y\n",
    );
    let ws = fx.discover();

    let written = checks::counts::write_counts(&ws).expect("write counts");
    assert_eq!(written.len(), 1, "the stale page should be rewritten");

    let updated =
        std::fs::read_to_string(ws.docs_dir().join("architecture.md")).expect("read updated page");
    assert!(
        updated.contains("<!-- forge:count:ext_crates -->2<!-- /forge:count -->"),
        "marker should be corrected to 2: {updated}"
    );
    // Idempotent: a second pass changes nothing.
    assert!(
        checks::counts::write_counts(&ws)
            .expect("write counts again")
            .is_empty(),
        "second pass must be a no-op"
    );
}

#[test]
fn example_block_check_matches_app_imports() {
    use forge_docs_check::exampleblock;
    let fx = Fixture::new();
    fx.write(
        "examples/demo/manifest.app.toml",
        "[app]\nname = \"demo\"\n",
    );
    fx.write(
        "examples/demo/src/main.ts",
        "import { readText } from \"runtime:fs\";\nimport { open } from \"runtime:window\";\n",
    );
    let ws = fx.discover();

    // A page whose block matches the app's real imports must not be flagged.
    let expected = exampleblock::render_block_body(
        "demo",
        &exampleblock::runtime_modules(&ws.root.join("examples/demo")),
    );
    let page = format!(
        "# Demo\n\n## Capabilities\n\n{}\n{}\n{}\n\nprose\n",
        exampleblock::BLOCK_OPEN,
        expected,
        exampleblock::BLOCK_CLOSE
    );
    fx.write("site/src/content/docs/examples/demo.md", &page);
    let ws = fx.discover();
    assert!(
        exampleblock::check(&ws).is_empty(),
        "current example block must not be flagged: {:?}",
        messages(&exampleblock::check(&ws))
    );

    // A stale block (missing runtime:window) is flagged.
    let stale = format!(
        "## Capabilities\n\n{}\n**Runtime modules used:** `runtime:fs`\n{}\n",
        exampleblock::BLOCK_OPEN,
        exampleblock::BLOCK_CLOSE
    );
    fx.write("site/src/content/docs/examples/demo.md", &stale);
    let ws = fx.discover();
    assert!(
        exampleblock::check(&ws)
            .iter()
            .any(|f| f.rule == "example-block"),
        "stale example block must be flagged"
    );
}

#[test]
fn count_marker_finding_uses_forward_slashes() {
    let mut fx = Fixture::new();
    fx.add_crate("ext_a");
    fx.add_crate("ext_b");
    fx.add_crate("forge_cli");
    // Wrong marker value (claims 9; the fixture has 2 ext crates), in a nested page.
    fx.write(
        "site/src/content/docs/guides/platform.md",
        "x <!-- forge:count:ext_crates -->9<!-- /forge:count --> y\n",
    );
    let ws = fx.discover();
    let msg = checks::counts::check(&ws)
        .into_iter()
        .find(|f| f.rule == "count")
        .map(|f| f.message)
        .unwrap_or_default();
    assert!(
        msg.contains("guides/platform.md"),
        "finding should reference the nested page with forward slashes: {msg}"
    );
    // The protective assertion on the Windows matrix: never OS-native backslashes.
    assert!(
        !msg.contains('\\'),
        "finding paths must not use backslashes: {msg}"
    );
}
