use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[track_caller]
pub fn contains_str(haystack: &str, needle: &str) {
    if !haystack.contains(needle) {
        panic!(
            "assertion failed: `contains_str`\n  haystack: `{haystack}`\n    needle: `{needle}`"
        );
    }
}

#[track_caller]
pub fn contains<T: PartialEq + Debug>(slice: &[T], item: &T) {
    if !slice.contains(item) {
        panic!(
            "assertion failed: `contains`\n  slice: `{:?}`\n missing: `{:?}`",
            slice, item
        );
    }
}

#[track_caller]
pub fn contains_key<K: Eq + Hash + Debug, V>(map: &HashMap<K, V>, key: &K) {
    if !map.contains_key(key) {
        panic!(
            "assertion failed: `contains_key`\n  missing key: `{:?}`",
            key
        );
    }
}

#[track_caller]
pub fn not_contains_str(haystack: &str, needle: &str) {
    if haystack.contains(needle) {
        panic!(
            "assertion failed: `not_contains_str`\n  haystack: `{haystack}`\n contains: `{needle}`"
        );
    }
}

#[track_caller]
pub fn not_contains<T: PartialEq + Debug>(slice: &[T], item: &T) {
    if slice.contains(item) {
        panic!(
            "assertion failed: `not_contains`\n  slice: `{:?}`\n has: `{:?}`",
            slice, item
        );
    }
}

#[track_caller]
pub fn len<T>(slice: &[T], n: usize) {
    if slice.len() != n {
        panic!(
            "assertion failed: `len`\n  expected: {n}\n    actual: {}",
            slice.len()
        );
    }
}

#[track_caller]
pub fn len_string(s: &str, n: usize) {
    if s.len() != n {
        panic!(
            "assertion failed: `len_string`\n  expected byte len: {n}\n    actual: {}",
            s.len()
        );
    }
}

#[track_caller]
pub fn empty_str(s: &str) {
    if !s.is_empty() {
        panic!("assertion failed: `empty_str`: expected empty, got `{s}`");
    }
}

#[track_caller]
pub fn not_empty_str(s: &str) {
    if s.is_empty() {
        panic!("assertion failed: `not_empty_str`: expected non-empty");
    }
}

#[track_caller]
pub fn empty_slice<T>(s: &[T]) {
    if !s.is_empty() {
        panic!(
            "assertion failed: `empty_slice`: expected len 0, got {}",
            s.len()
        );
    }
}

#[track_caller]
pub fn not_empty_slice<T>(s: &[T]) {
    if s.is_empty() {
        panic!("assertion failed: `not_empty_slice`: expected non-empty slice");
    }
}

#[track_caller]
pub fn subset<T>(superset: &[T], subset: &[T])
where
    T: Eq + Hash + Debug,
{
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for x in superset {
        *counts.entry(x).or_insert(0) += 1;
    }
    for x in subset {
        let c = counts.get_mut(x).unwrap_or_else(|| {
            panic!(
                "assertion failed: `subset`: missing element `{:?}` in superset",
                x
            )
        });
        if *c == 0 {
            panic!(
                "assertion failed: `subset`: insufficient multiplicity for `{:?}`",
                x
            );
        }
        *c -= 1;
    }
}

#[track_caller]
pub fn not_subset<T>(superset: &[T], subset: &[T])
where
    T: Eq + Hash + Debug,
{
    if multiset_subset(superset, subset) {
        panic!(
            "assertion failed: `not_subset`: `{:?}` is a multiset subset of the superset",
            subset
        );
    }
}

fn multiset_subset<T>(superset: &[T], subset: &[T]) -> bool
where
    T: Eq + Hash,
{
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for x in superset {
        *counts.entry(x).or_insert(0) += 1;
    }
    for x in subset {
        match counts.get_mut(x) {
            Some(c) if *c > 0 => *c -= 1,
            _ => return false,
        }
    }
    true
}

#[track_caller]
pub fn elements_match<T>(left: &[T], right: &[T])
where
    T: Eq + Hash + Debug,
{
    if left.len() != right.len() {
        panic!(
            "assertion failed: `elements_match` length mismatch\n  left: {}\n right: {}",
            left.len(),
            right.len()
        );
    }
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for x in left {
        *counts.entry(x).or_insert(0) += 1;
    }
    for x in right {
        let c = counts.get_mut(x).unwrap_or_else(|| {
            panic!("assertion failed: `elements_match` extra element `{:?}`", x);
        });
        if *c == 0 {
            panic!(
                "assertion failed: `elements_match` multiplicity mismatch for `{:?}`",
                x
            );
        }
        *c -= 1;
    }
}

#[track_caller]
pub fn not_elements_match<T>(left: &[T], right: &[T])
where
    T: Eq + Hash + Debug,
{
    if left.len() != right.len() {
        return;
    }
    let mut counts: HashMap<&T, usize> = HashMap::new();
    for x in left {
        *counts.entry(x).or_insert(0) += 1;
    }
    for x in right {
        match counts.get_mut(x) {
            Some(c) if *c > 0 => *c -= 1,
            _ => return,
        }
    }
    if counts.values().all(|&c| c == 0) {
        panic!("assertion failed: `not_elements_match`: multisets match");
    }
}
