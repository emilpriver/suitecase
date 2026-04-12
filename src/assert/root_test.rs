use super::*;

#[test]
#[should_panic(expected = "assertion failed: boom")]
fn fail_panics() {
    fail("boom");
}

#[test]
#[should_panic(expected = "assertion failed: x=1")]
fn fail_fmt_panics() {
    fail_fmt(format_args!("x={}", 1));
}

#[test]
fn condition_ok() {
    condition(true, "nope");
}

#[test]
#[should_panic(expected = "bad")]
fn condition_fails() {
    condition(false, "bad");
}

#[test]
fn condition_fn_ok() {
    condition_fn(|| true, "bad");
}

#[test]
fn same_delegates_to_same_ref() {
    let x = 1_i32;
    same(&x, &x);
}

#[test]
fn not_same_delegates() {
    let a = 1_i32;
    let b = 1_i32;
    not_same(&a, &b);
}
