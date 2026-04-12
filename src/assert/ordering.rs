//! Ordering and monotonicity assertions.

use std::cmp::Ordering;
use std::fmt::Debug;

fn cmp_or_panic<T: Ord + Debug>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

/// Asserts that `a` is strictly greater than `b`.
///
/// # Panics
///
/// Panics when `a <= b` in the `Ord` order.
///
/// # Examples
///
/// ```
/// use suitecase::assert::greater;
///
/// greater(&3, &2);
/// ```
#[track_caller]
pub fn greater<T: Ord + Debug>(a: &T, b: &T) {
    if cmp_or_panic(a, b) != Ordering::Greater {
        panic!(
            "assertion failed: `greater`\n  expected `{:?}` > `{:?}`",
            a, b
        );
    }
}

/// Asserts that `a` is greater than or equal to `b`.
///
/// # Panics
///
/// Panics when `a < b`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::greater_or_equal;
///
/// greater_or_equal(&2, &2);
/// ```
#[track_caller]
pub fn greater_or_equal<T: Ord + Debug>(a: &T, b: &T) {
    match cmp_or_panic(a, b) {
        Ordering::Greater | Ordering::Equal => {}
        Ordering::Less => panic!(
            "assertion failed: `greater_or_equal`\n  expected `{:?}` >= `{:?}`",
            a, b
        ),
    }
}

/// Asserts that `a` is strictly less than `b`.
///
/// # Panics
///
/// Panics when `a >= b`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::less;
///
/// less(&1, &9);
/// ```
#[track_caller]
pub fn less<T: Ord + Debug>(a: &T, b: &T) {
    if cmp_or_panic(a, b) != Ordering::Less {
        panic!("assertion failed: `less`\n  expected `{:?}` < `{:?}`", a, b);
    }
}

/// Asserts that `a` is less than or equal to `b`.
///
/// # Panics
///
/// Panics when `a > b`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::less_or_equal;
///
/// less_or_equal(&4, &4);
/// ```
#[track_caller]
pub fn less_or_equal<T: Ord + Debug>(a: &T, b: &T) {
    match cmp_or_panic(a, b) {
        Ordering::Less | Ordering::Equal => {}
        Ordering::Greater => panic!(
            "assertion failed: `less_or_equal`\n  expected `{:?}` <= `{:?}`",
            a, b
        ),
    }
}

/// Asserts that `v` is strictly greater than [`Default::default`].
///
/// # Panics
///
/// Panics when `v <= default`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::positive;
///
/// positive(&1_i32);
/// ```
#[track_caller]
pub fn positive<T: Ord + Debug + Default>(v: &T) {
    greater(v, &T::default());
}

/// Asserts that `v` is strictly less than [`Default::default`].
///
/// # Panics
///
/// Panics when `v >= default`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::negative;
///
/// negative(&-1_i32);
/// ```
#[track_caller]
pub fn negative<T>(v: &T)
where
    T: Ord + Debug + Default,
{
    less(v, &T::default());
}

/// Asserts that each consecutive pair in `slice` is strictly increasing.
///
/// # Panics
///
/// Panics when the slice is not strictly increasing.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_increasing;
///
/// is_increasing(&[1, 2, 3]);
/// ```
#[track_caller]
pub fn is_increasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] < w[1]) {
        panic!("assertion failed: `is_increasing`: {:?}", slice);
    }
}

/// Asserts that each consecutive pair in `slice` is strictly decreasing.
///
/// # Panics
///
/// Panics when the slice is not strictly decreasing.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_decreasing;
///
/// is_decreasing(&[3_i32, 2, 1]);
/// ```
#[track_caller]
pub fn is_decreasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] > w[1]) {
        panic!("assertion failed: `is_decreasing`: {:?}", slice);
    }
}

/// Asserts that each consecutive pair in `slice` is non-decreasing (`<=`).
///
/// # Panics
///
/// Panics when any pair violates `a <= b`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_non_decreasing;
///
/// is_non_decreasing(&[1, 2, 2, 3]);
/// ```
#[track_caller]
pub fn is_non_decreasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] <= w[1]) {
        panic!("assertion failed: `is_non_decreasing`: {:?}", slice);
    }
}

/// Asserts that each consecutive pair in `slice` is non-increasing (`>=`).
///
/// # Panics
///
/// Panics when any pair violates `a >= b`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_non_increasing;
///
/// is_non_increasing(&[3_i32, 2, 2, 1]);
/// ```
#[track_caller]
pub fn is_non_increasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] >= w[1]) {
        panic!("assertion failed: `is_non_increasing`: {:?}", slice);
    }
}
