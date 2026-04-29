//! Subprocess check: `cargo test` output lists each selected test name on its own line.
//!
//! Uses the `#[test]` fns emitted by [`test_suite!`] in [`super::suite_test`].

use std::process::Command;

#[test]
fn cargo_test_output_lists_both_filtered_lib_tests() {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(cargo)
        .current_dir(manifest_dir)
        .args([
            "test",
            "--lib",
            "--",
            "suite::suite_test::defaults_suite_test_run",
            "suite::suite_test::recorder_suite_test_run",
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
        combined.contains("defaults_suite_test_run"),
        "expected a line for defaults_suite_test_run in cargo output; got:\n{combined}",
    );
    assert!(
        combined.contains("recorder_suite_test_run"),
        "expected a line for recorder_suite_test_run in cargo output; got:\n{combined}",
    );
}
