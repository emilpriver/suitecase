use super::ordering::*;

#[test]
fn greater_ok() {
    greater(&3, &2);
}

#[test]
fn greater_or_equal_ok() {
    greater_or_equal(&2, &2);
}

#[test]
fn less_ok() {
    less(&1, &9);
}

#[test]
fn less_or_equal_ok() {
    less_or_equal(&4, &4);
}

#[test]
fn positive_ok() {
    positive(&1_i32);
}

#[test]
fn negative_ok() {
    negative(&-1_i32);
}

#[test]
fn is_increasing_ok() {
    is_increasing(&[1, 2, 3]);
}

#[test]
fn is_decreasing_ok() {
    is_decreasing(&[3_i32, 2, 1]);
}

#[test]
fn is_non_decreasing_ok() {
    is_non_decreasing(&[1, 2, 2, 3]);
}

#[test]
fn is_non_increasing_ok() {
    is_non_increasing(&[3_i32, 2, 2, 1]);
}
