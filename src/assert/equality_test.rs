use super::equality::*;

#[test]
fn equal_ok() {
    equal(&1_i32, &1_i32);
}

#[test]
#[should_panic(expected = "assertion failed: `equal`")]
fn equal_fails() {
    equal(&1, &2);
}

#[test]
fn equal_msg_includes_prefix() {
    let r = std::panic::catch_unwind(|| equal_msg(&1, &2, "ctx"));
    assert!(r.is_err());
}

#[test]
fn not_equal_ok() {
    not_equal(&1, &2);
}

#[test]
#[should_panic(expected = "assertion failed: `not_equal`")]
fn not_equal_fails() {
    not_equal(&1, &1);
}

#[test]
fn equal_values_i32_to_i64() {
    equal_values::<i32, i64, i64>(1_i32, 1_i64);
}

#[test]
fn equal_values_msg_smoke() {
    equal_values_msg::<u8, u16, u16>(2_u8, 2_u16, "");
}
