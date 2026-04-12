//! [`std::time::Duration`] assertions.

use std::time::Duration;

/// Asserts that `actual` is within `delta` of `expected` ([`Duration::abs_diff`]).
///
/// # Panics
///
/// Panics when the difference exceeds `delta`.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use suitecase::assert::within_duration;
///
/// within_duration(
///     Duration::from_secs(10),
///     Duration::from_secs(10),
///     Duration::from_millis(1),
/// );
/// ```
#[track_caller]
pub fn within_duration(expected: Duration, actual: Duration, delta: Duration) {
    let diff = actual.abs_diff(expected);
    if diff > delta {
        panic!(
            "assertion failed: `within_duration`\n  expected: {:?}\n    actual: {:?}\n   delta: {:?}",
            expected, actual, delta
        );
    }
}

/// Asserts that `actual` lies in the inclusive range `start..=end`.
///
/// # Panics
///
/// Panics when `actual < start` or `actual > end`.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use suitecase::assert::within_range;
///
/// let mid = Duration::from_secs(5);
/// let start = Duration::from_secs(1);
/// let end = Duration::from_secs(10);
/// within_range(mid, start, end);
/// ```
#[track_caller]
pub fn within_range(actual: Duration, start: Duration, end: Duration) {
    if actual < start || actual > end {
        panic!(
            "assertion failed: `within_range`\n  actual: {:?}\n  range: {:?} ..= {:?}",
            actual, start, end
        );
    }
}
