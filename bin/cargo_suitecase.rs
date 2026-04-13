//! `cargo-suitecase` — Cargo subcommand wrapper (`cargo suitecase …`).
//!
//! Install the binary on your `PATH` (e.g. `cargo install suitecase` from crates.io, or
//! `cargo install --path .` in this repo). Then:
//!
//! ```text
//! cargo suitecase test [CARGO_TEST_ARGS...]
//! ```
//!
//! In GitHub Actions (`GITHUB_ACTIONS=true`), this forwards to `cargo test` with defaults that
//! make **compiler** diagnostics easier to read in the Actions log viewer (ANSI-rendered JSON
//! diagnostics). Pair with a Rust [problem matcher](https://github.com/actions/toolkit/blob/main/docs/problem-matchers.md)
//! in your workflow if you want file/line annotations for `rustc` output.

use std::ffi::OsString;
use std::process::{Command, ExitCode, ExitStatus, Stdio};

fn main() -> ExitCode {
    let mut argv: Vec<OsString> = std::env::args_os().skip(1).collect();

    match argv.first().map(|s| s.as_os_str()) {
        None => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(s) if s == "-h" || s == "--help" => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(s) if s == "test" => {
            argv.remove(0);
            inject_github_actions_defaults(&mut argv);
            run_cargo_test(argv)
        }
        _ => {
            eprintln!(
                "suitecase: unknown subcommand (try `cargo suitecase test`, or `cargo suitecase --help`)"
            );
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    print!(
        "\
Usage:
    cargo suitecase test [OPTIONS] [-- TEST_ARGS...]

Runs `cargo test` with optional CI-friendly defaults when GITHUB_ACTIONS=true.

Environment:
    GITHUB_ACTIONS     When set to \"true\", injects --message-format for clearer logs.
    SUITECASE_NO_CI    Set to any value to skip those injections.
"
    );
}

fn inject_github_actions_defaults(args: &mut Vec<OsString>) {
    if !should_inject_ci_defaults() {
        return;
    }
    inject_message_format_before_libtest_split(args);
}

fn should_inject_ci_defaults() -> bool {
    std::env::var("SUITECASE_NO_CI").is_err()
        && std::env::var("GITHUB_ACTIONS").ok().as_deref() == Some("true")
}

fn inject_message_format_before_libtest_split(args: &mut Vec<OsString>) {
    if args.iter().any(|a| {
        let s = a.to_string_lossy();
        s.starts_with("--message-format") || s == "-q" || s.starts_with("-q=")
    }) {
        return;
    }

    let insert_at = args.iter().position(|a| a == "--").unwrap_or(args.len());
    args.insert(insert_at, OsString::from("json-diagnostic-rendered-ansi"));
    args.insert(insert_at, OsString::from("--message-format"));
}

fn run_cargo_test(cargo_and_test_args: Vec<OsString>) -> ExitCode {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(&cargo_and_test_args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    if should_inject_ci_defaults() {
        cmd.env("CARGO_TERM_COLOR", "always");
        if std::env::var("RUST_BACKTRACE").is_err() {
            cmd.env("RUST_BACKTRACE", "1");
        }
    }

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("suitecase: failed to run `cargo test`: {e}");
            return ExitCode::from(101);
        }
    };

    exit_code_from_status(status)
}

fn exit_code_from_status(status: ExitStatus) -> ExitCode {
    match status.code() {
        None => ExitCode::from(1),
        Some(0) => ExitCode::SUCCESS,
        Some(c) => match u8::try_from(c) {
            Ok(v) => ExitCode::from(v),
            Err(_) => ExitCode::from(1),
        },
    }
}

#[cfg(test)]
#[path = "cargo_suitecase_tests.rs"]
mod cargo_suitecase_tests;
