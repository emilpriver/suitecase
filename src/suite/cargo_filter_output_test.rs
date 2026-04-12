//! Subprocess check: `cargo test` output lists each selected test name on its own line.
//!
//! Uses the `#[test]` fns emitted by [`test_suite!`] in [`super::suite_test`] (`test_a`, `test_b`).

use std::process::Command;

#[test]
fn cargo_test_output_lists_both_filtered_lib_tests() {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(cargo)
        .current_dir(manifest_dir)
        // Cargo accepts only one `TESTNAME` before `--`; libtest filters go after `--`.
        .args([
            "test",
            "--lib",
            "--",
            "suite::suite_test::test_a",
            "suite::suite_test::test_b",
        ])
        .output()
        .expect("spawn cargo test");

    assert!(
        output.status.success(),
        "cargo test failed (status {:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert!(
        combined.contains("running 2 tests"),
        "expected harness to run exactly two tests; output:\n{combined}",
    );
    assert!(
        combined.contains("suite::suite_test::test_a"),
        "expected a line for test_a in cargo output; got:\n{combined}",
    );
    assert!(
        combined.contains("suite::suite_test::test_b"),
        "expected a line for test_b in cargo output; got:\n{combined}",
    );
}
