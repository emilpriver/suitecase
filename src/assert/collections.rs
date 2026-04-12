//! Assertions for strings, slices, maps, and multiset relationships.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Asserts that `haystack` contains `needle` as a substring.
///
/// # Panics
///
/// Panics when `needle` is not found in `haystack`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::contains_str;
///
/// contains_str("hello world", "world");
/// ```
#[track_caller]
pub fn contains_str(haystack: &str, needle: &str) {
    if !haystack.contains(needle) {
        panic!(
            "assertion failed: `contains_str`\n  haystack: `{haystack}`\n    needle: `{needle}`"
        );
    }
}

/// Asserts that `slice` contains `item` (same semantics as the slice `contains` method).
///
/// # Panics
///
/// Panics when `item` is not present in `slice`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::contains;
///
/// contains(&[1, 2, 3], &2);
/// ```
#[track_caller]
pub fn contains<T: PartialEq + Debug>(slice: &[T], item: &T) {
    if !slice.contains(item) {
        panic!(
            "assertion failed: `contains`\n  slice: `{:?}`\n missing: `{:?}`",
            slice, item
        );
    }
}

/// Asserts that `map` contains `key`.
///
/// # Panics
///
/// Panics when `key` is missing from `map`.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use suitecase::assert::contains_key;
///
/// let mut m = HashMap::new();
/// m.insert("k", 1);
/// contains_key(&m, &"k");
/// ```
#[track_caller]
pub fn contains_key<K: Eq + Hash + Debug, V>(map: &HashMap<K, V>, key: &K) {
    if !map.contains_key(key) {
        panic!(
            "assertion failed: `contains_key`\n  missing key: `{:?}`",
            key
        );
    }
}

/// Asserts that `haystack` does **not** contain `needle` as a substring.
///
/// # Panics
///
/// Panics when `needle` is found in `haystack`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_contains_str;
///
/// not_contains_str("abc", "z");
/// ```
#[track_caller]
pub fn not_contains_str(haystack: &str, needle: &str) {
    if haystack.contains(needle) {
        panic!(
            "assertion failed: `not_contains_str`\n  haystack: `{haystack}`\n contains: `{needle}`"
        );
    }
}

/// Asserts that `slice` does **not** contain `item`.
///
/// # Panics
///
/// Panics when `item` is present in `slice`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_contains;
///
/// not_contains(&[1, 2], &9);
/// ```
#[track_caller]
pub fn not_contains<T: PartialEq + Debug>(slice: &[T], item: &T) {
    if slice.contains(item) {
        panic!(
            "assertion failed: `not_contains`\n  slice: `{:?}`\n has: `{:?}`",
            slice, item
        );
    }
}

/// Asserts that `slice` has length `n`.
///
/// # Panics
///
/// Panics when `slice.len() != n`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::len;
///
/// len(&[1, 2, 3], 3);
/// ```
#[track_caller]
pub fn len<T>(slice: &[T], n: usize) {
    if slice.len() != n {
        panic!(
            "assertion failed: `len`\n  expected: {n}\n    actual: {}",
            slice.len()
        );
    }
}

/// Asserts that `s` has UTF-8 byte length `n` (`s.len()` in bytes).
///
/// # Panics
///
/// Panics when the byte length differs from `n`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::len_string;
///
/// len_string("abc", 3);
/// ```
#[track_caller]
pub fn len_string(s: &str, n: usize) {
    if s.len() != n {
        panic!(
            "assertion failed: `len_string`\n  expected byte len: {n}\n    actual: {}",
            s.len()
        );
    }
}

/// Asserts that `s` is empty.
///
/// # Panics
///
/// Panics when `s` is non-empty.
///
/// # Examples
///
/// ```
/// use suitecase::assert::empty_str;
///
/// empty_str("");
/// ```
#[track_caller]
pub fn empty_str(s: &str) {
    if !s.is_empty() {
        panic!("assertion failed: `empty_str`: expected empty, got `{s}`");
    }
}

/// Asserts that `s` is not empty.
///
/// # Panics
///
/// Panics when `s` is empty.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_empty_str;
///
/// not_empty_str("x");
/// ```
#[track_caller]
pub fn not_empty_str(s: &str) {
    if s.is_empty() {
        panic!("assertion failed: `not_empty_str`: expected non-empty");
    }
}

/// Asserts that `s` has length zero.
///
/// # Panics
///
/// Panics when the slice is non-empty.
///
/// # Examples
///
/// ```
/// use suitecase::assert::empty_slice;
///
/// empty_slice::<i32>(&[]);
/// ```
#[track_caller]
pub fn empty_slice<T>(s: &[T]) {
    if !s.is_empty() {
        panic!(
            "assertion failed: `empty_slice`: expected len 0, got {}",
            s.len()
        );
    }
}

/// Asserts that `s` is non-empty.
///
/// # Panics
///
/// Panics when the slice is empty.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_empty_slice;
///
/// not_empty_slice(&[1]);
/// ```
#[track_caller]
pub fn not_empty_slice<T>(s: &[T]) {
    if s.is_empty() {
        panic!("assertion failed: `not_empty_slice`: expected non-empty slice");
    }
}

/// Asserts that `subset` is a **multiset subset** of `superset` (each value in `subset` must
/// appear in `superset` at least as many times).
///
/// # Panics
///
/// Panics when an element is missing or has insufficient multiplicity.
///
/// # Examples
///
/// ```
/// use suitecase::assert::subset;
///
/// subset(&[1, 2, 2, 3], &[2, 1]);
/// ```
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

/// Asserts that `subset` is **not** a multiset subset of `superset`.
///
/// # Panics
///
/// Panics when `subset` is a multiset subset of `superset`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_subset;
///
/// not_subset(&[1, 2], &[1, 2, 3]);
/// ```
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

/// Asserts that `left` and `right` have the same length and the same multiset of elements.
///
/// # Panics
///
/// Panics on length mismatch or multiset mismatch.
///
/// # Examples
///
/// ```
/// use suitecase::assert::elements_match;
///
/// elements_match(&[1, 2, 2], &[2, 1, 2]);
/// ```
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

/// Asserts that `left` and `right` do **not** have the same multiset of elements (when lengths
/// match); if lengths differ, this returns without panicking.
///
/// # Panics
///
/// Panics when both slices have the same length and the same multiset of elements.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_elements_match;
///
/// not_elements_match(&[1, 2], &[1, 3]);
/// ```
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
