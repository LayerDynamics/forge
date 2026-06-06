//! Pipeline orchestration: discover → transpile → materialize.

use std::path::Path;

use crate::binary::{materialize, SmeltOutput};
use crate::parse::ModuleGraph;
use crate::{SmeltError, SmeltResult};

/// Smelt an app's TypeScript into a compiled JavaScript tree.
///
/// `app_dir` is the app root (containing `src/main.ts`). The compiled `.js` tree
/// is written under `out_dir`, mirroring the app's `src/` layout, with the entry
/// emitted as `main.js`. Relative `./x.ts` imports are rewritten to `./x.js`;
/// `runtime:*` and other external specifiers are left untouched.
///
/// Returns a [`SmeltOutput`] describing the compiled entry and modules.
pub fn smelt(app_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> SmeltResult<SmeltOutput> {
    let app_dir = app_dir.as_ref();
    let src_root = app_dir.join("src");
    let entry = src_root.join("main.ts");
    if !entry.is_file() {
        return Err(SmeltError::entry_not_found(&entry));
    }
    smelt_entry(&entry, &src_root, out_dir.as_ref())
}

/// Smelt starting from an explicit entry file and source root.
///
/// Use this when the entry is not the conventional `<app>/src/main.ts` (e.g. a
/// custom entry name, or a non-`src` root). `src_root` is the directory the
/// mirrored output layout is computed against; the entry must live within it.
pub fn smelt_entry(
    entry: impl AsRef<Path>,
    src_root: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> SmeltResult<SmeltOutput> {
    let entry = entry.as_ref();
    let src_root = src_root.as_ref();
    if !entry.is_file() {
        return Err(SmeltError::entry_not_found(entry));
    }
    let graph = ModuleGraph::discover(entry, src_root)?;
    materialize(&graph, out_dir.as_ref())
}
