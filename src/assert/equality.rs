use std::fmt::Debug;

/// Asserts that `expected` and `actual` compare equal with `==`.
///
/// # Panics
///
/// Panics when `expected != actual`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::equal;
///
/// equal(&"hello", &"hello");
/// ```
#[track_caller]
pub fn equal<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T) {
    equal_msg(expected, actual, "");
}

/// Like [`equal`], but includes `msg` in the panic message when the assertion fails.
///
/// # Panics
///
/// Panics when `expected != actual`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::equal_msg;
///
/// equal_msg(&10_i32, &10, "count");
/// ```
#[track_caller]
pub fn equal_msg<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T, msg: &str) {
    if expected != actual {
        if msg.is_empty() {
            panic!(
                "assertion failed: `equal`\n  expected: `{:?}`\n    actual: `{:?}`",
                expected, actual
            );
        }
        panic!(
            "assertion failed: `equal`: {msg}\n  expected: `{:?}`\n    actual: `{:?}`",
            expected, actual
        );
    }
}

/// Asserts that `expected` and `actual` compare unequal with `==`.
///
/// # Panics
///
/// Panics when `expected == actual`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_equal;
///
/// not_equal(&1, &2);
/// ```
#[track_caller]
pub fn not_equal<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T) {
    not_equal_msg(expected, actual, "");
}

/// Like [`not_equal`], but includes `msg` in the panic message when the assertion fails.
///
/// # Panics
///
/// Panics when `expected == actual`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_equal_msg;
///
/// not_equal_msg(&1, &2, "must differ");
/// ```
#[track_caller]
pub fn not_equal_msg<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T, msg: &str) {
    if expected == actual {
        if msg.is_empty() {
            panic!(
                "assertion failed: `not_equal`\n  both values: `{:?}`",
                expected
            );
        }
        panic!(
            "assertion failed: `not_equal`: {msg}\n  both values: `{:?}`",
            expected
        );
    }
}

/// Converts both values to a common type `V` via [`Into`] and asserts they are equal.
///
/// # Panics
///
/// Panics when the converted values are not equal.
///
/// # Examples
///
/// ```
/// use suitecase::assert::equal_values;
///
/// equal_values::<u32, u64, u64>(1_u32, 1_u64);
/// ```
#[track_caller]
pub fn equal_values<T, U, V>(expected: T, actual: U)
where
    T: Into<V>,
    U: Into<V>,
    V: PartialEq + Debug,
{
    equal_values_msg(expected, actual, "");
}

/// Like [`equal_values`], but includes `msg` in the panic message when the assertion fails.
///
/// # Panics
///
/// Panics when the converted values are not equal.
///
/// # Examples
///
/// ```
/// use suitecase::assert::equal_values_msg;
///
/// equal_values_msg::<u32, u64, u64>(1_u32, 1_u64, "same magnitude");
/// ```
#[track_caller]
pub fn equal_values_msg<T, U, V>(expected: T, actual: U, msg: &str)
where
    T: Into<V>,
    U: Into<V>,
    V: PartialEq + Debug,
{
    let e: V = expected.into();
    let a: V = actual.into();
    equal_msg(&e, &a, msg);
}
