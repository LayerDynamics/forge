use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Copy sdk/preload.ts to OUT_DIR for inclusion in main.rs
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let preload_src = Path::new(&manifest_dir).join("../../sdk/preload.ts");
    let preload_dest = out_path.join("preload.ts");

    if preload_src.exists() {
        fs::copy(&preload_src, &preload_dest).expect("Failed to copy preload.ts to OUT_DIR");
        println!("cargo:rerun-if-changed={}", preload_src.display());
    } else {
        panic!("sdk/preload.ts not found at {}", preload_src.display());
    }

    let dest_path = out_path.join("assets.rs");
    let mut f = File::create(&dest_path).unwrap();

    // Check if FORGE_EMBED_DIR is set (for release builds with embedded assets)
    if let Ok(embed_dir) = env::var("FORGE_EMBED_DIR") {
        let embed_path = Path::new(&embed_dir);
        if embed_path.exists() && embed_path.is_dir() {
            println!("cargo:rerun-if-changed={}", embed_dir);

            // Collect all files in the embed directory
            let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
            collect_files(embed_path, embed_path, &mut entries);

            // Generate the assets module
            writeln!(f, "pub const ASSET_EMBEDDED: bool = true;").unwrap();
            writeln!(f).unwrap();

            // Generate static byte arrays for each asset
            for (i, (_path, bytes)) in entries.iter().enumerate() {
                writeln!(f, "static ASSET_{}: &[u8] = &{:?};", i, bytes).unwrap();
            }

            writeln!(f).unwrap();
            writeln!(
                f,
                "pub fn get_asset(path: &str) -> Option<&'static [u8]> {{"
            )
            .unwrap();
            writeln!(f, "    match path {{").unwrap();
            for (i, (path, _)) in entries.iter().enumerate() {
                writeln!(f, "        {:?} => Some(ASSET_{}),", path, i).unwrap();
            }
            writeln!(f, "        _ => None,").unwrap();
            writeln!(f, "    }}").unwrap();
            writeln!(f, "}}").unwrap();

            println!(
                "cargo:warning=Embedded {} assets from {}",
                entries.len(),
                embed_dir
            );
            return;
        }
    }

    // Default: no embedded assets (dev mode)
    writeln!(f, "pub const ASSET_EMBEDDED: bool = false;").unwrap();
    writeln!(f).unwrap();
    writeln!(f, "#[allow(unused_variables)]").unwrap();
    writeln!(
        f,
        "pub fn get_asset(path: &str) -> Option<&'static [u8]> {{"
    )
    .unwrap();
    writeln!(f, "    None").unwrap();
    writeln!(f, "}}").unwrap();
}

fn collect_files(base: &Path, current: &Path, entries: &mut Vec<(String, Vec<u8>)>) {
    if let Ok(dir) = fs::read_dir(current) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(base, &path, entries);
            } else if path.is_file() {
                if let Ok(bytes) = fs::read(&path) {
                    let relative = path.strip_prefix(base).unwrap();
                    let key = relative.to_string_lossy().replace('\\', "/");
                    entries.push((key, bytes));
                }
            }
        }
    }
}
