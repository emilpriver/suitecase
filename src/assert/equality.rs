use std::fmt::Debug;

#[track_caller]
pub fn equal<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T) {
    equal_msg(expected, actual, "");
}

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

#[track_caller]
pub fn not_equal<T: PartialEq + Debug + ?Sized>(expected: &T, actual: &T) {
    not_equal_msg(expected, actual, "");
}

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

#[track_caller]
pub fn equal_values<T, U, V>(expected: T, actual: U)
where
    T: Into<V>,
    U: Into<V>,
    V: PartialEq + Debug,
{
    equal_values_msg(expected, actual, "");
}

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
