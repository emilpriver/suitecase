//! Floating-point comparisons for tests (absolute delta and relative epsilon).

/// Asserts that `actual` is within `delta` of `expected` (absolute tolerance, `f64`).
///
/// # Panics
///
/// Panics when `|expected - actual| > delta`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::in_delta_f64;
///
/// in_delta_f64(1.0, 1.0001, 1e-3);
/// ```
#[track_caller]
pub fn in_delta_f64(expected: f64, actual: f64, delta: f64) {
    if (expected - actual).abs() > delta {
        panic!(
            "assertion failed: `in_delta_f64`\n  expected: `{expected}`\n    actual: `{actual}`\n   delta: `{delta}`"
        );
    }
}

/// Asserts that `actual` is within `delta` of `expected` (absolute tolerance, `f32`).
///
/// # Panics
///
/// Panics when `|expected - actual| > delta`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::in_delta_f32;
///
/// in_delta_f32(2.0_f32, 2.0001, 1e-3);
/// ```
#[track_caller]
pub fn in_delta_f32(expected: f32, actual: f32, delta: f32) {
    if (expected - actual).abs() > delta {
        panic!(
            "assertion failed: `in_delta_f32`\n  expected: `{expected}`\n    actual: `{actual}`\n   delta: `{delta}`"
        );
    }
}

/// Like [`in_delta_f64`], but compares two slices element-wise (lengths must match).
///
/// # Panics
///
/// Panics on length mismatch or when any pair exceeds `delta`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::in_delta_slice_f64;
///
/// in_delta_slice_f64(&[1.0, 2.0], &[1.0, 2.0001], 1e-2);
/// ```
#[track_caller]
pub fn in_delta_slice_f64(expected: &[f64], actual: &[f64], delta: f64) {
    if expected.len() != actual.len() {
        panic!(
            "assertion failed: `in_delta_slice_f64` length mismatch {} vs {}",
            expected.len(),
            actual.len()
        );
    }
    for (i, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        if (e - a).abs() > delta {
            panic!(
                "assertion failed: `in_delta_slice_f64` at index {i}\n  expected: `{e}`\n    actual: `{a}`"
            );
        }
    }
}

/// Asserts relative closeness: `|expected - actual| <= epsilon * max(|expected|, |actual|, 1e-12)`.
///
/// # Panics
///
/// Panics when the inequality does not hold.
///
/// # Examples
///
/// ```
/// use suitecase::assert::in_epsilon_f64;
///
/// in_epsilon_f64(100.0, 100.0001, 1e-5);
/// ```
#[track_caller]
pub fn in_epsilon_f64(expected: f64, actual: f64, epsilon: f64) {
    let denom = expected.abs().max(actual.abs()).max(1e-12);
    if (expected - actual).abs() > epsilon * denom {
        panic!(
            "assertion failed: `in_epsilon_f64`\n  expected: `{expected}`\n    actual: `{actual}`\n epsilon: `{epsilon}`"
        );
    }
}

/// Like [`in_epsilon_f64`], but compares two slices element-wise.
///
/// # Panics
///
/// Panics on length mismatch or when any pair fails the epsilon check.
///
/// # Examples
///
/// ```
/// use suitecase::assert::in_epsilon_slice_f64;
///
/// in_epsilon_slice_f64(&[10.0, 20.0], &[10.0, 20.0000001], 1e-6);
/// ```
#[track_caller]
pub fn in_epsilon_slice_f64(expected: &[f64], actual: &[f64], epsilon: f64) {
    if expected.len() != actual.len() {
        panic!(
            "assertion failed: `in_epsilon_slice_f64` length mismatch {} vs {}",
            expected.len(),
            actual.len()
        );
    }
    for (i, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        let denom = e.abs().max(a.abs()).max(1e-12);
        if (e - a).abs() > epsilon * denom {
            panic!("assertion failed: `in_epsilon_slice_f64` at index {i}: {e} vs {a}");
        }
    }
}
