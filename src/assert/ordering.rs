use std::cmp::Ordering;
use std::fmt::Debug;

fn cmp_or_panic<T: Ord + Debug>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

#[track_caller]
pub fn greater<T: Ord + Debug>(a: &T, b: &T) {
    if cmp_or_panic(a, b) != Ordering::Greater {
        panic!(
            "assertion failed: `greater`\n  expected `{:?}` > `{:?}`",
            a, b
        );
    }
}

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

#[track_caller]
pub fn less<T: Ord + Debug>(a: &T, b: &T) {
    if cmp_or_panic(a, b) != Ordering::Less {
        panic!("assertion failed: `less`\n  expected `{:?}` < `{:?}`", a, b);
    }
}

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

#[track_caller]
pub fn positive<T: Ord + Debug + Default>(v: &T) {
    greater(v, &T::default());
}

#[track_caller]
pub fn negative<T>(v: &T)
where
    T: Ord + Debug + Default,
{
    less(v, &T::default());
}

#[track_caller]
pub fn is_increasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] < w[1]) {
        panic!("assertion failed: `is_increasing`: {:?}", slice);
    }
}

#[track_caller]
pub fn is_decreasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] > w[1]) {
        panic!("assertion failed: `is_decreasing`: {:?}", slice);
    }
}

#[track_caller]
pub fn is_non_decreasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] <= w[1]) {
        panic!("assertion failed: `is_non_decreasing`: {:?}", slice);
    }
}

#[track_caller]
pub fn is_non_increasing<T: Ord + Debug>(slice: &[T]) {
    if !slice.windows(2).all(|w| w[0] >= w[1]) {
        panic!("assertion failed: `is_non_increasing`: {:?}", slice);
    }
}
