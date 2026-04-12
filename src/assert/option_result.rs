//! Assertions for [`Option`], [`Result`], and “non-zero” numeric checks.

use std::fmt::Debug;

/// Asserts that `opt` is `None`.
///
/// # Panics
///
/// Panics when `opt` is `Some`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_none;
///
/// is_none(&None::<i32>);
/// ```
#[track_caller]
pub fn is_none<T>(opt: &Option<T>) {
    if opt.is_some() {
        panic!("assertion failed: `is_none`: expected None, got Some(...)");
    }
}

/// Asserts that `opt` is `Some`.
///
/// # Panics
///
/// Panics when `opt` is `None`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_some;
///
/// is_some(&Some(7_i32));
/// ```
#[track_caller]
pub fn is_some<T>(opt: &Option<T>) {
    if opt.is_none() {
        panic!("assertion failed: `is_some`: expected Some, got None");
    }
}

/// Unwraps `opt` or panics with a stable assertion message.
///
/// # Panics
///
/// Panics when `opt` is `None`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::unwrap_some;
///
/// assert_eq!(unwrap_some(Some(3_i32)), 3);
/// ```
#[track_caller]
pub fn unwrap_some<T>(opt: Option<T>) -> T {
    opt.expect("assertion failed: `unwrap_some`")
}

/// Asserts that `value` equals [`Default::default`].
///
/// # Panics
///
/// Panics when `value` is not the default.
///
/// # Examples
///
/// ```
/// use suitecase::assert::zero;
///
/// zero(&0_i32);
/// ```
#[track_caller]
pub fn zero<T: Default + PartialEq + Debug>(value: &T) {
    let z = T::default();
    if value != &z {
        panic!(
            "assertion failed: `zero`\n  expected default: `{:?}`\n    actual: `{:?}`",
            z, value
        );
    }
}

/// Asserts that `value` does **not** equal [`Default::default`].
///
/// # Panics
///
/// Panics when `value` equals the default.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_zero;
///
/// not_zero(&1_i32);
/// ```
#[track_caller]
pub fn not_zero<T: Default + PartialEq + Debug>(value: &T) {
    let z = T::default();
    if value == &z {
        panic!("assertion failed: `not_zero`: value is default `{:?}`", z);
    }
}

/// Unwraps `Ok` or panics printing the `Err` value.
///
/// # Panics
///
/// Panics when `result` is `Err`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::assert_ok;
///
/// assert_eq!(assert_ok(Ok::<_, ()>(9_i32)), 9);
/// ```
#[track_caller]
pub fn assert_ok<T, E: Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("assertion failed: `assert_ok`: {e:?}"),
    }
}

/// Alias for [`assert_ok`]: unwraps `Ok` or panics with the error debug-formatted.
///
/// # Panics
///
/// Panics when `result` is `Err`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::no_error;
///
/// assert_eq!(no_error(Ok::<i32, std::convert::Infallible>(3)), 3);
/// ```
#[inline]
#[track_caller]
pub fn no_error<T, E: Debug>(result: Result<T, E>) -> T {
    assert_ok(result)
}

/// Unwraps `Err` or panics if `Ok`.
///
/// # Panics
///
/// Panics when `result` is `Ok`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::assert_err;
///
/// let e = assert_err::<i32, &str>(Err("bad"));
/// assert_eq!(e, "bad");
/// ```
#[track_caller]
pub fn assert_err<T: Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(v) => panic!("assertion failed: `assert_err`: expected Err, got Ok({v:?})"),
        Err(e) => e,
    }
}
