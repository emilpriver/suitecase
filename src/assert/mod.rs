//! Panicking assertion helpers for tests (in the spirit of Go’s
//! [testify/assert](https://pkg.go.dev/github.com/stretchr/testify/assert)).
//!
//! Import the names you need from this module (`suitecase::assert`):
//!
//! ```
//! use suitecase::assert::{equal, contains, assert_ok};
//! ```
//!
//! # When to use
//!
//! Use these inside [`#[test]`](https://doc.rust-lang.org/reference/attributes/testing.html#the-test-attribute)
//! functions, suite case bodies, or anywhere you want a **named** check that **fails the test by
//! panicking** with a clear message. They complement `assert!` / `assert_eq!` with richer diagnostics
//! for common patterns (collections, errors, floats, options, ordering, …).
//!
//! # Panics and caller location
//!
//! Every public function in this module **panics** when its condition is not met (see each item’s
//! **Panics** section). Functions are annotated with [`#[track_caller]`](https://doc.rust-lang.org/std/panic/struct.Location.html),
//! so the panic points at the **call site** in your test, not inside this crate.
//!
//! # Submodules
//!
//! Implementation is split across private submodules (`boolean`, `collections`, `equality`, …);
//! every public helper is re-exported from this module so you can use a flat import path.
//!
//! | Subdirectory | Role |
//! |--------------|------|
//! | `boolean` | `true_`, `false_`, and `_with_msg` variants |
//! | `collections` | Strings, slices, maps, multiset subset / equality |
//! | `equality` | `equal`, `not_equal`, `equal_values`, messages |
//! | `errors` | [`std::error::Error`] chains, downcasts, I/O kinds |
//! | `floats` | Absolute delta and relative epsilon for floats |
//! | `fs` | File and directory existence |
//! | `option_result` | [`Option`], [`Result`], “is zero” checks |
//! | `ordering` | Comparisons and monotonic slices |
//! | `panic` | Expect panics / no panic, substring match |
//! | `pointer` | Same allocation (`std::sync::Arc`, `std::sync::Weak`, references) |
//! | `time` | [`std::time::Duration`] tolerances |
//!
//! Root-level helpers such as [`fail`], [`condition`], and [`same`] are defined in this module.
//!
//! # Example
//!
//! ```
//! use suitecase::assert::{contains_str, equal, assert_ok};
//!
//! let value = assert_ok(Ok::<_, &str>(42));
//! equal(&value, &42);
//! contains_str("hello world", "world");
//! ```

mod boolean;
mod collections;
mod equality;
mod errors;
mod floats;
mod fs;
mod option_result;
mod ordering;
mod panic;
mod pointer;
mod time;

pub use boolean::{false_, false_with_msg, true_, true_with_msg};
pub use collections::{
    contains, contains_key, contains_str, elements_match, empty_slice, empty_str, len, len_string,
    not_contains, not_contains_str, not_elements_match, not_empty_slice, not_empty_str, not_subset,
    subset,
};
pub use equality::{equal, equal_msg, equal_values, equal_values_msg, not_equal, not_equal_msg};
pub use errors::{
    equal_error_display, error_as, error_as_chain, error_chain_contains, error_contains, error_is,
    io_error_kind, is_error, not_error_as, not_error_is,
};
pub use floats::{
    in_delta_f32, in_delta_f64, in_delta_slice_f64, in_epsilon_f64, in_epsilon_slice_f64,
};
pub use fs::{dir_exists, file_exists, no_dir_exists, no_file_exists};
pub use option_result::{
    assert_err, assert_ok, is_none, is_some, no_error, not_zero, unwrap_some, zero,
};
pub use ordering::{
    greater, greater_or_equal, is_decreasing, is_increasing, is_non_decreasing, is_non_increasing,
    less, less_or_equal, negative, positive,
};
pub use panic::{not_panics, panics, panics_with_substring};
pub use pointer::{not_same_ref, same_arc, same_ref, same_weak};

pub use time::{within_duration, within_range};

/// Fails the test immediately with `msg`.
///
/// # Panics
///
/// Always panics.
///
/// # Examples
///
/// ```should_panic
/// use suitecase::assert::fail;
///
/// fail("expected failure");
/// ```
#[track_caller]
pub fn fail(msg: &str) -> ! {
    panic!("assertion failed: {msg}");
}

/// Fails the test immediately with a formatted message.
///
/// # Panics
///
/// Always panics.
///
/// # Examples
///
/// ```should_panic
/// use suitecase::assert::fail_fmt;
///
/// fail_fmt(format_args!("n={}", 3));
/// ```
#[track_caller]
pub fn fail_fmt(args: std::fmt::Arguments<'_>) -> ! {
    panic!("assertion failed: {args}");
}

/// Asserts that `ok` is `true`, otherwise fails with `msg`.
///
/// # Panics
///
/// Panics when `ok` is `false`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::condition;
///
/// condition(2 > 1, "ordering");
/// ```
#[track_caller]
pub fn condition(ok: bool, msg: &str) {
    if !ok {
        fail(msg);
    }
}

/// Asserts that `f()` returns `true`, otherwise fails with `msg`.
///
/// # Panics
///
/// Panics when `f()` is `false`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::condition_fn;
///
/// condition_fn(|| 1 + 1 == 2, "math");
/// ```
#[track_caller]
pub fn condition_fn(f: impl FnOnce() -> bool, msg: &str) {
    condition(f(), msg);
}

/// Asserts that `a` and `b` point to the same allocation (see [`same_ref`]).
///
/// # Panics
///
/// Panics when the two references do not point to the same allocation.
///
/// # Examples
///
/// ```
/// use suitecase::assert::same;
///
/// let x = 1_i32;
/// same(&x, &x);
/// ```
#[inline]
pub fn same<T: ?Sized>(a: &T, b: &T) {
    same_ref(a, b);
}

/// Asserts that `a` and `b` do not point to the same allocation (see [`not_same_ref`]).
///
/// # Panics
///
/// Panics when the two references point to the same allocation.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_same;
///
/// let a = 1_i32;
/// let b = 1_i32;
/// not_same(&a, &b);
/// ```
#[inline]
pub fn not_same<T: ?Sized>(a: &T, b: &T) {
    not_same_ref(a, b);
}

#[cfg(test)]
mod boolean_test;
#[cfg(test)]
mod collections_test;
#[cfg(test)]
mod equality_test;
#[cfg(test)]
mod errors_test;
#[cfg(test)]
mod floats_test;
#[cfg(test)]
mod fs_test;
#[cfg(test)]
mod option_result_test;
#[cfg(test)]
mod ordering_test;
#[cfg(test)]
mod panic_test;
#[cfg(test)]
mod pointer_test;
#[cfg(test)]
mod root_test;
#[cfg(test)]
mod time_test;
