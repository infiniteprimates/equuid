//! Process-boundary tests for `argv[0]` dispatch.
//!
//! Only a real `exec` can hand a binary a name other than the one on disk, so
//! dispatch is tested here rather than in `main.rs`'s unit tests. Unix-only
//! because `CommandExt::arg0` overrides `argv[0]` without renaming the
//! executable; the Windows equivalent would be copying the binary to
//! `uuidgen.exe`, which belongs in the distro checklist, not the test suite.

#![cfg(unix)]

use std::os::unix::process::CommandExt;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_equuid");

/// Run the built binary under a chosen `argv[0]`.
///
/// `.arg0()` rewrites `argv[0]` while the kernel still executes `BIN` — the same
/// shape as a symlink or a hard link, which is how the aliases will ship.
fn run_as(argv0: &str, args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(BIN)
        .arg0(argv0)
        .args(args)
        .output()
        .expect("failed to spawn equuid");

    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The contract a caller can observe: which failure exits with which code.
///
/// Deliberately asserted end to end — these codes are produced by `main`'s
/// classification step, so a unit test on `parse` would not cover them.
#[test]
fn exit_codes_hold_across_the_process_boundary() {
    assert_eq!(run_as("equuid", &["--bogus"]).0, Some(2), "equuid usage");
    assert_eq!(run_as("uuidgen", &["--bogus"]).0, Some(1), "uuidgen usage");
    assert_eq!(
        run_as("notaprog", &["--help"]).0,
        Some(1),
        "bad program name"
    );
    assert_eq!(run_as("equuid", &[]).0, Some(0), "bare invocation");
}

/// Dispatch is by stem, so a Windows-style `argv[0]` still reaches its parser,
/// while the usage line echoes the name it was actually invoked under.
#[test]
fn a_name_with_an_extension_dispatches_by_its_stem() {
    let (code, stdout, stderr) = run_as("equuid.exe", &["--help"]);

    assert_eq!(code, Some(0), "stderr: {stderr}");
    assert!(
        stdout.contains("Usage: equuid.exe"),
        "argv[0] should be echoed verbatim, got: {stdout}"
    );
}

/// The regression this suite exists for: a usage error used to be printed twice,
/// the second copy prefixed `Failed to render output:` and carrying raw ANSI
/// because it bypassed anstream.
#[test]
fn a_usage_error_is_reported_exactly_once() {
    let (code, stdout, stderr) = run_as("equuid", &["--bogus"]);

    assert_eq!(code, Some(2));
    assert!(stdout.is_empty(), "usage error wrote to stdout: {stdout}");
    assert_eq!(
        stderr.matches("unexpected argument").count(),
        1,
        "error text printed more than once: {stderr}"
    );
    assert!(
        !stderr.contains("Failed to render output"),
        "a usage error was reported as a render failure: {stderr}"
    );
}

/// Success and failure take opposite streams. Asserting the empty side matters:
/// a stream mix-up would otherwise still satisfy a `contains` assertion.
#[test]
fn help_and_errors_go_to_opposite_streams() {
    let (code, stdout, stderr) = run_as("equuid", &["--help"]);
    assert_eq!(code, Some(0));
    assert!(
        stdout.contains("Usage: equuid"),
        "no help on stdout: {stdout}"
    );
    assert!(stderr.is_empty(), "help wrote to stderr: {stderr}");

    let (code, stdout, stderr) = run_as("uuidgen", &["--bogus"]);
    assert_eq!(code, Some(1));
    assert!(stdout.is_empty(), "usage error wrote to stdout: {stdout}");
    assert!(stderr.contains("Usage: uuidgen"), "no usage text: {stderr}");
}
