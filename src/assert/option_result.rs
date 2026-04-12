use std::fmt::Debug;

#[track_caller]
pub fn is_none<T>(opt: &Option<T>) {
    if opt.is_some() {
        panic!("assertion failed: `is_none`: expected None, got Some(...)");
    }
}

#[track_caller]
pub fn is_some<T>(opt: &Option<T>) {
    if opt.is_none() {
        panic!("assertion failed: `is_some`: expected Some, got None");
    }
}

#[track_caller]
pub fn unwrap_some<T>(opt: Option<T>) -> T {
    opt.expect("assertion failed: `unwrap_some`")
}

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

#[track_caller]
pub fn not_zero<T: Default + PartialEq + Debug>(value: &T) {
    let z = T::default();
    if value == &z {
        panic!("assertion failed: `not_zero`: value is default `{:?}`", z);
    }
}

#[track_caller]
pub fn assert_ok<T, E: Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("assertion failed: `assert_ok`: {e:?}"),
    }
}

#[inline]
#[track_caller]
pub fn no_error<T, E: Debug>(result: Result<T, E>) -> T {
    assert_ok(result)
}

#[track_caller]
pub fn assert_err<T: Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(v) => panic!("assertion failed: `assert_err`: expected Err, got Ok({v:?})"),
        Err(e) => e,
    }
}
