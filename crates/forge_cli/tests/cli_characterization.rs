//! Characterization of the `forge` CLI's argument-parsing contract.
//!
//! Originally (P5.1) these snapshots captured the hand-rolled `env::args()`
//! parser. P5.2 migrated the CLI to **clap derive**, which deliberately changes
//! the *edge-case* behavior to clap's conventions (better help + errors):
//!
//! | case | old hand-parser | clap (now) |
//! |------|-----------------|------------|
//! | no args | usage to stderr, **exit 0** | help to stderr, **exit != 0** |
//! | unknown subcommand | usage, **exit 0** | error, **exit != 0** |
//! | unknown flag | `Unknown flag: X` | `unexpected argument '--X'` |
//! | missing flag value | `--identity requires a value` | `a value is required for ...` |
//!
//! The **real contract** — the set of subcommands, their flags, the `-o`/`-i`
//! short aliases, and the positionals — is preserved exactly, and is asserted
//! below so a future change can't silently drop a command or flag.

use std::process::{Command, Output};

fn forge(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(args)
        .output()
        .expect("run the forge binary")
}

fn combined(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

// --- Edge-case behavior (clap conventions) ---------------------------------

#[test]
fn no_args_shows_help_and_exits_nonzero() {
    // clap `arg_required_else_help`: print help, exit non-zero.
    let out = forge(&[]);
    assert!(!out.status.success(), "no-args now exits non-zero (clap)");
    assert!(
        combined(&out).contains("Usage: forge"),
        "no-args must show usage: {}",
        combined(&out)
    );
}

#[test]
fn help_documents_every_subcommand() {
    let out = forge(&["--help"]);
    assert!(out.status.success(), "--help exits 0");
    let text = combined(&out);
    for cmd in ["dev", "build", "bundle", "smelt", "sign", "icon", "docs"] {
        assert!(
            text.contains(&format!("  {cmd}")),
            "help must list `{cmd}`: {text}"
        );
    }
    // The hand-written extras were preserved via clap `after_help`.
    assert!(text.contains("Getting Started:"));
    assert!(text.contains("Bundle output formats:"));
}

#[test]
fn unknown_subcommand_is_an_error() {
    let out = forge(&["definitely-not-a-subcommand"]);
    assert!(
        !out.status.success(),
        "unknown subcommand now errors (clap)"
    );
}

#[test]
fn smelt_rejects_unknown_flag() {
    let out = forge(&["smelt", "app", "--bogus"]);
    assert!(!out.status.success());
    assert!(
        combined(&out).contains("--bogus"),
        "error should name the offending flag: {}",
        combined(&out)
    );
}

#[test]
fn sign_identity_flag_requires_a_value() {
    let out = forge(&["sign", "--identity"]);
    assert!(!out.status.success());
    assert!(
        combined(&out).contains("--identity"),
        "error should reference --identity: {}",
        combined(&out)
    );
}

// --- The real contract: commands, flags, aliases, positionals --------------

#[test]
fn version_flag_works() {
    assert!(forge(&["--version"]).status.success());
}

#[test]
fn every_subcommand_has_help() {
    for cmd in ["dev", "build", "bundle", "smelt", "sign", "icon"] {
        let out = forge(&[cmd, "--help"]);
        assert!(out.status.success(), "`forge {cmd} --help` must succeed");
        assert!(
            combined(&out).contains(&format!("forge {cmd}")),
            "`forge {cmd} --help` should show its usage: {}",
            combined(&out)
        );
    }
}

#[test]
fn smelt_exposes_out_and_embed_flags_with_short_alias() {
    let help = combined(&forge(&["smelt", "--help"]));
    assert!(help.contains("--out"), "smelt keeps --out: {help}");
    assert!(help.contains("-o"), "smelt keeps the -o alias: {help}");
    assert!(help.contains("--embed"), "smelt keeps --embed: {help}");
}

#[test]
fn sign_exposes_identity_flag_with_short_alias_and_artifact_positional() {
    let help = combined(&forge(&["sign", "--help"]));
    assert!(help.contains("--identity"), "sign keeps --identity: {help}");
    assert!(help.contains("-i"), "sign keeps the -i alias: {help}");
    assert!(
        help.to_lowercase().contains("artifact"),
        "sign keeps the artifact positional: {help}"
    );
}

#[test]
fn icon_has_create_and_validate_subcommands() {
    let help = combined(&forge(&["icon", "--help"]));
    assert!(help.contains("create"), "icon keeps `create`: {help}");
    assert!(help.contains("validate"), "icon keeps `validate`: {help}");
}
