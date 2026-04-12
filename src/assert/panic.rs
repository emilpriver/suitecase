//! Assertions about whether code panics, using [`std::panic::catch_unwind`].

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Asserts that invoking `f` unwinds with a panic.
///
/// # Panics
///
/// Panics when `f` returns normally.
///
/// # Examples
///
/// ```
/// use suitecase::assert::panics;
///
/// panics(|| panic!("boom"));
/// ```
#[track_caller]
pub fn panics(f: impl FnOnce()) {
    let r = catch_unwind(AssertUnwindSafe(f));
    if r.is_ok() {
        panic!("assertion failed: `panics`: expected closure to panic");
    }
}

/// Asserts that invoking `f` completes without panicking.
///
/// # Panics
///
/// Panics when `f` unwinds.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_panics;
///
/// not_panics(|| {
///     let _ = 1 + 1;
/// });
/// ```
#[track_caller]
pub fn not_panics(f: impl FnOnce()) {
    let r = catch_unwind(AssertUnwindSafe(f));
    if let Err(payload) = r {
        panic!(
            "assertion failed: `not_panics`: closure panicked: {:?}",
            panic_payload_to_string(payload)
        );
    }
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&'static str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(opaque panic payload)".to_string())
}

/// Asserts that `f` panics and the panic message contains `needle`.
///
/// # Panics
///
/// Panics when `f` does not panic, or the message does not contain `needle`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::panics_with_substring;
///
/// panics_with_substring(|| panic!("expected token"), "token");
/// ```
#[track_caller]
pub fn panics_with_substring(f: impl FnOnce(), needle: &str) {
    let r = catch_unwind(AssertUnwindSafe(f));
    match r {
        Ok(()) => panic!("assertion failed: `panics_with_substring`: expected panic"),
        Err(payload) => {
            let s = panic_payload_to_string(payload);
            if !s.contains(needle) {
                panic!(
                    "assertion failed: `panics_with_substring`\n  needle: `{needle}`\n message: `{s}`"
                );
            }
        }
    }
}
