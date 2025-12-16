//! Path utilities for normalizing and extracting components.

use deno_core::{op2, Extension};
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[weld_struct]
#[derive(Serialize)]
struct PathParts {
    dir: String,
    base: String,
    ext: String,
}

fn to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn path_join(base: String, segments: Vec<String>) -> String {
    let mut pb = PathBuf::from(base);
    for seg in segments {
        pb.push(seg);
    }
    to_string(pb)
}

fn path_dirname(path: &str) -> String {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string())
}

fn path_basename(path: &str) -> String {
    Path::new(&path)
        .file_name()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string())
}

fn path_extname(path: &str) -> String {
    Path::new(&path)
        .extension()
        .map(|p| format!(".{}", p.to_string_lossy()))
        .unwrap_or_else(|| "".to_string())
}

#[weld_op]
#[op2]
#[string]
fn op_path_join(#[string] base: String, #[serde] segments: Vec<String>) -> String {
    path_join(base, segments)
}

#[weld_op]
#[op2]
#[string]
fn op_path_dirname(#[string] path: String) -> String {
    path_dirname(&path)
}

#[weld_op]
#[op2]
#[string]
fn op_path_basename(#[string] path: String) -> String {
    path_basename(&path)
}

#[weld_op]
#[op2]
#[string]
fn op_path_extname(#[string] path: String) -> String {
    path_extname(&path)
}

#[weld_op]
#[op2]
#[serde]
fn op_path_parts(#[string] path: String) -> PathParts {
    PathParts {
        dir: path_dirname(&path),
        base: path_basename(&path),
        ext: path_extname(&path),
    }
}

deno_core::extension!(
    ext_path,
    ops = [
        op_path_join,
        op_path_dirname,
        op_path_basename,
        op_path_extname,
        op_path_parts
    ]
);

pub fn path_extension() -> Extension {
    ext_path::init()
}
