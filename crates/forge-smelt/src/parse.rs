//! Module-graph discovery.
//!
//! Starting from an app's entry (`src/main.ts`), parse each module with
//! `deno_ast`, extract its static `import`/`export ... from` specifiers (with
//! their byte ranges in the source, so [`crate::transpile`] can rewrite them),
//! classify each specifier, resolve the relative ones to real files, and recurse
//! — producing the set of TypeScript modules reachable from the entry.
//!
//! `runtime:*`, bare (`react`), and URL/scheme (`npm:`, `jsr:`, `node:`,
//! `https:`) specifiers are recorded as **external** and left untouched: the
//! runtime, import maps, and Deno resolve those. Only relative (`./`, `../`)
//! specifiers point at files this crate compiles.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use deno_ast::swc::ast::{ModuleDecl, ModuleItem, Str};
use deno_ast::{MediaType, ParseParams, ProgramRef, SourceRangedForSpanned};

use crate::{SmeltError, SmeltResult};

/// How an import specifier is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepKind {
    /// A relative import resolved to a local source file (compiled by smelt).
    Relative(PathBuf),
    /// An external specifier left untouched (`runtime:*`, bare, npm:, url, ...).
    External,
}

/// One `import`/`export ... from` specifier found in a module.
#[derive(Debug, Clone)]
pub struct DepRef {
    /// The raw specifier text as written, e.g. `./util.ts` or `runtime:fs`.
    pub specifier: String,
    /// Byte range of the *string literal contents* (excluding quotes) within the
    /// module source, used for in-place rewriting.
    pub value_range: Range<usize>,
    /// Classification / resolution target.
    pub kind: DepKind,
}

/// A single TypeScript module in the graph.
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Absolute, canonicalized path to the source file.
    pub path: PathBuf,
    /// Path relative to the source root (e.g. `main.ts`, `lib/util.ts`); drives
    /// the mirrored output layout.
    pub rel_path: PathBuf,
    /// The module's TypeScript source text.
    pub source: String,
    /// Specifiers found in this module (relative ones resolved).
    pub deps: Vec<DepRef>,
}

/// The reachable module graph rooted at an app entry point.
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    /// The source root the relative paths are computed against.
    pub src_root: PathBuf,
    /// Absolute path to the entry module.
    pub entry: PathBuf,
    /// All reachable modules, entry first, in discovery order.
    pub modules: Vec<ModuleNode>,
}

impl ModuleGraph {
    /// Discover the module graph reachable from `entry`.
    ///
    /// `src_root` is the directory relative output paths are computed against
    /// (typically the app's `src/` directory).
    pub fn discover(entry: &Path, src_root: &Path) -> SmeltResult<ModuleGraph> {
        let entry = canonicalize(entry)?;
        let src_root = canonicalize(src_root)?;

        let mut modules = Vec::new();
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut queue: Vec<PathBuf> = vec![entry.clone()];

        while let Some(path) = queue.pop_front_like() {
            if !visited.insert(path.clone()) {
                continue;
            }

            let source = std::fs::read_to_string(&path).map_err(|e| SmeltError::io(&path, e))?;
            let deps = analyze_module(&path, &source)?;

            // Enqueue newly-discovered relative dependencies, but only recurse
            // into TypeScript-family modules — `.js`/`.json`/asset targets are
            // copied verbatim (see [`crate::binary`]) and must not be parsed as
            // TypeScript.
            for dep in &deps {
                if let DepKind::Relative(target) = &dep.kind {
                    if is_ts_family(target) && !visited.contains(target) {
                        queue.push(target.clone());
                    }
                }
            }

            // Every compiled module must live within the source root so its
            // mirrored output path stays relative (otherwise `out_dir.join(..)`
            // could escape to an absolute location).
            let rel_path = path
                .strip_prefix(&src_root)
                .map_err(|_| {
                    SmeltError::invalid_input(format!(
                        "module '{}' is outside the source root '{}'",
                        path.display(),
                        src_root.display()
                    ))
                })?
                .to_path_buf();

            modules.push(ModuleNode {
                path,
                rel_path,
                source,
                deps,
            });
        }

        Ok(ModuleGraph {
            src_root,
            entry,
            modules,
        })
    }

    /// Look up a module by its absolute path.
    pub fn module(&self, path: &Path) -> Option<&ModuleNode> {
        self.modules.iter().find(|m| m.path == path)
    }
}

/// Parse one module and extract its static import/export specifiers.
fn analyze_module(path: &Path, source: &str) -> SmeltResult<Vec<DepRef>> {
    let media_type = MediaType::from_path(path);
    let specifier = deno_ast::ModuleSpecifier::from_file_path(path)
        .map_err(|_| SmeltError::parse(path, "path is not a valid file URL"))?;

    let parsed = deno_ast::parse_program(ParseParams {
        specifier,
        text: source.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(|e| SmeltError::parse(path, e.to_string()))?;

    let text_info = parsed.text_info_lazy().clone();
    let base = text_info.range().start;

    let module = match parsed.program_ref() {
        ProgramRef::Module(m) => m,
        // Scripts (no ESM) carry no static import/export specifiers.
        ProgramRef::Script(_) => return Ok(Vec::new()),
    };

    let referrer_dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut deps = Vec::new();

    for item in &module.body {
        let ModuleItem::ModuleDecl(decl) = item else {
            continue;
        };
        // Collect the string-literal source node for each `... from "x"` form.
        let src_str: Option<&Str> = match decl {
            ModuleDecl::Import(import) => Some(import.src.as_ref()),
            ModuleDecl::ExportNamed(named) => named.src.as_deref(),
            ModuleDecl::ExportAll(all) => Some(all.src.as_ref()),
            _ => None,
        };
        let Some(src_str) = src_str else { continue };

        // `range` spans the literal *including* quotes; narrow to the contents
        // and read the specifier text directly from the source. (Reading the
        // source avoids `Str.value`'s `Wtf8Atom`, which is not `Display`.)
        let range = src_str.range();
        let lit_start = range.start.as_byte_index(base);
        let lit_end = range.end.as_byte_index(base);
        let value_range = narrow_to_contents(source, lit_start, lit_end);
        let raw = source[value_range.clone()].to_string();

        let kind = classify(&raw, &referrer_dir, path)?;
        deps.push(DepRef {
            specifier: raw,
            value_range,
            kind,
        });
    }

    Ok(deps)
}

/// Classify a specifier and, for relative ones, resolve it to a real file.
fn classify(specifier: &str, referrer_dir: &Path, referrer: &Path) -> SmeltResult<DepKind> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        let resolved = resolve_relative(specifier, referrer_dir)
            .ok_or_else(|| SmeltError::unresolved_import(specifier, referrer))?;
        Ok(DepKind::Relative(resolved))
    } else {
        // runtime:*, bare module names, npm:/jsr:/node:/http(s): — all external.
        Ok(DepKind::External)
    }
}

/// Resolve a relative specifier against the referrer's directory, trying the
/// usual TypeScript extension/`index` candidates.
fn resolve_relative(specifier: &str, referrer_dir: &Path) -> Option<PathBuf> {
    let joined = referrer_dir.join(specifier);

    // 1. Exact path as written (e.g. `./util.ts`, `./data.json`).
    if joined.is_file() {
        return canonicalize(&joined).ok();
    }

    // 2. A written `.js`/`.mjs` specifier whose on-disk source is `.ts`/`.tsx`.
    if let Some(swapped) = swap_js_for_ts(&joined) {
        if swapped.is_file() {
            return canonicalize(&swapped).ok();
        }
    }

    // 3. Extensionless: try source extensions, then `index` in a directory.
    for ext in ["ts", "tsx", "mts", "cts", "js", "mjs", "cjs", "jsx"] {
        let candidate = with_appended_ext(&joined, ext);
        if candidate.is_file() {
            return canonicalize(&candidate).ok();
        }
    }
    for idx in ["index.ts", "index.tsx", "index.js", "index.mjs"] {
        let candidate = joined.join(idx);
        if candidate.is_file() {
            return canonicalize(&candidate).ok();
        }
    }

    None
}

/// Whether a path is a TypeScript-family module that smelt transpiles.
/// JavaScript, JSON, and other targets are copied verbatim instead.
pub fn is_ts_family(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts") | Some("tsx") | Some("mts") | Some("cts")
    )
}

/// If `path` ends in a JS extension, return the same path with the matching TS
/// extension (so `./util.js` can resolve to an on-disk `util.ts`).
fn swap_js_for_ts(path: &Path) -> Option<PathBuf> {
    let ext = path.extension()?.to_str()?;
    let ts_ext = match ext {
        "js" => "ts",
        "mjs" => "mts",
        "cjs" => "cts",
        "jsx" => "tsx",
        _ => return None,
    };
    Some(path.with_extension(ts_ext))
}

/// Append an extension to a path that has none (preserving any existing stem),
/// e.g. `dir/util` + `ts` -> `dir/util.ts`.
fn with_appended_ext(path: &Path, ext: &str) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".");
    s.push(ext);
    PathBuf::from(s)
}

/// Given byte offsets of a string literal *including* its surrounding quotes,
/// narrow them to the literal contents (the specifier text).
fn narrow_to_contents(source: &str, start: usize, end: usize) -> Range<usize> {
    // The literal always opens and closes with a quote byte (`"`, `'`, or `` ` ``).
    // Guard against unexpected shapes by only trimming when the ends are quotes.
    let bytes = source.as_bytes();
    let is_quote = |i: usize| matches!(bytes.get(i), Some(b'"') | Some(b'\'') | Some(b'`'));
    if end > start + 1 && is_quote(start) && is_quote(end - 1) {
        (start + 1)..(end - 1)
    } else {
        start..end
    }
}

/// Canonicalize, mapping I/O failures (e.g. missing file) to a smelt error.
fn canonicalize(path: &Path) -> SmeltResult<PathBuf> {
    path.canonicalize().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SmeltError::entry_not_found(path)
        } else {
            SmeltError::io(path, e)
        }
    })
}

/// Small helper: pop from the front to keep discovery order stable without
/// pulling in `VecDeque` ceremony at every call site.
trait PopFront<T> {
    fn pop_front_like(&mut self) -> Option<T>;
}
impl<T> PopFront<T> for Vec<T> {
    fn pop_front_like(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }
}

/// Build a quick path->index map (used by callers that need random access).
pub fn index_by_path(graph: &ModuleGraph) -> HashMap<PathBuf, usize> {
    graph
        .modules
        .iter()
        .enumerate()
        .map(|(i, m)| (m.path.clone(), i))
        .collect()
}
