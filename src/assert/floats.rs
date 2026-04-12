#[track_caller]
pub fn in_delta_f64(expected: f64, actual: f64, delta: f64) {
    if (expected - actual).abs() > delta {
        panic!(
            "assertion failed: `in_delta_f64`\n  expected: `{expected}`\n    actual: `{actual}`\n   delta: `{delta}`"
        );
    }
}

#[track_caller]
pub fn in_delta_f32(expected: f32, actual: f32, delta: f32) {
    if (expected - actual).abs() > delta {
        panic!(
            "assertion failed: `in_delta_f32`\n  expected: `{expected}`\n    actual: `{actual}`\n   delta: `{delta}`"
        );
    }
}

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

#[track_caller]
pub fn in_epsilon_f64(expected: f64, actual: f64, epsilon: f64) {
    let denom = expected.abs().max(actual.abs()).max(1e-12);
    if (expected - actual).abs() > epsilon * denom {
        panic!(
            "assertion failed: `in_epsilon_f64`\n  expected: `{expected}`\n    actual: `{actual}`\n epsilon: `{epsilon}`"
        );
    }
}

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
