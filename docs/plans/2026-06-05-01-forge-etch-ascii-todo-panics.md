# C1 — Remove `todo!()` panics in `forge-etch` ASCII doc renderer

**Severity:** CRITICAL (unrecoverable panic) · **Source:** Fix.md › C1 · **Crate:** `forge-etch`

## Goal
Make the ASCII documentation renderer total over `EtchNodeKind` so no input can panic. Render `Module`, `Import`, and `Reference` nodes with real labels and content instead of `todo!()`.

## Root cause (verified)
`crates/forge-etch/src/docgen/ascii.rs`:
- `render_node_ascii()` lines **170–172** — `EtchNodeKind::Module | Import | Reference => todo!()`
- `render_summary_table()` lines **260–262** — same three arms `=> todo!()`

`todo!()` = `panic!("not yet implemented")`. Both functions are public (exported in `crates/forge-etch/src/docgen/mod.rs:21`) and the three kinds are real, constructed variants: `crates/forge-etch/src/node.rs:551–554` maps `EtchNodeDef::Module/Import/Reference` (and `ModuleDoc` → `Module`) to these kinds. Definitions: `ModuleDef` (node.rs:179), `ImportDef` (220), `ReferenceDef` (229).

## Affected files
- `crates/forge-etch/src/docgen/ascii.rs` (the two `match` blocks)
- `crates/forge-etch/src/node.rs` (read-only: confirm field names on `ModuleDef`/`ImportDef`/`ReferenceDef` for what to print)

## Implementation steps
1. In `render_node_ascii()` (the `kind_str` match, ~160–173), replace the three `todo!()` arms with literal labels consistent with the existing style: `EtchNodeKind::Module => "module"`, `Import => "import"`, `Reference => "reference"`.
2. The function already renders title + optional signature + description + params/returns. For `Module`/`Import`/`Reference`, the `params`/`return_type` matches (lines 199–203, 232–236) fall through to `None`, so they degrade cleanly. Additionally surface their specific data: read `node.def` and, for `EtchNodeDef::Module { module_def }`, print the module specifier/child count; for `Import { import_def }`, print the imported symbols/source; for `Reference { reference_def }`, print the referenced name/target. Use the actual public fields on those structs (read `node.rs:177–229`).
3. In `render_summary_table()` (the `kind` match, ~252–263), replace the three `todo!()` arms with short labels matching the table's compact style: `Module => "mod"`, `Import => "import"`, `Reference => "ref"`.
4. Grep for any other exhaustive `match node.kind()` / `match …EtchNodeKind` in the docgen module and confirm none also use `todo!()`/`unimplemented!()` (`grep -rn "todo!\|unimplemented!" crates/forge-etch/src/docgen/`).

## Regression test (mandatory)
Add to `ascii.rs` under `#[cfg(test)]`:
- Construct an `EtchNode` of each kind `Module`, `Import`, `Reference` (via the same constructors `node.rs` uses, e.g. `EtchNodeDef::Module { module_def: ModuleDef { … } }`).
- Assert `render_node_ascii(&node)` returns a non-empty `String` containing the expected label (`"module"`/`"import"`/`"reference"`) and does **not** panic.
- Assert `render_summary_table(&[module_node, import_node, reference_node])` returns a table containing all three rows.
A pre-fix run of these tests panics; post-fix they pass — that is the regression proof.

## Done criteria
- `cargo test -p forge-etch` passes incl. the new tests.
- `grep -rn "todo!\|unimplemented!" crates/forge-etch/src/docgen/` returns nothing.
- `cargo clippy -p forge-etch -- -D warnings` clean.

## Notes / risks
Low risk — pure additive rendering. Only subtlety is reading the exact public field names on `ModuleDef`/`ImportDef`/`ReferenceDef`; do not invent fields, read `node.rs` first.
