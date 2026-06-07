//! Characterization snapshot of the `forge` CLI's argument-parsing contract.
//!
//! This is the safety net for the planned clap-derive migration (Phase 5 of
//! `docs/plans/2026-06-06-19-docs-autogeneration-pipeline.md`): it records the
//! *current* hand-parser's observable behavior — exit status, usage banner, and
//! error messages for the parse paths that don't require a real app — so the
//! migration can be verified byte-compatible. These tests touch no production
//! code; they only run the built binary and assert its output.
//!
//! Behaviors are chosen to be deterministic (no real app dir, no environment
//! dependence): no-args usage, unknown subcommand, and missing/invalid flags.

use std::process::{Command, Output};

/// Run the `forge` binary with `args` and capture its output.
fn forge(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(args)
        .output()
        .expect("run the forge binary")
}

#[test]
fn no_args_prints_usage_to_stderr_and_succeeds() {
    let out = forge(&[]);
    assert!(out.status.success(), "no-args invocation must exit 0");
    assert!(
        out.stdout.is_empty(),
        "the usage banner is written to stderr, not stdout"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("forge <dev|build|bundle|smelt|sign|icon|docs> [options] <app-dir>"),
        "missing the usage banner: {stderr}"
    );
}

#[test]
fn usage_documents_every_subcommand() {
    let out = forge(&[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    for cmd in ["dev", "build", "bundle", "smelt", "sign", "icon", "docs"] {
        assert!(
            stderr.contains(&format!("  {cmd} ")),
            "usage banner must document `{cmd}`: {stderr}"
        );
    }
}

#[test]
fn unknown_subcommand_falls_through_to_usage() {
    // Current contract: an unrecognized subcommand prints usage and exits 0
    // (the `_ => usage()` arm). The migration must preserve this, not error.
    let out = forge(&["definitely-not-a-subcommand"]);
    assert!(
        out.status.success(),
        "unknown subcommand currently exits 0 (prints usage)"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("forge <dev|build|bundle"));
}

#[test]
fn smelt_rejects_unknown_flag() {
    let out = forge(&["smelt", "--bogus"]);
    assert!(!out.status.success(), "an unknown flag must fail");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Unknown flag"),
        "expected an 'Unknown flag' error: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn sign_identity_flag_requires_a_value() {
    let out = forge(&["sign", "--identity"]);
    assert!(!out.status.success(), "a dangling --identity must fail");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--identity requires a value"),
        "expected the '--identity requires a value' error: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
