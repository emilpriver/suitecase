use std::collections::HashMap;

use super::collections::*;

#[test]
fn contains_str_ok() {
    contains_str("hello world", "world");
}

#[test]
#[should_panic(expected = "`contains_str`")]
fn contains_str_fails() {
    contains_str("a", "b");
}

#[test]
fn contains_slice_ok() {
    contains(&[1, 2, 3], &2);
}

#[test]
fn contains_key_ok() {
    let mut m = HashMap::new();
    m.insert("k", 1);
    contains_key(&m, &"k");
}

#[test]
fn not_contains_str_ok() {
    not_contains_str("abc", "z");
}

#[test]
fn not_contains_slice_ok() {
    not_contains(&[1, 2], &9);
}

#[test]
fn len_ok() {
    len(&[1, 2, 3], 3);
}

#[test]
fn len_string_ok() {
    len_string("abc", 3);
}

#[test]
fn empty_str_ok() {
    empty_str("");
}

#[test]
fn not_empty_str_ok() {
    not_empty_str("x");
}

#[test]
fn empty_slice_ok() {
    empty_slice::<i32>(&[]);
}

#[test]
fn not_empty_slice_ok() {
    not_empty_slice(&[1]);
}

#[test]
fn subset_ok() {
    subset(&[1, 2, 2, 3], &[2, 1]);
}

#[test]
fn not_subset_ok() {
    not_subset(&[1, 2], &[1, 2, 3]);
}

#[test]
fn elements_match_ok() {
    elements_match(&[1, 2, 2], &[2, 1, 2]);
}

#[test]
fn not_elements_match_ok() {
    not_elements_match(&[1, 2], &[1, 3]);
}
