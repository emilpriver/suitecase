use std::time::Duration;

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

#[track_caller]
pub fn within_range(actual: Duration, start: Duration, end: Duration) {
    if actual < start || actual > end {
        panic!(
            "assertion failed: `within_range`\n  actual: {:?}\n  range: {:?} ..= {:?}",
            actual, start, end
        );
    }
}
